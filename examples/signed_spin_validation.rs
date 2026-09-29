//! Validation driver only. Calls the unchanged generator/solver/archive APIs.
//! Each invocation handles one run so all full trajectories are freed on exit.
use kerr_ray::{session::SessionConfig, simulation::rays::RaySimulationConfig, standard_run};
use serde_json::json;
use std::{fs, io::BufWriter, path::Path, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mode = args.first().ok_or("run/reproduce")?;
    if mode == "reproduce" {
        let root = Path::new(args.get(1).ok_or("run directory")?);
        let loaded = standard_run::load(&root.join("simulation_1"), "sim_1.bin")?;
        let report = standard_run::reproduce(&loaded)?;
        fs::write(root.join("reproduction.txt"), report.summary())?;
        println!("{}", report.summary());
        if !report.passed() {
            return Err("reproduction failed".into());
        }
        return Ok(());
    }
    if mode != "run" {
        return Err("run/reproduce".into());
    }
    let root = Path::new(args.get(1).ok_or("new output directory")?);
    let spin: f64 = args.get(2).ok_or("chi")?.parse()?;
    let factor: f64 = args.get(3).ok_or("tolerance divisor")?.parse()?;
    let mirror = args.get(4).is_some_and(|x| x == "mirror");
    fs::create_dir(root)?;
    let session = SessionConfig::default();
    let mut spec = session.generator.unwrap();
    if mirror {
        spec.axis_1[1] = -spec.axis_1[1];
    }
    let mut cfg = RaySimulationConfig {
        spin,
        integration: session.experiment.integration,
        detector: session.experiment.detector,
    };
    cfg.integration.rtol /= factor;
    cfg.integration.atol /= factor;
    // Reuse the previous 192x192 benchmark seed; never use saved results as input.
    let baseline = standard_run::load(
        Path::new("validation/parallel_measurements_20260926/larger/simulation_1"),
        "sim_1.bin",
    )?;
    let seed = baseline.seed_metadata.as_ref().ok_or("missing seed")?.seed;
    drop(baseline);
    let timer = Instant::now();
    let mut start = None;
    let mut finish = None;
    let (run, outcomes) =
        standard_run::seeded::execute_with_outcomes(&spec, seed, &cfg, |done, total| {
            if start.is_none() {
                start = Some(Instant::now());
            }
            if done == total {
                finish = Some(Instant::now());
            }
            true
        })?;
    let calculate_seconds = timer.elapsed().as_secs_f64();
    let physics_seconds = finish
        .ok_or("no completion")?
        .duration_since(start.unwrap())
        .as_secs_f64();
    let saved = standard_run::save_new(root, &run)?;
    let records: Vec<_> = outcomes.iter().map(|o| {
        let c = o.ray.condition();
        let d = &o.result.diagnostics;
        let last = o.result.samples.last().unwrap();
        json!({
            "ray_id":o.ray.ray_id(),
            "input":[c.position_x,c.position_y,c.position_z,c.direction_x,c.direction_y,c.direction_z,c.t_emit],
            "initial_state":o.ray.initial().0,
            "status":o.result.status,"reason":format!("{:?}",o.result.stop_reason),
            "failure":o.result.failure_message,
            "min_r":o.result.samples.iter().map(|s|s.state.radius()).fold(f64::INFINITY,f64::min),
            "last_state":last.state.0, "affine":last.affine,
            "min_polar_distance":o.result.samples.iter().map(|s|s.state.theta().min(std::f64::consts::PI-s.state.theta())).fold(f64::INFINITY,f64::min),
            "max_null":d.max_null_energy_scaled,"max_q_rel":d.max_q_relative,
            "max_energy_rel":d.max_energy_relative,"max_lz_rel":d.max_lz_relative,
            "accepted_steps":d.accepted_steps,"rejected_steps":d.rejected_steps
        })
    }).collect();
    serde_json::to_writer(
        BufWriter::new(fs::File::create(root.join("diagnostics.json"))?),
        &records,
    )?;
    let hex = |v: &[u8]| v.iter().map(|x| format!("{x:02x}")).collect::<String>();
    let meta = run.seed_metadata.as_ref().unwrap();
    let measure = json!({
        "chi":spin,"factor":factor,"mirror":mirror,"rays":run.rows.len(),
        "seed_hex":hex(&seed), "initial_conditions_hash":hex(&meta.initial_conditions_hash),
        "counts":standard_run::counts(&run.rows), "physics_seconds":physics_seconds,
        "calculate_seconds":calculate_seconds,"save_seconds":saved.seconds,
        "finite_hits":run.rows.iter().filter_map(|r|r.hit).flatten().all(f64::is_finite),
        "threads":kerr_ray::simulation::ray_execution::RayExecution::default_threads(),
        "integration":cfg.integration,"detector":cfg.detector,
        "max_null":outcomes.iter().map(|o|o.result.diagnostics.max_null_energy_scaled).fold(0.0,f64::max)
    });
    fs::write(
        root.join("measurement.json"),
        serde_json::to_vec_pretty(&measure)?,
    )?;
    println!("{measure}");
    Ok(())
}
