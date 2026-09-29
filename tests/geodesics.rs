use kerr_ray::{
    compute::{RayIntegrator, RayStatus, StopReason, cpu::CpuReferenceIntegrator},
    config::IntegratorConfig,
    physics::{
        geodesic::{State, inverse_radial_derivative, rhs},
        kerr::Kerr,
        metric::Metric,
        tetrad::{equatorial_ray, ray_with_impact, zamo_frame},
        validation::{critical_schwarzschild_impact, photon_radius, radial_potential},
    },
};
use std::f64::consts::{FRAC_PI_2, PI};

#[test]
fn analytic_derivative_matches_independent_five_point_difference() {
    let mut worst: f64 = 0.0;
    for a in [0.0, 0.5, 0.99] {
        let k = Kerr::new(a).unwrap();
        for r in [k.horizon() + 0.3, 4.0, 20.0, 200.0] {
            let h = 1e-4 * r;
            let m2 = k.inverse(r - 2.0 * h, FRAC_PI_2).unwrap();
            let m1 = k.inverse(r - h, FRAC_PI_2).unwrap();
            let p1 = k.inverse(r + h, FRAC_PI_2).unwrap();
            let p2 = k.inverse(r + 2.0 * h, FRAC_PI_2).unwrap();
            let analytic = inverse_radial_derivative(k, r).unwrap();
            for i in 0..4 {
                for j in 0..4 {
                    let numeric =
                        (m2[i][j] - 8.0 * m1[i][j] + 8.0 * p1[i][j] - p2[i][j]) / (12.0 * h);
                    worst = worst
                        .max((numeric - analytic[i][j]).abs() / analytic[i][j].abs().max(1e-8));
                }
            }
        }
    }
    println!("analytic derivative scaled residual={worst:e}, limit=2e-7");
    assert!(worst < 2e-7);
}

#[test]
fn zamo_orthonormal_and_null_including_ergoregion() {
    let mut worst: f64 = 0.0;
    for a in [0.0, 0.7, 0.99] {
        let k = Kerr::new(a).unwrap();
        for r in [k.horizon() + 0.05, 5.0, 100.0] {
            for th in [0.3, FRAC_PI_2, 2.4] {
                let g = k.covariant(r, th).unwrap();
                let e = zamo_frame(k, r, th).unwrap();
                for i in 0..4 {
                    for j in 0..4 {
                        let mut dot = 0.0;
                        for m in 0..4 {
                            for n in 0..4 {
                                dot += g[m][n] * e[m][i] * e[n][j];
                            }
                        }
                        let expected = if i != j {
                            0.0
                        } else if i == 0 {
                            -1.0
                        } else {
                            1.0
                        };
                        worst = worst.max((dot - expected).abs());
                    }
                }
            }
            for angle in [0.0, 0.7, FRAC_PI_2, PI, -FRAC_PI_2] {
                let s = equatorial_ray(k, r, 0.0, angle, 1.0).unwrap();
                assert!(s.null_constraint(k).unwrap().abs() < 2e-12);
                assert!(rhs(k, &s.0).unwrap()[0] > 0.0);
            }
        }
    }
    println!("ZAMO orthonormal residual={worst:e}, limit=2e-12");
    assert!(worst < 2e-12);
}

#[test]
fn circular_photon_orbits_are_hamiltonian_equilibria_and_stay_circular() {
    let mut worst_r: f64 = 0.0;
    let mut worst_c: f64 = 0.0;
    for a in [0.0, 0.5, 0.9, 0.99] {
        let k = Kerr::new(a).unwrap();
        for pro in [true, false] {
            let r = photon_radius(a, pro).unwrap();
            let s =
                equatorial_ray(k, r, 0.0, if pro { FRAC_PI_2 } else { -FRAC_PI_2 }, 1.0).unwrap();
            let v = rhs(k, &s.0).unwrap();
            assert!(
                v[1].abs() < 1e-13 && v[4].abs() < 2e-11,
                "a={a}, r={r}, dp_r={}",
                v[4]
            );
            let cfg = IntegratorConfig {
                max_affine: 10.0,
                ..Default::default()
            };
            let result = CpuReferenceIntegrator.integrate(k, s, &cfg);
            assert_eq!(
                result.status,
                RayStatus::Active,
                "{:?}",
                result.failure_message
            );
            for sample in &result.samples {
                worst_r = worst_r.max((sample.state.radius() - r).abs());
            }
            worst_c = worst_c.max(result.diagnostics.max_null_abs);
        }
    }
    println!("circular orbit max |r-r_ph|={worst_r:e} limit=2e-7; max null={worst_c:e}");
    assert!(worst_r < 2e-7 && worst_c < 1e-9);
    for pro in [true, false] {
        assert!((photon_radius(0.0, pro).unwrap() - 3.0).abs() < 1e-14);
    }
    assert!((photon_radius(1.0, true).unwrap() - 1.0).abs() < 1e-14);
    assert!((photon_radius(1.0, false).unwrap() - 4.0).abs() < 1e-14);
    assert!(photon_radius(1.0 - 1e-8, true).unwrap() < 1.0002);
    assert!(photon_radius(1.0 - 1e-8, false).unwrap() > 3.9999);
}

#[test]
fn schwarzschild_critical_capture_boundary_and_reflection_symmetry() {
    let k = Kerr::new(0.0).unwrap();
    let bc = critical_schwarzschild_impact();
    let cfg = IntegratorConfig {
        escape_radius: 50.0,
        ..Default::default()
    };
    let mut worst_c: f64 = 0.0;
    for (b, status) in [
        (bc - 1e-3, RayStatus::Captured),
        (bc + 1e-3, RayStatus::Escaped),
    ] {
        let mut pair = Vec::new();
        for sign in [1.0, -1.0] {
            let s = ray_with_impact(k, 50.0, 0.0, sign * b, true).unwrap();
            assert!((s.angular_momentum() / s.energy() - sign * b).abs() < 1e-13);
            let out = CpuReferenceIntegrator.integrate(k, s, &cfg);
            assert_eq!(out.status, status, "{:?}", out.failure_message);
            let d = &out.diagnostics;
            worst_c = worst_c.max(d.max_null_abs);
            assert_eq!(d.max_energy_relative, 0.0);
            assert_eq!(d.max_lz_relative, 0.0);
            assert_eq!(d.max_q_abs, 0.0);
            if status == RayStatus::Captured {
                let r = out.samples.last().unwrap().state.radius();
                assert!(r > k.horizon() && r <= k.horizon() + cfg.horizon_epsilon);
                assert!((r - k.horizon() - cfg.horizon_epsilon).abs() < 3e-11);
            }
            pair.push(out);
        }
        let p = pair[0].samples.last().unwrap().state;
        let m = pair[1].samples.last().unwrap().state;
        assert!((p.radius() - m.radius()).abs() < 1e-9);
        assert!((p.phi() + m.phi()).abs() < 1e-7);
    }
    println!("critical b bracket +/-1e-3 around {bc:.15}; max null={worst_c:e}, limit=1e-5");
    assert!(worst_c < 1e-5);
}

#[test]
fn independent_radial_potential_and_tolerance_convergence() {
    let k = Kerr::new(0.7).unwrap();
    let s = ray_with_impact(k, 40.0, 0.0, 6.0, true).unwrap();
    let solve = |tol| {
        CpuReferenceIntegrator.integrate(
            k,
            s,
            &IntegratorConfig {
                rtol: tol,
                atol: tol * 0.01,
                max_step: 8.0,
                escape_radius: 40.0,
                ..Default::default()
            },
        )
    };
    let coarse = solve(1e-6);
    let fine = solve(1e-9);
    let reference = solve(1e-12);
    for out in [&coarse, &fine, &reference] {
        assert_eq!(out.status, RayStatus::Escaped, "{:?}", out.failure_message);
    }
    let phi = |o: &kerr_ray::compute::PhysicsResult| o.samples.last().unwrap().state.phi();
    let ec = (phi(&coarse) - phi(&reference)).abs();
    let ef = (phi(&fine) - phi(&reference)).abs();
    println!("convergence final phi error: rtol1e-6={ec:e}; rtol1e-9={ef:e}; reference rtol1e-12");
    assert!(ef < 2e-7 && ef < ec / 20.0);
    let mut worst: f64 = 0.0;
    for sample in &reference.samples {
        let s = sample.state;
        let dr = rhs(k, &s.0).unwrap()[1];
        let potential = radial_potential(k, s);
        let kinetic = s.radius().powi(4) * dr * dr;
        // Scale by r^4 E^2, including at radial turning points where R~0.
        worst = worst.max((kinetic - potential).abs() / (s.radius().powi(4) * s.energy().powi(2)));
    }
    println!("independent radial potential normalized residual={worst:e}, limit=2e-10");
    assert!(worst < 2e-10);
}

#[test]
fn far_field_approaches_straight_coordinate_line() {
    for a in [0.0, 0.99] {
        let k = Kerr::new(a).unwrap();
        let r = 1e6;
        let s = equatorial_ray(k, r, 0.0, PI - 0.3, 1.0).unwrap();
        let v = rhs(k, &s.0).unwrap();
        let cfg = IntegratorConfig {
            escape_radius: 2e6,
            max_affine: 1000.0,
            max_step: 25.0,
            ..Default::default()
        };
        let out = CpuReferenceIntegrator.integrate(k, s, &cfg);
        assert_eq!(out.status, RayStatus::Active);
        let end = out.samples.last().unwrap();
        let t = end.affine;
        let dx = end.state.radius() * end.state.phi().cos() - (r + v[1] * t);
        let dy = end.state.radius() * end.state.phi().sin() - (r * v[2] * t);
        let relative = dx.hypot(dy) / t;
        println!("far-field a={a}: straight-line displacement/affine={relative:e}, limit=2e-8");
        assert!(relative < 2e-8);
    }
}

#[test]
fn weak_schwarzschild_deflection_matches_four_over_b() {
    let k = Kerr::new(0.0).unwrap();
    let b = 1000.0;
    let s = ray_with_impact(k, 1e6, 0.0, b, true).unwrap();
    let cfg = IntegratorConfig {
        escape_radius: 1e6,
        max_affine: 3e6,
        max_step: 20_000.0,
        ..Default::default()
    };
    let out = CpuReferenceIntegrator.integrate(k, s, &cfg);
    assert_eq!(out.status, RayStatus::Escaped, "{:?}", out.failure_message);
    let heading = |s: State| {
        let v = rhs(k, &s.0).unwrap();
        s.phi() + (s.radius() * v[2]).atan2(v[1])
    };
    let angle = (heading(out.samples.last().unwrap().state) - heading(s))
        .sin()
        .atan2((heading(out.samples.last().unwrap().state) - heading(s)).cos())
        .abs();
    let relative = (angle / (4.0 / b) - 1.0).abs();
    println!(
        "weak deflection={angle:e}, 4/b={}, relative correction={relative:e}, limit=0.006",
        4.0 / b
    );
    assert!(relative < 0.006);
}

#[test]
fn failures_are_not_escape_and_affine_limit_is_active() {
    let k = Kerr::new(0.5).unwrap();
    let s = equatorial_ray(k, 10.0, 0.0, PI, 1.0).unwrap();
    let out = CpuReferenceIntegrator.integrate(
        k,
        s,
        &IntegratorConfig {
            max_steps: 1,
            ..Default::default()
        },
    );
    assert_eq!(out.status, RayStatus::NumericalFailure);
    assert_eq!(out.stop_reason, StopReason::StepLimit);
    let mut bad = s;
    bad.0[4] = 0.0;
    assert_eq!(
        CpuReferenceIntegrator
            .integrate(k, bad, &IntegratorConfig::default())
            .stop_reason,
        StopReason::NullViolation
    );
    bad.0[1] = f64::NAN;
    assert_eq!(
        CpuReferenceIntegrator
            .integrate(k, bad, &IntegratorConfig::default())
            .status,
        RayStatus::NumericalFailure
    );
    let out = CpuReferenceIntegrator.integrate(
        k,
        s,
        &IntegratorConfig {
            max_affine: 0.1,
            ..Default::default()
        },
    );
    assert_eq!(out.status, RayStatus::Active);
    assert_eq!(out.stop_reason, StopReason::AffineLimit);
    assert!((out.samples.last().unwrap().affine - 0.1).abs() < 1e-15);
}

#[test]
fn frame_dragging_is_from_metric_with_zero_lz() {
    let k = Kerr::new(0.8).unwrap();
    let mut s = equatorial_ray(k, 8.0, 0.0, 0.0, 1.0).unwrap();
    // Local radial emission has Lz=0 to tetrad roundoff.
    assert!(s.angular_momentum().abs() < 1e-15);
    s.0[5] = 0.0;
    let v = rhs(k, &s.0).unwrap();
    let omega = 2.0 * k.spin() * s.radius() / k.area_function(s.radius(), FRAC_PI_2);
    assert!((v[2] / v[0] - omega).abs() < 1e-15 && v[2] > 0.0);
    let cfg = IntegratorConfig {
        max_affine: 5.0,
        ..Default::default()
    };
    let out = CpuReferenceIntegrator.integrate(k, s, &cfg);
    assert_eq!(out.diagnostics.max_lz_relative, 0.0);
    assert!(out.samples.last().unwrap().state.phi() > 0.0);
}
