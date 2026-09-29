use kerr_ray::{
    experiments::chi_sampling::{CHI_LIMIT, ChiSampling, chi_at, chi_grid},
    session::SessionConfig,
    standard_run::{
        self,
        batch::{self, BatchReport, BatchSpecification},
        seeded,
    },
};
use std::{fs, path::PathBuf, sync::OnceLock};

fn temp() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "kerr-batch-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    root
}
fn fixture() -> &'static BatchReport {
    static REPORT: OnceLock<BatchReport> = OnceLock::new();
    REPORT.get_or_init(|| {
        let mut session = SessionConfig::default();
        session.generator.as_mut().unwrap().cell_count = [2, 2];
        batch::execute(
            &temp(),
            &session,
            &BatchSpecification {
                chi_indices: vec![-76, 0, 76],
                ..BatchSpecification::full(2)
            },
            |_| {},
        )
        .unwrap()
    })
}

#[test]
fn grid_has_exact_count_endpoints_center_and_strict_order() {
    let grid = chi_grid();
    assert_eq!(grid.len(), 153);
    assert_eq!(grid[0].to_bits(), (-0.999_f64).to_bits());
    assert_eq!(grid[76].to_bits(), 0.0_f64.to_bits());
    assert_eq!(grid[152].to_bits(), 0.999_f64.to_bits());
    assert!(grid.windows(2).all(|w| w[0] < w[1]));
    assert!(
        grid.iter()
            .all(|c| c.is_finite() && (-CHI_LIMIT..=CHI_LIMIT).contains(c))
    );
    for i in 1..=76 {
        assert_eq!(grid[76 - i].to_bits(), (-grid[76 + i]).to_bits());
    }
}

#[test]
fn grid_follows_formula_and_is_denser_towards_extremes() {
    let grid = chi_grid();
    for i in -76..=76 {
        let direct = (f64::from(i) * CHI_LIMIT.atanh() / 76.).tanh();
        assert!((chi_at(i).unwrap() - direct).abs() <= 2. * f64::EPSILON);
    }
    let gaps: Vec<_> = grid[76..].windows(2).map(|w| w[1] - w[0]).collect();
    assert!(gaps.windows(2).all(|w| w[0] > w[1]));
    assert!(chi_at(-77).is_err());
    assert!(chi_at(77).is_err());
    assert!(chi_at(i32::MIN).is_err());
}

#[test]
fn plan_orders_chi_then_repetition_and_rejects_invalid_settings() {
    let spec = BatchSpecification::full(3);
    assert_eq!(spec.simulation_count().unwrap(), 459);
    for number in 1..=459 {
        let entry = spec.entry(number).unwrap();
        assert_eq!(entry.chi_index, ((number - 1) / 3) as i32 - 76);
        assert_eq!(entry.repetition, (number - 1) % 3 + 1);
        assert_eq!(entry.chi, chi_at(entry.chi_index).unwrap());
    }
    assert!(spec.entry(0).is_err());
    assert!(spec.entry(460).is_err());
    assert!(BatchSpecification::full(0).validate().is_err());
    assert!(BatchSpecification::full(usize::MAX).validate().is_err());
    for indices in [vec![], vec![0, -1], vec![1, 1], vec![77]] {
        assert!(
            BatchSpecification {
                chi_indices: indices,
                ..BatchSpecification::full(1)
            }
            .validate()
            .is_err()
        );
    }
    let sampling = ChiSampling {
        chi_max: 0.99,
        ..Default::default()
    };
    assert!(sampling.validate().is_err());
}

#[test]
fn batch_records_plan_multiple_seeds_and_exact_binary_layout() {
    let report = fixture();
    let names = standard_run::list_simulations(&report.directory).unwrap();
    assert_eq!(
        names,
        (1..=6).map(|n| format!("sim_{n}.bin")).collect::<Vec<_>>()
    );
    assert_eq!(fs::read_dir(&report.directory).unwrap().count(), 7);
    let mut seeds = std::collections::HashSet::new();
    for (n, r) in report.simulations.iter().enumerate() {
        let run = standard_run::load(&report.directory, &names[n]).unwrap();
        let raw = fs::read(report.directory.join(&names[n])).unwrap();
        let meta = run.seed_metadata.as_ref().unwrap();
        assert!(seeds.insert(meta.seed));
        assert_eq!(meta.seed, r.seed);
        assert_eq!(meta.initial_conditions_hash, r.initial_conditions_hash);
        assert_eq!(raw[0..8], r.entry.chi.to_le_bytes());
        assert_eq!(raw[8..40], r.seed);
        assert_eq!(raw[40..72], r.initial_conditions_hash);
        assert_eq!(raw.len(), 72 + r.rays + r.counts[1] * 24);
        assert_eq!(run.rows.len(), 4);
        assert_eq!(run.chi.to_bits(), r.entry.chi.to_bits());
        assert!(run.rows.iter().all(|r| r.ray_id.is_none()));
        let plan = run.common.batch.as_ref().unwrap();
        assert_eq!(plan.chi_sampling, ChiSampling::default());
        assert_eq!(plan.chi_indices, [-76, 0, 76]);
        assert_eq!(plan.simulations_per_chi, 2);
        assert_eq!(standard_run::counts(&run.rows), r.counts);
    }
}

#[test]
fn batch_reproduction_regenerates_hash_canonical_order_status_and_hit_bits() {
    let report = fixture();
    for number in 1..=6 {
        let run = standard_run::load(&report.directory, &format!("sim_{number}.bin")).unwrap();
        let result = standard_run::reproduce(&run).unwrap();
        assert!(result.passed(), "{}", result.summary());
        assert!(result.initial_conditions_hash_verified);
        assert_eq!(result.hit_bit_mismatches, [0; 3]);
        assert_eq!(result.canonical_mismatches, 0);
        assert_eq!(result.initial_state_bit_mismatches, 0);
        assert!(result.same_code);
    }
}

#[test]
fn batch_and_existing_single_path_produce_identical_binary_bytes() {
    let report = fixture();
    let loaded = standard_run::load(&report.directory, "sim_1.bin").unwrap();
    let (single, _) = seeded::execute_with_outcomes(
        loaded.common.ray_generator.as_ref().unwrap(),
        loaded.seed_metadata.as_ref().unwrap().seed,
        &loaded.common.config(loaded.chi).unwrap(),
        |_, _| true,
    )
    .unwrap();
    assert!(single.common.batch.is_none());
    let root = temp();
    let first = standard_run::save_new(&root, &single).unwrap();
    let second = standard_run::save_new(&root, &single).unwrap();
    assert_ne!(first.directory, second.directory);
    let expected = fs::read(report.directory.join("sim_1.bin")).unwrap();
    for dir in [&first.directory, &second.directory] {
        assert_eq!(fs::read(dir.join("sim_1.bin")).unwrap(), expected);
        let json: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("common.json")).unwrap()).unwrap();
        assert!(json.get("batch").is_none());
        assert!(
            standard_run::load(dir, "sim_1.bin")
                .unwrap()
                .common
                .batch
                .is_none()
        );
    }
}

#[test]
fn invalid_batch_fails_before_creating_archive_and_never_overwrites_existing_folder() {
    let root = temp();
    assert!(
        batch::execute(
            &root,
            &SessionConfig::default(),
            &BatchSpecification::full(0),
            |_| {}
        )
        .is_err()
    );
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    let existing = root.join("simulation_1");
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("common.json"), "keep").unwrap();
    let mut session = SessionConfig::default();
    session.generator.as_mut().unwrap().cell_count = [1, 1];
    let plan = BatchSpecification {
        chi_indices: vec![0],
        ..BatchSpecification::full(1)
    };
    let mut completed = 0;
    let output = batch::execute(&root, &session, &plan, |_| {
        completed += 1;
    })
    .unwrap();
    assert_eq!(completed, 1);
    assert_eq!(output.directory, root.join("simulation_2"));
    assert_eq!(
        fs::read_to_string(existing.join("common.json")).unwrap(),
        "keep"
    );
}

#[test]
fn loading_rejects_wrong_batch_spin_file_number_and_sampling_specification() {
    let report = fixture();
    let root = temp();
    fs::copy(
        report.directory.join("common.json"),
        root.join("common.json"),
    )
    .unwrap();
    fs::copy(report.directory.join("sim_1.bin"), root.join("sim_3.bin")).unwrap();
    assert!(
        standard_run::load(&root, "sim_3.bin")
            .unwrap_err()
            .to_string()
            .contains("batch numbering")
    );
    fs::copy(report.directory.join("sim_1.bin"), root.join("sim_7.bin")).unwrap();
    assert!(standard_run::load(&root, "sim_7.bin").is_err());
    let mut common: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("common.json")).unwrap()).unwrap();
    common["batch"]["chi_sampling"]["total_chi_count"] = 152.into();
    fs::write(
        root.join("common.json"),
        serde_json::to_vec(&common).unwrap(),
    )
    .unwrap();
    assert!(standard_run::list_simulations(&root).is_err());
}

#[test]
fn cli_requires_repetition_and_dry_run_performs_no_calculation_or_save() {
    let root = temp();
    let exe = env!("CARGO_BIN_EXE_kerr-ray");
    for args in [
        vec![],
        vec!["--simulations-per-chi", "0"],
        vec!["--simulations-per-chi", "1", "--chi-indices", "0,-76"],
    ] {
        let output = std::process::Command::new(exe)
            .arg("batch-simulate")
            .args(args)
            .args(["--output", root.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
    let output = std::process::Command::new(exe)
        .args([
            "batch-simulate",
            "--simulations-per-chi",
            "2",
            "--dry-run",
            "--output",
            root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("306 simulations; 36864 physical rays"));
    assert_eq!(
        text.lines().filter(|l| l.starts_with("chi index ")).count(),
        153
    );
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
}
