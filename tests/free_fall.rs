use kerr_ray::{
    physics::{
        field::frame_dragging,
        geodesic::{State, rhs},
        kerr::Kerr,
        metric::{Metric, contract, multiply_vector},
    },
    session::{FreeFallConfig, SessionConfig, physics_key},
    visualization::{
        free_fall::{Cache, DURATION, HORIZON_MARGIN, advect, flow},
        free_fall_ui::FreeFall,
    },
};

#[test]
fn rain_velocity_has_unit_timelike_norm_energy_one_zero_lz_and_inward_branch() {
    let mut maximum: f64 = 0.;
    for chi in [0., 0.6, 0.99] {
        let k = Kerr::new(chi).unwrap();
        for r in [k.horizon() + HORIZON_MARGIN + 1e-6, 3., 10., 100.] {
            for th in [0.1, 0.7, std::f64::consts::FRAC_PI_2, 2.4, 3.04] {
                let f = flow(k, [r, th, 0.3]).unwrap();
                let g = k.covariant(r, th).unwrap();
                let p = multiply_vector(&g, &f.velocity);
                maximum = maximum.max((contract(&g, &f.velocity) + 1.).abs());
                assert!((p[0] + 1.).abs() < 2e-11);
                assert!(p[3].abs() < 2e-11 && p[2].abs() < 1e-13);
                assert!(f.momentum[1] < 0. && f.velocity[0] > 0. && f.coordinate_velocity[0] < 0.);
                assert_eq!(f.spatial_diagonal, [g[1][1], g[2][2], g[3][3]]);
                assert!(f.spatial_diagonal.iter().all(|x| x.is_finite() && *x > 0.));
                assert!(
                    (f.coordinate_velocity[2] - frame_dragging(k, r, th).unwrap()).abs() < 1e-13
                );
            }
        }
    }
    println!("rain metric normalization max error {maximum:e}, limit 2e-10");
    assert!(maximum < 2e-10);
}

#[test]
fn schwarzschild_rain_falls_radially_and_kerr_angular_velocity_is_not_doubled() {
    for r in [2.1_f64, 3., 10., 100.] {
        let zero = flow(Kerr::new(0.).unwrap(), [r, 1.1, 0.]).unwrap();
        let expected = -(1. - 2. / r) * (2. / r).sqrt();
        assert!((zero.coordinate_velocity[0] - expected).abs() < 1e-14);
        assert_eq!(zero.coordinate_velocity[1], 0.);
        assert_eq!(zero.coordinate_velocity[2], 0.);
        let k = Kerr::new(0.6).unwrap();
        let spinning = flow(k, [r, 1.1, 0.]).unwrap();
        let omega = frame_dragging(k, r, 1.1).unwrap();
        assert!(omega > 0.);
        assert!((spinning.coordinate_velocity[2] / omega - 1.).abs() < 1e-13);
    }
}

#[test]
fn rain_field_satisfies_existing_hamilton_equations_for_massive_geodesics() {
    // The canonical Hamiltonian RHS is mass independent; only H=-1/2 differs.
    // Independently check dp/dtau along the field, not just pointwise normalization.
    for chi in [0., 0.6, 0.99] {
        let k = Kerr::new(chi).unwrap();
        for r in [3., 5., 15.] {
            for th in [0.4, 1.2, 2.3] {
                let f = flow(k, [r, th, 0.2]).unwrap();
                let s = State::from_bl([0., r, th, 0.2], f.momentum);
                let h = 1e-5;
                let dpdr = (flow(k, [r + h, th, 0.2]).unwrap().momentum[1]
                    - flow(k, [r - h, th, 0.2]).unwrap().momentum[1])
                    / (2. * h);
                let dy = rhs(k, &s.0).unwrap();
                assert!((dy[4] - dpdr * f.velocity[1]).abs() < 2e-8);
                assert!(dy[7].abs() < 1e-11);
                assert_eq!(dy[3], 0.);
                assert_eq!(dy[5], 0.);
            }
        }
    }
}

#[test]
fn advection_converges_and_stops_outside_the_bl_chart_boundary() {
    let k = Kerr::new(0.6).unwrap();
    let solve = |step: f64| {
        let mut q = [3.2, 1.1, 0.];
        for _ in 0..(6. / step).round() as usize {
            q = advect(k, q, step).unwrap().unwrap();
        }
        q
    };
    let coarse = solve(0.2);
    let fine = solve(0.1);
    let reference = solve(0.025);
    let error = |q: [f64; 3]| {
        q.iter()
            .zip(reference)
            .map(|(a, b)| (a - b).abs())
            .fold(0., f64::max)
    };
    println!(
        "rain RK4 step .2/.1 errors: {:e} / {:e}",
        error(coarse),
        error(fine)
    );
    assert!(error(fine) < error(coarse) / 8. && error(fine) < 1e-7);
    let k = Kerr::new(0.).unwrap();
    assert!(flow(k, [k.horizon() + HORIZON_MARGIN, 1., 0.]).is_err());
    assert!(flow(k, [5., 0., 0.]).is_err());
    assert!(flow(k, [f64::NAN, 1., 0.]).is_err());
    assert!(advect(k, [2.02001, 1., 0.], 10.).unwrap().is_none());
}

#[test]
fn cached_lattice_keeps_topology_real_flow_and_finite_interpolated_vertices() {
    let cfg = FreeFallConfig {
        density: 5,
        extent: 8.,
        ..Default::default()
    };
    for chi in [0., 0.6] {
        let c = Cache::build(chi, &cfg, || true).unwrap();
        assert_eq!(c.paths.len(), 125);
        assert_eq!(c.edges.len(), 300);
        assert!(c.excluded_initial > 0);
        let mut moved = 0;
        for (i, path) in c.paths.iter().enumerate() {
            for pair in path.windows(2) {
                assert!(pair[1][0] < pair[0][0]);
                assert_eq!(pair[1][1], pair[0][1]);
                if chi == 0. {
                    assert_eq!(pair[1][2], pair[0][2]);
                } else {
                    assert!(pair[1][2] > pair[0][2]);
                }
            }
            if path.len() > 2 {
                moved += 1;
            }
            for t in [0., 0.125, 10.125, 50., DURATION] {
                if let Some(q) = c.coordinate(i, t) {
                    assert!(q[0] > c.kerr.horizon() + HORIZON_MARGIN);
                    assert!(c.position(i, t).unwrap().iter().all(|v| v.is_finite()));
                }
            }
            if path.len() < 321 {
                assert!(c.position(i, DURATION).is_none());
            }
        }
        assert!(moved > 80);
        // A half-output-step interpolation must agree with direct finer advection.
        let initial = c.paths[0][0];
        let mut q = initial;
        for _ in 0..5 {
            q = advect(c.kerr, q, 0.025).unwrap().unwrap();
        }
        let interpolated = c.coordinate(0, 0.125).unwrap();
        let err = q
            .iter()
            .zip(interpolated)
            .map(|(a, b)| (a - b).abs())
            .fold(0., f64::max);
        assert!(err < 2e-5, "interpolation error {err}");
    }
}

#[test]
fn free_fall_settings_are_display_only_and_old_json_keeps_defaults() {
    let a = SessionConfig::regression();
    let key = physics_key(&a.experiment);
    let mut b = a.clone();
    b.view.free_fall = FreeFallConfig {
        visible: true,
        extent: 8.,
        density: 7,
        speed: 12.,
    };
    b.view.visibility[0] = false;
    b.view.visibility[10] = false;
    assert_eq!(physics_key(&b.experiment), key);
    b.validate_view().unwrap();
    let mut old = serde_json::to_value(a).unwrap();
    old["view"].as_object_mut().unwrap().remove("free_fall");
    let old: SessionConfig = serde_json::from_value(old).unwrap();
    assert!(!old.view.free_fall.visible);
    assert_eq!(old.view.free_fall.density, 9);
    assert!(Cache::build(0., &b.view.free_fall, || false).is_err());
}

#[test]
fn independent_cache_clock_reset_and_latest_generation_do_not_recalculate_rays() {
    let mut grid = FreeFall::default();
    let mut cfg = FreeFallConfig {
        visible: true,
        density: 3,
        ..Default::default()
    };
    grid.update(0., &cfg, 0.);
    grid.update(0.6, &cfg, 0.); // supersede first request
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while grid.cache.is_none() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
        grid.update(0.6, &cfg, 0.);
    }
    assert_eq!(grid.cache.as_ref().unwrap().kerr.spin(), 0.6);
    assert_eq!(grid.builds, 2);
    let initial: Vec<_> = grid.segments.iter().map(|s| (s.a, s.b)).collect();
    grid.playing = true;
    assert!(grid.update(0.6, &cfg, 0.1));
    assert!(grid.time > 0.);
    grid.playing = false;
    let paused = grid.time;
    assert!(!grid.update(0.6, &cfg, 0.1));
    assert_eq!(grid.time, paused);
    cfg.speed = 10.;
    cfg.visible = false;
    grid.update(0.6, &cfg, 0.1);
    cfg.visible = true;
    grid.update(0.6, &cfg, 0.);
    grid.reset();
    grid.update(0.6, &cfg, 0.);
    assert_eq!(grid.time, 0.);
    assert_eq!(grid.builds, 2);
    assert_eq!(
        initial,
        grid.segments.iter().map(|s| (s.a, s.b)).collect::<Vec<_>>()
    );
}
