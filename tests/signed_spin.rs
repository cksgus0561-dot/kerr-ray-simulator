use kerr_ray::{
    physics::{
        field::frame_dragging,
        geodesic::{inverse_derivatives, rhs},
        kerr::Kerr,
        metric::Metric,
    },
    session::SessionConfig,
    simulation::rays::{RayInitialCondition, RaySimulationConfig, prepare_rays},
};

#[test]
fn signed_spin_domain_and_metric_reflection() {
    for a in [0.0, 0.6, 0.99, 0.9999] {
        let p = Kerr::new(a).unwrap();
        let m = Kerr::new(-a).unwrap();
        assert_eq!(p.horizon(), m.horizon());
        for r in [p.horizon() + 0.001, 2.5, 80.0] {
            for theta in [0.3, 1.1, 2.8] {
                assert_eq!(p.ergosphere(theta), m.ergosphere(theta));
                let sign = [1.0, 1.0, 1.0, -1.0];
                let pg = [p.covariant(r, theta).unwrap(), p.inverse(r, theta).unwrap()];
                let mg = [m.covariant(r, theta).unwrap(), m.inverse(r, theta).unwrap()];
                let pd = inverse_derivatives(p, r, theta).unwrap();
                let md = inverse_derivatives(m, r, theta).unwrap();
                for n in 0..2 {
                    for i in 0..4 {
                        for j in 0..4 {
                            assert_eq!(pg[n][i][j] * sign[i] * sign[j], mg[n][i][j]);
                            assert_eq!(pd[n][i][j] * sign[i] * sign[j], md[n][i][j]);
                        }
                    }
                }
                let omega = frame_dragging(p, r, theta).unwrap();
                assert!(omega.is_finite());
                assert_eq!(omega, -frame_dragging(m, r, theta).unwrap());
                // Cancellation near the BL horizon makes this absolute residual
                // larger than the original r_plus+0.01 test; record it separately.
                for (i, row) in pg[0].iter().enumerate() {
                    for (j, _) in pg[1].iter().enumerate() {
                        let v: f64 = (0..4).map(|n| row[n] * pg[1][n][j]).sum();
                        assert!((v - if i == j { 1.0 } else { 0.0 }).abs() < 1e-8);
                    }
                }
            }
        }
    }
}

#[test]
fn reflected_cartesian_initial_state_and_hamiltonian_flow() {
    let e = SessionConfig::default().experiment;
    let mut cfg = RaySimulationConfig {
        spin: 0.9999,
        integration: e.integration,
        detector: e.detector,
    };
    let ray = RayInitialCondition {
        position_x: 80.0,
        position_y: 2.7,
        position_z: -1.2,
        direction_x: -1.0,
        direction_y: 0.04,
        direction_z: 0.03,
        t_emit: 2.0,
    };
    let p = prepare_rays(vec![ray], &cfg).unwrap()[0].initial();
    cfg.spin = -cfg.spin;
    let mirror = RayInitialCondition {
        position_y: -ray.position_y,
        direction_y: -ray.direction_y,
        ..ray
    };
    let m = prepare_rays(vec![mirror], &cfg).unwrap()[0].initial();
    let sign = [1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0];
    let pr = rhs(Kerr::new(0.9999).unwrap(), &p.0).unwrap();
    let mr = rhs(Kerr::new(-0.9999).unwrap(), &m.0).unwrap();
    for i in 0..8 {
        assert!((p.0[i] * sign[i] - m.0[i]).abs() < 1e-12);
        assert!((pr[i] * sign[i] - mr[i]).abs() < 1e-12);
    }
}
