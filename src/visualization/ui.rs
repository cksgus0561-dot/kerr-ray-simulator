//! UI edits shared settings; it never calls the geodesic integrator.
use crate::{
    detector::plane::Plane,
    experiments::source_detector::{LaunchDirection, SourcePattern},
    session::{DetectorMode, PlaybackMode, SessionConfig, VISIBILITY_NAMES},
};
use eframe::egui::{self, Ui};

fn vector(ui: &mut Ui, label: &str, value: &mut [f64; 3]) -> bool {
    let mut changed = false;
    ui.label(label);
    ui.horizontal(|ui| {
        for v in value {
            changed |= ui
                .add(egui::DragValue::new(v).speed(0.1).max_decimals(4))
                .changed();
        }
    });
    changed
}
fn plane(ui: &mut Ui, p: &mut Plane) {
    vector(ui, "Center (coordinate M)", &mut p.center);
    let mut normal = p.normal;
    let mut up = p.e_v;
    let changed =
        vector(ui, "Normal (XYZ)", &mut normal) | vector(ui, "Local v / up (XYZ)", &mut up);
    if changed {
        match Plane::from_normal_up(p.center, normal, up, p.width, p.height) {
            Ok(next) => *p = next,
            Err(e) => {
                ui.colored_label(egui::Color32::LIGHT_RED, e);
            }
        }
    }
    ui.horizontal(|ui| {
        ui.label("Width / height");
        ui.add(
            egui::DragValue::new(&mut p.width)
                .range(0.01..=10000.)
                .speed(0.5),
        );
        ui.add(
            egui::DragValue::new(&mut p.height)
                .range(0.01..=10000.)
                .speed(0.5),
        );
    });
}
pub fn settings(ui: &mut Ui, cfg: &mut SessionConfig) {
    ui.heading("Scene");
    egui::CollapsingHeader::new("Visibility")
        .default_open(false)
        .show(ui, |ui| {
            for (value, label) in cfg.view.visibility.iter_mut().zip(VISIBILITY_NAMES) {
                ui.checkbox(value, label);
            }
            ui.checkbox(&mut cfg.view.detector_overlay, "Detector world overlay");
            ui.checkbox(&mut cfg.view.detector_panel, "Detector image panel");
        });
    egui::CollapsingHeader::new("Display settings (no new physics)").default_open(true).show(ui,|ui|{
        ui.label("Rendered trajectories (physical rays unchanged)");
        ui.add(egui::DragValue::new(&mut cfg.view.max_rendered_trajectories).range(1..=65536).speed(8));
        ui.add(egui::Slider::new(&mut cfg.view.trajectory_thickness,0.5..=5.).text("Line px"));
        ui.checkbox(&mut cfg.view.grid_auto,"Grid auto-fit");
        ui.add_enabled_ui(!cfg.view.grid_auto,|ui|{
            ui.add(egui::DragValue::new(&mut cfg.view.grid_extent).range(1.0..=10000.).prefix("Extent "));
            ui.add(egui::DragValue::new(&mut cfg.view.grid_spacing).range(0.1..=10000.).prefix("Spacing "));
            ui.small("At most 40 grid intervals/axis; effective spacing shown below.");
        });
        ui.add(egui::Slider::new(&mut cfg.view.frame_dragging_density,2..=35).text("Field samples/axis"));
        ui.checkbox(&mut cfg.view.field_3d,"3D field (otherwise z=0 slice)");
        ui.add(egui::DragValue::new(&mut cfg.view.field_extent).range(2.1..=1000.).prefix("Field extent "));
        ui.add(egui::DragValue::new(&mut cfg.view.field_time_scale).range(0.1..=1000.).prefix("Arrow time scale M "));
        ui.small("Arrows: omega * (-Y,X,0) * display interval. Coordinate angular velocity, not a force.");
    });
    egui::CollapsingHeader::new("Simulation (background recomputation)").show(ui, |ui| {
        let e = &mut cfg.experiment;
        ui.add(egui::Slider::new(&mut e.spin, -crate::physics::constants::MAX_SPIN..=crate::physics::constants::MAX_SPIN).text("chi = a/M"));
        if let Some(g)=&mut cfg.generator {
            ui.collapsing("Stratified ray generator",|ui| {
                vector(ui,"Launch region center (M)",&mut g.center);
                vector(ui,"Unit axis 1 (XYZ)",&mut g.axis_1);
                vector(ui,"Unit axis 2 (XYZ)",&mut g.axis_2);
                ui.horizontal(|ui| {ui.label("Cell count 1 / 2");for n in &mut g.cell_count {ui.add(egui::DragValue::new(n).range(1..=256));}});
                ui.horizontal_wrapped(|ui| {for n in [16,32,64,128,256] {if ui.button(format!("{n} x {n}")).clicked() {g.cell_count=[n,n];}}});
                ui.horizontal(|ui| {ui.label("Cell size (M)");for d in &mut g.cell_size {ui.add(egui::DragValue::new(d).range(0.001..=100.).speed(0.01));}});
                ui.label(format!("Extent {:?} M; one uniform point per cell",g.extent()));
                vector(ui,"Fixed CartesianSpatial direction (no noise)",&mut g.fixed_direction);
                ui.add(egui::DragValue::new(&mut g.t_emit).prefix("t_emit "));
                ui.small("New calculation: fresh OS 256-bit seed; cell size stays fixed when count changes.");
                if let Err(error)=g.validate() {ui.colored_label(egui::Color32::LIGHT_RED,error);}
            });
        } else { ui.collapsing("SourcePlane (legacy preset)", |ui| {
            plane(ui, &mut e.source.plane);
            if let SourcePattern::RectangularGrid { nu, nv } = &mut e.source.pattern {
                ui.horizontal(|ui| {
                    ui.label("Ray grid X / Y");
                    ui.add(egui::DragValue::new(nu).range(1..=256));
                    ui.add(egui::DragValue::new(nv).range(1..=256));
                });
                ui.horizontal_wrapped(|ui| {
                    for n in [16, 32, 64, 128, 256] {
                        if ui.button(format!("{n} x {n}")).clicked() {
                            *nu = n;
                            *nv = n;
                        }
                    }
                });
            } else if ui.button("Use rectangular launch grid").clicked() {
                e.source.pattern = SourcePattern::RectangularGrid { nu: 64, nv: 64 };
            }
            ui.small("16x16 is the preserved 256-ray regression preset.");
            let mut local = matches!(e.source.direction, LaunchDirection::LocalZamo(_));
            if ui
                .checkbox(&mut local, "Direction in local ZAMO basis")
                .changed()
            {
                e.source.direction = if local {
                    LaunchDirection::LocalZamo([-1., 0., 0.])
                } else {
                    LaunchDirection::CartesianSpatial([-1., 0., 0.])
                };
            }
            match &mut e.source.direction {
                LaunchDirection::LocalZamo(n) => {
                    vector(ui, "(radial, polar, azimuthal)", n);
                }
                LaunchDirection::CartesianSpatial(n) => {
                    vector(ui, "Coordinate spatial direction (XYZ)", n);
                }
            }
            ui.add(
                egui::DragValue::new(&mut e.source.t_emit)
                    .speed(0.1)
                    .prefix("t_emit "),
            );
        });
        }
        ui.collapsing("DetectorPlane", |ui| {
            plane(ui, &mut e.detector.plane);
        });
        ui.collapsing("Reference accuracy", |ui| {
            ui.add(
                egui::DragValue::new(&mut e.integration.rtol)
                    .range(1e-13..=1e-3)
                    .speed(1e-11)
                    .prefix("rtol "),
            );
            ui.add(
                egui::DragValue::new(&mut e.integration.atol)
                    .range(1e-15..=1e-3)
                    .speed(1e-13)
                    .prefix("atol "),
            );
            ui.add(
                egui::DragValue::new(&mut e.integration.escape_radius)
                    .range(3.0..=100000.)
                    .prefix("r_escape "),
            );
        });
    });
    ui.separator();
    ui.heading("Detector / playback");
    egui::ComboBox::from_id_salt("playback-mode")
        .selected_text(format!("{:?}", cfg.view.playback_mode))
        .show_ui(ui, |ui| {
            for (v, label) in [
                (PlaybackMode::Static, "Full trajectories"),
                (PlaybackMode::Propagation, "Ray propagation"),
                (PlaybackMode::Detector, "Detector playback"),
            ] {
                ui.selectable_value(&mut cfg.view.playback_mode, v, label);
            }
        });
    egui::ComboBox::from_id_salt("detector-mode")
        .selected_text(format!("{:?}", cfg.view.detector_mode))
        .show_ui(ui, |ui| {
            for (v, label) in [
                (DetectorMode::Accumulated, "All accumulated hits"),
                (DetectorMode::Instantaneous, "Instantaneous time bin"),
                (DetectorMode::Cumulative, "Cumulative to current time"),
            ] {
                ui.selectable_value(&mut cfg.view.detector_mode, v, label);
            }
        });
    ui.add(
        egui::DragValue::new(&mut cfg.view.playback_speed)
            .range(0.01..=10000.)
            .prefix("Physical M / wall s "),
    );
    ui.add(
        egui::DragValue::new(&mut cfg.experiment.postprocess.time_bin_width)
            .range(0.0001..=10000.)
            .speed(0.1)
            .prefix("Delta t "),
    );
    ui.add(
        egui::DragValue::new(&mut cfg.experiment.postprocess.counts_per_white)
            .range(1..=100000)
            .prefix("Counts / white "),
    );
    ui.horizontal(|ui| {
        ui.label("Pixels X / Y");
        for n in &mut cfg.experiment.detector.resolution {
            ui.add(egui::DragValue::new(n).range(1..=1024));
        }
    });
    ui.small("Time controls and detector binning reuse every stored event.");
}
