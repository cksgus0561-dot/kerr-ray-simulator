//! Compact raw storage. In-memory rows are reconstructed, never written as inputs.
use super::{Common, RayRow, RecordedRun, Result, SaveReport, key};
use crate::{
    compute::RayStatus,
    experiments::stratified::GeneratorSpec,
    simulation::rays::{RayInitialCondition, RayOutcome, RaySimulationConfig, prepare_rays},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    time::Instant,
};

pub const FORMAT_VERSION: &str = "2-seeded-bin";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryFormat {
    pub endianness: String,
    pub layout: String,
    pub chi_dtype: String,
    pub seed_dtype: String,
    pub hash_dtype: String,
    pub status_dtype: String,
    pub hits_dtype: String,
    pub status_codes: [String; 5],
    pub hit_columns: [String; 3],
    pub hash_algorithm: String,
    pub hash_sequence: String,
}
impl Default for BinaryFormat {
    fn default() -> Self {
        Self {
            endianness:"little".into(), layout:"chi[8]; seed[32]; initial_conditions_hash[32]; status[N]; detected_hits[K][3]; no padding/header/trailer; N=cell_count[0]*cell_count[1]; K=count(status==1); hits follow DET order".into(),
            chi_dtype:"float64 IEEE-754".into(),seed_dtype:"uint8[32], unchanged OS bytes".into(),hash_dtype:"uint8[32], SHA-256 digest byte order".into(),
            status_dtype:"uint8".into(),hits_dtype:"float64 IEEE-754".into(),
            status_codes:["0=ACT","1=DET","2=CAP","3=ESC","4=NUM"].map(str::to_owned),
            hit_columns:["u_hit","v_hit","t_hit (BL coordinate time / M)"].map(str::to_owned),
            hash_algorithm:"SHA-256".into(),
            hash_sequence:"prepare_rays canonical sequence; per ray position_x,y,z,direction_x,y,z,t_emit; normalized direction and +0; each IEEE-754 f64 bits as 8 little-endian bytes; no IDs/count/separators".into(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct SeedMetadata {
    pub seed: [u8; 32],
    pub initial_conditions_hash: [u8; 32],
}

/// Compute the fingerprint of the actual canonical conditions, before integration.
pub fn initial_hash(
    inputs: Vec<RayInitialCondition>,
    cfg: &RaySimulationConfig,
) -> Result<[u8; 32]> {
    let prepared = prepare_rays(inputs, cfg)?;
    let mut h = Sha256::new();
    for p in prepared {
        for bits in key(p.condition()) {
            h.update(bits.to_le_bytes());
        }
    }
    Ok(h.finalize().into())
}

/// Regenerate *raw* fixed directions, not twice-normalized PreparedRay directions.
/// Ordering is taken from existing prepare_rays; no saved ID participates.
fn canonical_inputs(
    spec: &GeneratorSpec,
    seed: [u8; 32],
    cfg: &RaySimulationConfig,
) -> Result<Vec<RayInitialCondition>> {
    Ok(prepare_rays(spec.generate(seed)?, cfg)?
        .into_iter()
        .map(|p| {
            let mut c = p.condition();
            [c.direction_x, c.direction_y, c.direction_z] = spec.fixed_direction;
            c
        })
        .collect())
}
/// Verify again at reproduction time. Returns generator inputs, never stored row inputs.
pub fn verify(run: &RecordedRun) -> Result<Vec<RayInitialCondition>> {
    let cfg = run.common.config(run.chi)?;
    let spec = run
        .common
        .ray_generator
        .as_ref()
        .ok_or("missing generator")?;
    let meta = run.seed_metadata.as_ref().ok_or("missing seed/hash")?;
    let inputs = canonical_inputs(spec, meta.seed, &cfg)?;
    if initial_hash(inputs.clone(), &cfg)? != meta.initial_conditions_hash {
        return Err("INITIAL CONDITIONS HASH MISMATCH: initial-condition reproduction failed; geodesic integration not started".into());
    }
    if inputs.len() != run.rows.len() {
        return Err("seeded result row count mismatch".into());
    }
    // Results remain bound to canonical order; IDs are neither stored nor consulted.
    for (input, row) in inputs.iter().zip(&run.rows) {
        if key(*input) != key(row.input) {
            return Err("seeded canonical row association mismatch".into());
        }
    }
    Ok(inputs)
}
pub fn execute_with_outcomes(
    spec: &GeneratorSpec,
    seed: [u8; 32],
    cfg: &RaySimulationConfig,
    progress: impl FnMut(usize, usize) -> bool,
) -> Result<(RecordedRun, Vec<RayOutcome>)> {
    let inputs = spec.generate(seed)?;
    let hash = initial_hash(inputs.clone(), cfg)?;
    // Reuse association + integration without changing the physical engine.
    let (mut run, outcomes) = super::execute_with_outcomes(inputs, cfg, progress)?;
    run.rows.sort_by_key(|r| r.ray_id);
    // Canonical raw inputs make binary and in-memory order precisely identical.
    let ordered = canonical_inputs(spec, seed, cfg)?;
    for (row, input) in run.rows.iter_mut().zip(ordered) {
        row.input = input;
        row.ray_id = None;
    }
    run.common.ray_generator = Some(spec.clone());
    run.common.binary_format = Some(BinaryFormat::default());
    run.common.data_format_version = FORMAT_VERSION.into();
    run.seed_metadata = Some(SeedMetadata {
        seed,
        initial_conditions_hash: hash,
    });
    run.validate()?;
    Ok((run, outcomes))
}
pub fn status_code(status: RayStatus) -> u8 {
    crate::experiments::source_detector::status_index(status) as u8
}
fn status_from(code: u8) -> Result<RayStatus> {
    Ok(match code {
        0 => RayStatus::Active,
        1 => RayStatus::Detected,
        2 => RayStatus::Captured,
        3 => RayStatus::Escaped,
        4 => RayStatus::NumericalFailure,
        _ => return Err("invalid binary status code".into()),
    })
}
pub(super) fn save_new(root: &Path, run: &RecordedRun) -> Result<SaveReport> {
    let timer = Instant::now();
    let dir = super::reserve(root)?;
    let mut json = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("common.json"))?,
    );
    serde_json::to_writer_pretty(&mut json, &run.common)?;
    json.flush()?;
    json.get_ref().sync_all()?;
    write_binary(&dir.join("sim_1.bin"), run)?;
    Ok(SaveReport {
        seconds: timer.elapsed().as_secs_f64(),
        common_bytes: fs::metadata(dir.join("common.json"))?.len(),
        parquet_bytes: 0,
        binary_bytes: fs::metadata(dir.join("sim_1.bin"))?.len(),
        directory: dir,
    })
}
/// The existing binary encoder, shared by single-run and batch archives.
/// Caller validates the run. create_new prevents replacing any saved result.
pub(super) fn write_binary(path: &Path, run: &RecordedRun) -> Result<()> {
    let mut w = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    let meta = run.seed_metadata.as_ref().ok_or("missing seed")?;
    w.write_all(&run.chi.to_le_bytes())?;
    w.write_all(&meta.seed)?;
    w.write_all(&meta.initial_conditions_hash)?;
    for r in &run.rows {
        w.write_all(&[status_code(r.status)])?;
    }
    for r in &run.rows {
        if let Some(hit) = r.hit {
            for x in hit {
                w.write_all(&x.to_le_bytes())?;
            }
        }
    }
    w.flush()?;
    w.get_ref().sync_all()?;
    Ok(())
}
fn read_array<const N: usize>(r: &mut impl Read) -> Result<[u8; N]> {
    let mut b = [0; N];
    r.read_exact(&mut b)?;
    Ok(b)
}
pub(super) fn load(directory: &Path, simulation: &str) -> Result<RecordedRun> {
    let n = simulation
        .strip_prefix("sim_")
        .and_then(|s| s.strip_suffix(".bin"))
        .ok_or("expected sim_N.bin")?;
    let number = n.parse::<u64>()?;
    if number == 0 || number.to_string() != n {
        return Err("simulation number must be positive without padding".into());
    }
    let common: Common =
        serde_json::from_reader(BufReader::new(File::open(directory.join("common.json"))?))?;
    common.validate_format()?;
    let spec = common
        .ray_generator
        .as_ref()
        .ok_or("binary file requires seeded common.json")?;
    let count = spec.count()?;
    let file = File::open(directory.join(simulation))?;
    let len = file.metadata()?.len();
    let mut r = BufReader::new(file);
    let chi = f64::from_le_bytes(read_array(&mut r)?);
    if let Some(batch) = &common.batch {
        let ordinal = usize::try_from(number).map_err(|_| "batch number overflow")?;
        if chi.to_bits() != batch.entry(ordinal)?.chi.to_bits() {
            return Err("binary chi does not match batch numbering/specification".into());
        }
    }
    let seed = read_array(&mut r)?;
    let initial_conditions_hash = read_array(&mut r)?;
    let cfg = common.config(chi)?;
    let inputs = canonical_inputs(spec, seed, &cfg)?;
    if initial_hash(inputs.clone(), &cfg)? != initial_conditions_hash {
        return Err("INITIAL CONDITIONS HASH MISMATCH: initial-condition reproduction failed; geodesic integration not started".into());
    }
    let mut codes = vec![0; count];
    r.read_exact(&mut codes)?;
    let statuses = codes
        .into_iter()
        .map(status_from)
        .collect::<Result<Vec<_>>>()?;
    let detected = statuses
        .iter()
        .filter(|s| **s == RayStatus::Detected)
        .count();
    if len != 72 + count as u64 + 24 * detected as u64 {
        return Err("binary length mismatch (truncated/trailing data)".into());
    }
    let mut rows = Vec::with_capacity(count);
    for (input, status) in inputs.into_iter().zip(statuses) {
        let hit = if status == RayStatus::Detected {
            Some([
                f64::from_le_bytes(read_array(&mut r)?),
                f64::from_le_bytes(read_array(&mut r)?),
                f64::from_le_bytes(read_array(&mut r)?),
            ])
        } else {
            None
        };
        rows.push(RayRow {
            ray_id: None,
            input,
            energy: 1.0,
            status,
            hit,
        });
    }
    let run = RecordedRun {
        common,
        chi,
        rows,
        seed_metadata: Some(SeedMetadata {
            seed,
            initial_conditions_hash,
        }),
    };
    run.validate()?;
    Ok(run)
}
