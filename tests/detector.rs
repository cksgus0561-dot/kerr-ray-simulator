use kerr_ray::{
    compute::{RayStatus, cpu::CpuReferenceIntegrator},
    config::IntegratorConfig,
    detector::{
        binning::{TimeBasis, TimeBins, bin},
        events::HitEvent,
        plane::{DetectorPlane, Plane},
    },
    physics::{
        coordinates::{cartesian_to_local, from_cartesian},
        kerr::Kerr,
        tetrad::ray_3d,
    },
};
fn plane(x: f64) -> DetectorPlane {
    DetectorPlane {
        plane: Plane::from_normal_up(
            [x, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            200.0,
            200.0,
        )
        .unwrap(),
        resolution: [16, 16],
        root_tolerance: 1e-10,
    }
}
fn launch(k: Kerr, x: [f64; 3], d: [f64; 3]) -> kerr_ray::physics::geodesic::State {
    let q = from_cartesian(k, x).unwrap();
    ray_3d(
        k,
        [0.0, q[0], q[1], q[2]],
        cartesian_to_local(k, q, d).unwrap(),
        1.0,
    )
    .unwrap()
}

#[test]
fn plane_basis_rectangle_edges_and_pixels() {
    let mut d = plane(30.0);
    d.validate(Kerr::new(0.8).unwrap()).unwrap();
    assert_eq!(d.pixel([-100.0, 100.0]), Some([0, 0]));
    assert_eq!(d.pixel([100.0, -100.0]), Some([15, 15]));
    assert!(d.pixel([100.0 + 1e-10, 0.0]).is_none());
    assert!(d.pixel([0.0, -100.0 - 1e-10]).is_none());
    assert_eq!(d.plane.uv(d.plane.position(12.0, -3.0)), [12.0, -3.0]);
    d.plane.e_u = [1.0, 0.0, 0.0];
    assert!(d.plane.validate().is_err());
    assert!(plane(1.0).validate(Kerr::new(0.0).unwrap()).is_err());
}

#[test]
fn far_field_oblique_hit_matches_flat_limit_position_and_time() {
    let k = Kerr::new(0.6).unwrap();
    let r = 1e8;
    let mut d = plane(r + 100.0);
    d.root_tolerance = 5e-8;
    let s = launch(k, [r, 1.5, -2.0], [1.0, 0.1, 0.2]);
    let cfg = IntegratorConfig {
        escape_radius: 2e8,
        max_affine: 200.0,
        max_step: 20.0,
        ..Default::default()
    };
    let out = CpuReferenceIntegrator.integrate_with_detector(k, s, &cfg, &d);
    assert_eq!(out.status, RayStatus::Detected, "{:?}", out.failure_message);
    let hit = out.detection.unwrap();
    let eu = (hit.uv[0] - 11.5).abs();
    let ev = (hit.uv[1] - 18.0).abs();
    let et = (hit.state.time() - 100.0 * 1.05_f64.sqrt()).abs();
    println!(
        "far plane |du|={eu:e}, |dv|={ev:e}, |dt|={et:e}; limits=2e-6,2e-6,1e-5 (finite M/r included)"
    );
    assert!(eu < 2e-6 && ev < 2e-6 && et < 1e-5);
    assert_eq!(out.samples.last().unwrap().state, hit.state);
}

#[test]
fn radial_schwarzschild_hit_time_matches_exact_logarithmic_delay() {
    let k = Kerr::new(0.0).unwrap();
    let d = plane(60.0);
    let s = launch(k, [30.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    let out =
        CpuReferenceIntegrator.integrate_with_detector(k, s, &IntegratorConfig::default(), &d);
    assert_eq!(out.status, RayStatus::Detected);
    let hit = out.detection.unwrap();
    let expected = 30.0 + 2.0 * (58.0_f64 / 28.0).ln();
    let error = (hit.state.time() - expected).abs();
    println!("exact Schwarzschild radial t_hit error={error:e}, limit=2e-9");
    assert!(error < 2e-9);
}

#[test]
fn detector_hit_position_and_time_converge() {
    let k = Kerr::new(0.8).unwrap();
    let d = plane(-50.0);
    let s = launch(k, [50.0, 10.0, 5.0], [-1.0, 0.0, 0.0]);
    let solve = |rtol, step| {
        let cfg = IntegratorConfig {
            rtol,
            atol: rtol * 0.01,
            max_step: step,
            escape_radius: 300.0,
            ..Default::default()
        };
        let out = CpuReferenceIntegrator.integrate_with_detector(k, s, &cfg, &d);
        assert_eq!(out.status, RayStatus::Detected, "{:?}", out.failure_message);
        out.detection.unwrap()
    };
    let a = solve(1e-7, 4.0);
    let b = solve(1e-10, 1.0);
    let reference = solve(1e-13, 0.25);
    let errors = |h: kerr_ray::detector::intersection::Intersection| {
        [
            (h.uv[0] - reference.uv[0]).abs(),
            (h.uv[1] - reference.uv[1]).abs(),
            (h.state.time() - reference.state.time()).abs(),
        ]
    };
    let coarse = errors(a);
    let fine = errors(b);
    println!(
        "detector convergence coarse du,dv,dt={coarse:?}; fine={fine:?}; fine limits=2e-7 each"
    );
    for i in 0..3 {
        assert!(fine[i] < 2e-7 && fine[i] < coarse[i] / 5.0);
    }
}

#[test]
fn outside_plane_crossing_does_not_absorb_and_start_on_plane_is_immediate() {
    let k = Kerr::new(0.0).unwrap();
    let mut d = plane(50.0);
    d.plane.width = 1.0;
    d.plane.height = 1.0;
    let s = launch(k, [30.0, 5.0, 2.0], [1.0, 0.0, 0.0]);
    let out =
        CpuReferenceIntegrator.integrate_with_detector(k, s, &IntegratorConfig::default(), &d);
    assert_eq!(out.status, RayStatus::Escaped);
    assert!(out.detection.is_none());
    let s = launch(k, [50.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    let out =
        CpuReferenceIntegrator.integrate_with_detector(k, s, &IntegratorConfig::default(), &d);
    assert_eq!(out.status, RayStatus::Detected);
    assert_eq!(out.detection.unwrap().affine, 0.0);
}

fn event(id: usize, t: f64) -> HitEvent {
    HitEvent {
        ray_id: id,
        u_source: 0.0,
        v_source: 0.0,
        t_emit: 0.0,
        u_hit: 0.0,
        v_hit: 0.0,
        t_hit: t,
        delta_t: t,
        status: RayStatus::Detected,
    }
}
#[test]
fn time_binning_exact_edges_rebinning_and_count_conservation() {
    let d = plane(30.0);
    let t = TimeBins {
        width: 0.1,
        start: 0.0,
        end: 1.0,
        basis: TimeBasis::Arrival,
    };
    for k in 0..10 {
        assert_eq!(t.index(k as f64 * 0.1), Some(k));
    }
    assert_eq!(t.index(1.0), None);
    assert_eq!(t.index(0.3), Some(3));
    assert_eq!(t.index(0.3 - 1e-12), Some(2));
    let events = vec![
        event(0, 0.0),
        event(1, 0.1),
        event(2, 0.1 * 3.0),
        event(3, 0.95),
    ];
    let b = bin(&events, &d, t).unwrap();
    assert_eq!(b.by_bin[0].len(), 1);
    assert_eq!(b.by_bin[1].len(), 1);
    assert_eq!(b.by_bin[3].len(), 1);
    assert_eq!(b.accumulated.iter().sum::<u64>(), 4);
    assert_eq!(b.by_bin.iter().map(Vec::len).sum::<usize>(), 4);
    let rebinned = bin(&events, &d, TimeBins { width: 0.5, ..t }).unwrap();
    assert_eq!(rebinned.by_bin[0].len(), 3);
    assert_eq!(rebinned.by_bin[1].len(), 1);
    let crop = bin(
        &events,
        &d,
        TimeBins {
            start: 0.2,
            end: 0.8,
            ..t
        },
    )
    .unwrap();
    assert_eq!(crop.before_window.iter().sum::<u64>(), 2);
    assert_eq!(crop.window_count, 1);
    assert_eq!(crop.after_window, 1);
    let mut cumulative = crop.before_window.clone();
    for frame in &crop.by_bin {
        for index in frame {
            cumulative[*index] += 1;
        }
    }
    assert_eq!(cumulative.iter().sum::<u64>(), 3);
}

#[test]
fn time_basis_invalid_events_and_outside_pixels() {
    let d = plane(30.0);
    let t = TimeBins {
        width: 1.0,
        start: 0.0,
        end: 5.0,
        basis: TimeBasis::Travel,
    };
    let mut e = event(0, 12.0);
    e.t_emit = 10.0;
    e.delta_t = 2.0;
    let b = bin(&[e.clone()], &d, t).unwrap();
    assert_eq!(b.by_bin[2].len(), 1);
    e.u_hit = 101.0;
    assert_eq!(bin(&[e.clone()], &d, t).unwrap().outside_detector, 1);
    assert!(bin(&[e.clone(), e.clone()], &d, t).is_err());
    e.t_hit = f64::NAN;
    assert!(bin(&[e], &d, t).is_err());
    assert!(TimeBins { width: 0.0, ..t }.count().is_err());
}
