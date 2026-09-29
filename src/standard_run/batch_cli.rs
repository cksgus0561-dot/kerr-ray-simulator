//! Batch controls are separate from the single-run CLI and from physics settings.
use super::{
    Result,
    batch::{self, BatchSpecification},
};
use crate::{experiments::chi_sampling::chi_at, session::SessionConfig};
use std::path::{Path, PathBuf};

pub fn run(args: &[String]) -> Result<()> {
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!(
            "batch-simulate --simulations-per-chi N [--config SESSION_JSON] [--output ROOT]\n  [--chi-indices -76,0,76] [--dry-run]\nN is required, with no production default. Without --chi-indices the complete 153-point grid is used.\nDefault physical settings: unchanged 192x192 seeded rays, 0.5M cells.\nOne new simulation_N/common.json plus sim_1.bin, sim_2.bin, ... per batch.\nSmall validation: --simulations-per-chi 1 --chi-indices -76,0,76\n--dry-run prints the plan without generating seeds, calculating rays or writing files.\nReproduce: reproduce --input DIRECTORY --sim sim_N.bin. See BATCH_SIMULATION.md."
        );
        return Ok(());
    }
    let mut session = SessionConfig::default();
    let mut output = PathBuf::from("results/standard");
    let mut repetitions = None;
    let mut indices = None;
    let mut dry_run = false;
    let mut i = 0;
    while i < args.len() {
        let key = args[i].as_str();
        if key == "--dry-run" {
            dry_run = true;
            i += 1;
            continue;
        }
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {key}"))?;
        match key {
            "--simulations-per-chi" => repetitions = Some(value.parse::<usize>()?),
            "--chi-indices" => {
                indices = Some(
                    value
                        .split(',')
                        .map(str::parse::<i32>)
                        .collect::<std::result::Result<Vec<_>, _>>()?,
                )
            }
            "--config" => session = SessionConfig::load(Path::new(value))?,
            "--output" => output = PathBuf::from(value),
            _ => return Err(format!("unknown batch option {key}").into()),
        }
        i += 2;
    }
    let mut spec = BatchSpecification::full(
        repetitions.ok_or("--simulations-per-chi N is required; no production default")?,
    );
    if let Some(indices) = indices {
        spec.chi_indices = indices;
    }
    let total = spec.simulation_count()?;
    let generator = session
        .generator
        .as_ref()
        .ok_or("batch requires a seeded session config")?;
    generator.validate()?;
    println!(
        "batch plan: {} chi values x {} = {total} simulations; {} physical rays each; output root: {}",
        spec.chi_indices.len(),
        spec.simulations_per_chi,
        generator.count()?,
        output.display()
    );
    if dry_run {
        for &index in &spec.chi_indices {
            println!("chi index {index}: {:.17}", chi_at(index)?);
        }
        return Ok(());
    }
    let report = batch::execute(&output, &session, &spec, |r| {
        println!(
            "sim_{}.bin chi={:.17} index={} repetition={} rays={} counts [ACT,DET,CAP,ESC,NUM]={:?} seconds={:.9} bytes={} seed={} initial_conditions_hash={}",
            r.entry.number,
            r.entry.chi,
            r.entry.chi_index,
            r.entry.repetition,
            r.rays,
            r.counts,
            r.calculation_seconds,
            r.binary_bytes,
            hex(&r.seed),
            hex(&r.initial_conditions_hash)
        );
    })?;
    println!(
        "batch saved: {}; completed {} / {total}",
        report.directory.display(),
        report.simulations.len()
    );
    Ok(())
}
fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
