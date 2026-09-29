use kerr_ray::{
    compute::{RayIntegrator, RayStatus, cpu::CpuReferenceIntegrator},
    experiments::{
        output,
        scenarios::{self, ExperimentSettings},
    },
};
use std::{
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "batch-simulate") {
        return kerr_ray::standard_run::batch_cli::run(&args[1..]);
    }
    if args.first().is_some_and(|s| {
        matches!(
            s.as_str(),
            "simulate-save" | "inspect-simulation" | "reproduce"
        )
    }) {
        return kerr_ray::standard_run::cli::run(&args[0], &args[1..]);
    }
    if args
        .first()
        .is_some_and(|s| s == "visualize" || s == "visualize-prepare")
    {
        return kerr_ray::visualization::cli::run(&args[1..], args[0] == "visualize-prepare");
    }
    if args.first().is_some_and(|s| s == "rebin") {
        return kerr_ray::experiments::source_detector_cli::run(&args[1..], true);
    }
    if args.first().is_some_and(|s| s == "experiment")
        && args.get(1).is_some_and(|s| s == "source_to_detector")
    {
        return kerr_ray::experiments::source_detector_cli::run(&args[2..], false);
    }
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "kerr-ray: fixed Kerr, 3+1 f64 reference; G=c=M=1; (-,+,+,+)\n\
Usage:\n  cargo run --release -- experiment <name> [options]\n  cargo run --release -- benchmark [options]\n\
Experiments: {}\n\
Options: --spin N --rays N --source-radius N --angle-deg N --b-min N --b-max N\n\
  --comparison-b N --beam-width N --rtol N --atol N --epsilon N --escape N\n\
  --max-affine N --max-step N --max-steps N --null-tolerance N\n\
  --output DIRECTORY --trajectories --repeats N (benchmark only)\n\
New: experiment source_to_detector --help; rebin --help; simulate-save --help; reproduce --help; batch-simulate --help\n\
3D GUI: visualize --help; reusable runs: visualize-prepare --help. See README.md.",
            scenarios::NAMES.join(", ")
        );
        return Ok(());
    }
    let benchmark = args[0] == "benchmark";
    if args[0] != "experiment" && !benchmark {
        return Err("expected experiment or benchmark; see --help".into());
    }
    let name = if benchmark {
        "parallel_beam"
    } else {
        args.get(1).ok_or("missing experiment name")?
    };
    let mut p = ExperimentSettings::default();
    let mut trajectories = false;
    let mut repeats = 30_usize;
    let mut directory = None;
    let mut i = if benchmark { 1 } else { 2 };
    while i < args.len() {
        let key = &args[i];
        if key == "--trajectories" {
            trajectories = true;
            i += 1;
            continue;
        }
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {key}"))?;
        match key.as_str() {
            "--spin" => p.spin = value.parse()?,
            "--rays" => p.ray_count = value.parse()?,
            "--source-radius" => p.source_radius = value.parse()?,
            "--angle-deg" => p.angle_degrees = value.parse()?,
            "--b-min" => p.impact_min = value.parse()?,
            "--b-max" => p.impact_max = value.parse()?,
            "--comparison-b" => p.comparison_impact = value.parse()?,
            "--beam-width" => p.beam_half_width = value.parse()?,
            "--rtol" => p.integration.rtol = value.parse()?,
            "--atol" => p.integration.atol = value.parse()?,
            "--epsilon" => p.integration.horizon_epsilon = value.parse()?,
            "--escape" => p.integration.escape_radius = value.parse()?,
            "--max-affine" => p.integration.max_affine = value.parse()?,
            "--max-step" => p.integration.max_step = value.parse()?,
            "--max-steps" => p.integration.max_steps = value.parse()?,
            "--null-tolerance" => p.integration.null_tolerance = value.parse()?,
            "--output" => directory = Some(PathBuf::from(value)),
            "--repeats" if benchmark => repeats = value.parse()?,
            _ => return Err(format!("unknown option {key}").into()),
        }
        i += 2;
    }
    let cases = scenarios::select(name, &p)?;
    let dir = directory.unwrap_or_else(|| {
        PathBuf::from("results").join(format!(
            "{name}_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ))
    });
    fs::create_dir_all(&dir)?;
    println!(
        "kerr-ray {} | CPU f64 Dormand-Prince 5(4) | sequential | G=c=M=1 | (-,+,+,+)",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "scenario={name}; rays={}; rtol={:e}; atol={:e}; epsilon={:e}; r_escape={}; per-ray spins below",
        cases.len(),
        p.integration.rtol,
        p.integration.atol,
        p.integration.horizon_epsilon,
        p.integration.escape_radius
    );
    if benchmark {
        if !(2..=10_000).contains(&repeats) {
            return Err("benchmark repeats must be 2..10000".into());
        }
        return run_benchmark(&cases, repeats, &dir);
    }
    let mut summary = BufWriter::new(File::create(dir.join("summary.csv"))?);
    writeln!(summary, "{}", output::SUMMARY_HEADER)?;
    let started = Instant::now();
    let mut counts = [0; 4];
    let mut max_null: f64 = 0.0;
    for (id, case) in cases.iter().enumerate() {
        let result = CpuReferenceIntegrator.integrate(case.kerr, case.initial, &case.config);
        counts[match result.status {
            RayStatus::Active | RayStatus::Detected => 0,
            RayStatus::Captured => 1,
            RayStatus::Escaped => 2,
            RayStatus::NumericalFailure => 3,
        }] += 1;
        max_null = max_null.max(result.diagnostics.max_null_abs);
        println!(
            "ray {id:04} chi={:.2} {:24} {:16?} steps={} null={:.3e} Eerr={:.1e} Lerr={:.1e} Qerr={:.1e} {:.3} ms{}",
            case.kerr.spin(),
            case.label,
            result.status,
            result.diagnostics.accepted_steps,
            result.diagnostics.max_null_abs,
            result.diagnostics.max_energy_relative,
            result.diagnostics.max_lz_relative,
            result.diagnostics.max_q_abs,
            result.elapsed_seconds * 1000.0,
            result
                .failure_message
                .as_ref()
                .map(|s| format!(" ERROR: {s}"))
                .unwrap_or_default()
        );
        writeln!(summary, "{}", output::summary_row(id, case, &result))?;
        if trajectories {
            output::write_trajectory(&dir.join(format!("ray_{id:04}.csv")), case, &result)?;
        }
    }
    summary.flush()?;
    println!(
        "Active={} Captured={} Escaped={} NumericalFailure={}; max |C_null|={max_null:e}; wall (including I/O)={:.3} ms\nCSV: {}",
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        started.elapsed().as_secs_f64() * 1000.0,
        dir.display()
    );
    if counts[3] > 0 {
        return Err("one or more rays failed numerically; inspect summary.csv".into());
    }
    Ok(())
}

fn run_benchmark(
    cases: &[scenarios::Case],
    repeats: usize,
    dir: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    // One excluded warmup batch. Timing includes integration, diagnostics,
    // trajectory allocation and dropping, but excludes filesystem/terminal output.
    for c in cases {
        std::hint::black_box(CpuReferenceIntegrator.integrate(c.kerr, c.initial, &c.config));
    }
    let mut times = Vec::with_capacity(repeats);
    let mut max_null: f64 = 0.0;
    let mut failures = 0;
    for _ in 0..repeats {
        let start = Instant::now();
        for c in cases {
            let result = CpuReferenceIntegrator.integrate(c.kerr, c.initial, &c.config);
            max_null = max_null.max(result.diagnostics.max_null_abs);
            failures += usize::from(result.status == RayStatus::NumericalFailure);
            std::hint::black_box(result);
        }
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let mut w = BufWriter::new(File::create(dir.join("benchmark.csv"))?);
    writeln!(w, "repeat,rays,batch_ms")?;
    for (i, t) in times.iter().enumerate() {
        writeln!(w, "{i},{},{t:.9}", cases.len())?;
    }
    w.flush()?;
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    times.sort_by(f64::total_cmp);
    let p95 = times[(0.95 * times.len() as f64).ceil() as usize - 1];
    let report = format!(
        "CPU sequential f64; {} rays; {repeats} measured batches; 1 excluded warmup\nmean_batch_ms={mean:.6}\np95_batch_ms={p95:.6}\nmean_ms_per_ray={:.6}\nmax_null_abs={max_null:e}\nnumerical_failures={failures}\nFPS/frame time/GPU: not measured (no renderer/backend implemented)\n",
        cases.len(),
        mean / cases.len() as f64
    );
    fs::write(dir.join("benchmark.txt"), &report)?;
    let mut inputs = BufWriter::new(File::create(dir.join("inputs.csv"))?);
    writeln!(inputs, "{}", output::SUMMARY_HEADER)?;
    for (i, c) in cases.iter().enumerate() {
        let out = CpuReferenceIntegrator.integrate(c.kerr, c.initial, &c.config);
        writeln!(inputs, "{}", output::summary_row(i, c, &out))?;
    }
    inputs.flush()?;
    println!("{report}CSV: {}", dir.display());
    if failures > 0 {
        return Err("benchmark has numerical failures".into());
    }
    Ok(())
}
