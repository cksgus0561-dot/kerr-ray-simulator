use kerr_ray::{
    compute::RayStatus,
    config::IntegratorConfig,
    detector::plane::{DetectorPlane, Plane},
    simulation::rays::{RayInitialCondition, RaySimulationConfig, prepare_rays},
    standard_run::{self, RecordedRun},
};
use parquet::{
    file::reader::{FileReader, SerializedFileReader},
    record::Field,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
fn temp() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let p = std::env::temp_dir().join(format!(
        "kerr-standard-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&p).unwrap();
    p
}
fn config() -> RaySimulationConfig {
    RaySimulationConfig {
        spin: 0.6,
        integration: IntegratorConfig {
            escape_radius: 100.0,
            max_affine: 300.0,
            ..Default::default()
        },
        detector: DetectorPlane {
            plane: Plane::from_normal_up(
                [80.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                40.0,
                40.0,
            )
            .unwrap(),
            resolution: [512, 512],
            root_tolerance: 1e-10,
        },
    }
}
fn inputs() -> Vec<RayInitialCondition> {
    let a = RayInitialCondition {
        position_x: 60.0,
        position_y: 2.3,
        position_z: -1.1,
        direction_x: 7.0,
        direction_y: 0.14,
        direction_z: -0.07,
        t_emit: 2.123456789012345,
    };
    let mut b = a;
    b.position_x = 90.0;
    b.direction_y = -0.0;
    b.t_emit = -0.0;
    let mut c = a;
    c.direction_x = 14.0;
    c.direction_y = 0.28;
    c.direction_z = -0.14;
    vec![b, a, c, a]
}
fn bits(c: RayInitialCondition) -> [u64; 7] {
    [
        c.position_x,
        c.position_y,
        c.position_z,
        c.direction_x,
        c.direction_y,
        c.direction_z,
        c.t_emit,
    ]
    .map(f64::to_bits)
}
fn save_sample(root: &Path) -> PathBuf {
    let run = standard_run::execute(inputs(), &config()).unwrap();
    standard_run::save_new(root, &run).unwrap().directory
}
#[test]
fn disk_only_reproduction_after_original_inputs_and_results_are_dropped() {
    let root = temp();
    let (dir, initial_bits) = {
        let cfg = config();
        let raw = inputs();
        let prepared = prepare_rays(raw.clone(), &cfg).unwrap();
        let audit: Vec<_> = prepared
            .iter()
            .map(|r| (r.ray_id(), r.initial().0.map(f64::to_bits)))
            .collect();
        let original = standard_run::execute(raw, &cfg).unwrap();
        let dir = standard_run::save_new(&root, &original).unwrap().directory;
        (dir, audit)
    }; // A config, raw input Vec and all A results have left scope.
    let loaded = standard_run::load(&dir, "sim_1.parquet").unwrap();
    let cfg = loaded.common.config(loaded.chi).unwrap();
    let prepared = prepare_rays(loaded.rows.iter().map(|r| r.input).collect(), &cfg).unwrap();
    let actual: Vec<_> = prepared
        .iter()
        .map(|r| (r.ray_id(), r.initial().0.map(f64::to_bits)))
        .collect();
    assert_eq!(actual, initial_bits); // Audit only; never used as solver input.
    let report = standard_run::reproduce(&loaded).unwrap();
    assert!(report.passed(), "{}", report.summary());
    assert_eq!(report.stored_ids_checked, 4);
    assert_eq!(report.stored_id_mismatches, 0);
    assert_eq!(report.hit_bit_mismatches, [0; 3]);
}
#[test]
fn parquet_preserves_raw_f64_and_common_actual_values() {
    let root = temp();
    let mut cfg = config();
    cfg.spin = 0.6345678901234567;
    cfg.integration = IntegratorConfig {
        rtol: 3e-11,
        atol: 7e-13,
        initial_step: 0.07,
        min_step: 2e-13,
        max_step: 0.4,
        max_steps: 12345,
        max_affine: 321.0,
        horizon_epsilon: 0.002,
        escape_radius: 102.0,
        null_tolerance: 2e-5,
    };
    cfg.detector.root_tolerance = 2e-10;
    cfg.detector.plane = Plane::from_normal_up(
        [81.12345678901234, 0.1234567890123456, -0.0],
        [1.0, 0.03, 0.04],
        [0.0, 0.0, 1.0],
        41.5,
        42.5,
    )
    .unwrap();
    let raw = inputs();
    let run = standard_run::execute(raw.clone(), &cfg).unwrap();
    let dir = standard_run::save_new(&root, &run).unwrap().directory;
    let loaded = standard_run::load(&dir, "sim_1.parquet").unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.common).unwrap(),
        serde_json::to_value(&run.common).unwrap()
    );
    let restored = loaded.common.config(loaded.chi).unwrap();
    assert_eq!(
        serde_json::to_value(restored.integration).unwrap(),
        serde_json::to_value(cfg.integration).unwrap()
    );
    assert_eq!(
        serde_json::to_value(restored.detector.plane).unwrap(),
        serde_json::to_value(cfg.detector.plane).unwrap()
    );
    assert_eq!(
        restored.detector.root_tolerance.to_bits(),
        cfg.detector.root_tolerance.to_bits()
    );
    assert_eq!(loaded.chi.to_bits(), cfg.spin.to_bits());
    for ((a, b), raw) in run.rows.iter().zip(&loaded.rows).zip(raw) {
        assert_eq!(bits(b.input), bits(raw));
        assert_eq!(a.ray_id, b.ray_id);
        assert_eq!(a.status, b.status);
        assert_eq!(b.energy.to_bits(), 1.0f64.to_bits());
        assert_eq!(
            a.hit.map(|v| v.map(f64::to_bits)),
            b.hit.map(|v| v.map(f64::to_bits))
        );
    }
    assert_ne!(loaded.rows[1].input.direction_x, 1.0);
}
#[test]
fn actual_parquet_schema_metadata_and_nulls_are_standard() {
    let root = temp();
    let dir = save_sample(&root);
    let reader =
        SerializedFileReader::new(fs::File::open(dir.join("sim_1.parquet")).unwrap()).unwrap();
    let meta = reader.metadata().file_metadata();
    assert_eq!(meta.num_rows(), 4);
    let fields = meta.schema_descr().root_schema().get_fields();
    assert_eq!(fields.len(), 12);
    assert!(!fields.iter().any(|f| f.name() == "energy"));
    assert_eq!(fields[0].name(), "ray_id");
    assert!(!fields.iter().any(|f| f.name() == "chi"));
    let kv = meta.key_value_metadata().unwrap();
    let chi = kv.iter().filter(|v| v.key == "chi").collect::<Vec<_>>();
    assert_eq!(chi.len(), 1);
    assert_eq!(
        chi[0]
            .value
            .as_ref()
            .unwrap()
            .parse::<f64>()
            .unwrap()
            .to_bits(),
        0.6f64.to_bits()
    );
    let mut null_rows = 0;
    for row in reader.get_row_iter(None).unwrap() {
        let row = row.unwrap();
        let values = row.get_column_iter().map(|(_, v)| v).collect::<Vec<_>>();
        assert!(matches!(values[0], Field::ULong(_)));
        assert!(values[1..8].iter().all(|v| matches!(v, Field::Double(_))));
        if let Field::Str(status) = values[8] {
            if status == "DET" {
                assert!(values[9..].iter().all(|v| matches!(v, Field::Double(_))));
            } else {
                assert!(values[9..].iter().all(|v| matches!(v, Field::Null)));
                null_rows += 1;
            }
        } else {
            panic!("UTF8 status required");
        }
    }
    assert_eq!(null_rows, 1);
    let text = fs::read_to_string(dir.join("common.json")).unwrap();
    for forbidden in [
        "resolution",
        "camera",
        "ray_id",
        "time_bin",
        "trajectory",
        "chi",
    ] {
        assert!(!text.contains(forbidden));
    }
}
#[test]
fn permuted_disk_rows_regenerate_ids_without_row_index_identity() {
    let root = temp();
    let mut run = standard_run::execute(inputs(), &config()).unwrap();
    run.rows.reverse();
    let dir = standard_run::save_new(&root, &run).unwrap().directory;
    drop(run);
    let loaded = standard_run::load(&dir, "sim_1.parquet").unwrap();
    let report = standard_run::reproduce(&loaded).unwrap();
    assert!(report.passed(), "{}", report.summary());
    assert_eq!(report.stored_ids_checked, 4);
    assert_eq!(report.stored_id_mismatches, 0);
}
#[test]
fn changed_saved_results_cannot_drive_the_fresh_solver() {
    let root = temp();
    let mut run = standard_run::execute(inputs(), &config()).unwrap();
    // Valid but false reference data; must not become new physics input.
    run.rows[1].hit.as_mut().unwrap()[0] += 0.125;
    run.rows[0].status = RayStatus::Captured;
    let dir = standard_run::save_new(&root, &run).unwrap().directory;
    drop(run);
    let report =
        standard_run::reproduce(&standard_run::load(&dir, "sim_1.parquet").unwrap()).unwrap();
    assert!(!report.passed());
    assert_eq!(report.status_mismatches, 1);
    assert!(report.hit_max_abs[0] >= 0.125 - 1e-12);
    assert_eq!(report.actual_counts, [0, 3, 0, 1, 0]);
}
#[test]
fn numbering_never_overwrites_and_uses_unpadded_unused_positive_numbers() {
    let root = temp();
    fs::create_dir(root.join("simulation_1")).unwrap();
    fs::write(root.join("simulation_1/marker"), b"preserve").unwrap();
    fs::create_dir(root.join("simulation_3")).unwrap();
    let run = standard_run::execute(inputs(), &config()).unwrap();
    let first = standard_run::save_new(&root, &run).unwrap();
    let second = standard_run::save_new(&root, &run).unwrap();
    assert_eq!(first.directory.file_name().unwrap(), "simulation_2");
    assert_eq!(second.directory.file_name().unwrap(), "simulation_4");
    assert_eq!(
        fs::read(root.join("simulation_1/marker")).unwrap(),
        b"preserve"
    );
}
#[test]
fn simultaneous_saves_reserve_distinct_directories() {
    let root = temp();
    let run = standard_run::execute(inputs(), &config()).unwrap();
    let dirs = std::thread::scope(|s| {
        let a = s.spawn(|| standard_run::save_new(&root, &run).unwrap().directory);
        let b = s.spawn(|| standard_run::save_new(&root, &run).unwrap().directory);
        (a.join().unwrap(), b.join().unwrap())
    });
    assert_ne!(dirs.0, dirs.1);
}
#[test]
fn unknown_version_convention_and_nonunit_energy_are_rejected() {
    let root = temp();
    let dir = save_sample(&root);
    let original = fs::read(dir.join("common.json")).unwrap();
    for field in ["data_format_version", "convention"] {
        let mut json: serde_json::Value = serde_json::from_slice(&original).unwrap();
        if field == "convention" {
            json[field]["M"] = 2.0.into();
        } else {
            json[field] = "future".into();
        }
        fs::write(dir.join("common.json"), serde_json::to_vec(&json).unwrap()).unwrap();
        assert!(standard_run::load(&dir, "sim_1.parquet").is_err());
    }
    fs::write(dir.join("common.json"), original).unwrap();
    let mut run = standard_run::load(&dir, "sim_1.parquet").unwrap();
    run.rows[0].energy = 2.0;
    assert!(standard_run::save_new(&root, &run).is_err());
}
#[test]
fn malformed_inputs_and_hit_nullability_are_rejected_before_saving() {
    let run = standard_run::execute(inputs(), &config()).unwrap();
    let mut bad = run.clone();
    bad.rows[0].input.t_emit = f64::NAN;
    assert!(bad.validate().is_err());
    let mut bad = run.clone();
    bad.rows[1].hit = None;
    assert!(bad.validate().is_err());
    let mut bad = run.clone();
    bad.rows[0].hit = Some([0.0, 0.0, 100.0]);
    assert!(bad.validate().is_err());
    let mut bad = run;
    bad.rows[1].hit.as_mut().unwrap()[0] = f64::INFINITY;
    assert!(bad.validate().is_err());
}
#[test]
fn truncated_parquet_and_invalid_simulation_names_fail_closed() {
    let root = temp();
    let dir = save_sample(&root);
    for name in [
        "../sim_1.parquet",
        "sim_0.parquet",
        "sim_01.parquet",
        "sim_-1.parquet",
    ] {
        assert!(standard_run::load(&dir, name).is_err());
    }
    fs::copy(dir.join("sim_1.parquet"), dir.join("sim_2.parquet")).unwrap();
    assert!(standard_run::load(&dir, "sim_2.parquet").is_ok());
    fs::write(dir.join("sim_1.parquet"), b"PAR1broken").unwrap();
    assert!(standard_run::load(&dir, "sim_1.parquet").is_err());
}
#[test]
fn actual_active_and_numerical_failure_statuses_round_trip() {
    for (limit, status) in [
        (true, RayStatus::Active),
        (false, RayStatus::NumericalFailure),
    ] {
        let root = temp();
        let mut cfg = config();
        if limit {
            cfg.integration.max_affine = 0.01;
        } else {
            cfg.integration.max_steps = 1;
        }
        let run = standard_run::execute(vec![inputs()[1]], &cfg).unwrap();
        assert_eq!(run.rows[0].status, status);
        let dir = standard_run::save_new(&root, &run).unwrap().directory;
        let loaded = standard_run::load(&dir, "sim_1.parquet").unwrap();
        assert_eq!(loaded.rows[0].ray_id, Some(0));
        assert_eq!(loaded.rows[0].status, status);
        assert!(loaded.rows[0].hit.is_none());
        assert!(standard_run::reproduce(&loaded).unwrap().passed());
    }
}
#[test]
fn every_status_is_written_as_three_letters_and_restored_from_parquet() {
    let root = temp();
    let mut run = standard_run::execute(vec![inputs()[1]; 5], &config()).unwrap();
    let statuses = [
        RayStatus::Active,
        RayStatus::Detected,
        RayStatus::Captured,
        RayStatus::Escaped,
        RayStatus::NumericalFailure,
    ];
    // A serialization fixture, not a claim that these are solver outcomes.
    for (row, status) in run.rows.iter_mut().zip(statuses) {
        row.status = status;
        if status != RayStatus::Detected {
            row.hit = None;
        }
    }
    let dir = standard_run::save_new(&root, &run).unwrap().directory;
    let reader =
        SerializedFileReader::new(fs::File::open(dir.join("sim_1.parquet")).unwrap()).unwrap();
    let disk: Vec<_> = reader
        .get_row_iter(None)
        .unwrap()
        .map(|row| {
            let row = row.unwrap();
            match row
                .get_column_iter()
                .find(|(name, _)| *name == "status")
                .unwrap()
                .1
            {
                Field::Str(s) => s.clone(),
                _ => panic!("UTF8 status required"),
            }
        })
        .collect();
    assert_eq!(disk, ["ACT", "DET", "CAP", "ESC", "NUM"]);
    let loaded = standard_run::load(&dir, "sim_1.parquet").unwrap();
    assert_eq!(
        loaded.rows.iter().map(|r| r.status).collect::<Vec<_>>(),
        statuses
    );
    for (original, restored) in run.rows.iter().zip(&loaded.rows) {
        assert_eq!(original.ray_id, restored.ray_id);
        assert_eq!(bits(original.input), bits(restored.input));
        assert_eq!(original.hit, restored.hit);
    }
}
#[test]
fn user_cli_load_reproduce_and_failure_exit_codes() {
    let root = temp();
    let dir = save_sample(&root);
    for command in ["inspect-simulation", "reproduce"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_kerr-ray"))
            .args([
                command,
                "--input",
                dir.to_str().unwrap(),
                "--sim",
                "sim_1.parquet",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if command == "reproduce" {
            assert!(String::from_utf8_lossy(&output.stdout).contains("result: PASS"));
        }
    }
    let mut bad: RecordedRun = standard_run::load(&dir, "sim_1.parquet").unwrap();
    bad.rows[0].status = RayStatus::Captured;
    let dir = standard_run::save_new(&root, &bad).unwrap().directory;
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_kerr-ray"))
        .args(["reproduce", "--input", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("result: FAIL"));
}

#[test]
fn stored_ids_are_dense_unique_and_link_raw_inputs_and_results_after_shuffle() {
    let root = temp();
    let mut run = standard_run::execute(inputs(), &config()).unwrap();
    // Original input order differs from canonical order; three canonical-equal
    // copies (including differently scaled raw direction) are all retained.
    assert_eq!(
        run.rows.iter().map(|r| r.ray_id).collect::<Vec<_>>(),
        [Some(3), Some(0), Some(1), Some(2)]
    );
    let reference = run.rows.clone();
    run.rows.rotate_left(2);
    run.rows.reverse();
    let dir = standard_run::save_new(&root, &run).unwrap().directory;
    let loaded = standard_run::load(&dir, "sim_1.parquet").unwrap();
    let ids: std::collections::BTreeSet<_> =
        loaded.rows.iter().map(|r| r.ray_id.unwrap()).collect();
    assert_eq!(ids, (0..4).collect());
    assert_eq!(ids.len(), loaded.rows.len());
    for row in &loaded.rows {
        let original = reference.iter().find(|r| r.ray_id == row.ray_id).unwrap();
        assert_eq!(bits(original.input), bits(row.input));
        assert_eq!(original.energy.to_bits(), row.energy.to_bits());
        assert_eq!(original.status, row.status);
        assert_eq!(
            original.hit.map(|h| h.map(f64::to_bits)),
            row.hit.map(|h| h.map(f64::to_bits))
        );
    }
    let report = standard_run::reproduce(&loaded).unwrap();
    assert!(report.passed(), "{}", report.summary());
    assert_eq!(report.stored_ids_checked, 4);
    assert_eq!(report.stored_id_mismatches, 0);
    assert_eq!(report.hit_bit_mismatches, [0; 3]);
}

#[test]
fn swapped_stored_ids_are_detected_without_changing_fresh_physics_or_generated_ids() {
    let root = temp();
    let mut run = standard_run::execute(inputs(), &config()).unwrap();
    // Exchange IDs between distinct physical conditions, preserving unique range.
    let id = run.rows[0].ray_id;
    run.rows[0].ray_id = run.rows[1].ray_id;
    run.rows[1].ray_id = id;
    let dir = standard_run::save_new(&root, &run).unwrap().directory;
    let loaded = standard_run::load(&dir, "sim_1.parquet").unwrap();
    let report = standard_run::reproduce(&loaded).unwrap();
    assert!(!report.passed());
    assert!(report.stored_id_mismatches >= 2);
    assert_eq!(report.stored_ids_checked, 4);
    assert_eq!(report.id_mismatches, 0);
    assert_eq!(report.canonical_mismatches, 0);
    assert_eq!(report.initial_state_bit_mismatches, 0);
    assert_eq!(report.status_mismatches, 0);
    assert_eq!(report.actual_counts, [0, 3, 0, 1, 0]);
    assert_eq!(report.hit_bit_mismatches, [0; 3]);
    assert_eq!(report.hit_max_abs, [0.0; 3]);
}

#[test]
fn duplicate_out_of_range_and_missing_ids_cannot_be_written() {
    let root = temp();
    let run = standard_run::execute(inputs(), &config()).unwrap();
    for bad_id in [Some(0), Some(4), Some(u64::MAX), None] {
        let mut bad = run.clone();
        bad.rows[0].ray_id = bad_id;
        assert!(bad.validate().is_err());
        assert!(standard_run::save_new(&root, &bad).is_err());
    }
    let mut missing = run;
    for row in &mut missing.rows {
        row.ray_id = None;
    }
    // Legacy files can be inspected/reproduced but cannot masquerade as newly
    // calculated archives with persisted IDs. No ID is synthesized by the writer.
    assert!(missing.validate().is_ok());
    assert!(standard_run::save_new(&root, &missing).is_err());
    assert_eq!(fs::read_dir(root).unwrap().count(), 0);
}

#[test]
fn historical_archive_without_ids_remains_readable_and_reports_unverified_ids() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/standard_without_id");
    let loaded = standard_run::load(&dir, "sim_1.parquet").unwrap();
    assert_eq!(loaded.rows.len(), 256);
    assert!(loaded.rows.iter().all(|r| r.ray_id.is_none()));
    assert_eq!(standard_run::counts(&loaded.rows), [0, 216, 20, 20, 0]);
    let report = standard_run::reproduce(&loaded).unwrap();
    assert!(report.passed(), "{}", report.summary());
    assert_eq!(report.stored_ids_checked, 0);
    assert!(report.summary().contains("NOT VERIFIED"));
    // This fixture was integrated in release mode. The existing comparison
    // tolerance applies across build profiles; bit equality is reported, not
    // required. Same-build archive round trips above still require exact bits.
    assert_eq!(report.hit_tolerance_mismatches, 0);
}
