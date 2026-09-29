use kerr_ray::physics::{kerr::Kerr, metric::Metric};

#[test]
fn metric_inverse_identity() {
    let mut worst: f64 = 0.0;
    for a in [0.0, 0.2, 0.7, 0.99] {
        let k = Kerr::new(a).unwrap();
        for r in [k.horizon() + 0.01, 3.0, 10.0, 1e4] {
            for th in [0.1, 0.7, std::f64::consts::FRAC_PI_2, 2.7, 3.04] {
                let g = k.covariant(r, th).unwrap();
                let inv = k.inverse(r, th).unwrap();
                for (i, row) in g.iter().enumerate() {
                    for (j, _) in inv.iter().enumerate() {
                        let product: f64 = (0..4).map(|m| row[m] * inv[m][j]).sum();
                        worst = worst.max((product - if i == j { 1.0 } else { 0.0 }).abs());
                    }
                }
            }
        }
    }
    println!("metric inverse max absolute residual={worst:e}, limit=2e-11");
    assert!(worst < 2e-11);
}

#[test]
fn schwarzschild_all_components() {
    let k = Kerr::new(0.0).unwrap();
    for r in [2.1, 3.0, 10.0, 1e6] {
        for th in [0.3_f64, 1.2, 2.9] {
            let f = 1.0 - 2.0 / r;
            let diagonal = [-f, 1.0 / f, r * r, r * r * th.sin().powi(2)];
            let g = k.covariant(r, th).unwrap();
            let inv = k.inverse(r, th).unwrap();
            for i in 0..4 {
                for j in 0..4 {
                    let expected = if i == j { diagonal[i] } else { 0.0 };
                    let expected_inv = if i == j { 1.0 / diagonal[i] } else { 0.0 };
                    assert!((g[i][j] - expected).abs() < 5e-14 * expected.abs().max(1.0));
                    assert!((inv[i][j] - expected_inv).abs() < 5e-14 * expected_inv.abs().max(1.0));
                }
            }
        }
    }
}

#[test]
fn exact_structures_and_domain() {
    for a in [0.0, 0.5, 0.99] {
        let k = Kerr::new(a).unwrap();
        assert!(k.delta(k.horizon()).abs() < 2e-15);
        assert!((k.ergosphere(std::f64::consts::FRAC_PI_2) - 2.0).abs() < 1e-15);
        assert!((k.ergosphere(0.0) - k.horizon()).abs() < 1e-15);
        assert!(k.covariant(k.horizon(), 1.0).is_err());
        assert!(k.inverse(5.0, 0.0).is_err());
    }
    for a in [-1.0, -0.99991, 0.99991, 1.0, f64::NAN, f64::INFINITY] {
        assert!(Kerr::new(a).is_err());
    }
}
