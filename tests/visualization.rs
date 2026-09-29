//! UI-free regression tests for the simulation -> display boundary.
use kerr_ray::{
    experiments::source_detector::SourcePattern,
    physics::{
        coordinates::{bl_to_cartesian, from_cartesian},
        field::frame_dragging,
        kerr::Kerr,
    },
    session::{DetectorMode, SessionConfig, physics_key},
    simulation::{
        self, SimulationData, storage,
        worker::{Request, Worker},
    },
    visualization::{
        camera,
        detector_view::DetectorView,
        geometry,
        playback::{Playback, position_at},
    },
};
use std::{
    sync::{OnceLock, atomic::Ordering},
    time::Duration,
};
fn small() -> &'static SimulationData {
    static DATA: OnceLock<SimulationData> = OnceLock::new();
    DATA.get_or_init(|| {
        let mut c = SessionConfig::regression();
        c.experiment.source.pattern = SourcePattern::RectangularGrid { nu: 4, nv: 4 };
        simulation::calculate(&c.experiment, |_, _| true).unwrap()
    })
}
fn config() -> SessionConfig {
    let mut c = SessionConfig::regression();
    c.experiment = small().experiment.clone();
    c
}
fn temporary(label: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "kerr-viz-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(!p.exists());
    p
}

#[test]
fn defaults_separate_physical_and_display_counts() {
    let c = SessionConfig::default();
    assert_eq!(c.generator.as_ref().unwrap().count().unwrap(), 36_864);
    assert_eq!(c.view.max_rendered_trajectories, 256);
    assert_eq!(
        SessionConfig::regression()
            .experiment
            .source
            .emissions()
            .unwrap()
            .len(),
        256
    );
}
#[test]
fn display_postprocess_and_camera_do_not_change_physics_key() {
    let a = config();
    let mut b = a.clone();
    b.camera.yaw += 1.;
    b.view.max_rendered_trajectories = 512;
    b.view.grid_spacing = 3.;
    b.experiment.postprocess.time_bin_width = 0.1;
    b.experiment.detector.resolution = [64, 64];
    assert_eq!(physics_key(&a.experiment), physics_key(&b.experiment));
    b.experiment.spin += 0.1;
    assert_ne!(physics_key(&a.experiment), physics_key(&b.experiment));
}
#[test]
fn shared_json_roundtrip_preserves_simulation() {
    let c = config();
    let p = temporary("json");
    std::fs::write(&p, serde_json::to_vec(&c).unwrap()).unwrap();
    let d = SessionConfig::load(&p).unwrap();
    assert_eq!(physics_key(&c.experiment), physics_key(&d.experiment));
    std::fs::remove_file(p).unwrap();
}
#[test]
fn source_orientation_uses_existing_right_handed_plane() {
    let mut c = config();
    c.experiment.source.plane = kerr_ray::detector::plane::Plane::from_normal_up(
        [80., 4., 5.],
        [1., 0.2, 0.3],
        [0., 0., 1.],
        20.,
        30.,
    )
    .unwrap();
    let p = &c.experiment.source.plane;
    p.validate().unwrap();
    for corner in geometry::plane_corners(p) {
        assert!(p.signed_distance(corner.to_array().map(f64::from)).abs() < 1e-5);
    }
    assert!(c.experiment.initials().is_ok());
}
#[test]
fn morton_subset_covers_all_source_quadrants() {
    let data = small();
    let s = geometry::subset(data, 4, None);
    assert_eq!(s.len(), 4);
    let quadrants: std::collections::HashSet<_> = s
        .iter()
        .map(|&i| {
            (
                data.rays[i].source_uv[0] > 0.,
                data.rays[i].source_uv[1] > 0.,
            )
        })
        .collect();
    assert_eq!(quadrants.len(), 4);
}
#[test]
fn display_subset_never_mutates_events_or_counts() {
    let d = small();
    let before = serde_json::to_vec(&d.events).unwrap();
    let counts = d.counts();
    let _ = geometry::ray_segments(d, &geometry::subset(d, 3, Some(0)));
    let _ = geometry::ray_segments(d, &geometry::subset(d, 12, None));
    assert_eq!(before, serde_json::to_vec(&d.events).unwrap());
    assert_eq!(counts, d.counts());
}
#[test]
fn selected_ray_is_included_by_identity_not_array_index() {
    let mut data = small().clone();
    data.rays.reverse();
    let indices = geometry::subset(&data, 2, Some(0));
    assert!(indices.iter().any(|&i| data.rays[i].ray_id == 0));
    let segments = geometry::ray_segments(&data, &indices);
    assert!(segments.iter().all(|s| {
        indices
            .iter()
            .any(|&i| data.rays[i].ray_id == s.meta[1] as usize)
    }));
}
#[test]
fn grid_lines_are_straight_coordinate_references() {
    for s in geometry::grid(10., 2.) {
        let changed = (0..3).filter(|&i| s.a[i] != s.b[i]).count();
        assert_eq!(changed, 1);
        assert_eq!(s.meta[0], 0);
    }
}
#[test]
fn auto_bounds_include_planes_center_and_selected_paths() {
    let d = small();
    let ids = geometry::subset(d, 8, None);
    let (lo, hi) = geometry::bounds(d, &ids);
    for p in [&d.experiment.source.plane, &d.experiment.detector.plane] {
        for v in geometry::plane_corners(p) {
            assert!(v.cmpge(lo).all() && v.cmple(hi).all());
        }
    }
    assert!(glam::Vec3::ZERO.cmpge(lo).all() && glam::Vec3::ZERO.cmple(hi).all());
}
#[test]
fn kerr_surfaces_follow_exact_spin_dependent_radii() {
    for spin in [0., 0.6, 0.99] {
        let mut d = small().clone();
        d.experiment.spin = spin;
        let k = Kerr::new(spin).unwrap();
        let (lines, surface) = geometry::static_geometry(&d, 100., 20.);
        for v in surface {
            let p = v.position.map(f64::from);
            let h = k.horizon();
            let level = (p[0] * p[0] + p[1] * p[1]) / (h * h + spin * spin) + p[2] * p[2] / (h * h);
            assert!((level - 1.).abs() < 4e-7);
        }
        for s in lines.iter().filter(|s| s.meta[0] == 4) {
            for p in [s.a, s.b] {
                let p = p.map(f64::from);
                let b = p[0] * p[0] + p[1] * p[1] + p[2] * p[2] - spin * spin;
                let r = (0.5 * (b + b.hypot(2. * spin * p[2]))).sqrt();
                let theta = (p[2] / r).clamp(-1., 1.).acos();
                assert!((r - k.ergosphere(theta)).abs() < 5e-7);
            }
        }
    }
}
#[test]
fn cartesian_conversion_is_existing_oblate_bl_convention() {
    let k = Kerr::new(0.8).unwrap();
    let xyz = bl_to_cartesian(k, 5., std::f64::consts::FRAC_PI_2, 0.);
    assert!((xyz[0] - (25.64_f64).sqrt()).abs() < 1e-13);
    let q = from_cartesian(k, xyz).unwrap();
    assert!((q[0] - 5.).abs() < 1e-13);
}
#[test]
fn frame_dragging_is_finite_positive_and_zero_in_schwarzschild() {
    for spin in [0., 0.6, 0.99] {
        let k = Kerr::new(spin).unwrap();
        for r in [k.horizon() + 0.01, 3., 10., 100.] {
            let w = frame_dragging(k, r, 1.1).unwrap();
            assert!(w.is_finite() && w >= 0.);
            if spin == 0. {
                assert_eq!(w, 0.);
            }
        }
        let (v, m) = geometry::field_segments(k, &config().view);
        assert!(m.is_finite());
        assert!(
            v.iter()
                .all(|s| s.a.iter().chain(&s.b).all(|x| x.is_finite()))
        );
    }
}
#[test]
fn camera_changes_are_finite_and_fit_visible_bounds() {
    let mut c = config().camera;
    let (a, b) = geometry::bounds(small(), &[0, 1]);
    camera::fit(&mut c, a, b, 1.3);
    let before = camera::eye(&c);
    camera::orbit(&mut c, [20., -10.]);
    camera::pan(&mut c, [5., 4.], 800.);
    camera::zoom(&mut c, 100.);
    assert_ne!(camera::eye(&c), before);
    assert!(camera::matrix(&c, 1.3).is_finite());
}
#[test]
fn playback_uses_physical_time_and_stops_at_end() {
    let mut p = Playback {
        time: 10.,
        playing: true,
        range: [10., 15.],
    };
    p.tick(0.1, 2.);
    assert!((p.time - 10.2).abs() < 1e-13);
    p.tick(0.25, 100.);
    assert_eq!(p.time, 15.);
    assert!(!p.playing);
    p.reset();
    assert_eq!(p.time, 10.);
}
#[test]
fn interpolation_matches_saved_endpoints_without_extrapolation() {
    let d = small();
    let r = &d.rays[0];
    let k = Kerr::new(d.experiment.spin).unwrap();
    let s = &r.samples[0];
    assert_eq!(
        position_at(r, k, s.state[0]).unwrap(),
        bl_to_cartesian(k, s.state[1], s.state[6], s.state[2])
    );
    assert!(position_at(r, k, s.state[0] - 1.).is_none());
    let last = r.samples.last().unwrap();
    assert!(position_at(r, k, last.state[0] + 1.).is_none());
}
#[test]
fn all_ray_detector_histograms_equal_reference_bins() {
    let d = small();
    let mut c = config();
    let bins = c.experiment.postprocess.resolve(&d.events).unwrap();
    let expected =
        kerr_ray::detector::binning::bin(&d.events, &d.experiment.detector, bins).unwrap();
    let mut v = DetectorView::default();
    v.update(d, &c, 0.).unwrap();
    assert_eq!(v.counts, expected.accumulated);
    assert_eq!(v.total, d.events.len() as u64);
    c.view.detector_mode = DetectorMode::Instantaneous;
    let mut sum = 0;
    for i in 0..expected.by_bin.len() {
        v.update(d, &c, bins.start + (i as f64 + 0.5) * bins.width)
            .unwrap();
        assert_eq!(v.displayed, expected.by_bin[i].len() as u64);
        sum += v.displayed;
    }
    assert_eq!(sum, expected.window_count);
    c.view.detector_mode = DetectorMode::Cumulative;
    v.update(d, &c, d.time_range()[1]).unwrap();
    assert_eq!(v.counts, expected.accumulated);
}
#[test]
fn legacy_load_preserves_events_but_never_invents_paths() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("results/source_to_detector");
    let d = storage::load(&path).unwrap();
    assert_eq!(d.rays.len(), 256);
    assert_eq!(d.counts(), [0, 216, 20, 20, 0]);
    assert!(
        d.rays
            .iter()
            .all(|r| r.samples.is_empty() && r.final_state.is_none())
    );
    assert!(d.trajectory_note.contains("absent"));
    assert_eq!(
        d.diagnostic_csv,
        std::fs::read_to_string(path.join("ray_diagnostics.csv")).unwrap()
    );
}
#[test]
fn new_run_roundtrip_preserves_all_f64_samples_and_diagnostics() {
    let d = small();
    let c = config();
    let p = temporary("roundtrip");
    storage::save(d, &c, &p).unwrap();
    let loaded = storage::load(&p).unwrap();
    assert_eq!(d.events, loaded.events);
    assert_eq!(d.diagnostic_csv, loaded.diagnostic_csv);
    assert_eq!(d.hit_state_csv, loaded.hit_state_csv);
    for (a, b) in d.rays.iter().zip(&loaded.rays) {
        assert_eq!(a.samples.len(), b.samples.len());
        for (x, y) in a.samples.iter().zip(&b.samples) {
            assert_eq!(x.state, y.state);
            assert_eq!(x.affine, y.affine);
        }
    }
    assert!(storage::save(d, &c, &p).is_err());
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
fn trajectory_export_stride_keeps_endpoints_and_full_physics() {
    let d = small();
    let mut c = config();
    c.trajectory_export_stride = 7;
    let p = temporary("stride");
    storage::save(d, &c, &p).unwrap();
    let loaded = storage::load(&p).unwrap();
    assert_eq!(d.events, loaded.events);
    assert_eq!(d.counts(), loaded.counts());
    for (a, b) in d.rays.iter().zip(&loaded.rays) {
        assert!(b.samples.len() <= a.samples.len());
        assert_eq!(a.samples[0].state, b.samples[0].state);
        assert_eq!(
            a.samples.last().unwrap().state,
            b.samples.last().unwrap().state
        );
        assert_eq!(a.original_sample_count, b.original_sample_count);
    }
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
fn generation_worker_delivers_only_latest_request() {
    let w = Worker::default();
    let mut old = config();
    old.experiment.source.pattern = SourcePattern::RectangularGrid { nu: 64, nv: 64 };
    w.submit(Request::Calculate(Box::new(old)));
    let mut new = config();
    new.experiment.spin = 0.2;
    let id = w.submit(Request::Calculate(Box::new(new)));
    let result = w.receiver.recv_timeout(Duration::from_secs(30)).unwrap();
    assert_eq!(result.generation, id);
    let d = result.result.unwrap();
    assert_eq!(d.experiment.spin, 0.2);
    assert_eq!(d.rays.len(), 16);
    assert_eq!(w.generation.load(Ordering::Acquire), id);
}
#[test]
fn cancellation_stops_between_independent_rays() {
    let c = config();
    let mut calls = 0;
    let result = simulation::calculate(&c.experiment, |_, _| {
        calls += 1;
        calls < 3
    });
    assert!(result.unwrap_err().contains("cancelled"));
    assert_eq!(calls, 3);
}
#[test]
fn duplicate_identity_is_rejected_in_saved_scene() {
    let mut d = small().clone();
    d.rays[1].ray_id = d.rays[0].ray_id;
    assert!(d.validate().unwrap_err().contains("duplicate"));
}
