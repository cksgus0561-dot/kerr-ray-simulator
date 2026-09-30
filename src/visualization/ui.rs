//! UI edits shared settings; it never calls the geodesic integrator.
use super::i18n::{Language, collapsing, header, numeric};
use crate::{
    detector::plane::Plane,
    experiments::source_detector::{LaunchDirection, SourcePattern},
    session::{DetectorMode, PlaybackMode, SessionConfig, VISIBILITY_NAMES},
};
use eframe::egui::{self, Ui};

fn vector(ui: &mut Ui, lang: Language, label: &str, value: &mut [f64; 3]) -> bool {
    let mut changed = false;
    ui.label(lang.text(label));
    ui.horizontal_wrapped(|ui| {
        for v in value {
            changed |=
                numeric(ui, lang, egui::DragValue::new(v).speed(0.1).max_decimals(4)).changed();
        }
    });
    changed
}
fn plane(ui: &mut Ui, lang: Language, p: &mut Plane) {
    vector(ui, lang, "Center (coordinate M)", &mut p.center);
    let mut normal = p.normal;
    let mut up = p.e_v;
    let changed = vector(ui, lang, "Normal (XYZ)", &mut normal)
        | vector(ui, lang, "Local v / up (XYZ)", &mut up);
    if changed {
        match Plane::from_normal_up(p.center, normal, up, p.width, p.height) {
            Ok(next) => *p = next,
            Err(e) => {
                ui.colored_label(egui::Color32::LIGHT_RED, lang.diagnostic(e.as_str()));
            }
        }
    }
    ui.horizontal_wrapped(|ui| {
        ui.label(lang.text("Width / height"));
        numeric(
            ui,
            lang,
            egui::DragValue::new(&mut p.width)
                .range(0.01..=10000.)
                .speed(0.5),
        );
        numeric(
            ui,
            lang,
            egui::DragValue::new(&mut p.height)
                .range(0.01..=10000.)
                .speed(0.5),
        );
    });
}
pub fn settings(ui: &mut Ui, cfg: &mut SessionConfig, lang: Language) {
    ui.heading(lang.text("Scene"));
    header(ui, lang, "Visibility")
        .default_open(false)
        .show(ui, |ui| {
            for (value, label) in cfg.view.visibility.iter_mut().zip(VISIBILITY_NAMES) {
                ui.checkbox(value, lang.text(label));
            }
            ui.checkbox(
                &mut cfg.view.detector_overlay,
                lang.text("Detector world overlay"),
            );
            ui.checkbox(
                &mut cfg.view.detector_panel,
                lang.text("Detector image panel"),
            );
        });
    header(ui, lang, "Display settings (no new physics)").default_open(true).show(ui,|ui|{
        ui.label(lang.text("Rendered trajectories (physical rays unchanged)"));
        numeric(ui, lang, egui::DragValue::new(&mut cfg.view.max_rendered_trajectories).range(1..=65536).speed(8));
        ui.label(lang.text("Line px"));
        numeric(ui, lang, egui::Slider::new(&mut cfg.view.trajectory_thickness,0.5..=5.));
        ui.checkbox(&mut cfg.view.grid_auto,lang.text("Grid auto-fit"));
        ui.add_enabled_ui(!cfg.view.grid_auto,|ui|{
            numeric(ui, lang, egui::DragValue::new(&mut cfg.view.grid_extent).range(1.0..=10000.).prefix(lang.text("Extent ")));
            numeric(ui, lang, egui::DragValue::new(&mut cfg.view.grid_spacing).range(0.1..=10000.).prefix(lang.text("Spacing ")));
            ui.small(lang.text("At most 40 grid intervals/axis; effective spacing shown below."));
        });
        ui.label(lang.text("Field samples/axis"));
        numeric(ui, lang, egui::Slider::new(&mut cfg.view.frame_dragging_density,2..=35));
        ui.checkbox(&mut cfg.view.field_3d,lang.text("3D field (otherwise z=0 slice)"));
        numeric(ui, lang, egui::DragValue::new(&mut cfg.view.field_extent).range(2.1..=1000.).prefix(lang.text("Field extent ")));
        numeric(ui, lang, egui::DragValue::new(&mut cfg.view.field_time_scale).range(0.1..=1000.).prefix(lang.text("Arrow time scale M ")));
        ui.small(lang.text("Arrows: omega * (-Y,X,0) * display interval. Coordinate angular velocity, not a force."));
    });
    header(ui, lang, "Simulation (background recomputation)").show(ui, |ui| {
        let e = &mut cfg.experiment;
        ui.label("chi = a/M");
        numeric(ui, lang, egui::Slider::new(&mut e.spin, -crate::physics::constants::MAX_SPIN..=crate::physics::constants::MAX_SPIN));
        if let Some(g)=&mut cfg.generator {
            collapsing(ui, lang, "Stratified ray generator",|ui| {
                vector(ui, lang,"Launch region center (M)",&mut g.center);
                vector(ui, lang,"Unit axis 1 (XYZ)",&mut g.axis_1);
                vector(ui, lang,"Unit axis 2 (XYZ)",&mut g.axis_2);
                ui.horizontal_wrapped(|ui| {ui.label(lang.text("Cell count 1 / 2"));for n in &mut g.cell_count {numeric(ui, lang, egui::DragValue::new(n).range(1..=256));}});
                ui.horizontal_wrapped(|ui| {for n in [16,32,64,128,256] {if ui.button(lang.text(&format!("{n} x {n}"))).clicked() {g.cell_count=[n,n];}}});
                ui.horizontal_wrapped(|ui| {ui.label(lang.text("Cell size (M)"));for d in &mut g.cell_size {numeric(ui, lang, egui::DragValue::new(d).range(0.001..=100.).speed(0.01));}});
                ui.label(lang.text(&format!("Extent {:?} M; one uniform point per cell",g.extent())));
                vector(ui, lang,"Fixed CartesianSpatial direction (no noise)",&mut g.fixed_direction);
                numeric(ui, lang, egui::DragValue::new(&mut g.t_emit).prefix(lang.text("t_emit ")));
                ui.small(lang.text("New calculation: fresh OS 256-bit seed; cell size stays fixed when count changes."));
                if let Err(error)=g.validate() {ui.colored_label(egui::Color32::LIGHT_RED,lang.diagnostic(error.as_str()));}
            });
        } else { collapsing(ui, lang, "SourcePlane (legacy preset)", |ui| {
            plane(ui, lang, &mut e.source.plane);
            if let SourcePattern::RectangularGrid { nu, nv } = &mut e.source.pattern {
                ui.horizontal_wrapped(|ui| {
                    ui.label(lang.text("Ray grid X / Y"));
                    numeric(ui, lang, egui::DragValue::new(nu).range(1..=256));
                    numeric(ui, lang, egui::DragValue::new(nv).range(1..=256));
                });
                ui.horizontal_wrapped(|ui| {
                    for n in [16, 32, 64, 128, 256] {
                        if ui.button(lang.text(&format!("{n} x {n}"))).clicked() {
                            *nu = n;
                            *nv = n;
                        }
                    }
                });
            } else if ui.button(lang.text("Use rectangular launch grid")).clicked() {
                e.source.pattern = SourcePattern::RectangularGrid { nu: 64, nv: 64 };
            }
            ui.small(lang.text("16x16 is the preserved 256-ray regression preset."));
            let mut local = matches!(e.source.direction, LaunchDirection::LocalZamo(_));
            if ui
                .checkbox(&mut local,lang.text("Direction in local ZAMO basis"))
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
                    vector(ui, lang, "(radial, polar, azimuthal)", n);
                }
                LaunchDirection::CartesianSpatial(n) => {
                    vector(ui, lang, "Coordinate spatial direction (XYZ)", n);
                }
            }
            numeric(ui, lang, egui::DragValue::new(&mut e.source.t_emit)
                    .speed(0.1)
                    .prefix(lang.text("t_emit ")));
        });
        }
        collapsing(ui, lang, "DetectorPlane", |ui| {
            plane(ui, lang, &mut e.detector.plane);
        });
        collapsing(ui, lang, "Reference accuracy", |ui| {
            numeric(ui, lang, egui::DragValue::new(&mut e.integration.rtol)
                    .range(1e-13..=1e-3)
                    .speed(1e-11)
                    .prefix(lang.text("rtol ")));
            numeric(ui, lang, egui::DragValue::new(&mut e.integration.atol)
                    .range(1e-15..=1e-3)
                    .speed(1e-13)
                    .prefix(lang.text("atol ")));
            numeric(ui, lang, egui::DragValue::new(&mut e.integration.escape_radius)
                    .range(3.0..=100000.)
                    .prefix(lang.text("r_escape ")));
        });
    });
    ui.separator();
    ui.heading(lang.text("Detector / playback"));
    egui::ComboBox::from_id_salt("playback-mode")
        .selected_text(lang.text(&format!("{:?}", cfg.view.playback_mode)))
        .show_ui(ui, |ui| {
            for (v, label) in [
                (PlaybackMode::Static, "Full trajectories"),
                (PlaybackMode::Propagation, "Ray propagation"),
                (PlaybackMode::Detector, "Detector playback"),
            ] {
                ui.selectable_value(&mut cfg.view.playback_mode, v, lang.text(label));
            }
        });
    egui::ComboBox::from_id_salt("detector-mode")
        .selected_text(lang.text(&format!("{:?}", cfg.view.detector_mode)))
        .show_ui(ui, |ui| {
            for (v, label) in [
                (DetectorMode::Accumulated, "All accumulated hits"),
                (DetectorMode::Instantaneous, "Instantaneous time bin"),
                (DetectorMode::Cumulative, "Cumulative to current time"),
            ] {
                ui.selectable_value(&mut cfg.view.detector_mode, v, lang.text(label));
            }
        });
    numeric(
        ui,
        lang,
        egui::DragValue::new(&mut cfg.view.playback_speed)
            .range(0.01..=10000.)
            .prefix(lang.text("Physical M / wall s ")),
    );
    numeric(
        ui,
        lang,
        egui::DragValue::new(&mut cfg.experiment.postprocess.time_bin_width)
            .range(0.0001..=10000.)
            .speed(0.1)
            .prefix(lang.text("Delta t ")),
    );
    numeric(
        ui,
        lang,
        egui::DragValue::new(&mut cfg.experiment.postprocess.counts_per_white)
            .range(1..=100000)
            .prefix(lang.text("Counts / white ")),
    );
    ui.horizontal_wrapped(|ui| {
        ui.label(lang.text("Pixels X / Y"));
        for n in &mut cfg.experiment.detector.resolution {
            numeric(ui, lang, egui::DragValue::new(n).range(1..=1024));
        }
    });
    ui.small(lang.text("Time controls and detector binning reuse every stored event."));
}
