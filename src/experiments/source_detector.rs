//! 학생이 광원/검출면과 시간·공간 표본 설정을 한 곳에서 수정하는 실험.
//! 기본 평면은 먼 영역의 좌표에 고정된 검출면이며 면적은 좌표 면적입니다.
use crate::{
    compute::{RayStatus, cpu::CpuReferenceIntegrator},
    config::IntegratorConfig,
    detector::{
        events::HitEvent,
        plane::{DetectorPlane, Plane},
    },
    output::{
        OutputResult, csv, metadata,
        time_frames::{self, PostprocessConfig},
    },
    physics::{
        coordinates::{Vec3, cartesian_to_local, from_cartesian},
        geodesic::State,
        kerr::Kerr,
        tetrad::ray_3d,
    },
};
use serde_json::json;
use std::{
    fs,
    io::{BufWriter, Write},
    path::PathBuf,
    time::Instant,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum LaunchDirection {
    LocalZamo(Vec3),
    CartesianSpatial(Vec3),
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Emission {
    pub u: f64,
    pub v: f64,
    pub t_emit: Option<f64>,
    pub direction: Option<LaunchDirection>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SourcePattern {
    Single,
    SquareGrid { side: usize },
    RectangularGrid { nu: usize, nv: usize },
    Explicit(Vec<Emission>),
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SourcePlane {
    pub plane: Plane,
    pub pattern: SourcePattern,
    pub direction: LaunchDirection,
    pub t_emit: f64,
    pub local_energy: f64,
}
impl SourcePlane {
    pub fn emissions(&self) -> Result<Vec<Emission>, String> {
        let grid = |nu: usize, nv: usize| -> Result<Vec<Emission>, String> {
            if nu == 0 || nv == 0 || nu.checked_mul(nv).is_none_or(|n| n > 100_000) {
                return Err("grid count must be 1..100000".into());
            }
            Ok((0..nv)
                .flat_map(|v| {
                    (0..nu).map(move |u| Emission {
                        u: ((u as f64 + 0.5) / nu as f64 - 0.5) * self.plane.width,
                        v: ((v as f64 + 0.5) / nv as f64 - 0.5) * self.plane.height,
                        t_emit: None,
                        direction: None,
                    })
                })
                .collect())
        };
        let list = match &self.pattern {
            SourcePattern::Single => grid(1, 1)?,
            SourcePattern::SquareGrid { side } => grid(*side, *side)?,
            SourcePattern::RectangularGrid { nu, nv } => grid(*nu, *nv)?,
            SourcePattern::Explicit(list) => list.clone(),
        };
        if list.is_empty()
            || list.len() > 100_000
            || list.iter().any(|e| {
                !self.plane.contains([e.u, e.v]) || e.t_emit.is_some_and(|t| !t.is_finite())
            })
        {
            return Err("invalid emission list".into());
        }
        Ok(list)
    }
    pub fn initial(&self, k: Kerr, e: &Emission) -> Result<State, String> {
        let q = from_cartesian(k, self.plane.position(e.u, e.v))?;
        let n = match e.direction.as_ref().unwrap_or(&self.direction) {
            LaunchDirection::LocalZamo(n) => *n,
            LaunchDirection::CartesianSpatial(d) => cartesian_to_local(k, q, *d)?,
        };
        ray_3d(
            k,
            [e.t_emit.unwrap_or(self.t_emit), q[0], q[1], q[2]],
            n,
            self.local_energy,
        )
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDetectorExperiment {
    pub spin: f64,
    pub source: SourcePlane,
    pub detector: DetectorPlane,
    pub integration: IntegratorConfig,
    pub postprocess: PostprocessConfig,
    pub output: PathBuf,
}
impl Default for SourceDetectorExperiment {
    fn default() -> Self {
        let plane = |x, width, height| {
            Plane::from_normal_up(
                [x, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                width,
                height,
            )
            .expect("orthonormal default plane")
        };
        Self {
            spin: 0.6,
            source: SourcePlane {
                plane: plane(80.0, 32.0, 32.0),
                pattern: SourcePattern::RectangularGrid { nu: 16, nv: 16 },
                direction: LaunchDirection::CartesianSpatial([-1.0, 0.0, 0.0]),
                t_emit: 0.0,
                local_energy: 1.0,
            },
            detector: DetectorPlane {
                plane: plane(-80.0, 300.0, 300.0),
                resolution: [128, 128],
                root_tolerance: 1e-10,
            },
            integration: IntegratorConfig {
                escape_radius: 400.0,
                max_affine: 1500.0,
                ..Default::default()
            },
            postprocess: PostprocessConfig::default(),
            output: PathBuf::from("results/source_to_detector"),
        }
    }
}
impl SourceDetectorExperiment {
    pub fn initials(&self) -> Result<Vec<(Emission, State)>, String> {
        let k = Kerr::new(self.spin)?;
        self.source.plane.validate_stationary(k)?;
        self.detector.validate(k)?;
        self.integration.validate(k)?;
        let list = self.source.emissions()?;
        let mut rays = Vec::with_capacity(list.len());
        for e in list {
            let s = self.source.initial(k, &e)?;
            if s.radius() <= k.horizon() + self.integration.horizon_epsilon
                || s.radius() >= self.integration.escape_radius
            {
                return Err("source must be between capture cutoff and escape radius".into());
            }
            rays.push((e, s));
        }
        Ok(rays)
    }
}
pub fn simulate(exp: &SourceDetectorExperiment) -> OutputResult<serde_json::Value> {
    let rays = exp.initials()?;
    let k = Kerr::new(exp.spin)?;
    fs::create_dir_all(&exp.output)?;
    // All primary filenames use create_new: an existing run is never overwritten.
    let started = metadata::unix_seconds();
    let timer = Instant::now();
    let provenance = json!({"schema_version":2,"experiment_id":format!("source_to_detector-{started:.6}"),"experiment":exp,"ray_count":rays.len(),"M":1.0,"G":1.0,"c":1.0,
        "coordinates":"Boyer-Lindquist (t,r,theta,phi), signature (-,+,+,+)",
        "plane_map":"X=sqrt(r^2+a^2)*sin(theta)*cos(phi), Y=sqrt(r^2+a^2)*sin(theta)*sin(phi), Z=r*cos(theta); T=t_BL; NOT Kerr-Schild",
        "plane_worldlines":"fixed Cartesian-like BL coordinates; guaranteed static timelike exterior r>2; coordinate areas, not proper areas",
        "time_convention":"f64 t_hit=t_BL; delta_t=t_hit-t_emit; affine parameter is not photon proper time",
        "integrator":"CPU f64 Dormand-Prince 5(4), adaptive; 8 canonical variables; no constraint projection",
        "detector":"absorbing, first valid transverse crossing; both rectangle edges included",
        "code_version":env!("CARGO_PKG_VERSION"),"build_source_fnv1a64":env!("KERR_SOURCE_FINGERPRINT"),
        "executed_at_unix_seconds":started,"date_encoding":"seconds since 1970-01-01T00:00:00Z (UTC)","build_mode":if cfg!(debug_assertions){"debug"}else{"release"}});
    metadata::write(&exp.output.join("run_started.json"), &provenance)?;
    let events_path = exp.output.join("detector_events.csv");
    let mut ew = ::csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(csv::new_file(&events_path)?);
    ew.write_record([
        "ray_id", "u_source", "v_source", "t_emit", "u_hit", "v_hit", "t_hit", "delta_t", "status",
    ])?;
    let mut diagnostics = BufWriter::new(csv::new_file(&exp.output.join("ray_diagnostics.csv"))?);
    writeln!(diagnostics, "{}", super::output::SUMMARY_HEADER)?;
    let mut hw = BufWriter::new(csv::new_file(&exp.output.join("detector_hit_states.csv"))?);
    writeln!(
        hw,
        "ray_id,affine,t,r,theta,phi,pt,pr,ptheta,pphi,C_null,Q,root_signed_residual"
    )?;
    let mut events = Vec::new();
    let mut counts = [0usize; 5];
    let mut compute_seconds = 0.0;
    let mut errors = [0.0_f64; 5];
    for (id, (emission, initial)) in rays.iter().enumerate() {
        let out = CpuReferenceIntegrator.integrate_with_detector(
            k,
            *initial,
            &exp.integration,
            &exp.detector,
        );
        counts[status_index(out.status)] += 1;
        compute_seconds += out.elapsed_seconds;
        let d = &out.diagnostics;
        for (max, value) in errors.iter_mut().zip([
            d.max_null_abs,
            d.max_energy_relative,
            d.max_lz_relative,
            d.max_q_abs,
            d.max_q_relative,
        ]) {
            *max = max.max(value);
        }
        let c = super::scenarios::Case {
            label: format!("source_ray_{id}"),
            kerr: k,
            initial: *initial,
            config: exp.integration,
        };
        writeln!(diagnostics, "{}", super::output::summary_row(id, &c, &out))?;
        if let Some(hit) = out.detection {
            let event = HitEvent {
                ray_id: id,
                u_source: emission.u,
                v_source: emission.v,
                t_emit: initial.time(),
                u_hit: hit.uv[0],
                v_hit: hit.uv[1],
                t_hit: hit.state.time(),
                delta_t: hit.state.time() - initial.time(),
                status: RayStatus::Detected,
            };
            event.validate()?;
            ew.serialize(&event)?;
            ew.flush()?;
            events.push(event);
            let s = hit.state;
            writeln!(
                hw,
                "{id},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                hit.affine,
                s.time(),
                s.radius(),
                s.theta(),
                s.phi(),
                s.0[3],
                s.0[4],
                s.0[7],
                s.0[5],
                s.null_constraint(k)?,
                s.carter_q(k),
                hit.signed_residual
            )?;
        }
    }
    ew.flush()?;
    ew.get_ref().sync_all()?;
    diagnostics.flush()?;
    hw.flush()?;
    let mut report = provenance;
    report["outcomes"] = json!({"Active":counts[0],"Detected":counts[1],"Captured":counts[2],"Escaped":counts[3],"NumericalFailure":counts[4]});
    report["max_errors"] = json!({"null_abs":errors[0],"E_relative":errors[1],"Lz_relative":errors[2],"Q_abs":errors[3],"Q_relative":errors[4]});
    report["physics_seconds"] = json!(compute_seconds);
    report["events_fnv1a64"] = json!(metadata::fingerprint(&events_path)?);
    // Final primary metadata is available even if optional image generation subsequently fails.
    report["physics_and_primary_output_seconds"] = json!(timer.elapsed().as_secs_f64());
    metadata::write(&exp.output.join("run_metadata.json"), &report)?;
    let post = Instant::now();
    let derived = time_frames::generate(
        &events_path,
        &events,
        &exp.detector,
        &exp.postprocess,
        &exp.output,
    )?;
    metadata::write(
        &exp.output.join("output_timing.json"),
        &json!({"postprocess_seconds":post.elapsed().as_secs_f64(),"total_seconds":timer.elapsed().as_secs_f64()}),
    )?;
    println!(
        "source_to_detector: {} rays; outcomes={}\nmax errors={}\nphysics={compute_seconds:.6}s; frames={}\nraw events={}",
        rays.len(),
        report["outcomes"],
        report["max_errors"],
        derived["frame_count"],
        events_path.display()
    );
    if counts[4] > 0 {
        return Err(
            "numerical failures recorded; see ray_diagnostics.csv (raw events preserved)".into(),
        );
    }
    Ok(report)
}
pub fn status_index(s: RayStatus) -> usize {
    match s {
        RayStatus::Active => 0,
        RayStatus::Detected => 1,
        RayStatus::Captured => 2,
        RayStatus::Escaped => 3,
        RayStatus::NumericalFailure => 4,
    }
}

pub fn rebin(
    input: &std::path::Path,
    output: &std::path::Path,
    post: &PostprocessConfig,
    resolution: Option<[usize; 2]>,
) -> OutputResult<serde_json::Value> {
    let meta: serde_json::Value =
        serde_json::from_reader(fs::File::open(input.join("run_metadata.json"))?)?;
    let exp: SourceDetectorExperiment = serde_json::from_value(meta["experiment"].clone())?;
    let events_path = input.join("detector_events.csv");
    if meta["events_fnv1a64"] != metadata::fingerprint(&events_path)? {
        return Err("raw event checksum differs from run metadata".into());
    }
    let events = csv::read_events(&events_path)?;
    let mut detector = exp.detector;
    if let Some(r) = resolution {
        detector.resolution = r;
    }
    detector.validate(Kerr::new(exp.spin)?)?;
    let out = time_frames::generate(&events_path, &events, &detector, post, output)?;
    println!(
        "Rebinned {} saved events; no geodesic integration; frames={} -> {}",
        events.len(),
        out["frame_count"],
        output.display()
    );
    Ok(out)
}
