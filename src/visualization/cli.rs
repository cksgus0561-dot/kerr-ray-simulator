use crate::{session::SessionConfig, simulation};
use eframe::{
    egui,
    egui_wgpu::{self, wgpu},
};
use std::{path::PathBuf, sync::Arc};
pub fn run(args: &[String], prepare: bool) -> Result<(), Box<dyn std::error::Error>> {
    if args.iter().any(|a| a == "--help") {
        println!(
            "visualize [--config JSON | --input RUN_DIRECTORY] [--preset regression] [--grid N] [--rendered N] [--measure NEW_DIRECTORY]\nvisualize-prepare [--config JSON] [--grid N] --output NEW_DIRECTORY\nDefault: 192x192 stratified physical rays in 96x96M / 256 displayed trajectories. New GUI calculations auto-save common.json + sim_1.bin. Load Folder/Reproduce supports binary and legacy Parquet. --input reads legacy visualization exports; --measure measures rendering; prepare explicitly exports trajectories. See SEEDED_RUN.md."
        );
        return Ok(());
    }
    let mut cfg = SessionConfig::default();
    let mut input = None;
    let mut output = None;
    let mut measure = None;
    let mut grid = None;
    let mut rendered = None;
    let mut i = 0;
    while i < args.len() {
        let key = &args[i];
        let value = args.get(i + 1).ok_or("missing option value")?;
        match key.as_str() {
            "--config" => cfg = SessionConfig::load(std::path::Path::new(value))?,
            "--input" => input = Some(PathBuf::from(value)),
            "--output" => output = Some(PathBuf::from(value)),
            "--measure" => measure = Some(PathBuf::from(value)),
            "--preset" => {
                if value != "regression" {
                    return Err("only preset regression is defined".into());
                }
                cfg = SessionConfig::regression();
            }
            "--grid" => grid = Some(value.parse::<usize>()?),
            "--rendered" => rendered = Some(value.parse::<usize>()?),
            _ => return Err(format!("unknown visualization option {key}").into()),
        }
        i += 2;
    }
    if let Some(n) = grid {
        cfg.set_grid(n);
    }
    if let Some(n) = rendered {
        cfg.view.max_rendered_trajectories = n;
    }
    // Absolute arrival time is the shared scene clock. Travel-time histograms remain
    // available in the existing rebin CLI, where different emission times are explicit.
    if !matches!(
        cfg.experiment.postprocess.basis,
        crate::detector::binning::TimeBasis::Arrival
    ) {
        return Err("3D scene playback requires Arrival time basis; use rebin for Travel-time distributions".into());
    }
    cfg.validate_view()?;
    if prepare {
        let output = output.ok_or("visualize-prepare requires --output NEW_DIRECTORY")?;
        let data = simulation::worker::calculate_session(&cfg, |_, _| true)?;
        println!(
            "CPU f64: rays={} seconds={:.9} counts={:?} errors={:?}",
            data.rays.len(),
            data.seconds,
            data.counts(),
            data.max_errors()
        );
        let timer = std::time::Instant::now();
        simulation::storage::save(&data, &cfg, &output)?;
        println!("Export seconds: {:.6}", timer.elapsed().as_secs_f64());
        return Ok(());
    }
    if output.is_some() {
        return Err(
            "use --output with visualize-prepare; interactive export is in the panel".into(),
        );
    }
    if let Some(path) = &input {
        let saved = path.join("visualization.json");
        if !args.iter().any(|s| s == "--config") {
            cfg = SessionConfig::load(&if saved.exists() {
                saved
            } else {
                path.join("run_metadata.json")
            })?;
            if let Some(n) = rendered {
                cfg.view.max_rendered_trajectories = n;
            }
        }
        if !matches!(
            cfg.experiment.postprocess.basis,
            crate::detector::binning::TimeBasis::Arrival
        ) {
            return Err(
                "3D playback requires Arrival time; use rebin for Travel distributions".into(),
            );
        }
    }
    let mut setup = egui_wgpu::WgpuSetupCreateNew::without_display_handle();
    setup.power_preference = wgpu::PowerPreference::HighPerformance;
    setup.native_adapter_selector = Some(Arc::new(|adapters, surface| {
        for a in adapters {
            println!("Available adapter: {:?}", a.get_info());
        }
        adapters
            .iter()
            .filter(|a| surface.is_none_or(|s| a.is_surface_supported(s)))
            .max_by_key(|a| {
                let i = a.get_info();
                let discrete = i.device_type == wgpu::DeviceType::DiscreteGpu;
                (
                    if discrete && i.vendor == 0x10de {
                        3
                    } else if discrete {
                        2
                    } else {
                        1
                    },
                    i.backend == wgpu::Backend::Vulkan,
                )
            })
            .cloned()
            .ok_or_else(|| "no surface-compatible GPU adapter".into())
    }));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Kerr Geodesic Lab")
            .with_visible(true)
            .with_inner_size([1440., 960.])
            .with_min_inner_size([1000., 650.]),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: egui_wgpu::WgpuConfiguration {
            wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(setup),
            ..Default::default()
        },
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(
        "Kerr Geodesic Lab",
        options,
        Box::new(move |cc| Ok(Box::new(super::app::App::new(cc, cfg, input, measure)))),
    )
    .map_err(|e| e.to_string().into())
}
