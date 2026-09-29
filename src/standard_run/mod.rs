//! Standard input/results archive and fresh-physics reproduction. Legacy exports
//! remain separate. No saved State, trajectory, ray ID or pixel data is an input.
pub mod batch;
pub mod batch_cli;
pub mod cli;
mod common;
mod parquet_io;
mod reproduce;
pub mod seeded;
use crate::{
    compute::RayStatus,
    simulation::rays::{
        RayInitialCondition, RayOutcome, RaySimulationConfig, calculate_rays, prepare_rays,
    },
};
pub use common::{Common, Convention, FORMAT_VERSION};
pub use reproduce::{Comparison, reproduce, reproduce_with_outcomes};
use std::{
    collections::{HashMap, VecDeque},
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Clone, Debug)]
pub struct RayRow {
    /// Canonical ID produced by the solver, never an input to it. None is only
    /// for reading historical archives whose schema had no ray_id column.
    pub ray_id: Option<u64>,
    /// Original unnormalized f64 input; not PreparedRay::condition().
    pub input: RayInitialCondition,
    pub energy: f64,
    pub status: RayStatus,
    pub hit: Option<[f64; 3]>,
}
#[derive(Clone, Debug)]
pub struct RecordedRun {
    pub common: Common,
    pub chi: f64,
    pub rows: Vec<RayRow>,
    pub seed_metadata: Option<seeded::SeedMetadata>,
}
impl RecordedRun {
    pub fn validate(&self) -> Result<()> {
        let config = self.common.config(self.chi)?;
        if self.seed_metadata.is_some() {
            seeded::verify(self)?;
        } else if self.common.ray_generator.is_some() {
            return Err("missing seed metadata".into());
        }
        prepare_rays(self.rows.iter().map(|r| r.input).collect(), &config)?;
        let id_count = self.rows.iter().filter(|r| r.ray_id.is_some()).count();
        if id_count != 0 && id_count != self.rows.len() {
            return Err("mixed present/missing ray_id values".into());
        }
        let mut seen = vec![false; self.rows.len()];
        for r in &self.rows {
            if let Some(id) = r.ray_id {
                let index = usize::try_from(id).map_err(|_| "ray_id out of range")?;
                let slot = seen.get_mut(index).ok_or("ray_id out of range")?;
                if *slot {
                    return Err("duplicate ray_id".into());
                }
                *slot = true;
            }
            if r.energy.to_bits() != 1.0_f64.to_bits() {
                return Err("only local energy=1 is supported".into());
            }
            if (r.status == RayStatus::Detected) != r.hit.is_some() {
                return Err("status/hit nullability mismatch".into());
            }
            if let Some(hit) = r.hit
                && (hit.iter().any(|v| !v.is_finite())
                    || hit[2] < r.input.t_emit
                    || !config.detector.plane.contains([hit[0], hit[1]]))
            {
                return Err("invalid detector hit".into());
            }
        }
        Ok(())
    }
}
pub(crate) fn key(c: RayInitialCondition) -> [u64; 7] {
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
/// Match original rows to canonical conditions without injecting row indices as IDs.
/// Identical canonical duplicates form a queue; each still contributes one ray.
pub(crate) fn grouped(
    rows: &[RayRow],
    config: &RaySimulationConfig,
) -> Result<HashMap<[u64; 7], VecDeque<usize>>> {
    let mut groups: HashMap<_, VecDeque<_>> = HashMap::new();
    for (i, row) in rows.iter().enumerate() {
        let singleton = prepare_rays(vec![row.input], config)?;
        groups
            .entry(key(singleton[0].condition()))
            .or_default()
            .push_back(i);
    }
    Ok(groups)
}
/// Run once using only raw inputs/config. Attach outcomes to their physical input,
/// preserving all original bits, including direction length and signed zeros.
pub fn execute(
    inputs: Vec<RayInitialCondition>,
    config: &RaySimulationConfig,
) -> Result<RecordedRun> {
    execute_with_outcomes(inputs, config, |_, _| true).map(|(run, _)| run)
}
/// Same calculation, retaining its already computed trajectories for the viewer.
pub fn execute_with_outcomes(
    inputs: Vec<RayInitialCondition>,
    config: &RaySimulationConfig,
    keep_going: impl FnMut(usize, usize) -> bool,
) -> Result<(RecordedRun, Vec<RayOutcome>)> {
    let mut rows: Vec<_> = inputs
        .iter()
        .map(|c| RayRow {
            ray_id: None,
            input: *c,
            energy: 1.0,
            status: RayStatus::Active,
            hit: None,
        })
        .collect();
    let mut groups = grouped(&rows, config)?;
    let outcomes = calculate_rays(inputs, config, keep_going)?;
    for out in &outcomes {
        let i = groups
            .get_mut(&key(out.ray.condition()))
            .and_then(VecDeque::pop_front)
            .ok_or("canonical input association failed")?;
        rows[i].ray_id = Some(u64::try_from(out.ray.ray_id())?);
        rows[i].status = out.result.status;
        rows[i].hit = out
            .detection()
            .map(|d| [d.hit.uv[0], d.hit.uv[1], d.hit.state.time()]);
    }
    let run = RecordedRun {
        common: Common::from_config(config),
        chi: config.spin,
        rows,
        seed_metadata: None,
    };
    run.validate()?;
    Ok((run, outcomes))
}
pub fn counts(rows: &[RayRow]) -> [usize; 5] {
    let mut c = [0; 5];
    for r in rows {
        c[crate::experiments::source_detector::status_index(r.status)] += 1;
    }
    c
}
/// Atomically reserve the first unused positive directory number. Existing files
/// and directories are never replaced, including simultaneous writers.
fn reserve(root: &Path) -> Result<PathBuf> {
    fs::create_dir_all(root)?;
    for n in 1..=u64::MAX {
        let p = root.join(format!("simulation_{n}"));
        match fs::create_dir(&p) {
            Ok(()) => return Ok(p),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Err("simulation numbering exhausted".into())
}
pub struct SaveReport {
    pub directory: PathBuf,
    pub seconds: f64,
    pub common_bytes: u64,
    pub parquet_bytes: u64,
    pub binary_bytes: u64,
}
/// Each new run reserves its own directory. Failed writes remain reserved and
/// cannot overwrite a prior archive; loaders reject incomplete files.
pub fn save_new(root: &Path, run: &RecordedRun) -> Result<SaveReport> {
    run.validate()?;
    if run.seed_metadata.is_some() {
        return seeded::save_new(root, run);
    }
    if run.rows.iter().any(|r| r.ray_id.is_none()) {
        return Err("new archives require solver-generated ray_id on every row".into());
    }
    let timer = Instant::now();
    let dir = reserve(root)?;
    let mut writer = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("common.json"))?,
    );
    serde_json::to_writer_pretty(&mut writer, &run.common)?;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    parquet_io::write(&dir.join("sim_1.parquet"), run)?;
    Ok(SaveReport {
        seconds: timer.elapsed().as_secs_f64(),
        common_bytes: fs::metadata(dir.join("common.json"))?.len(),
        parquet_bytes: fs::metadata(dir.join("sim_1.parquet"))?.len(),
        binary_bytes: 0,
        directory: dir,
    })
}
/// A numbered basename selects a single simulation, never an arbitrary path.
pub fn load(directory: &Path, simulation: &str) -> Result<RecordedRun> {
    if simulation.ends_with(".bin") {
        return seeded::load(directory, simulation);
    }
    let n = simulation
        .strip_prefix("sim_")
        .and_then(|s| s.strip_suffix(".parquet"))
        .ok_or("expected sim_N.parquet")?;
    let number = n.parse::<u64>()?;
    if number == 0 || number.to_string() != n {
        return Err("simulation number must be positive without padding".into());
    }
    let common: Common =
        serde_json::from_reader(BufReader::new(File::open(directory.join("common.json"))?))?;
    let (chi, rows) = parquet_io::read(&directory.join(simulation))?;
    let run = RecordedRun {
        common,
        chi,
        rows,
        seed_metadata: None,
    };
    run.validate()?;
    Ok(run)
}

/// Discover numbered files only; loading a selection performs full schema validation.
/// Parsing common.json here makes a missing/broken folder fail before selection.
pub fn list_simulations(directory: &Path) -> Result<Vec<String>> {
    let common: Common =
        serde_json::from_reader(BufReader::new(File::open(directory.join("common.json"))?))?;
    common.validate_format()?;
    if common.convention != Convention::default() {
        return Err("unsupported common.json version/convention".into());
    }
    let mut files = Vec::new();
    let extension = if common.ray_generator.is_some() {
        ".bin"
    } else {
        ".parquet"
    };
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(n) = name
            .strip_prefix("sim_")
            .and_then(|s| s.strip_suffix(extension))
            && !n.is_empty()
            && n.bytes().all(|c| c.is_ascii_digit())
            && let Ok(number) = n.parse::<u64>()
            && number > 0
            && number.to_string() == n
        {
            files.push((number, name));
        }
    }
    files.sort_by_key(|(n, _)| *n);
    if files.is_empty() {
        return Err(format!("no sim_<positive integer>{extension} files in folder").into());
    }
    Ok(files.into_iter().map(|(_, name)| name).collect())
}
