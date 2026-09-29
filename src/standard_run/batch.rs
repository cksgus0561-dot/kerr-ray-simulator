//! Sequential simulations using the existing ray-parallel single-run engine.
//! Only one simulation's trajectories are held in memory at a time.
use super::{Common, Result, counts, seeded};
use crate::{
    experiments::{
        chi_sampling::{ChiSampling, POSITIVE_INTERVAL_COUNT, chi_at},
        stratified::master_seed,
    },
    session::SessionConfig,
    simulation::rays::RaySimulationConfig,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchSpecification {
    pub chi_sampling: ChiSampling,
    /// Required from the caller; there is deliberately no production default.
    pub simulations_per_chi: usize,
    /// Full grid in production; explicit ascending subset for small validation.
    /// File number = selected-index position * simulations_per_chi + repetition + 1.
    pub chi_indices: Vec<i32>,
}
impl BatchSpecification {
    pub fn full(simulations_per_chi: usize) -> Self {
        Self {
            chi_sampling: ChiSampling::default(),
            simulations_per_chi,
            chi_indices: (-POSITIVE_INTERVAL_COUNT..=POSITIVE_INTERVAL_COUNT).collect(),
        }
    }
    pub fn validate(&self) -> std::result::Result<(), String> {
        self.chi_sampling.validate()?;
        if self.simulations_per_chi == 0 || self.chi_indices.is_empty() {
            return Err("simulations_per_chi and chi selection must be nonzero".into());
        }
        for &index in &self.chi_indices {
            chi_at(index)?;
        }
        if !self.chi_indices.windows(2).all(|w| w[0] < w[1]) {
            return Err("chi indices must be strictly increasing without duplicates".into());
        }
        self.chi_indices
            .len()
            .checked_mul(self.simulations_per_chi)
            .ok_or("batch simulation count overflow")?;
        Ok(())
    }
    pub fn simulation_count(&self) -> std::result::Result<usize, String> {
        self.validate()?;
        Ok(self.chi_indices.len() * self.simulations_per_chi)
    }
    /// A 1-based file number identifies the planned spin and repetition only.
    /// It is NEVER used as entropy or as a seed for ray generation.
    pub fn entry(&self, number: usize) -> std::result::Result<BatchEntry, String> {
        if number == 0 || number > self.simulation_count()? {
            return Err("simulation number outside batch plan".into());
        }
        let index = self.chi_indices[(number - 1) / self.simulations_per_chi];
        Ok(BatchEntry {
            number,
            chi_index: index,
            chi: chi_at(index)?,
            repetition: (number - 1) % self.simulations_per_chi + 1,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BatchEntry {
    pub number: usize,
    pub chi_index: i32,
    pub chi: f64,
    pub repetition: usize,
}
#[derive(Debug, Serialize)]
pub struct SimulationReport {
    pub entry: BatchEntry,
    pub seed: [u8; 32],
    pub initial_conditions_hash: [u8; 32],
    pub rays: usize,
    /// ACT / DET / CAP / ESC / NUM; failures are recorded, never reclassified.
    pub counts: [usize; 5],
    pub calculation_seconds: f64,
    pub binary_bytes: u64,
}
#[derive(Debug)]
pub struct BatchReport {
    pub directory: PathBuf,
    pub simulations: Vec<SimulationReport>,
}

pub fn execute(
    root: &Path,
    session: &SessionConfig,
    spec: &BatchSpecification,
    mut completed: impl FnMut(&SimulationReport),
) -> Result<BatchReport> {
    let total = spec.simulation_count()?;
    let generator = session
        .generator
        .as_ref()
        .ok_or("batch requires the seeded generator")?;
    generator.validate()?;
    let e = &session.experiment;
    let mut common = Common::from_config(&RaySimulationConfig {
        spin: e.spin,
        integration: e.integration,
        detector: e.detector.clone(),
    });
    common.ray_generator = Some(generator.clone());
    common.binary_format = Some(seeded::BinaryFormat::default());
    common.data_format_version = seeded::FORMAT_VERSION.into();
    common.batch = Some(spec.clone());
    // Validate the whole plan before reserving a directory or generating a seed.
    for &index in &spec.chi_indices {
        common.config(chi_at(index)?)?;
    }
    let directory = super::reserve(root)?;
    let mut json = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("common.json"))?,
    );
    serde_json::to_writer_pretty(&mut json, &common)?;
    json.flush()?;
    json.get_ref().sync_all()?;
    let mut simulations = Vec::new();
    for number in 1..=total {
        let entry = spec.entry(number)?;
        // Exactly the existing OS entropy source, independent of chi and number.
        let seed = master_seed()?;
        let config = common.config(entry.chi)?;
        let timer = Instant::now();
        let (mut run, outcomes) =
            seeded::execute_with_outcomes(generator, seed, &config, |_, _| true)?;
        let calculation_seconds = timer.elapsed().as_secs_f64();
        drop(outcomes);
        run.common = common.clone();
        let path = directory.join(format!("sim_{number}.bin"));
        seeded::write_binary(&path, &run)?;
        let report = SimulationReport {
            entry,
            seed,
            initial_conditions_hash: run
                .seed_metadata
                .as_ref()
                .expect("seeded run")
                .initial_conditions_hash,
            rays: run.rows.len(),
            counts: counts(&run.rows),
            calculation_seconds,
            binary_bytes: fs::metadata(path)?.len(),
        };
        completed(&report);
        simulations.push(report);
    }
    Ok(BatchReport {
        directory,
        simulations,
    })
}
