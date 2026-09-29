use kerr_ray::{
    experiments::stratified::GeneratorSpec,
    session::SessionConfig,
    simulation::{
        ray_execution::RayExecution,
        rays::{RayOutcome, RaySimulationConfig, calculate_rays_with_execution},
    },
    standard_run,
};

fn config() -> RaySimulationConfig {
    let e = SessionConfig::default().experiment;
    RaySimulationConfig {
        spin: e.spin,
        integration: e.integration,
        detector: e.detector,
    }
}
fn compare(a: &[RayOutcome], b: &[RayOutcome]) {
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(b) {
        assert_eq!(a.ray, b.ray);
        let (a, b) = (&a.result, &b.result);
        assert_eq!(a.status, b.status);
        assert_eq!(a.stop_reason, b.stop_reason);
        assert_eq!(a.failure_message, b.failure_message);
        assert_eq!(
            format!("{:?}", a.diagnostics),
            format!("{:?}", b.diagnostics)
        );
        assert_eq!(a.samples.len(), b.samples.len());
        for (a, b) in a.samples.iter().zip(&b.samples) {
            assert_eq!(a.affine.to_bits(), b.affine.to_bits());
            assert_eq!(a.state.0.map(f64::to_bits), b.state.0.map(f64::to_bits));
        }
        match (&a.detection, &b.detection) {
            (Some(a), Some(b)) => {
                assert_eq!(a.state.0.map(f64::to_bits), b.state.0.map(f64::to_bits));
                assert_eq!(a.uv.map(f64::to_bits), b.uv.map(f64::to_bits));
                assert_eq!(a.affine.to_bits(), b.affine.to_bits());
                assert_eq!(a.signed_residual.to_bits(), b.signed_residual.to_bits());
                assert_eq!(a.iterations, b.iterations);
            }
            (None, None) => {}
            _ => panic!("hit presence differs"),
        }
    }
}
#[test]
fn arbitrary_lengths_threads_and_full_physics_bits_match_serial() {
    let g = GeneratorSpec {
        cell_count: [7, 9],
        cell_size: [5., 4.],
        ..Default::default()
    };
    let mut inputs = g.generate([29; 32]).unwrap();
    for (i, c) in inputs.iter_mut().enumerate() {
        c.position_x += i as f64 / 100.;
        c.direction_y = (i as f64 - 31.) * 0.001;
        c.t_emit = i as f64 * 0.25;
    }
    for n in [1, 2, 17, 63] {
        let input = inputs[..n].to_vec();
        let serial = calculate_rays_with_execution(
            input.clone(),
            &config(),
            RayExecution::Serial,
            |_, _| true,
        )
        .unwrap();
        for threads in [1, 2, 4, 8] {
            let mut last = 0;
            let parallel = calculate_rays_with_execution(
                input.clone(),
                &config(),
                RayExecution::Parallel { threads },
                |done, total| {
                    assert_eq!(total, n);
                    assert!(done >= last && done <= total);
                    last = done;
                    true
                },
            )
            .unwrap();
            assert_eq!(last, n);
            compare(&serial, &parallel);
        }
    }
}
#[test]
fn reversed_inputs_and_duplicates_keep_canonical_result_positions() {
    let g = GeneratorSpec {
        cell_count: [3, 4],
        cell_size: [6., 6.],
        ..Default::default()
    };
    let mut inputs = g.generate([5; 32]).unwrap();
    inputs.push(inputs[2]);
    inputs.push(inputs[2]);
    let a =
        calculate_rays_with_execution(inputs.clone(), &config(), RayExecution::Serial, |_, _| true)
            .unwrap();
    inputs.reverse();
    let b = calculate_rays_with_execution(
        inputs,
        &config(),
        RayExecution::Parallel { threads: 3 },
        |_, _| true,
    )
    .unwrap();
    compare(&a, &b);
    for (i, r) in b.iter().enumerate() {
        assert_eq!(r.ray.ray_id(), i);
    }
}
#[test]
fn numerical_failure_is_preserved_not_introduced_or_hidden_by_scheduler() {
    let g = GeneratorSpec {
        cell_count: [2, 3],
        cell_size: [8., 8.],
        ..Default::default()
    };
    let inputs = g.generate([17; 32]).unwrap();
    let mut c = config();
    c.integration.null_tolerance = 1e-30;
    let a = calculate_rays_with_execution(inputs.clone(), &c, RayExecution::Serial, |_, _| true)
        .unwrap();
    let b =
        calculate_rays_with_execution(inputs, &c, RayExecution::Parallel { threads: 3 }, |_, _| {
            true
        })
        .unwrap();
    assert!(
        a.iter()
            .any(|r| r.result.status == kerr_ray::compute::RayStatus::NumericalFailure)
    );
    compare(&a, &b);
}
#[test]
fn cancellation_returns_error_and_zero_threads_is_rejected() {
    let inputs = GeneratorSpec {
        cell_count: [9, 7],
        cell_size: [3., 3.],
        ..Default::default()
    }
    .generate([9; 32])
    .unwrap();
    let mut calls = 0;
    assert!(
        calculate_rays_with_execution(
            inputs.clone(),
            &config(),
            RayExecution::Parallel { threads: 4 },
            |_, _| {
                calls += 1;
                false
            }
        )
        .unwrap_err()
        .contains("cancelled")
    );
    assert_eq!(calls, 1);
    assert!(
        calculate_rays_with_execution(
            inputs.clone(),
            &config(),
            RayExecution::Parallel { threads: 3 },
            |done, _| done < 2
        )
        .unwrap_err()
        .contains("cancelled")
    );
    assert!(
        calculate_rays_with_execution(
            inputs,
            &config(),
            RayExecution::Parallel { threads: 0 },
            |_, _| true
        )
        .is_err()
    );
}
#[test]
fn parallel_results_keep_existing_seed_hash_and_binary_reproduction() {
    let g = GeneratorSpec {
        cell_count: [5, 7],
        cell_size: [6., 4.],
        ..Default::default()
    };
    let (run, parallel) =
        standard_run::seeded::execute_with_outcomes(&g, [43; 32], &config(), |_, _| true).unwrap();
    let serial = calculate_rays_with_execution(
        g.generate([43; 32]).unwrap(),
        &config(),
        RayExecution::Serial,
        |_, _| true,
    )
    .unwrap();
    compare(&serial, &parallel);
    assert_eq!(
        run.seed_metadata.as_ref().unwrap().initial_conditions_hash,
        standard_run::seeded::initial_hash(g.generate([43; 32]).unwrap(), &config()).unwrap()
    );
    let root = std::env::temp_dir().join(format!(
        "kerr-parallel-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let saved = standard_run::save_new(&root, &run).unwrap();
    let loaded = standard_run::load(&saved.directory, "sim_1.bin").unwrap();
    let report = standard_run::reproduce(&loaded).unwrap();
    assert!(report.passed());
    assert!(report.initial_conditions_hash_verified);
    assert_eq!(report.hit_bit_mismatches, [0; 3]);
}
