use kerr_ray::{
    compute::RayStatus,
    experiments::stratified::{GeneratorSpec, master_seed},
    session::SessionConfig,
    simulation::{
        rays::{RayInitialCondition, RaySimulationConfig},
        worker::{Request, Worker},
    },
    standard_run::{self, RecordedRun, seeded},
};
use std::{fs, path::PathBuf, sync::OnceLock, time::Duration};

fn config() -> RaySimulationConfig {
    let e = SessionConfig::regression().experiment;
    RaySimulationConfig {
        spin: e.spin,
        integration: e.integration,
        detector: e.detector,
    }
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
fn fixture() -> &'static RecordedRun {
    static RUN: OnceLock<RecordedRun> = OnceLock::new();
    RUN.get_or_init(|| {
        seeded::execute_with_outcomes(
            &GeneratorSpec {
                cell_count: [8, 8],
                cell_size: [4., 4.],
                ..Default::default()
            },
            [73; 32],
            &config(),
            |_, _| true,
        )
        .unwrap()
        .0
    })
}
fn temp() -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "kerr-seeded-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&p).unwrap();
    p
}
#[test]
fn default_expands_coverage_preserving_old_physical_density() {
    let old = SessionConfig::regression().experiment.source.plane;
    let g = GeneratorSpec::default();
    assert_eq!([old.width / 64., old.height / 64.], g.cell_size);
    assert_eq!(g.cell_count, [192, 192]);
    assert_eq!(g.cell_size, [0.5, 0.5]);
    assert_eq!(g.extent(), [96., 96.]);
    assert_eq!(g.count().unwrap(), 36_864);
    assert_eq!(g.center, [80., 0., 0.]);
    assert_eq!(g.axis_1, [0., 1., 0.]);
    assert_eq!(g.axis_2, [0., 0., 1.]);
    let session = SessionConfig::default();
    assert_eq!(session.generator.as_ref().unwrap(), &g);
    let scene = session.scene_experiment();
    assert_eq!(
        [scene.source.plane.width, scene.source.plane.height],
        g.extent()
    );
    let json = SessionConfig::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/seeded.json"),
    )
    .unwrap();
    assert_eq!(json.generator.unwrap(), g);
}
#[test]
fn one_ray_in_each_cell_exact_consumption_and_fixed_direction() {
    let g = GeneratorSpec::default();
    for seed in [[0; 32], [1; 32], [255; 32]] {
        let rays = g.generate(seed).unwrap();
        assert_eq!(rays.len(), 36_864);
        for (index, ray) in rays.iter().enumerate() {
            let i = index % g.cell_count[0];
            let j = index / g.cell_count[0];
            assert_eq!(ray.position_x, 80.);
            assert!((i as f64 * 0.5 - 48. ..(i + 1) as f64 * 0.5 - 48.).contains(&ray.position_y));
            assert!((j as f64 * 0.5 - 48. ..(j + 1) as f64 * 0.5 - 48.).contains(&ray.position_z));
            assert_eq!(ray.direction(), [-1., 0., 0.]);
            assert_eq!(ray.t_emit, 0.);
        }
    }
}
#[test]
fn chacha_known_answer_and_canonical_hash_golden() {
    // Zero-key/zero-counter ChaCha20 block vector, independently converted with Python struct/hashlib.
    use rand_core::{RngCore, SeedableRng};
    let mut rng = rand_chacha::ChaCha20Rng::from_seed([0; 32]);
    assert_eq!(rng.next_u64(), 0x903df1a0ade0b876);
    assert_eq!(rng.next_u64(), 0x28bd8653e56a5d40);
    let g = GeneratorSpec {
        cell_count: [2, 2],
        ..Default::default()
    };
    let rays = g.generate([0; 32]).unwrap();
    assert_eq!(
        bits(rays[0]),
        [
            0x4054000000000000,
            0xbfcbf08397d487d2,
            0xbfdae84f358352b5,
            0xbff0000000000000,
            0,
            0,
            0
        ]
    );
    let hex = seeded::initial_hash(rays, &config())
        .unwrap()
        .map(|v| format!("{v:02x}"))
        .join("");
    assert_eq!(
        hex,
        "9e758150552f566b59041361a92538fe632010bb2888814bc11191c6af81e4cc"
    );
}
#[test]
fn repeated_seed_bit_identical_different_seed_differs_and_direction_constant() {
    let g = GeneratorSpec {
        cell_count: [7, 9],
        ..Default::default()
    };
    let a = g.generate([27; 32]).unwrap();
    let b = g.generate([27; 32]).unwrap();
    let c = g.generate([28; 32]).unwrap();
    assert_eq!(
        a.iter().copied().map(bits).collect::<Vec<_>>(),
        b.iter().copied().map(bits).collect::<Vec<_>>()
    );
    assert_ne!(a, c);
    for (a, c) in a.iter().zip(c) {
        assert_eq!(a.direction(), c.direction());
        assert_eq!(a.t_emit, c.t_emit);
    }
}
#[test]
fn many_seeds_have_balanced_within_cell_positions() {
    let g = GeneratorSpec {
        center: [80., 0., 0.],
        cell_count: [1, 1],
        cell_size: [1., 1.],
        ..Default::default()
    };
    let mut hist = [[0usize; 10]; 2];
    let mut mean = [0.; 2];
    for i in 0u64..10000 {
        // Deterministic test corpus, not the production OS seed source.
        use sha2::{Digest, Sha256};
        let seed = Sha256::digest(i.to_le_bytes()).into();
        let c = g.generate(seed).unwrap()[0];
        for (axis, u) in [c.position_y + 0.5, c.position_z + 0.5]
            .into_iter()
            .enumerate()
        {
            mean[axis] += u / 10000.;
            hist[axis][(u * 10.) as usize] += 1;
        }
    }
    for a in 0..2 {
        assert!((mean[a] - 0.5).abs() < 0.015);
        for n in hist[a] {
            assert!((850..=1150).contains(&n), "{hist:?}");
        }
    }
}
#[test]
fn os_seed_uses_all_256_bits_and_generator_has_no_chi_input() {
    let seeds: Vec<_> = (0..16).map(|_| master_seed().unwrap()).collect();
    for i in 0..seeds.len() {
        for j in 0..i {
            assert_ne!(seeds[i], seeds[j]);
        }
    }
    let g = GeneratorSpec {
        cell_count: [2, 2],
        ..Default::default()
    };
    let inputs = g.generate(seeds[0]).unwrap();
    let mut c = config();
    let a = seeded::initial_hash(inputs.clone(), &c).unwrap();
    c.spin = 0.;
    assert_eq!(a, seeded::initial_hash(inputs, &c).unwrap());
}
#[test]
fn invalid_or_unknown_generator_spec_is_rejected() {
    let g = GeneratorSpec::default();
    for mut bad in [g.clone(), g.clone(), g.clone(), g.clone()]
        .into_iter()
        .enumerate()
    {
        match bad.0 {
            0 => bad.1.ray_generator_version = "future".into(),
            1 => bad.1.axis_1 = [0.; 3],
            2 => bad.1.cell_count = [0, 2],
            _ => bad.1.direction_noise = "random".into(),
        };
        assert!(bad.1.generate([0; 32]).is_err());
    }
}
#[test]
fn compact_roundtrip_and_fresh_reproduction_preserve_all_results() {
    let original = fixture();
    let dir = standard_run::save_new(&temp(), original).unwrap().directory;
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
    let loaded = standard_run::load(&dir, "sim_1.bin").unwrap();
    assert_eq!(loaded.seed_metadata.as_ref().unwrap().seed, [73; 32]);
    for (a, b) in original.rows.iter().zip(&loaded.rows) {
        assert_eq!(bits(a.input), bits(b.input));
        assert_eq!(a.status, b.status);
        assert_eq!(
            a.hit.map(|x| x.map(f64::to_bits)),
            b.hit.map(|x| x.map(f64::to_bits))
        );
        assert!(b.ray_id.is_none());
    }
    let mut calls = 0;
    let (report, outcomes) = standard_run::reproduce_with_outcomes(&loaded, |_, _| {
        calls += 1;
        true
    })
    .unwrap();
    assert!(calls > 0);
    assert!(report.passed(), "{}", report.summary());
    assert!(report.initial_conditions_hash_verified);
    assert_eq!(report.hit_bit_mismatches, [0; 3]);
    assert_eq!(report.stored_ids_checked, 0);
    for (i, out) in outcomes.iter().enumerate() {
        assert_eq!(out.ray.ray_id(), i);
        assert!(!out.result.samples.is_empty());
        assert_eq!(out.result.status, loaded.rows[i].status);
    }
}
#[test]
fn binary_layout_packs_only_detected_hits_in_canonical_order() {
    let mut run = fixture().clone();
    let states = [
        RayStatus::Detected,
        RayStatus::Captured,
        RayStatus::Detected,
        RayStatus::Escaped,
        RayStatus::Active,
        RayStatus::NumericalFailure,
    ];
    for (i, row) in run.rows.iter_mut().enumerate() {
        row.status = states[i % 6];
        row.hit = (row.status == RayStatus::Detected).then_some([i as f64, 0., 200. + i as f64]);
    }
    let report = standard_run::save_new(&temp(), &run).unwrap();
    let k = run.rows.iter().filter(|r| r.hit.is_some()).count();
    assert_eq!(
        report.binary_bytes,
        72 + run.rows.len() as u64 + 24 * k as u64
    );
    let raw = fs::read(report.directory.join("sim_1.bin")).unwrap();
    assert_eq!(&raw[..8], run.chi.to_le_bytes());
    assert_eq!(&raw[8..40], [73; 32]);
    assert_eq!(&raw[72..78], [1, 2, 1, 3, 0, 4]);
    let loaded = standard_run::load(&report.directory, "sim_1.bin").unwrap();
    for (a, b) in run.rows.iter().zip(loaded.rows) {
        assert_eq!(a.status, b.status);
        assert_eq!(a.hit, b.hit);
    }
    let common = fs::read_to_string(report.directory.join("common.json")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&common).unwrap();
    for excluded in [
        "resolution",
        "camera",
        "free_fall",
        "ray_id",
        "position_x",
        "energy",
    ] {
        assert!(!value.as_object().unwrap().contains_key(excluded));
    }
    assert!(value["detector"].get("resolution").is_none());
}
#[test]
fn bit_flipped_seed_hash_or_changed_generator_stops_before_integration() {
    for mutation in 0..3 {
        let mut run = fixture().clone();
        match mutation {
            0 => run.seed_metadata.as_mut().unwrap().seed[31] ^= 1,
            1 => run.seed_metadata.as_mut().unwrap().initial_conditions_hash[0] ^= 1,
            _ => run.common.ray_generator.as_mut().unwrap().cell_size[0] += 0.01,
        }
        let mut calls = 0;
        let result = standard_run::reproduce_with_outcomes(&run, |_, _| {
            calls += 1;
            true
        });
        assert!(result.unwrap_err().to_string().contains("HASH MISMATCH"));
        assert_eq!(calls, 0);
    }
}
#[test]
fn seed_inputs_are_regenerated_instead_of_using_cached_rows_or_ids() {
    let mut run = fixture().clone();
    run.rows[0].input.position_y += 0.01;
    let mut called = false;
    assert!(
        standard_run::reproduce_with_outcomes(&run, |_, _| {
            called = true;
            true
        })
        .is_err()
    );
    assert!(!called);
    let mut run = fixture().clone();
    run.rows.swap(0, 1);
    assert!(run.validate().is_err());
}
#[test]
fn non_axis_fixed_direction_is_not_double_normalized_during_replay() {
    let g = GeneratorSpec {
        cell_count: [2, 2],
        cell_size: [5., 5.],
        fixed_direction: [-1.7, 0.04, 0.03],
        t_emit: 2.25,
        ..Default::default()
    };
    let (run, _) = seeded::execute_with_outcomes(&g, [18; 32], &config(), |_, _| true).unwrap();
    let dir = standard_run::save_new(&temp(), &run).unwrap().directory;
    let report = standard_run::reproduce(&standard_run::load(&dir, "sim_1.bin").unwrap()).unwrap();
    assert!(report.passed());
    assert_eq!(report.hit_bit_mismatches, [0; 3]);
}
#[test]
fn loader_rejects_corruption_and_sorts_multiple_simulations_numerically() {
    let dir = standard_run::save_new(&temp(), fixture())
        .unwrap()
        .directory;
    for n in [10, 3, 2] {
        fs::copy(dir.join("sim_1.bin"), dir.join(format!("sim_{n}.bin"))).unwrap();
    }
    assert_eq!(
        standard_run::list_simulations(&dir).unwrap(),
        ["sim_1.bin", "sim_2.bin", "sim_3.bin", "sim_10.bin"]
    );
    let original = fs::read(dir.join("sim_1.bin")).unwrap();
    for kind in 0..4 {
        let mut raw = original.clone();
        match kind {
            0 => raw[72] = 9,
            1 => raw[8] ^= 1,
            2 => {
                raw.pop();
            }
            _ => raw.push(0),
        };
        fs::write(dir.join("sim_2.bin"), raw).unwrap();
        assert!(standard_run::load(&dir, "sim_2.bin").is_err());
    }
    assert!(standard_run::load(&dir, "../sim_1.bin").is_err());
}
#[test]
fn session_json_and_physics_key_use_generator_without_sourceplane() {
    let a = SessionConfig::default();
    let mut b = a.clone();
    b.experiment.source.plane.width = 1.;
    b.view.free_fall.visible = true;
    b.experiment.detector.resolution = [512, 512];
    assert_eq!(a.physics_key(), b.physics_key());
    b.generator.as_mut().unwrap().cell_count[0] += 1;
    assert_ne!(a.physics_key(), b.physics_key());
    let decoded: SessionConfig = serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
    assert_eq!(decoded.generator, a.generator);
    let mut old = serde_json::to_value(SessionConfig::regression()).unwrap();
    old.as_object_mut().unwrap().remove("generator");
    let old: SessionConfig = serde_json::from_value(old).unwrap();
    assert!(old.generator.is_none());
}
#[test]
fn worker_new_seed_autosaves_once_per_run_and_reproduces_without_saving() {
    let root = temp();
    let w = Worker::default();
    let mut cfg = SessionConfig::default();
    cfg.generator.as_mut().unwrap().cell_count = [2, 2];
    for _ in 0..2 {
        w.submit(Request::CalculateAndSave(
            Box::new(cfg.clone()),
            root.clone(),
        ));
        let c = w.receiver.recv_timeout(Duration::from_secs(30)).unwrap();
        assert!(c.result.is_ok(), "{:?}", c.result.err());
        assert!(c.archive_message.unwrap().contains("sim_1.bin"));
    }
    let a = standard_run::load(&root.join("simulation_1"), "sim_1.bin").unwrap();
    let b = standard_run::load(&root.join("simulation_2"), "sim_1.bin").unwrap();
    assert_ne!(a.seed_metadata.unwrap().seed, b.seed_metadata.unwrap().seed);
    w.submit(Request::Reproduce {
        directory: root.join("simulation_1"),
        simulation: "sim_1.bin".into(),
        view: Box::new(cfg),
    });
    let c = w.receiver.recv_timeout(Duration::from_secs(30)).unwrap();
    assert!(c.result.is_ok());
    assert!(c.comparison.unwrap().passed());
    assert!(c.generator.is_some());
    assert_eq!(fs::read_dir(root).unwrap().count(), 2);
}

#[test]
fn seeded_cli_saves_loads_and_reproduces_default_binary_selection() {
    let root = temp();
    let exe = env!("CARGO_BIN_EXE_kerr-ray");
    let output = std::process::Command::new(exe)
        .args([
            "simulate-save",
            "--grid",
            "2",
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
    assert!(String::from_utf8_lossy(&output.stdout).contains("sim_1.bin bytes"));
    let dir = root.join("simulation_1");
    for command in ["inspect-simulation", "reproduce"] {
        let output = std::process::Command::new(exe)
            .args([command, "--input", dir.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if command == "reproduce" {
            assert!(
                String::from_utf8_lossy(&output.stdout)
                    .contains("Initial conditions SHA-256: PASS")
            );
        }
    }
    assert_eq!(fs::read_dir(root).unwrap().count(), 1);
}
