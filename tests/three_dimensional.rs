use kerr_ray::{
    compute::{RayIntegrator, RayStatus, cpu::CpuReferenceIntegrator},
    config::IntegratorConfig,
    physics::{
        coordinates::{cartesian_to_local, from_cartesian, to_cartesian, unit},
        geodesic::{State, inverse_derivatives},
        kerr::Kerr,
        metric::Metric,
        tetrad::ray_3d,
    },
};

#[test]
fn general_metric_derivatives_match_five_point_differences() {
    let mut worst: f64 = 0.0;
    for a in [0.0, 0.6, 0.99] {
        let k = Kerr::new(a).unwrap();
        for (r, theta) in [
            (k.horizon() + 0.4, 0.3),
            (3.0, 1.1),
            (12.0, 2.3),
            (100.0, 2.9),
        ] {
            let ds = inverse_derivatives(k, r, theta).unwrap();
            for (mu, d) in ds.iter().enumerate() {
                let h = if mu == 0 { r * 1e-4 } else { 1e-4 };
                let m = |offset: f64| {
                    k.inverse(
                        r + if mu == 0 { offset * h } else { 0.0 },
                        theta + if mu == 1 { offset * h } else { 0.0 },
                    )
                    .unwrap()
                };
                let (m2, m1, p1, p2) = (m(-2.0), m(-1.0), m(1.0), m(2.0));
                for i in 0..4 {
                    for j in 0..4 {
                        let fd =
                            (m2[i][j] - 8.0 * m1[i][j] + 8.0 * p1[i][j] - p2[i][j]) / (12.0 * h);
                        // Mixed absolute/relative tolerance also covers exactly zero
                        // derivatives: subtracting identical f64 values has roundoff.
                        worst = worst.max((d[i][j] - fd).abs() / (3e-10 + 3e-7 * d[i][j].abs()));
                    }
                }
            }
        }
    }
    println!("3D derivative error/(3e-10+3e-7*abs(analytic))={worst:e}, limit=1");
    assert!(worst < 1.0);
}

#[test]
fn general_rays_conserve_null_energy_lz_and_carter_q() {
    let mut max_c: f64 = 0.0;
    let mut max_q: f64 = 0.0;
    let mut max_qrel: f64 = 0.0;
    for a in [0.0, 0.6, 0.99] {
        let k = Kerr::new(a).unwrap();
        for theta in [0.8, 1.3, 2.1] {
            for sign in [-1.0, 1.0] {
                let s = ray_3d(
                    k,
                    [0.0, 30.0, theta, 0.2],
                    unit([-1.0, sign * 0.15, 0.22]).unwrap(),
                    1.0,
                )
                .unwrap();
                assert!(s.null_constraint(k).unwrap().abs() < 1e-13);
                let out = CpuReferenceIntegrator.integrate(k, s, &IntegratorConfig::default());
                assert_eq!(out.status, RayStatus::Escaped, "{:?}", out.failure_message);
                assert!(
                    out.samples
                        .iter()
                        .any(|p| (p.state.theta() - theta).abs() > 0.01)
                );
                assert_eq!(out.diagnostics.max_energy_relative, 0.0);
                assert_eq!(out.diagnostics.max_lz_relative, 0.0);
                max_c = max_c.max(out.diagnostics.max_null_abs);
                max_q = max_q.max(out.diagnostics.max_q_abs);
                max_qrel = max_qrel.max(out.diagnostics.max_q_relative);
            }
        }
    }
    println!(
        "3D 18 rays: max_null={max_c:e}, max_E_relative=0, max_Lz_relative=0, max_Q_abs={max_q:e}, max_Q_relative={max_qrel:e}"
    );
    assert!(max_c < 1e-9 && max_q < 1e-7 && max_qrel < 1e-9);
}

#[test]
fn schwarzschild_rotated_initial_conditions_have_rotated_outcomes() {
    let k = Kerr::new(0.0).unwrap();
    let angle = 0.57_f64;
    let rotate = |v: [f64; 3]| {
        [
            v[0] * angle.cos() + v[2] * angle.sin(),
            v[1],
            -v[0] * angle.sin() + v[2] * angle.cos(),
        ]
    };
    let x = [40.0, 0.0, 0.0];
    let d = [-1.0, 0.25, 0.13];
    let launch = |x, d| {
        let q = from_cartesian(k, x).unwrap();
        ray_3d(
            k,
            [0.0, q[0], q[1], q[2]],
            cartesian_to_local(k, q, d).unwrap(),
            1.0,
        )
        .unwrap()
    };
    let cfg = IntegratorConfig {
        escape_radius: 70.0,
        ..Default::default()
    };
    let s = launch(x, d);
    let sr = launch(rotate(x), rotate(d));
    let one = CpuReferenceIntegrator.integrate(k, s, &cfg);
    let two = CpuReferenceIntegrator.integrate(k, sr, &cfg);
    assert_eq!(one.status, RayStatus::Escaped);
    assert_eq!(two.status, RayStatus::Escaped);
    let end = one.samples.last().unwrap().state;
    let rotated = two.samples.last().unwrap().state;
    let expected = rotate(to_cartesian(k, end));
    let actual = to_cartesian(k, rotated);
    let dx = expected
        .iter()
        .zip(actual)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    let dt = (end.time() - rotated.time()).abs();
    let l2 = |s: State| s.carter_q(k) + s.angular_momentum().powi(2);
    println!("Schwarzschild rotation max Cartesian error={dx:e}, time error={dt:e}");
    assert!(dx < 1e-7 && dt < 1e-7 && (l2(s) - l2(sr)).abs() < 1e-10);
}

#[test]
fn preserved_equatorial_v1_fixtures_match_the_same_3d_engine() {
    let mut reader =
        csv::Reader::from_reader(include_bytes!("fixtures/equatorial_v1.csv").as_slice());
    let mut worst_phi: f64 = 0.0;
    let mut worst_t: f64 = 0.0;
    for row in reader.records() {
        let row = row.unwrap();
        let val = |i: usize| row[i].parse::<f64>().unwrap();
        let k = Kerr::new(val(2)).unwrap();
        let s = State([
            val(4),
            val(5),
            val(6),
            val(7),
            val(8),
            val(9),
            std::f64::consts::FRAC_PI_2,
            0.0,
        ]);
        let out = CpuReferenceIntegrator.integrate(k, s, &IntegratorConfig::default());
        assert_eq!(format!("{:?}", out.status), row[3]);
        let end = out.samples.last().unwrap();
        let ephi = (end.state.phi() - val(13)).abs();
        let et = (end.state.time() - val(11)).abs();
        worst_phi = worst_phi.max(ephi);
        worst_t = worst_t.max(et);
        assert!((end.state.radius() - val(12)).abs() < 1e-8 && ephi < 2e-7 && et < 1e-5);
        assert!((end.affine - val(10)).abs() < 1e-7);
        for p in out.samples {
            assert_eq!(p.state.theta(), std::f64::consts::FRAC_PI_2);
            assert_eq!(p.state.0[7], 0.0);
        }
    }
    println!("equatorial baseline max phi difference={worst_phi:e}, time difference={worst_t:e}");
}

#[test]
fn zamo_3d_directions_and_oblate_roundtrip() {
    let k = Kerr::new(0.9).unwrap();
    for theta in [0.2, 1.1, 2.7] {
        for dir in [
            [1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            unit([-1.0, 2.0, 3.0]).unwrap(),
        ] {
            let s = ray_3d(k, [7.0, 12.0, theta, 0.8], dir, 1.0).unwrap();
            assert!(s.null_constraint(k).unwrap().abs() < 1e-13);
            let q = from_cartesian(k, to_cartesian(k, s)).unwrap();
            assert!((q[0] - 12.0).abs() < 1e-13);
            assert!((q[1] - theta).abs() < 1e-13);
            assert!((q[2] - 0.8).abs() < 1e-13);
        }
    }
    assert!(ray_3d(k, [0.0, 20.0, 1.0, 0.0], [1.0, 1.0, 1.0], 1.0).is_err());
}

#[test]
fn non_equatorial_capture_conserves_q_through_high_spin_exterior() {
    let mut null: f64 = 0.0;
    let mut q: f64 = 0.0;
    for spin in [0.0, 0.6, 0.99] {
        let k = Kerr::new(spin).unwrap();
        for theta in [0.5, 1.2, 2.2] {
            let s = ray_3d(
                k,
                [0.0, 30.0, theta, 0.0],
                unit([-1.0, 0.03, 0.04]).unwrap(),
                1.0,
            )
            .unwrap();
            let out = CpuReferenceIntegrator.integrate(k, s, &IntegratorConfig::default());
            assert_eq!(out.status, RayStatus::Captured, "{:?}", out.failure_message);
            null = null.max(out.diagnostics.max_null_abs);
            q = q.max(out.diagnostics.max_q_abs);
            assert_eq!(out.diagnostics.max_energy_relative, 0.0);
            assert_eq!(out.diagnostics.max_lz_relative, 0.0);
        }
    }
    println!(
        "3D captured rays through a=.99: max_null={null:e}, max_Q_abs={q:e}; limits=2e-6,1e-7"
    );
    assert!(null < 2e-6 && q < 1e-7);
}
