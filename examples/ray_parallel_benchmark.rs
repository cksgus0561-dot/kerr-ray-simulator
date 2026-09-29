//! Same release binary, same saved seed/config, serial then parallel. No format changes.
//! cargo run --release --example ray_parallel_benchmark -- VALIDATION_OUTPUT_DIRECTORY
use kerr_ray::{simulation::ray_execution::RayExecution, standard_run};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
const BASELINE: &str = "results/seeded_validation_20260925/new/simulation_1";
#[derive(Serialize, Deserialize)]
struct Measurement {
    rays: usize,
    threads: usize,
    physics_seconds: f64,
    total_calculate_save_seconds: f64,
    save_seconds: f64,
    counts: [usize; 5],
    binary_bytes: u64,
    archive: PathBuf,
    // Validation evidence only, outside the standard archive. Timings excluded.
    ray_physics_sha256: Vec<String>,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "--child") {
        let side = args.get(1).ok_or("missing side")?.parse::<usize>()?;
        let root = Path::new(args.get(2).ok_or("missing output")?);
        let baseline = standard_run::load(Path::new(BASELINE), "sim_1.bin")?;
        let seed = baseline.seed_metadata.as_ref().ok_or("seed required")?.seed;
        let cfg = baseline.common.config(baseline.chi)?;
        let mut generator = baseline
            .common
            .ray_generator
            .clone()
            .ok_or("generator required")?;
        generator.cell_count = [side, side];
        let threads = std::env::var("KERR_RAY_THREADS")?.parse()?;
        drop(baseline);
        let timer = Instant::now();
        let mut start = None;
        let mut end = None;
        let (run, outcomes) =
            standard_run::seeded::execute_with_outcomes(&generator, seed, &cfg, |done, total| {
                if start.is_none() {
                    start = Some(Instant::now());
                }
                if done == total {
                    end = Some(Instant::now());
                }
                true
            })?;
        let saved = standard_run::save_new(root, &run)?;
        let total_calculate_save_seconds = timer.elapsed().as_secs_f64();
        let physics_seconds = end
            .ok_or("no final physics progress")?
            .duration_since(start.unwrap())
            .as_secs_f64();
        let mut fingerprints = Vec::with_capacity(outcomes.len());
        for out in outcomes {
            let mut h = Sha256::new();
            h.update((out.ray.ray_id() as u64).to_le_bytes());
            for x in out.ray.initial().0 {
                h.update(x.to_le_bytes());
            }
            h.update([standard_run::seeded::status_code(out.result.status)]);
            h.update(
                format!(
                    "{:?}{:?}{:?}",
                    out.result.stop_reason, out.result.failure_message, out.result.diagnostics
                )
                .as_bytes(),
            );
            for sample in &out.result.samples {
                h.update(sample.affine.to_le_bytes());
                for x in sample.state.0 {
                    h.update(x.to_le_bytes());
                }
            }
            if let Some(hit) = out.result.detection {
                for x in hit
                    .state
                    .0
                    .into_iter()
                    .chain(hit.uv)
                    .chain([hit.affine, hit.signed_residual])
                {
                    h.update(x.to_le_bytes());
                }
                h.update((hit.iterations as u64).to_le_bytes());
            }
            fingerprints.push(h.finalize().iter().map(|x| format!("{x:02x}")).collect());
        }
        let measurement = Measurement {
            rays: run.rows.len(),
            threads,
            physics_seconds,
            total_calculate_save_seconds,
            save_seconds: saved.seconds,
            counts: standard_run::counts(&run.rows),
            binary_bytes: saved.binary_bytes,
            archive: saved.directory,
            ray_physics_sha256: fingerprints,
        };
        std::fs::write(
            root.join("measurement.json"),
            serde_json::to_vec_pretty(&measurement)?,
        )?;
        return Ok(());
    }
    let root = PathBuf::from(
        args.first()
            .ok_or("usage: ray_parallel_benchmark NEW_OUTPUT_DIRECTORY")?,
    );
    std::fs::create_dir(&root)?;
    let parallel = RayExecution::default_threads();
    let mut runs = Vec::new();
    for (label, side, threads) in [
        ("serial", 128, 1),
        ("parallel", 128, parallel),
        ("larger", 192, parallel),
    ] {
        let dir = root.join(label);
        let status = std::process::Command::new(std::env::current_exe()?)
            .arg("--child")
            .arg(side.to_string())
            .arg(&dir)
            .env("KERR_RAY_THREADS", threads.to_string())
            .status()?;
        if !status.success() {
            return Err(format!("{label} benchmark failed: {status}").into());
        }
        let m: Measurement = serde_json::from_slice(&std::fs::read(dir.join("measurement.json"))?)?;
        println!(
            "{label}: rays={} threads={} physics={:.6}s total_calculate_save={:.6}s counts={:?}",
            m.rays, m.threads, m.physics_seconds, m.total_calculate_save_seconds, m.counts
        );
        runs.push(m);
    }
    assert_eq!(
        runs[0].ray_physics_sha256, runs[1].ray_physics_sha256,
        "per-ray physics fingerprint mismatch"
    );
    assert_eq!(runs[0].counts, runs[1].counts);
    let old = std::fs::read(Path::new(BASELINE).join("sim_1.bin"))?;
    for m in &runs[..2] {
        assert_eq!(
            old,
            std::fs::read(m.archive.join("sim_1.bin"))?,
            "pre-parallel archive bytes changed"
        );
    }
    let summary = serde_json::json!({
        "logical_cpus":std::thread::available_parallelism()?.get(),"threads":parallel,
        "serial_physics_seconds":runs[0].physics_seconds,"parallel_physics_seconds":runs[1].physics_seconds,
        "physics_speedup":runs[0].physics_seconds/runs[1].physics_seconds,
        "serial_total_calculate_save_seconds":runs[0].total_calculate_save_seconds,
        "parallel_total_calculate_save_seconds":runs[1].total_calculate_save_seconds,
        "total_speedup":runs[0].total_calculate_save_seconds/runs[1].total_calculate_save_seconds,
        "per_ray_physics_fingerprint":"PASS","pre_parallel_binary_bytes":"PASS",
        "counts":runs[1].counts,"larger_rays":runs[2].rays,"larger_counts":runs[2].counts,
        "larger_physics_seconds":runs[2].physics_seconds,"larger_total_seconds":runs[2].total_calculate_save_seconds,
    });
    std::fs::write(
        root.join("summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    println!("{summary}");
    Ok(())
}
