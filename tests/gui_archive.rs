use kerr_ray::{
    experiments::{independent_rays::from_source_plane, source_detector::SourcePattern},
    session::{SessionConfig, physics_key},
    simulation::{
        rays::RaySimulationConfig,
        worker::{Completion, Request, Worker},
    },
    standard_run,
};
use std::{fs, path::PathBuf, time::Duration};
fn temp() -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "kerr-gui-archive-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&p).unwrap();
    p
}
fn config() -> SessionConfig {
    let mut cfg = SessionConfig::regression();
    cfg.experiment.source.pattern = SourcePattern::RectangularGrid { nu: 2, nv: 2 };
    cfg
}
fn wait(w: &Worker) -> Completion {
    w.receiver.recv_timeout(Duration::from_secs(30)).unwrap()
}
fn calculate(w: &Worker, cfg: &SessionConfig, root: &std::path::Path) -> Completion {
    w.submit(Request::CalculateAndSave(
        Box::new(cfg.clone()),
        root.into(),
    ));
    wait(w)
}
#[test]
fn completed_calculations_auto_save_once_each_and_display_changes_do_not_submit_work() {
    let root = temp();
    let cfg = config();
    let w = Worker::default();
    for n in 1..=2 {
        let done = calculate(&w, &cfg, &root);
        let data = done.result.unwrap();
        assert!(done.archive_message.unwrap().contains("Auto-saved:"));
        assert_eq!(data.rays.len(), 4);
        assert!(data.rays.iter().all(|r| !r.samples.is_empty()));
        let dir = root.join(format!("simulation_{n}"));
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
        let loaded = standard_run::load(&dir, "sim_1.parquet").unwrap();
        assert_eq!(standard_run::counts(&loaded.rows), data.counts());
        for ray in &data.rays {
            let row = loaded
                .rows
                .iter()
                .find(|r| r.ray_id == Some(ray.ray_id as u64))
                .unwrap();
            assert_eq!(row.status, ray.status);
        }
    }
    let preserved = fs::read(root.join("simulation_1/sim_1.parquet")).unwrap();
    let mut display = cfg.clone();
    display.camera.distance *= 2.;
    display.view.max_rendered_trajectories = 32;
    display.view.trajectory_thickness = 4.;
    display.view.visibility[10] = false;
    display.view.grid_spacing = 7.;
    display.view.playback_speed = 3.;
    display.view.detector_overlay = false;
    assert_eq!(
        physics_key(&cfg.experiment),
        physics_key(&display.experiment)
    );
    assert!(w.receiver.recv_timeout(Duration::from_millis(40)).is_err());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    assert_eq!(
        fs::read(root.join("simulation_1/sim_1.parquet")).unwrap(),
        preserved
    );
}
#[test]
fn auto_save_failure_keeps_full_calculated_result() {
    let root = temp().join("not_a_directory");
    fs::write(&root, b"preserve").unwrap();
    let done = calculate(&Worker::default(), &config(), &root);
    assert!(done.archive_message.unwrap().contains("Auto-save FAILED"));
    let data = done.result.unwrap();
    data.validate().unwrap();
    assert_eq!(data.rays.len(), 4);
    assert!(data.rays.iter().all(|r| !r.samples.is_empty()));
    assert_eq!(fs::read(root).unwrap(), b"preserve");
}
#[test]
fn folder_discovery_is_numeric_and_ignores_invalid_names_and_directories() {
    let root = temp();
    let w = Worker::default();
    calculate(&w, &config(), &root).result.unwrap();
    let dir = root.join("simulation_1");
    assert_eq!(
        standard_run::list_simulations(&dir).unwrap(),
        ["sim_1.parquet"]
    );
    for name in [
        "sim_10.parquet",
        "sim_3.parquet",
        "sim_2.parquet",
        "sim_0.parquet",
        "sim_01.parquet",
        "sim_-1.parquet",
        "sim_1.parquet.bak",
        "other.parquet",
    ] {
        fs::copy(dir.join("sim_1.parquet"), dir.join(name)).unwrap();
    }
    fs::create_dir(dir.join("sim_4.parquet")).unwrap();
    assert_eq!(
        standard_run::list_simulations(&dir).unwrap(),
        [
            "sim_1.parquet",
            "sim_2.parquet",
            "sim_3.parquet",
            "sim_10.parquet"
        ]
    );
    assert!(standard_run::list_simulations(&temp()).is_err());
    fs::write(dir.join("common.json"), b"{}").unwrap();
    assert!(standard_run::list_simulations(&dir).is_err());
}
#[test]
fn selected_simulation_reproduces_real_trajectories_without_auto_save() {
    let root = temp();
    let w = Worker::default();
    let cfg = config();
    calculate(&w, &cfg, &root).result.unwrap();
    let mut inputs = from_source_plane(&cfg.experiment.source).unwrap();
    for c in &mut inputs {
        c.t_emit = 7.25;
    }
    let e = &cfg.experiment;
    let c = RaySimulationConfig {
        spin: e.spin,
        integration: e.integration,
        detector: e.detector.clone(),
    };
    let run = standard_run::execute(inputs, &c).unwrap();
    let second = standard_run::save_new(&root, &run).unwrap().directory;
    let first = root.join("simulation_1");
    fs::copy(second.join("sim_1.parquet"), first.join("sim_2.parquet")).unwrap();
    let original = fs::read(first.join("sim_2.parquet")).unwrap();
    w.submit(Request::Reproduce {
        directory: first.clone(),
        simulation: "sim_2.parquet".into(),
        view: Box::new(cfg),
    });
    let done = wait(&w);
    let report = done.comparison.unwrap();
    assert!(report.passed(), "{}", report.summary());
    assert_eq!(report.stored_ids_checked, 4);
    assert_eq!(report.hit_bit_mismatches, [0; 3]);
    let data = done.result.unwrap();
    assert!(!data.source_available);
    assert!(
        data.rays
            .iter()
            .all(|r| r.initial[0] == 7.25 && r.samples.len() > 1 && r.launch_direction.is_some())
    );
    assert_eq!(data.events.len(), report.actual_counts[1]);
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    assert_eq!(fs::read(first.join("sim_2.parquet")).unwrap(), original);
}
#[test]
fn invalid_selected_file_is_reported_and_worker_can_continue() {
    let root = temp();
    let w = Worker::default();
    let cfg = config();
    calculate(&w, &cfg, &root).result.unwrap();
    let dir = root.join("simulation_1");
    fs::write(dir.join("sim_2.parquet"), b"invalid").unwrap();
    w.submit(Request::Reproduce {
        directory: dir,
        simulation: "sim_2.parquet".into(),
        view: Box::new(cfg.clone()),
    });
    assert!(wait(&w).result.is_err());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    assert!(calculate(&w, &cfg, &root).result.is_ok());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
}
