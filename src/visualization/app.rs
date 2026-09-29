//! Native application; immutable simulation snapshots cross the worker boundary.
use super::{camera, detector_view::DetectorView, playback::Playback, renderer::Renderer, ui};
use crate::{
    session::{DetectorMode, PlaybackMode, SessionConfig},
    simulation::{
        SimulationData, storage,
        worker::{Request, Worker},
    },
};
use eframe::egui;
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{Arc, atomic::Ordering, mpsc},
    time::{Duration, Instant},
};

pub struct App {
    cfg: SessionConfig,
    archive: super::archive_ui::ArchiveUi,
    worker: Worker,
    data: Option<Arc<SimulationData>>,
    renderer: Renderer,
    free_fall: super::free_fall_ui::FreeFall,
    detector: DetectorView,
    playback: Playback,
    selected: Option<usize>,
    busy: bool,
    loading: bool,
    dirty: Option<Instant>,
    status: String,
    last_frame: Instant,
    frames: VecDeque<f64>,
    counts: [usize; 5],
    errors: [f64; 5],
    started: Instant,
    load_seconds: f64,
    fit_next: bool,
    pan_drag: bool,
    camera_drag: Option<(egui::PointerButton, egui::Pos2)>,
    export_path: String,
    export_rx: Option<mpsc::Receiver<Result<String, String>>>,
    screenshot: Option<PathBuf>,
    measure: Option<Measurement>,
    gpu: String,
}
struct Measurement {
    directory: PathBuf,
    phase: usize,
    start: Option<Instant>,
    samples: Vec<f64>,
    rows: Vec<serde_json::Value>,
    finishing: bool,
}
const PHASES: [(&str, usize, bool, bool, bool); 7] = [
    ("trajectories_256", 256, false, true, false),
    ("trajectories_512", 512, false, true, false),
    ("field_off", 256, false, true, false),
    ("field_on", 256, true, true, false),
    ("overlay_off", 256, false, false, false),
    ("overlay_on", 256, false, true, false),
    ("playback", 256, true, true, true),
];
impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        cfg: SessionConfig,
        input: Option<PathBuf>,
        measure: Option<PathBuf>,
    ) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        let state = cc
            .wgpu_render_state
            .clone()
            .expect("wgpu renderer requested");
        let info = state.adapter.get_info();
        let gpu = format!(
            "{} | {:?} | {:?} | vendor {:#x}",
            info.name, info.backend, info.device_type, info.vendor
        );
        println!(
            "Selected GPU: {gpu}\nFeatures: {:?}\nLimits: {:?}",
            state.adapter.features(),
            state.adapter.limits()
        );
        let worker = Worker::default();
        let archive = super::archive_ui::ArchiveUi::default();
        let loading = input.is_some();
        worker.submit(input.map_or_else(
            || {
                Request::CalculateAndSave(
                    Box::new(cfg.clone()),
                    PathBuf::from(&archive.auto_save_root),
                )
            },
            Request::Load,
        ));
        let now = Instant::now();
        let fit_next = cfg.view.fit_on_load;
        Self {
            cfg,
            archive,
            worker,
            data: None,
            renderer: Renderer::new(state),
            free_fall: super::free_fall_ui::FreeFall::default(),
            detector: DetectorView::default(),
            playback: Playback::default(),
            selected: None,
            busy: true,
            loading,
            dirty: None,
            status: "Loading / calculating on the reference worker".into(),
            last_frame: now,
            frames: VecDeque::with_capacity(240),
            counts: [0; 5],
            errors: [0.; 5],
            started: now,
            load_seconds: 0.,
            fit_next,
            pan_drag: false,
            camera_drag: None,
            export_path: "results/visualization_export".into(),
            export_rx: None,
            screenshot: None,
            measure: measure.map(|directory| Measurement {
                directory,
                phase: 0,
                start: None,
                samples: vec![],
                rows: vec![],
                finishing: false,
            }),
            gpu,
        }
    }
    fn submit(&mut self) {
        self.worker.submit(Request::CalculateAndSave(
            Box::new(self.cfg.clone()),
            PathBuf::from(&self.archive.auto_save_root),
        ));
        self.archive.comparison = None;
        self.archive.save_message = "Auto-save pending calculation completion".into();
        self.busy = true;
        self.loading = false;
        self.dirty = None;
        self.status = "CPU f64 reference calculation in background".into();
    }
    fn poll(&mut self) {
        while let Ok(done) = self.worker.receiver.try_recv() {
            if done.generation != self.worker.generation.load(Ordering::Acquire) {
                continue;
            }
            self.busy = false;
            if let Some(message) = done.archive_message {
                self.archive.save_message = message;
            }
            if let Some(report) = done.comparison {
                self.archive.comparison = Some(report);
            }
            match done.result {
                Ok(data) => {
                    if self.loading {
                        self.cfg.generator = done.generator;
                        let post = self.cfg.experiment.postprocess.clone();
                        let resolution = self.cfg.experiment.detector.resolution;
                        self.cfg.experiment = data.experiment.clone();
                        self.cfg.experiment.postprocess = post;
                        self.cfg.experiment.detector.resolution = resolution;
                        self.loading = false;
                    }
                    self.load_seconds = self.started.elapsed().as_secs_f64();
                    self.counts = data.counts();
                    self.errors = data.max_errors();
                    self.playback.range = data.time_range();
                    self.playback.reset();
                    self.status = data.trajectory_note.clone();
                    // Diagnostics are drawn before the viewport. Refresh cached
                    // indices before a smaller result can be read by that panel.
                    self.renderer.prepare(data.clone(), &self.cfg, None);
                    self.data = Some(data);
                    self.detector.invalidate();
                    self.selected = None;
                    println!(
                        "Scene ready: {:.6}s; counts {:?}",
                        self.load_seconds, self.counts
                    );
                }
                Err(e) => {
                    self.status = format!("ERROR: {e}");
                    eprintln!("{}", self.status);
                }
            }
        }
        if let Some(rx) = &self.export_rx {
            match rx.try_recv() {
                Ok(result) => {
                    self.status = result.unwrap_or_else(|e| format!("Export failed: {e}"));
                    self.export_rx = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status = "Export worker disconnected".into();
                    self.export_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if self
            .dirty
            .is_some_and(|t| t.elapsed() > Duration::from_millis(450))
        {
            self.submit();
        }
    }
    fn coherent_config(&self) -> SessionConfig {
        let mut config = self.cfg.clone();
        config.view.fit_on_load = false;
        if let Some(data) = &self.data {
            let post = config.experiment.postprocess.clone();
            let res = config.experiment.detector.resolution;
            config.experiment = data.experiment.clone();
            config.experiment.postprocess = post;
            config.experiment.detector.resolution = res;
        }
        config
    }
    fn export(&mut self, kind: u8, ctx: &egui::Context) {
        let path = PathBuf::from(&self.export_path);
        if let Err(e) = std::fs::create_dir_all(&path) {
            self.status = e.to_string();
            return;
        }
        if kind == 0 {
            self.screenshot = Some(path.join("screenshot.png"));
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            return;
        }
        if kind == 1 {
            self.status = write_json(&path.join("visualization.json"), &self.coherent_config())
                .map_or_else(|e| e, |()| "Configuration saved".into());
            return;
        }
        if kind == 2 {
            self.status = write_png(
                &path.join("detector_view.png"),
                self.detector.resolution,
                &self.detector.rgba,
            )
            .and_then(|()| {
                crate::output::csv::counts(
                    &path.join("detector_view.csv"),
                    &self.detector.counts,
                    self.detector.resolution[0],
                )
                .map_err(|e| e.to_string())
            })
            .map_or_else(|e| e, |()| "Current detector PNG + count CSV saved".into());
            return;
        }
        if let Some(data) = self.data.clone() {
            if self.export_rx.is_some() {
                self.status = "An export is already running".into();
                return;
            }
            let cfg = self.coherent_config();
            let (tx, rx) = mpsc::channel();
            self.export_rx = Some(rx);
            std::thread::spawn(move || {
                let result = storage::save(&data, &cfg, &path)
                    .map(|()| format!("Saved complete run to {}", path.display()));
                let _ = tx.send(result);
            });
            self.status = "Exporting trajectories, events, PNG/APNG on a worker".into();
        }
    }
    fn screenshots(&mut self, ctx: &egui::Context) {
        let images = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| {
                    if let egui::Event::Screenshot { image, .. } = e {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        });
        for image in images {
            if let Some(path) = self.screenshot.take() {
                let rgba = image
                    .pixels
                    .iter()
                    .flat_map(|p| p.to_array())
                    .collect::<Vec<_>>();
                self.status = write_png(&path, image.size, &rgba)
                    .map_or_else(|e| e, |()| format!("Screenshot saved: {}", path.display()));
            }
        }
    }
    fn measure(&mut self, ctx: &egui::Context, dt: f64) {
        if self.data.is_none() {
            return;
        }
        let Some(m) = &mut self.measure else {
            return;
        };
        if m.finishing {
            if self.screenshot.is_none() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            return;
        }
        let (name, trajectories, field, overlay, play) = PHASES[m.phase];
        // Exercise a moving camera; only camera uniforms may change during a phase.
        self.cfg.camera.yaw += (dt * 0.035) as f32;
        if m.start.is_none() {
            m.start = Some(Instant::now());
            m.samples.clear();
            self.cfg.view.max_rendered_trajectories = trajectories;
            self.cfg.view.visibility[10] = field;
            self.cfg.view.detector_overlay = overlay;
            self.cfg.view.playback_mode = if play {
                PlaybackMode::Propagation
            } else {
                PlaybackMode::Static
            };
            self.cfg.view.detector_mode = if play {
                DetectorMode::Cumulative
            } else {
                DetectorMode::Accumulated
            };
            self.playback.reset();
            self.playback.playing = play;
        }
        let elapsed = m.start.unwrap().elapsed().as_secs_f64();
        if elapsed > 1.5 {
            m.samples.push(dt);
        }
        if elapsed >= 5.5 {
            let mut sorted = m.samples.clone();
            sorted.sort_by(f64::total_cmp);
            let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
            let p95 = sorted[(sorted.len() * 95 / 100).min(sorted.len() - 1)];
            let row = serde_json::json!({"phase":name,"frames":sorted.len(),"fps":1./mean,"mean_frame_ms":mean*1000.,"p95_frame_ms":p95*1000.,"rendered_trajectories":self.renderer.rendered_ids.len(),"ray_buffer_uploads":self.renderer.ray_uploads,"geometry_uploads":self.renderer.geometry_uploads,"viewport_pixels":self.renderer.size});
            println!("MEASURE {row}");
            m.rows.push(row);
            m.phase += 1;
            m.start = None;
            if m.phase == PHASES.len() {
                let d = self.data.as_ref().unwrap();
                let report = serde_json::json!({"gpu":self.gpu,"adapter_features":format!("{:?}",self.renderer.state.adapter.features()),"adapter_limits":format!("{:?}",self.renderer.state.adapter.limits()),"scene_load_seconds":self.load_seconds,"physical_rays":d.rays.len(),"cpu_simulation_seconds":d.seconds,"counts":self.counts,"max_errors":self.errors,"frame_measurements":m.rows,"method":"Native window frame intervals, vsync, 1.5s warmup + 4s sample per phase; includes egui and presentation; no GPU timestamp claim"});
                let result = std::fs::create_dir_all(&m.directory)
                    .map_err(|e| e.to_string())
                    .and_then(|()| {
                        write_json(&m.directory.join("render_measurements.json"), &report)
                    });
                if let Err(e) = result {
                    self.status = e;
                }
                self.cfg.view.playback_mode = PlaybackMode::Static;
                self.cfg.view.detector_mode = DetectorMode::Accumulated;
                self.screenshot = Some(m.directory.join("scene.png"));
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                m.finishing = true;
            }
        }
    }
}
impl eframe::App for App {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
        if self.frames.len() == 240 {
            self.frames.pop_front();
        }
        self.frames.push_back(dt);
        self.poll();
        self.screenshots(&ctx);
        self.measure(&ctx, dt);
        self.playback.tick(dt, self.cfg.view.playback_speed as f64);
        let old_key = self.cfg.physics_key();
        egui::Panel::top("title").show(root,|ui|{ui.horizontal(|ui|{ui.heading("KERR / GEODESIC LAB");ui.separator();ui.label("CPU f64 physics  /  wgpu 3D");if self.busy{ui.spinner();ui.label(format!("{} / {} rays",self.worker.completed.load(Ordering::Relaxed),self.worker.total.load(Ordering::Relaxed)));}});ui.small("G = c = M = 1   |   +Z spin   |   Cartesian-like BL coordinates, not a Euclidean embedding or Kerr-Schild time");});
        egui::Panel::bottom("playback").show(root, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(if self.playback.playing {
                        "Pause"
                    } else {
                        "Play"
                    })
                    .clicked()
                {
                    if self.playback.time >= self.playback.range[1] {
                        self.playback.reset();
                    }
                    self.playback.playing = !self.playback.playing;
                }
                if ui.button("Stop").clicked() {
                    self.playback.reset();
                }
                if ui.button("Reset time").clicked() {
                    self.playback.reset();
                }
                ui.add(
                    egui::Slider::new(
                        &mut self.playback.time,
                        self.playback.range[0]..=self.playback.range[1],
                    )
                    .text("t_BL / M")
                    .fixed_decimals(3),
                );
            });
            ui.label(&self.status);
        });
        egui::Panel::left("controls")
            .default_size(310.)
            .resizable(true)
            .show(root, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some((directory, simulation)) = self.archive.show(ui, self.busy) {
                        self.worker.submit(Request::Reproduce { directory, simulation, view: Box::new(self.cfg.clone()) });
                        self.busy = true;
                        self.loading = true;
                        self.dirty = None;
                        self.status = "Reproducing saved initial conditions on CPU worker; no auto-save".into();
                        self.fit_next = true;
                    }
                    if self.data.as_ref().is_some_and(|d| !d.source_available) {
                        ui.small("SourcePlane is not stored. Source controls define a NEW experiment only.");
                    }
                    self.free_fall.show(ui, &mut self.cfg.view);
                    ui::settings(ui, &mut self.cfg);
                    if ui
                        .button("Recompute with current physical settings")
                        .clicked()
                    {
                        self.submit();
                    }
                    ui.separator();
                    ui.collapsing("Export (new filenames only)", |ui| {
                        ui.text_edit_singleline(&mut self.export_path);
                        for (kind, label) in [
                            (0, "Screenshot"),
                            (1, "Save visualization config"),
                            (2, "Export current detector image + CSV"),
                            (3, "Save complete run + frame sequences"),
                        ] {
                            if ui.button(label).clicked() {
                                self.export(kind, &ctx);
                            }
                        }
                        ui.add(
                            egui::DragValue::new(&mut self.cfg.trajectory_export_stride)
                                .range(1..=10000)
                                .prefix("Trajectory export stride "),
                        );
                    });
                });
            });
        egui::Panel::right("diagnostics")
            .default_size(260.)
            .resizable(true)
            .show(root, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Observation");
                    ui.label(&self.gpu);
                    let avg = self.frames.iter().sum::<f64>() / self.frames.len().max(1) as f64;
                    ui.label(format!(
                        "{:.1} FPS  |  {:.2} ms mean",
                        1. / avg,
                        avg * 1000.
                    ));
                    if let Some(data) = &self.data {
                        ui.label(format!(
                            "chi {:.4} | physical rays {}",
                            data.experiment.spin,
                            data.rays.len()
                        ));
                        ui.label(format!(
                            "Rendered trajectories {}",
                            self.renderer
                                .rendered_ids
                                .iter()
                                .filter(|&&i| !data.rays[i].samples.is_empty())
                                .count()
                        ));
                        for (label, n) in [
                            "Active",
                            "Detected",
                            "Captured",
                            "Escaped",
                            "NumericalFailure",
                        ]
                        .into_iter()
                        .zip(self.counts)
                        {
                            ui.label(format!("{label}: {n}"));
                        }
                        ui.label(format!("Reference calculation: {:.3} s", data.seconds));
                        ui.small(&data.trajectory_note);
                        if data.rays.iter().all(|r| r.samples.is_empty()) {
                            ui.colored_label(
                                egui::Color32::YELLOW,
                                "TRAJECTORY UNAVAILABLE\nEvents and detector counts remain valid.",
                            );
                        }
                        ui.separator();
                        let mut id = self.selected.unwrap_or(0);
                        ui.horizontal(|ui| {
                            ui.label("Ray ID");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut id)
                                        .range(0..=data.rays.len().saturating_sub(1)),
                                )
                                .changed()
                            {
                                self.selected = Some(id);
                            }
                            if ui.button("Select").clicked() {
                                self.selected = Some(id);
                            }
                            if ui.button("Clear").clicked() {
                                self.selected = None;
                            }
                        });
                        if let Some(ray) = self
                            .selected
                            .and_then(|id| data.rays.iter().find(|r| r.ray_id == id))
                        {
                            ui.colored_label(
                                egui::Color32::LIGHT_YELLOW,
                                format!("Ray {} / {:?}", ray.ray_id, ray.status),
                            );
                            if data.source_available {
                                ui.label(format!("Launch u,v: {:.4?}", ray.source_uv));
                            }
                            let k = crate::physics::kerr::Kerr::new(data.experiment.spin).unwrap();
                            let s = ray.initial;
                            ui.label(format!(
                                "Launch XYZ: {:.4?}",
                                crate::physics::coordinates::bl_to_cartesian(k, s[1], s[6], s[2])
                            ));
                            if let Some(direction) = ray.launch_direction {
                                ui.label(format!("Cartesian direction: {direction:?}"));
                            } else if data.source_available {
                                ui.label(format!(
                                    "Direction: {:?}",
                                    data.experiment.source.direction
                                ));
                            }
                            ui.label(format!("Stop: {}", ray.stop_reason));
                            if let Some(e) = data.events.iter().find(|e| e.ray_id == ray.ray_id) {
                                ui.label(format!(
                                    "u_hit {:.6}\nv_hit {:.6}\nt_hit {:.9}\ndelta_t {:.9}",
                                    e.u_hit, e.v_hit, e.t_hit, e.delta_t
                                ));
                            } else {
                                ui.label("No detector hit");
                            }
                            ui.label(format!(
                                "Steps {} accepted / {} rejected",
                                ray.accepted_steps, ray.rejected_steps
                            ));
                            if let Some(error) = &ray.failure {
                                ui.colored_label(egui::Color32::LIGHT_RED, error);
                            }
                        }
                    }
                    ui.collapsing("Conservation / GPU cache", |ui| {
                        for (label, error) in [
                            "max |null|",
                            "max rel E",
                            "max rel Lz",
                            "max |delta Q|",
                            "max rel Q",
                        ]
                        .into_iter()
                        .zip(self.errors)
                        {
                            ui.label(format!("{label}: {error:.4e}"));
                        }
                        ui.label(format!(
                            "Ray uploads: {}\nGeometry uploads: {}",
                            self.renderer.ray_uploads, self.renderer.geometry_uploads
                        ));
                        ui.label(format!(
                            "Grid extent {:.2}, spacing {:.2}",
                            self.renderer.grid_values[0], self.renderer.grid_values[1]
                        ));
                        ui.label(format!(
                            "max sampled omega {:.6e} / M",
                            self.renderer.max_omega
                        ));
                        let mut sorted = self.frames.iter().copied().collect::<Vec<_>>();
                        sorted.sort_by(f64::total_cmp);
                        if !sorted.is_empty() {
                            ui.label(format!(
                                "p95 frame {:.2} ms",
                                sorted[sorted.len() * 95 / 100] * 1000.
                            ));
                        }
                    });
                    if self.cfg.view.detector_panel && !self.detector.rgba.is_empty() {
                        ui.separator();
                        ui.label(format!("Detector {:?}", self.cfg.view.detector_mode));
                        ui.label(format!(
                            "Shown {} / total {} hits",
                            self.detector.displayed, self.detector.total
                        ));
                        let width = ui.available_width();
                        ui.image((
                            self.renderer.detector_texture_id,
                            egui::vec2(
                                width,
                                width * self.detector.resolution[1] as f32
                                    / self.detector.resolution[0] as f32,
                            ),
                        ));
                        ui.small(
                            "u right / v up; coordinate pixel counts. White level configurable.",
                        );
                    }
                });
            });
        if old_key != self.cfg.physics_key() {
            self.worker.cancel();
            self.dirty = Some(Instant::now());
            self.busy = true;
            self.status = "Physical settings changed; waiting 450 ms before reference solve".into();
        }
        egui::CentralPanel::default().show(root,|ui|{
            ui.horizontal_wrapped(|ui|{
                if ui.button("Fit scene").clicked(){self.fit_next=true;}
                ui.toggle_value(&mut self.pan_drag,"Pan drag");
                if ui.button("Reset camera").clicked(){self.cfg.camera=Default::default();self.fit_next=true;}
                if ui.button("Kerr center").clicked(){self.cfg.camera.target=[0.;3];self.cfg.camera.distance=24.;}
                if let Some(data)=&self.data{for(label,plane)in[("Source",&data.experiment.source.plane),("Detector",&data.experiment.detector.plane)]{if ui.add_enabled(label != "Source" || data.source_available, egui::Button::new(label)).clicked(){self.cfg.camera.target=plane.center.map(|x|x as f32);self.cfg.camera.distance=(plane.width.max(plane.height)*1.6)as f32;}}}
            });ui.small("Drag: orbit / Pan drag  |  Wheel: zoom  |  Arrows: orbit  |  Shift+arrows: pan  |  +/-: zoom");
            let size=ui.available_size().max(egui::vec2(16.,16.));
            if !ctx.egui_wants_keyboard_input(){
                let (movement,shift,zoom)=ctx.input(|i|{
                    let x=(i.key_pressed(egui::Key::ArrowRight) as i32-i.key_pressed(egui::Key::ArrowLeft) as i32) as f32;
                    let y=(i.key_pressed(egui::Key::ArrowDown) as i32-i.key_pressed(egui::Key::ArrowUp) as i32) as f32;
                    ([x*35.,y*35.],i.modifiers.shift,(i.key_pressed(egui::Key::Plus) as i32-i.key_pressed(egui::Key::Minus) as i32) as f32*120.)
                });
                if shift{camera::pan(&mut self.cfg.camera,movement,size.y);}else{camera::orbit(&mut self.cfg.camera,movement);}
                camera::zoom(&mut self.cfg.camera,zoom);
            }
            if let Some(data)=self.data.clone(){
                if self.free_fall.update(data.experiment.spin,&self.cfg.view.free_fall,dt) {
                    self.renderer.update_free_fall(&self.free_fall.segments);
                }
                self.renderer.prepare(data.clone(),&self.cfg,self.selected);
                if self.fit_next{camera::fit(&mut self.cfg.camera,self.renderer.bounds.0,self.renderer.bounds.1,size.x/size.y);self.fit_next=false;}
                match self.detector.update(&data,&self.cfg,self.playback.time){Ok(true)=>self.renderer.update_detector(&self.detector),Ok(false)=>{},Err(e)=>self.status=format!("Detector display: {e}")}
                self.renderer.render([(size.x*ctx.pixels_per_point())as u32,(size.y*ctx.pixels_per_point())as u32],&self.cfg,self.playback.time,self.detector.hit_filter,self.selected);
            }
            let response=ui.add(egui::Image::new((self.renderer.texture_id,size)).sense(egui::Sense::click_and_drag()));
            // A fast press/move/release can all arrive in one egui frame. In that
            // case the widget's dragged flag need not survive to this point.
            // Preserve event order and the last pointer position for the camera.
            let mut camera_moved=false;
            ctx.input(|input| {
                for event in &input.events {
                    match event {
                        egui::Event::PointerButton{pos,button,pressed:true,..}
                            if response.rect.contains(*pos) && matches!(button,egui::PointerButton::Primary|egui::PointerButton::Secondary|egui::PointerButton::Middle) => {
                                self.camera_drag=Some((*button,*pos));
                            }
                        egui::Event::PointerMoved(pos) => {
                            if let Some((button,last))=self.camera_drag {
                                let delta=*pos-last;
                                if delta!=egui::Vec2::ZERO {
                                    if button!=egui::PointerButton::Primary || self.pan_drag {
                                        camera::pan(&mut self.cfg.camera,[delta.x,delta.y],size.y);
                                    } else {
                                        camera::orbit(&mut self.cfg.camera,[delta.x,delta.y]);
                                    }
                                    camera_moved=true;
                                }
                                self.camera_drag=Some((button,*pos));
                            }
                        }
                        egui::Event::PointerButton{button,pressed:false,..} => {
                            if self.camera_drag.is_some_and(|(active,_)|active==*button) {
                                self.camera_drag=None;
                            }
                        }
                        egui::Event::PointerGone|egui::Event::WindowFocused(false) => self.camera_drag=None,
                        _=>{}
                    }
                }
            });
            if response.hovered(){camera::zoom(&mut self.cfg.camera,ctx.input(|i|i.smooth_scroll_delta.y));}
            if !camera_moved&&response.clicked()&&let Some(pointer)=response.interact_pointer_pos()&&let Some(data)=&self.data{
                let matrix=camera::matrix(&self.cfg.camera,size.x/size.y);let mut closest=(100.0_f32,None);
                for &i in &self.renderer.rendered_ids{let ray=&data.rays[i];let k=crate::physics::kerr::Kerr::new(data.experiment.spin).unwrap();for s in &ray.samples{let xyz=crate::physics::coordinates::bl_to_cartesian(k,s.state[1],s.state[6],s.state[2]);let q=matrix*glam::Vec3::from_array(xyz.map(|v|v as f32)).extend(1.);if q.w>0.{let p=response.rect.min+egui::vec2((q.x/q.w+1.)*0.5*size.x,(1.-q.y/q.w)*0.5*size.y);let d=p.distance_sq(pointer);if d<closest.0{closest=(d,Some(ray.ray_id));}}}}
                self.selected=closest.1;
            }
        });
        ctx.request_repaint();
    }
}
pub fn write_json(path: &std::path::Path, value: &impl serde::Serialize) -> Result<(), String> {
    let file = std::fs::File::create_new(path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(file, value).map_err(|e| e.to_string())
}
pub fn write_png(path: &std::path::Path, size: [usize; 2], rgba: &[u8]) -> Result<(), String> {
    if size.contains(&0) {
        return Err("No detector image available".into());
    }
    let file = std::fs::File::create_new(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(file),
        size[0] as u32,
        size[1] as u32,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgba).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}
