use kerr_ray::{
    compute::{RayStatus, cpu::CpuReferenceIntegrator},
    detector::{binning::TimeBasis, events::HitEvent},
    experiments::source_detector::{
        Emission, LaunchDirection, SourceDetectorExperiment, SourcePattern,
    },
    output::{
        accumulated_image::pixels,
        csv, metadata,
        time_frames::{PostprocessConfig, generate},
    },
    physics::kerr::Kerr,
};
use std::{
    fs,
    io::BufReader,
    path::{Path, PathBuf},
};
fn directory(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "target/test-artifacts/{name}_{}_{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
fn decode(path: &Path) -> (Vec<u16>, u32, u32) {
    let mut r = png::Decoder::new(BufReader::new(fs::File::open(path).unwrap()))
        .read_info()
        .unwrap();
    let mut data = vec![0; r.output_buffer_size().unwrap()];
    let info = r.next_frame(&mut data).unwrap();
    assert_eq!(info.bit_depth, png::BitDepth::Sixteen);
    assert_eq!(info.color_type, png::ColorType::Grayscale);
    (
        data[..info.buffer_size()]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
            .collect(),
        info.width,
        info.height,
    )
}
fn events() -> Vec<HitEvent> {
    [0.0, 0.1, 0.3]
        .into_iter()
        .enumerate()
        .map(|(id, t)| HitEvent {
            ray_id: id,
            u_source: 0.0,
            v_source: 0.0,
            t_emit: 0.0,
            u_hit: 0.0,
            v_hit: 0.0,
            t_hit: t,
            delta_t: t,
            status: RayStatus::Detected,
        })
        .collect()
}

#[test]
fn all_source_patterns_and_explicit_emission_controls() {
    let mut exp = SourceDetectorExperiment::default();
    for (pattern, count) in [
        (SourcePattern::Single, 1),
        (SourcePattern::SquareGrid { side: 3 }, 9),
        (SourcePattern::RectangularGrid { nu: 3, nv: 2 }, 6),
    ] {
        exp.source.pattern = pattern;
        assert_eq!(exp.initials().unwrap().len(), count);
    }
    exp.source.pattern = SourcePattern::Explicit(vec![
        Emission {
            u: 1.0,
            v: 2.0,
            t_emit: Some(17.0),
            direction: Some(LaunchDirection::LocalZamo([-1.0, 0.0, 0.0])),
        },
        Emission {
            u: -2.0,
            v: -3.0,
            t_emit: Some(18.0),
            direction: None,
        },
    ]);
    let rays = exp.initials().unwrap();
    assert_eq!(rays[0].1.time(), 17.0);
    assert_eq!(rays[1].1.time(), 18.0);
    assert_ne!(rays[0].1.theta(), std::f64::consts::FRAC_PI_2);
    exp.source.pattern = SourcePattern::RectangularGrid { nu: 0, nv: 3 };
    assert!(exp.initials().is_err());
}

#[test]
fn emission_time_shift_changes_t_hit_but_not_travel_time_or_position() {
    let mut exp = SourceDetectorExperiment::default();
    exp.source.pattern = SourcePattern::Explicit(vec![Emission {
        u: 10.0,
        v: 5.0,
        t_emit: None,
        direction: None,
    }]);
    let k = Kerr::new(exp.spin).unwrap();
    let first = exp.initials().unwrap()[0].1;
    exp.source.t_emit = 20.0;
    let second = exp.initials().unwrap()[0].1;
    let a = CpuReferenceIntegrator
        .integrate_with_detector(k, first, &exp.integration, &exp.detector)
        .detection
        .unwrap();
    let b = CpuReferenceIntegrator
        .integrate_with_detector(k, second, &exp.integration, &exp.detector)
        .detection
        .unwrap();
    assert!((b.state.time() - a.state.time() - 20.0).abs() < 2e-8);
    assert!((a.uv[0] - b.uv[0]).abs() < 2e-8 && (a.uv[1] - b.uv[1]).abs() < 2e-8);
}

#[test]
fn png_histograms_and_both_frame_modes_match_events() {
    let dir = directory("images");
    let raw = dir.join("events.csv");
    let es = events();
    let mut w = ::csv::Writer::from_writer(csv::new_file(&raw).unwrap());
    for e in &es {
        w.serialize(e).unwrap();
    }
    w.flush().unwrap();
    let mut det = SourceDetectorExperiment::default().detector;
    det.resolution = [4, 4];
    let cfg = PostprocessConfig {
        start: Some(0.1),
        end: Some(0.5),
        time_bin_width: 0.1,
        ..Default::default()
    };
    let out = dir.join("derived");
    let info = generate(&raw, &es, &det, &cfg, &out).unwrap();
    assert_eq!(info["frame_count"], 4);
    assert_eq!(info["before_window_count"], 1);
    assert_eq!(info["analysis_window_count"], 2);
    let (acc, nx, ny) = decode(&out.join("detector_accumulated.png"));
    assert_eq!((nx, ny), (4, 4));
    assert_eq!(*acc.iter().max().unwrap(), 49151);
    let count_sum: u64 = fs::read_to_string(out.join("detector_accumulated.csv"))
        .unwrap()
        .split([',', '\n'])
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u64>().unwrap())
        .sum();
    assert_eq!(count_sum, 3);
    let instantaneous = decode(&out.join("detector_frames/instantaneous/frame_000000.png")).0;
    let cumulative = decode(&out.join("detector_frames/cumulative/frame_000000.png")).0;
    assert_eq!(*instantaneous.iter().max().unwrap(), 16383);
    assert_eq!(*cumulative.iter().max().unwrap(), 32767);
    let last = decode(&out.join("detector_frames/cumulative/frame_000003.png")).0;
    assert_eq!(last, acc);
    for mode in ["instantaneous", "cumulative"] {
        let mut r = png::Decoder::new(BufReader::new(
            fs::File::open(out.join(format!("detector_{mode}.apng"))).unwrap(),
        ))
        .read_info()
        .unwrap();
        assert_eq!(r.info().animation_control.unwrap().num_frames, 4);
        for frame in 0..4 {
            let mut buf = vec![0; r.output_buffer_size().unwrap()];
            let info = r.next_frame(&mut buf).unwrap();
            let values: Vec<u16> = buf[..info.buffer_size()]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u16::from_be_bytes([b[0], b[1]]))
                .collect();
            assert_eq!(
                values,
                decode(&out.join(format!("detector_frames/{mode}/frame_{frame:06}.png"))).0
            );
        }
    }
    assert_eq!(
        pixels(&[0, 1, 2, 4, 100], 4),
        [0, 0, 63, 255, 127, 255, 255, 255, 255, 255]
    );
}

#[test]
fn reprocessing_preserves_raw_bytes_and_refuses_existing_outputs() {
    let dir = directory("preserve");
    let raw = dir.join("events.csv");
    let es = events();
    let mut w = ::csv::Writer::from_writer(csv::new_file(&raw).unwrap());
    for e in &es {
        w.serialize(e).unwrap();
    }
    w.flush().unwrap();
    let original = fs::read(&raw).unwrap();
    let mut det = SourceDetectorExperiment::default().detector;
    det.resolution = [4, 4];
    let cfg = PostprocessConfig {
        start: Some(0.0),
        end: Some(0.5),
        time_bin_width: 0.1,
        ..Default::default()
    };
    generate(&raw, &es, &det, &cfg, &dir.join("one")).unwrap();
    generate(
        &raw,
        &csv::read_events(&raw).unwrap(),
        &det,
        &PostprocessConfig {
            time_bin_width: 0.01,
            playback_fps: 60,
            ..cfg.clone()
        },
        &dir.join("two"),
    )
    .unwrap();
    assert_eq!(fs::read(&raw).unwrap(), original);
    assert!(generate(&raw, &es, &det, &cfg, &dir.join("one")).is_err());
    assert_eq!(fs::read(&raw).unwrap(), original);
}

#[test]
fn empty_and_cropped_data_have_explicit_zero_frames() {
    let dir = directory("empty");
    let raw = dir.join("events.csv");
    fs::write(
        &raw,
        "ray_id,u_source,v_source,t_emit,u_hit,v_hit,t_hit,delta_t,status\n",
    )
    .unwrap();
    let mut det = SourceDetectorExperiment::default().detector;
    det.resolution = [2, 2];
    let cfg = PostprocessConfig {
        time_bin_width: 0.1,
        start: Some(0.0),
        end: Some(0.2),
        basis: TimeBasis::Arrival,
        ..Default::default()
    };
    let info = generate(&raw, &[], &det, &cfg, &dir.join("derived")).unwrap();
    assert_eq!(info["frame_count"], 2);
    assert_eq!(info["event_count"], 0);
    assert_eq!(
        decode(&dir.join("derived/detector_accumulated.png")).0,
        vec![0; 4]
    );
}

#[test]
fn source_detector_cli_outputs_and_rebin_without_reintegration() {
    let dir = directory("cli");
    let run = dir.join("run");
    let exe = env!("CARGO_BIN_EXE_kerr-ray");
    let status = std::process::Command::new(exe)
        .args([
            "experiment",
            "source_to_detector",
            "--nu",
            "4",
            "--nv",
            "4",
            "--resolution",
            "8x8",
            "--dt",
            "5",
            "--output",
        ])
        .arg(&run)
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "{} {}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
    let meta: serde_json::Value =
        serde_json::from_reader(fs::File::open(run.join("run_metadata.json")).unwrap()).unwrap();
    assert_eq!(meta["ray_count"], 16);
    assert_eq!(meta["outcomes"]["NumericalFailure"], 0);
    assert!(
        meta["experiment_id"]
            .as_str()
            .unwrap()
            .starts_with("source_to_detector-")
    );
    let started: serde_json::Value =
        serde_json::from_reader(fs::File::open(run.join("run_started.json")).unwrap()).unwrap();
    assert_eq!(meta["experiment_id"], started["experiment_id"]);
    let before = metadata::fingerprint(&run.join("detector_events.csv")).unwrap();
    let status = std::process::Command::new(exe)
        .arg("rebin")
        .arg("--input")
        .arg(&run)
        .args([
            "--dt",
            "0.1",
            "--time-start",
            "168",
            "--time-end",
            "180",
            "--output",
        ])
        .arg(dir.join("rebin"))
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    assert!(String::from_utf8_lossy(&status.stdout).contains("no geodesic integration"));
    assert_eq!(
        before,
        metadata::fingerprint(&run.join("detector_events.csv")).unwrap()
    );
    assert!(
        csv::read_events(&run.join("detector_events.csv"))
            .unwrap()
            .iter()
            .all(|e| e.delta_t == e.t_hit)
    );
}
