//! No output or compilation inside the timed region. No physics checks disabled.
use kerr_ray::{
    compute::cpu::CpuReferenceIntegrator,
    experiments::source_detector::{SourceDetectorExperiment, status_index},
    output::{OutputResult, csv::new_file, metadata},
    physics::kerr::Kerr,
};
use std::{io::Write, path::PathBuf, time::Instant};
fn main() -> OutputResult<()> {
    let dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(format!(
                "results/benchmark_detector_{}",
                (metadata::unix_seconds() * 1000.0) as u64
            ))
        });
    std::fs::create_dir_all(&dir)?;
    let exp = SourceDetectorExperiment::default();
    let rays = exp.initials()?;
    let k = Kerr::new(exp.spin)?;
    let mut times = Vec::new();
    let mut max_null: f64 = 0.0;
    let mut q: f64 = 0.0;
    let mut counts = [0usize; 5];
    for repeat in 0..11 {
        let start = Instant::now();
        let mut current = [0usize; 5];
        for (_, s) in &rays {
            let result = CpuReferenceIntegrator.integrate_with_detector(
                k,
                *s,
                &exp.integration,
                &exp.detector,
            );
            current[status_index(result.status)] += 1;
            max_null = max_null.max(result.diagnostics.max_null_abs);
            q = q.max(result.diagnostics.max_q_abs);
            std::hint::black_box(result);
        }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        if repeat > 0 {
            times.push(elapsed);
        }
        counts = current;
    }
    let mut f = new_file(&dir.join("batches.csv"))?;
    writeln!(f, "batch,rays,batch_ms")?;
    for (i, t) in times.iter().enumerate() {
        writeln!(f, "{i},{},{t:.9}", rays.len())?;
    }
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    times.sort_by(f64::total_cmp);
    let p95 = times[(0.95 * times.len() as f64).ceil() as usize - 1];
    let report = serde_json::json!({"engine":"sequential CPU f64 DP5(4) with detector", "rays":rays.len(),"measured_batches":10,"excluded_warmup_batches":1,
        "mean_batch_ms":mean,"p95_batch_ms":p95,"mean_ms_per_ray":mean/rays.len() as f64,"max_null":max_null,"max_Q_abs":q,
        "outcomes":{"Active":counts[0],"Detected":counts[1],"Captured":counts[2],"Escaped":counts[3],"NumericalFailure":counts[4]},
        "experiment":exp,"code_version":env!("CARGO_PKG_VERSION"),"source_fingerprint":env!("KERR_SOURCE_FINGERPRINT"),"measured_at_unix_seconds":metadata::unix_seconds()});
    metadata::write(&dir.join("benchmark.json"), &report)?;
    println!(
        "256-ray detector batch: mean={mean:.6} ms, p95={p95:.6} ms\nresults={}",
        dir.display()
    );
    if counts[4] > 0 {
        return Err("numerical failures in benchmark".into());
    }
    Ok(())
}
