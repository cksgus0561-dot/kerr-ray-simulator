use kerr_ray::{
    compute::{RayIntegrator, RayStatus, cpu::CpuReferenceIntegrator},
    experiments::{
        observables, output, parameter_scan,
        scenarios::{self, ExperimentSettings},
    },
    physics::{kerr::Kerr, tetrad::ray_with_impact},
};

#[test]
fn all_example_scenarios_finish_and_export_observables() {
    let p = ExperimentSettings::default();
    let mut worst: f64 = 0.0;
    let mut rays = 0;
    for name in scenarios::NAMES {
        for (id, c) in scenarios::select(name, &p).unwrap().iter().enumerate() {
            let out = CpuReferenceIntegrator.integrate(c.kerr, c.initial, &c.config);
            assert!(
                matches!(out.status, RayStatus::Captured | RayStatus::Escaped),
                "{name} {} {:?}",
                c.label,
                out.failure_message
            );
            assert!(observables::minimum_radius(&out) > c.kerr.horizon());
            assert!(observables::affine_length(&out) > 0.0);
            assert_eq!(
                observables::final_scattering_angle(c.kerr, &out).is_some(),
                out.status == RayStatus::Escaped
            );
            assert_eq!(
                output::summary_row(id, c, &out).split(',').count(),
                output::SUMMARY_HEADER.split(',').count()
            );
            assert_eq!(out.diagnostics.max_energy_abs, 0.0);
            assert_eq!(out.diagnostics.max_lz_abs, 0.0);
            worst = worst.max(out.diagnostics.max_null_abs);
            rays += 1;
        }
    }
    println!("all 8 example scenarios: rays={rays}; max null={worst:e}, limit=2e-6");
    assert!(worst < 2e-6);
}

#[test]
fn high_spin_capture_and_cutoff_sensitivity() {
    let k = Kerr::new(0.99).unwrap();
    let s = ray_with_impact(k, 50.0, 0.0, -6.0, true).unwrap();
    let mut worst: f64 = 0.0;
    for epsilon in [0.01, 0.003, 0.001] {
        let cfg = kerr_ray::config::IntegratorConfig {
            horizon_epsilon: epsilon,
            ..Default::default()
        };
        let out = CpuReferenceIntegrator.integrate(k, s, &cfg);
        assert_eq!(out.status, RayStatus::Captured, "{:?}", out.failure_message);
        assert!(out.samples.iter().all(|p| p.state.radius() > k.horizon()));
        assert!((out.samples.last().unwrap().state.radius() - k.horizon() - epsilon).abs() < 3e-11);
        worst = worst.max(out.diagnostics.max_null_abs);
    }
    println!("a=0.99 capture, epsilon 0.01/0.003/0.001: max null={worst:e}, limit=2e-6");
    assert!(worst < 2e-6);
}

#[test]
fn scan_rejects_invalid_ranges_and_preserves_endpoints() {
    assert_eq!(
        parameter_scan::linear_values(0.0, 1.0, 3).unwrap(),
        [0.0, 0.5, 1.0]
    );
    assert_eq!(parameter_scan::linear_values(0.0, 1.0, 1).unwrap(), [0.0]);
    assert!(parameter_scan::linear_values(0.0, 1.0, 0).is_err());
    assert!(parameter_scan::linear_values(1.0, 0.0, 2).is_err());
    assert!(parameter_scan::linear_values(f64::NAN, 1.0, 2).is_err());
    assert!(scenarios::select("unknown", &ExperimentSettings::default()).is_err());
    assert!(
        scenarios::select(
            "single_ray",
            &ExperimentSettings {
                spin: 1.0,
                ..Default::default()
            }
        )
        .is_err()
    );
}

#[test]
fn cli_writes_summary_and_real_trajectory_and_reports_failures() {
    let dir = std::env::temp_dir().join(format!("kerr-ray-cli-{}", std::process::id()));
    let exe = env!("CARGO_BIN_EXE_kerr-ray");
    let out = std::process::Command::new(exe)
        .args(["experiment", "single_ray", "--trajectories", "--output"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let summary = std::fs::read_to_string(dir.join("summary.csv")).unwrap();
    assert_eq!(summary.lines().count(), 2);
    let trajectory = std::fs::read_to_string(dir.join("ray_0000.csv")).unwrap();
    assert!(trajectory.lines().count() > 100);
    assert!(trajectory.starts_with("affine,t,r,theta,phi"));
    let bad = std::process::Command::new(exe)
        .args(["experiment", "single_ray", "--max-steps", "1", "--output"])
        .arg(dir.join("failure"))
        .output()
        .unwrap();
    assert!(!bad.status.success());
    let failed = std::fs::read_to_string(dir.join("failure/summary.csv")).unwrap();
    assert!(failed.contains("NumericalFailure,StepLimit"));
    // Only remove these specific test files; no recursive path operations.
    for path in ["summary.csv", "ray_0000.csv", "failure/summary.csv"] {
        std::fs::remove_file(dir.join(path)).unwrap();
    }
    std::fs::remove_dir(dir.join("failure")).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
