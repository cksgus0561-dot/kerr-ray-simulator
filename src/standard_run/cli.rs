use super::{Result, counts, execute, load, reproduce, save_new};
use crate::{
    experiments::independent_rays::from_source_plane, session::SessionConfig,
    simulation::rays::RaySimulationConfig,
};
use std::{path::PathBuf, time::Instant};
pub fn run(command: &str, args: &[String]) -> Result<()> {
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!(
            "simulate-save [--config JSON] [--grid N | --preset regression] [--output ROOT]\ninspect-simulation --input DIRECTORY [--sim sim_1.bin]\nreproduce --input DIRECTORY [--sim sim_1.bin]\nDefault: 192x192 stratified rays, 0.5M cells, fresh OS 256-bit seed; common.json + sim_1.bin. Each calculation reserves a new simulation_N. Legacy regression/JSON and Parquet reproduction remain supported. See SEEDED_RUN.md."
        );
        return Ok(());
    }
    let mut config = SessionConfig::default();
    let mut input = None;
    let mut output = PathBuf::from("results/standard");
    let mut simulation = None;
    let mut grid = None;
    let mut regression = false;
    if !args.len().is_multiple_of(2) {
        return Err("options require values".into());
    }
    for pair in args.as_chunks::<2>().0 {
        match pair[0].as_str() {
            "--config" if command == "simulate-save" => {
                config = SessionConfig::load(std::path::Path::new(&pair[1]))?
            }
            "--output" if command == "simulate-save" => output = PathBuf::from(&pair[1]),
            "--grid" if command == "simulate-save" => grid = Some(pair[1].parse::<usize>()?),
            "--preset" if command == "simulate-save" && pair[1] == "regression" => {
                regression = true
            }
            "--input" if command != "simulate-save" => input = Some(PathBuf::from(&pair[1])),
            "--sim" if command != "simulate-save" => simulation = Some(pair[1].clone()),
            key => return Err(format!("unknown/inapplicable option {key}").into()),
        }
    }
    if command == "simulate-save" {
        if regression {
            config = SessionConfig::regression();
        }
        if let Some(n) = grid {
            config.set_grid(n);
        }
        let e = config.experiment;
        let cfg = RaySimulationConfig {
            spin: e.spin,
            integration: e.integration,
            detector: e.detector,
        };
        let timer = Instant::now();
        let run = if let Some(spec) = &config.generator {
            super::seeded::execute_with_outcomes(
                spec,
                crate::experiments::stratified::master_seed()?,
                &cfg,
                |_, _| true,
            )?
            .0
        } else {
            execute(from_source_plane(&e.source)?, &cfg)?
        };
        let calculation = timer.elapsed().as_secs_f64();
        let saved = save_new(&output, &run)?;
        if run.seed_metadata.is_some() {
            println!(
                "saved: {}\nrays: {}\ncounts [Active,Detected,Captured,Escaped,NumericalFailure]: {:?}\ncalculation/association seconds: {calculation:.9}\nsave seconds: {:.9}\ncommon.json bytes: {}\nsim_1.bin bytes: {}\nchi: {}",
                saved.directory.display(),
                run.rows.len(),
                counts(&run.rows),
                saved.seconds,
                saved.common_bytes,
                saved.binary_bytes,
                run.chi
            );
            return Ok(());
        }
        println!(
            "saved: {}\nrays: {}\ncounts [Active,Detected,Captured,Escaped,NumericalFailure]: {:?}\ncalculation/association seconds: {calculation:.9}\nsave seconds: {:.9}\ncommon.json bytes: {}\nsim_1.parquet bytes: {}\ncompression: SNAPPY\nchi: {}",
            saved.directory.display(),
            run.rows.len(),
            counts(&run.rows),
            saved.seconds,
            saved.common_bytes,
            saved.parquet_bytes,
            run.chi
        );
    } else {
        let directory = input.ok_or("--input DIRECTORY is required")?;
        let simulation = simulation.unwrap_or(super::list_simulations(&directory)?[0].clone());
        let timer = Instant::now();
        let saved = load(&directory, &simulation)?;
        let seconds = timer.elapsed().as_secs_f64();
        println!(
            "loaded: {} / {simulation}\nload/validation seconds: {seconds:.9}\nrays: {}\nchi: {}\ncounts [Active,Detected,Captured,Escaped,NumericalFailure]: {:?}\nformat: {}\nsimulator: {}",
            directory.display(),
            saved.rows.len(),
            saved.chi,
            counts(&saved.rows),
            saved.common.data_format_version,
            saved.common.simulator_version
        );
        if command == "reproduce" {
            let report = reproduce(&saved)?;
            println!("{}", report.summary());
            if !report.passed() {
                return Err("reproduction comparison failed".into());
            }
        }
    }
    Ok(())
}
