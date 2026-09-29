//! A single long-lived worker coalesces pending requests. Completed data is immutable.
use super::{SimulationData, calculate, storage};
use crate::session::SessionConfig;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
};
pub enum Request {
    Calculate(Box<SessionConfig>),
    Load(PathBuf),
    CalculateAndSave(Box<SessionConfig>, PathBuf),
    Reproduce {
        directory: PathBuf,
        simulation: String,
        view: Box<SessionConfig>,
    },
}
pub struct Completion {
    pub generator: Option<crate::experiments::stratified::GeneratorSpec>,
    pub generation: u64,
    pub result: Result<Arc<SimulationData>, String>,
    pub archive_message: Option<String>,
    pub comparison: Option<crate::standard_run::Comparison>,
}
pub struct Worker {
    sender: mpsc::Sender<(u64, Request)>,
    pub receiver: mpsc::Receiver<Completion>,
    pub generation: Arc<AtomicU64>,
    pub completed: Arc<AtomicUsize>,
    pub total: Arc<AtomicUsize>,
}
impl Default for Worker {
    fn default() -> Self {
        let (sender, requests) = mpsc::channel::<(u64, Request)>();
        let (finish, receiver) = mpsc::channel();
        let generation = Arc::new(AtomicU64::new(0));
        let completed = Arc::new(AtomicUsize::new(0));
        let total = Arc::new(AtomicUsize::new(0));
        let (g, c, t) = (generation.clone(), completed.clone(), total.clone());
        thread::Builder::new()
            .name("kerr-reference-worker".into())
            .spawn(move || {
                while let Ok(mut request) = requests.recv() {
                    while let Ok(newer) = requests.try_recv() {
                        request = newer;
                    }
                    let (id, request) = request;
                    c.store(0, Ordering::Relaxed);
                    t.store(0, Ordering::Relaxed);
                    let mut archive_message = None;
                    let mut comparison = None;
                    let mut generator = None;
                    let mut progress = |done, count| {
                        c.store(done, Ordering::Relaxed);
                        t.store(count, Ordering::Relaxed);
                        g.load(Ordering::Acquire) == id
                    };
                    let result = match request {
                        Request::Calculate(cfg) => {
                            generator = cfg.generator.clone();
                            calculate_session(&cfg, &mut progress)
                        }
                        Request::Load(path) => storage::load(&path),
                        Request::CalculateAndSave(cfg, root) => {
                            generator = cfg.generator.clone();
                            calculate_and_save(&cfg, &root, &mut progress).map(|(data, message)| {
                                archive_message = Some(message);
                                data
                            })
                        }
                        Request::Reproduce {
                            directory,
                            simulation,
                            view,
                        } => reproduce_scene(&directory, &simulation, &view, &mut progress).map(
                            |(data, report, spec)| {
                                generator = spec;
                                archive_message = Some(format!(
                                    "Reproduced {} / {} (not auto-saved)",
                                    directory.display(),
                                    simulation
                                ));
                                comparison = Some(report);
                                data
                            },
                        ),
                    };
                    if g.load(Ordering::Acquire) == id {
                        let _ = finish.send(Completion {
                            generator,
                            generation: id,
                            result: result.map(Arc::new),
                            archive_message,
                            comparison,
                        });
                    }
                }
            })
            .expect("spawn reference worker");
        Self {
            sender,
            receiver,
            generation,
            completed,
            total,
        }
    }
}
impl Worker {
    pub fn submit(&self, request: Request) -> u64 {
        let id = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let _ = self.sender.send((id, request));
        id
    }
    pub fn cancel(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn calculate_and_save(
    cfg: &SessionConfig,
    root: &std::path::Path,
    mut progress: impl FnMut(usize, usize) -> bool,
) -> Result<(SimulationData, String), String> {
    use crate::{experiments::independent_rays::from_source_plane, standard_run};
    if let Some(spec) = &cfg.generator {
        let timer = std::time::Instant::now();
        let (run, outcomes) = standard_run::seeded::execute_with_outcomes(
            spec,
            crate::experiments::stratified::master_seed()?,
            &simulation_config(cfg),
            &mut progress,
        )
        .map_err(|e| e.to_string())?;
        if !progress(outcomes.len(), outcomes.len()) {
            return Err("cancelled before auto-save".into());
        }
        let data = super::archive::scene(
            outcomes,
            cfg.scene_experiment(),
            true,
            timer.elapsed().as_secs_f64(),
        )?;
        let message = match standard_run::save_new(root, &run) {
            Ok(saved) => format!("Auto-saved: {} (sim_1.bin)", saved.directory.display()),
            Err(e) => format!("Auto-save FAILED: {e}; calculated result retained"),
        };
        println!("{message}");
        return Ok((data, message));
    }
    let inputs = match from_source_plane(&cfg.experiment.source) {
        Ok(inputs) => inputs,
        Err(error) => {
            // Preserve legacy LocalZamo/nonunit-energy experiments. Their inputs
            // cannot be represented by the verified fixed-energy Cartesian API.
            let data = calculate(&cfg.experiment, &mut progress)?;
            return Ok((
                data,
                format!("Auto-save FAILED: {error}; legacy calculation retained"),
            ));
        }
    };
    let e = &cfg.experiment;
    let config = super::rays::RaySimulationConfig {
        spin: e.spin,
        integration: e.integration,
        detector: e.detector.clone(),
    };
    let timer = std::time::Instant::now();
    let (run, outcomes) = standard_run::execute_with_outcomes(inputs, &config, &mut progress)
        .map_err(|e| e.to_string())?;
    let count = outcomes.len();
    if !progress(count, count) {
        return Err("cancelled before auto-save".into());
    }
    let data = super::archive::scene(outcomes, e.clone(), true, timer.elapsed().as_secs_f64())?;
    let message = match standard_run::save_new(root, &run) {
        Ok(saved) => format!("Auto-saved: {}", saved.directory.display()),
        Err(error) => format!("Auto-save FAILED: {error}; calculated result retained"),
    };
    println!("{message}");
    Ok((data, message))
}
fn reproduce_scene(
    directory: &std::path::Path,
    simulation: &str,
    view: &SessionConfig,
    progress: impl FnMut(usize, usize) -> bool,
) -> Result<
    (
        SimulationData,
        crate::standard_run::Comparison,
        Option<crate::experiments::stratified::GeneratorSpec>,
    ),
    String,
> {
    use crate::standard_run;
    let saved = standard_run::load(directory, simulation).map_err(|e| e.to_string())?;
    let config = saved.common.config(saved.chi)?;
    let (report, outcomes) =
        standard_run::reproduce_with_outcomes(&saved, progress).map_err(|e| e.to_string())?;
    let mut exp = view.experiment.clone();
    exp.spin = config.spin;
    exp.integration = config.integration;
    let resolution = exp.detector.resolution;
    exp.detector = config.detector;
    exp.detector.resolution = resolution;
    let mut scene_config = view.clone();
    scene_config.experiment = exp;
    scene_config.generator = saved.common.ray_generator.clone();
    let data = super::archive::scene(
        outcomes,
        scene_config.scene_experiment(),
        scene_config.generator.is_some(),
        report.calculation_seconds,
    )?;
    println!("{}", report.summary());
    Ok((data, report, scene_config.generator))
}

fn simulation_config(cfg: &SessionConfig) -> super::rays::RaySimulationConfig {
    super::rays::RaySimulationConfig {
        spin: cfg.experiment.spin,
        integration: cfg.experiment.integration,
        detector: cfg.experiment.detector.clone(),
    }
}
/// Explicit trajectory-export / non-saving callers share the same initial generator.
pub fn calculate_session(
    cfg: &SessionConfig,
    progress: impl FnMut(usize, usize) -> bool,
) -> Result<SimulationData, String> {
    if let Some(spec) = &cfg.generator {
        let timer = std::time::Instant::now();
        let inputs = spec.generate(crate::experiments::stratified::master_seed()?)?;
        let outcomes = super::rays::calculate_rays(inputs, &simulation_config(cfg), progress)?;
        super::archive::scene(
            outcomes,
            cfg.scene_experiment(),
            true,
            timer.elapsed().as_secs_f64(),
        )
    } else {
        calculate(&cfg.experiment, progress)
    }
}
