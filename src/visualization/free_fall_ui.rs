//! Independent visualization worker/cache/clock. Never submits a ray calculation.
use super::{
    free_fall::{Cache, DURATION, flow},
    geometry::Segment,
};
use crate::session::FreeFallConfig;
use eframe::egui;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
    mpsc,
};

type Key = (u64, u64, usize);
type Job = (u64, f64, FreeFallConfig);
pub struct FreeFall {
    sender: mpsc::Sender<Job>,
    receiver: mpsc::Receiver<(u64, Result<Cache, String>)>,
    generation: Arc<AtomicU64>,
    requested: Option<Key>,
    pub cache: Option<Cache>,
    pub time: f64,
    pub playing: bool,
    pub builds: u64,
    pending: bool,
    error: Option<String>,
    dirty: bool,
    positions: Vec<Option<[f64; 3]>>,
    pub segments: Vec<Segment>,
}
impl Default for FreeFall {
    fn default() -> Self {
        let (sender, jobs) = mpsc::channel::<Job>();
        let (finish, receiver) = mpsc::channel();
        let generation = Arc::new(AtomicU64::new(0));
        let g = generation.clone();
        std::thread::Builder::new()
            .name("kerr-free-fall-cache".into())
            .spawn(move || {
                while let Ok(mut job) = jobs.recv() {
                    while let Ok(newer) = jobs.try_recv() {
                        job = newer;
                    }
                    let (id, chi, cfg) = job;
                    let result = Cache::build(chi, &cfg, || g.load(Ordering::Acquire) == id);
                    if g.load(Ordering::Acquire) == id {
                        let _ = finish.send((id, result));
                    }
                }
            })
            .expect("spawn visualization cache worker");
        Self {
            sender,
            receiver,
            generation,
            requested: None,
            cache: None,
            time: 0.,
            playing: false,
            builds: 0,
            pending: false,
            error: None,
            dirty: true,
            positions: vec![],
            segments: vec![],
        }
    }
}
impl Drop for FreeFall {
    fn drop(&mut self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
    }
}
impl FreeFall {
    pub fn reset(&mut self) {
        self.time = 0.;
        self.playing = false;
        self.dirty = true;
    }
    /// Only spin/initial extent/density invalidate the precomputed flow.
    pub fn update(&mut self, chi: f64, cfg: &FreeFallConfig, wall_dt: f64) -> bool {
        let key = (chi.to_bits(), cfg.extent.to_bits(), cfg.density);
        if cfg.visible && self.requested != Some(key) {
            let id = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
            self.requested = Some(key);
            self.builds += 1;
            self.reset();
            self.cache = None;
            self.error = None;
            self.pending = true;
            if self.sender.send((id, chi, cfg.clone())).is_err() {
                self.pending = false;
                self.error = Some("free-fall worker disconnected".into());
            }
        }
        while let Ok((id, result)) = self.receiver.try_recv() {
            if id != self.generation.load(Ordering::Acquire) {
                continue;
            }
            self.pending = false;
            match result {
                Ok(cache) => {
                    self.cache = Some(cache);
                    self.dirty = true;
                }
                Err(e) => self.error = Some(e),
            }
        }
        if cfg.visible
            && self.playing
            && self.cache.is_some()
            && wall_dt.is_finite()
            && wall_dt > 0.
        {
            self.time = (self.time + wall_dt.min(0.25) * cfg.speed).min(DURATION);
            if self.time >= DURATION {
                self.playing = false;
            }
            self.dirty = true;
        }
        if !self.dirty {
            return false;
        }
        self.dirty = false;
        self.segments.clear();
        if let Some(cache) = &self.cache {
            self.positions.resize(cache.paths.len(), None);
            for (i, p) in self.positions.iter_mut().enumerate() {
                *p = cache.position(i, self.time);
            }
            let color = [0.45, 1.0, 0.55, 0.55];
            for &[i, j] in &cache.edges {
                if let (Some(a), Some(b)) = (self.positions[i], self.positions[j]) {
                    self.segments.push(Segment::line(
                        super::geometry::vector(a),
                        super::geometry::vector(b),
                        color,
                        12,
                        u32::MAX,
                    ));
                }
            }
            for p in self.positions.iter().flatten() {
                let p = super::geometry::vector(*p);
                self.segments.push(Segment::line(p, p, color, 12, u32::MAX));
            }
        }
        true
    }
    pub fn show(&mut self, ui: &mut egui::Ui, view: &mut crate::session::ViewConfig) {
        egui::CollapsingHeader::new("Kerr Free-Fall Grid (visualization only)").default_open(true).show(ui,|ui| {
            ui.checkbox(&mut view.visibility[0],"Cartesian Reference Grid");
            ui.checkbox(&mut view.visibility[10],"Frame Dragging");
            let cfg = &mut view.free_fall;
            ui.checkbox(&mut cfg.visible,"Kerr Free-Fall Grid");
            ui.small("E=1, Lz=0 rain observers; connected markers, not moving space or a Euclidean embedding.");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut cfg.extent).range(3.0..=100.0).speed(0.2).prefix("Extent "));
                ui.add(egui::DragValue::new(&mut cfg.density).range(3..=17).prefix("Nodes/axis "));
            });
            ui.add(egui::DragValue::new(&mut cfg.speed).range(0.01..=100.).speed(0.1).prefix("Grid M / wall s "));
            ui.horizontal(|ui| {
                if ui.add_enabled(self.cache.is_some(),egui::Button::new(if self.playing {"Pause grid"} else {"Play grid"})).clicked() {
                    if self.time>=DURATION {self.reset();}
                    self.playing = !self.playing;
                }
                if ui.button("Reset grid").clicked() {self.reset();}
            });
            ui.label(format!("Grid t_BL/M {:.3} / {DURATION}; cache builds {}",self.time,self.builds));
            if self.pending {ui.label("Precomputing free-fall cache (not ray physics)...");}
            if let Some(e)=&self.error {ui.colored_label(egui::Color32::LIGHT_RED,e);}
            if let Some(c)=&self.cache {
                ui.small(format!("chi {}; visible nodes {}/{}; initial exclusions {}; cache {:.3}s",
                    c.kerr.spin(),self.positions.iter().flatten().count(),c.paths.len(),c.excluded_initial,c.seconds));
                ui.collapsing("Free-fall metric diagnostics",|ui| {
                    ui.label(format!("max sampled |g(u,u)+1| {:.3e}",c.max_normalization_error));
                    // One explicitly identified probe; not a second metric implementation.
                    let probe=c.paths.iter().enumerate().filter_map(|(i,p)|p.first().map(|q|(i,q[0])))
                        .min_by(|a,b|a.1.total_cmp(&b.1)).map(|p|p.0);
                    if let Some(i)=probe && let Some(q)=c.coordinate(i,self.time) && let Ok(f)=flow(c.kerr,q) {
                        ui.label(format!("Probe {i}: r {:.5}, theta {:.5}, phi {:.5}",q[0],q[1],q[2]));
                        ui.label(format!("dr/dt {:.5e}; dphi/dt {:.5e}",f.coordinate_velocity[0],f.coordinate_velocity[2]));
                        ui.label(format!("t_BL slice gamma_rr/theta/phi: {:.5?}",f.spatial_diagonal));
                    } else {ui.label("Initial probe removed at numerical horizon margin.");}
                    ui.small("BL margin 0.02 M; axis excluded. Grid clock is independent of photon playback; hidden grid pauses. Reset restores ALL nodes.");
                });
            }
        });
    }
}
