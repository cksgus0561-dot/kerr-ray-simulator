//! Immutable research outputs plus an optional lossless f64 gzip trajectory stream.
//! Legacy event-only runs stay event-only: no path is invented from hit coordinates.
use super::{RayRecord, SimulationData, TraceSample};
use crate::{
    compute::RayStatus,
    experiments::source_detector::SourceDetectorExperiment,
    output::{csv as output_csv, metadata, time_frames},
    session::SessionConfig,
};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde_json::json;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
};

pub fn save(data: &SimulationData, config: &SessionConfig, directory: &Path) -> Result<(), String> {
    save_inner(data, config, directory).map_err(|e| e.to_string())
}
fn save_inner(
    data: &SimulationData,
    config: &SessionConfig,
    directory: &Path,
) -> crate::output::OutputResult<()> {
    data.validate()?;
    config.validate_view()?;
    if directory.exists() && fs::read_dir(directory)?.next().is_some() {
        return Err("export needs a new empty directory; saved data is immutable".into());
    }
    fs::create_dir_all(directory)?;
    use std::io::Write;
    output_csv::new_file(&directory.join("ray_diagnostics.csv"))?
        .write_all(data.diagnostic_csv.as_bytes())?;
    output_csv::new_file(&directory.join("detector_hit_states.csv"))?
        .write_all(data.hit_state_csv.as_bytes())?;
    let stamp = metadata::unix_seconds();
    metadata::write(
        &directory.join("run_started.json"),
        &json!({"experiment":data.experiment,"started":stamp,"code_version":env!("CARGO_PKG_VERSION")}),
    )?;
    let events_path = directory.join("detector_events.csv");
    let mut ew = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(output_csv::new_file(&events_path)?);
    ew.write_record([
        "ray_id", "u_source", "v_source", "t_emit", "u_hit", "v_hit", "t_hit", "delta_t", "status",
    ])?;
    for e in &data.events {
        ew.serialize(e)?;
    }
    ew.flush()?;
    ew.get_ref().sync_all()?;
    let trajectory_path = directory.join("trajectory_samples.csv.gz");
    let encoder = GzEncoder::new(output_csv::new_file(&trajectory_path)?, Compression::fast());
    let mut writer = csv::Writer::from_writer(encoder);
    writer.write_record([
        "ray_id", "affine", "t", "r", "theta", "phi", "p_t", "p_r", "p_theta", "p_phi",
    ])?;
    let stride = config.trajectory_export_stride;
    for ray in &data.rays {
        for (i, s) in ray.samples.iter().enumerate() {
            if i % stride != 0 && i + 1 != ray.samples.len() {
                continue;
            }
            let p = s.state;
            writer.serialize((
                ray.ray_id, s.affine, p[0], p[1], p[6], p[2], p[3], p[4], p[7], p[5],
            ))?;
        }
    }
    writer.flush()?;
    writer.into_inner()?.finish()?.sync_all()?;
    let counts = data.counts();
    let errors = data.max_errors();
    metadata::write(
        &directory.join("run_metadata.json"),
        &json!({
            "schema_version":3,"experiment_id":format!("visualize-{stamp:.6}"),"experiment":data.experiment,
            "ray_count":data.rays.len(),"G":1,"c":1,"M":1,"coordinates":"Boyer-Lindquist (t,r,theta,phi); signature (-,+,+,+)",
            "plane_map":"X=sqrt(r*r+a*a)sin(theta)cos(phi), Y=sqrt(r*r+a*a)sin(theta)sin(phi), Z=r*cos(theta), T=t_BL; NOT Kerr-Schild",
            "time_convention":"continuous f64 t_BL; delta_t=t_hit-t_emit; frame FPS is presentation only",
            "code_version":env!("CARGO_PKG_VERSION"),"build_source_fnv1a64":env!("KERR_SOURCE_FINGERPRINT"),"executed_at_unix_seconds":stamp,
            "events_fnv1a64":metadata::fingerprint(&events_path)?,"physics_seconds":data.seconds,
            "outcomes":{"Active":counts[0],"Detected":counts[1],"Captured":counts[2],"Escaped":counts[3],"NumericalFailure":counts[4]},
            "max_errors":{"null_abs":errors[0],"E_relative":errors[1],"Lz_relative":errors[2],"Q_abs":errors[3],"Q_relative":errors[4]}
        }),
    )?;
    metadata::write(
        &directory.join("scene_metadata.json"),
        &json!({"schema_version":1,"data":data,
        "trajectory_file":"trajectory_samples.csv.gz","trajectory_fnv1a64":metadata::fingerprint(&trajectory_path)?,
        "trajectory_stride":stride,"trajectory_format":"CSV gzip, canonical BL f64 samples. Both endpoints retained. Linear display interpolation between samples, no refitting."}),
    )?;
    let mut coherent = config.clone();
    coherent.experiment = data.experiment.clone();
    coherent.experiment.postprocess = config.experiment.postprocess.clone();
    coherent.experiment.detector.resolution = config.experiment.detector.resolution;
    metadata::write(&directory.join("visualization.json"), &coherent)?;
    let mut detector = data.experiment.detector.clone();
    detector.resolution = config.experiment.detector.resolution;
    time_frames::generate(
        &events_path,
        &data.events,
        &detector,
        &config.experiment.postprocess,
        directory,
    )?;
    Ok(())
}

pub fn load(directory: &Path) -> Result<SimulationData, String> {
    load_inner(directory).map_err(|e| e.to_string())
}
fn load_inner(directory: &Path) -> crate::output::OutputResult<SimulationData> {
    let run: serde_json::Value = serde_json::from_reader(std::io::BufReader::new(File::open(
        directory.join("run_metadata.json"),
    )?))?;
    let events_path = directory.join("detector_events.csv");
    if run["events_fnv1a64"] != metadata::fingerprint(&events_path)? {
        return Err("raw event checksum mismatch".into());
    }
    let events = output_csv::read_events(&events_path)?;
    let scene_path = directory.join("scene_metadata.json");
    if scene_path.exists() {
        let scene: serde_json::Value =
            serde_json::from_reader(std::io::BufReader::new(File::open(scene_path)?))?;
        let mut data: SimulationData = serde_json::from_value(scene["data"].clone())?;
        if data.events != events {
            return Err("scene/event list mismatch".into());
        }
        // Fixed filename: untrusted metadata must not direct reads outside this run.
        let path = directory.join("trajectory_samples.csv.gz");
        if scene["trajectory_file"] != "trajectory_samples.csv.gz"
            || scene["trajectory_fnv1a64"] != metadata::fingerprint(&path)?
        {
            return Err("trajectory checksum/filename mismatch".into());
        }
        let map: HashMap<_, _> = data
            .rays
            .iter()
            .enumerate()
            .map(|(i, r)| (r.ray_id, i))
            .collect();
        let reader = GzDecoder::new(std::io::BufReader::new(File::open(path)?));
        for row in csv::Reader::from_reader(reader).records() {
            let row = row?;
            if row.len() != 10 {
                return Err("trajectory requires 10 columns".into());
            }
            let id: usize = row[0].parse()?;
            let index = *map
                .get(&id)
                .ok_or("trajectory ray_id has no diagnostic record")?;
            let val = |i: usize| row[i].parse::<f64>();
            data.rays[index].samples.push(TraceSample {
                affine: val(1)?,
                state: [
                    val(2)?,
                    val(3)?,
                    val(5)?,
                    val(6)?,
                    val(7)?,
                    val(9)?,
                    val(4)?,
                    val(8)?,
                ],
            });
        }
        data.trajectory_note = format!(
            "loaded f64 trajectories; stored stride {}",
            scene["trajectory_stride"]
        );
        data.validate()?;
        return Ok(data);
    }
    // Earlier v0.2 output has complete launch/diagnostics/hit data but no full paths.
    let experiment: SourceDetectorExperiment = serde_json::from_value(run["experiment"].clone())?;
    let initials = experiment.initials()?;
    let mut reader = csv::Reader::from_path(directory.join("ray_diagnostics.csv"))?;
    let headers = reader.headers()?.clone();
    let mut rays = Vec::new();
    for row in reader.records() {
        let row = row?;
        let field = |name: &str| -> Result<&str, String> {
            let i = headers
                .iter()
                .position(|s| s == name)
                .ok_or_else(|| format!("missing diagnostic {name}"))?;
            row.get(i).ok_or_else(|| format!("short diagnostic {name}"))
        };
        let f = |name: &str| -> Result<f64, String> {
            field(name)?.parse::<f64>().map_err(|e| e.to_string())
        };
        let id: usize = field("ray_id")?.parse()?;
        let (emission, _) = initials
            .get(id)
            .ok_or("legacy ray ID out of source range")?;
        let initial = [
            f("initial_t")?,
            f("initial_r")?,
            f("initial_phi")?,
            f("initial_pt")?,
            f("initial_pr")?,
            f("initial_pphi")?,
            f("initial_theta")?,
            f("initial_ptheta")?,
        ];
        let status: RayStatus = serde_json::from_value(json!(field("status")?))?;
        rays.push(RayRecord {
            launch_direction: None,
            ray_id: id,
            source_uv: [emission.u, emission.v],
            initial,
            final_state: None,
            end_time: initial[0] + f("coordinate_time")?,
            status,
            stop_reason: field("stop_reason")?.into(),
            failure: if field("failure_message")?.is_empty() {
                None
            } else {
                Some(field("failure_message")?.into())
            },
            errors: [
                f("max_null_abs")?,
                f("max_E_relative")?,
                f("max_Lz_relative")?,
                f("max_Q_abs")?,
                f("max_Q_relative")?,
            ],
            accepted_steps: field("accepted_steps")?.parse()?,
            rejected_steps: field("rejected_steps")?.parse()?,
            compute_seconds: f("seconds")?,
            original_sample_count: 0,
            samples: Vec::new(),
        });
    }
    let data=SimulationData {experiment,source_available:true,rays,events,seconds:run["physics_seconds"].as_f64().unwrap_or(0.0),trajectory_note:"Legacy event-only data: trajectories are absent. Recompute explicitly to obtain paths; original files remain unchanged.".into(),diagnostic_csv:fs::read_to_string(directory.join("ray_diagnostics.csv"))?,hit_state_csv:fs::read_to_string(directory.join("detector_hit_states.csv"))?};
    data.validate()?;
    Ok(data)
}
