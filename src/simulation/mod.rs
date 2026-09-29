//! CPU scheduling/data boundary. No egui or wgpu types, and no display subsampling.
pub mod archive;
pub mod ray_execution;
pub mod rays;
pub mod storage;
pub mod worker;
use crate::{
    compute::{RayStatus, cpu::CpuReferenceIntegrator},
    detector::events::HitEvent,
    experiments::source_detector::{SourceDetectorExperiment, status_index},
    physics::{geodesic::State, kerr::Kerr},
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceSample {
    pub affine: f64,
    pub state: [f64; 8],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RayRecord {
    pub ray_id: usize,
    pub source_uv: [f64; 2],
    #[serde(default)]
    pub launch_direction: Option<[f64; 3]>,
    pub initial: [f64; 8],
    pub final_state: Option<[f64; 8]>,
    pub end_time: f64,
    pub status: RayStatus,
    pub stop_reason: String,
    pub failure: Option<String>,
    /// null absolute, E relative, Lz relative, Q absolute, Q relative.
    pub errors: [f64; 5],
    pub accepted_steps: usize,
    pub rejected_steps: usize,
    pub compute_seconds: f64,
    pub original_sample_count: usize,
    #[serde(skip)]
    pub samples: Vec<TraceSample>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationData {
    pub experiment: SourceDetectorExperiment,
    #[serde(default = "source_present")]
    pub source_available: bool,
    pub rays: Vec<RayRecord>,
    pub events: Vec<HitEvent>,
    pub seconds: f64,
    pub trajectory_note: String,
    /// Original diagnostic schema and continuous intersection state table are
    /// preserved, even when trajectory export is thinned or unavailable.
    #[serde(default)]
    pub diagnostic_csv: String,
    #[serde(default)]
    pub hit_state_csv: String,
}
fn source_present() -> bool {
    true
}
impl SimulationData {
    pub fn counts(&self) -> [usize; 5] {
        let mut counts = [0; 5];
        for ray in &self.rays {
            counts[status_index(ray.status)] += 1;
        }
        counts
    }
    pub fn max_errors(&self) -> [f64; 5] {
        let mut max = [0.0_f64; 5];
        for ray in &self.rays {
            for (m, e) in max.iter_mut().zip(ray.errors) {
                *m = m.max(e);
            }
        }
        max
    }
    pub fn time_range(&self) -> [f64; 2] {
        let mut range = [f64::INFINITY, f64::NEG_INFINITY];
        for r in &self.rays {
            range[0] = range[0].min(r.initial[0]);
            range[1] = range[1].max(r.end_time);
        }
        if !range[0].is_finite() {
            [0.0, 1.0]
        } else {
            [range[0], range[1].max(range[0] + 1e-6)]
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = std::collections::HashSet::new();
        for ray in &self.rays {
            if !ids.insert(ray.ray_id) {
                return Err("duplicate trajectory ray_id".into());
            }
            if ray
                .initial
                .iter()
                .chain(ray.final_state.iter().flatten())
                .any(|x| !x.is_finite())
                || !ray.end_time.is_finite()
            {
                return Err("invalid ray coordinates".into());
            }
            let mut last = f64::NEG_INFINITY;
            for s in &ray.samples {
                if s.state.iter().any(|v| !v.is_finite())
                    || !s.affine.is_finite()
                    || s.state[0] < last
                {
                    return Err("invalid or unordered trajectory times".into());
                }
                last = s.state[0];
            }
        }
        let statuses: std::collections::HashMap<_, _> =
            self.rays.iter().map(|r| (r.ray_id, r.status)).collect();
        let mut event_ids = std::collections::HashSet::new();
        for e in &self.events {
            e.validate()?;
            if !event_ids.insert(e.ray_id) || statuses.get(&e.ray_id) != Some(&RayStatus::Detected)
            {
                return Err("event/ray identity mismatch".into());
            }
        }
        if self.counts()[1] != self.events.len() {
            return Err("Detected/event count mismatch".into());
        }
        Ok(())
    }
}

/// Cancellation is checked between independent rays; an individual reference solve is untouched.
pub fn calculate(
    exp: &SourceDetectorExperiment,
    mut keep_going: impl FnMut(usize, usize) -> bool,
) -> Result<SimulationData, String> {
    let started = Instant::now();
    let initials = exp.initials()?;
    let k = Kerr::new(exp.spin)?;
    let mut rays = Vec::with_capacity(initials.len());
    let mut events = Vec::new();
    use std::fmt::Write;
    let mut diagnostic_csv = format!("{}\n", crate::experiments::output::SUMMARY_HEADER);
    let mut hit_state_csv = String::from(
        "ray_id,affine,t,r,theta,phi,pt,pr,ptheta,pphi,C_null,Q,root_signed_residual\n",
    );
    for (id, (emission, initial)) in initials.iter().enumerate() {
        if !keep_going(id, initials.len()) {
            return Err("cancelled by a newer generation".into());
        }
        let out = CpuReferenceIntegrator.integrate_with_detector(
            k,
            *initial,
            &exp.integration,
            &exp.detector,
        );
        if let Some(hit) = &out.detection {
            let s = hit.state;
            writeln!(hit_state_csv,"{id},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",hit.affine,s.time(),s.radius(),s.theta(),s.phi(),s.0[3],s.0[4],s.0[7],s.0[5],s.null_constraint(k)?,s.carter_q(k),hit.signed_residual).unwrap();
            events.push(HitEvent {
                ray_id: id,
                u_source: emission.u,
                v_source: emission.v,
                t_emit: initial.time(),
                u_hit: hit.uv[0],
                v_hit: hit.uv[1],
                t_hit: hit.state.time(),
                delta_t: hit.state.time() - initial.time(),
                status: RayStatus::Detected,
            });
        }
        let d = &out.diagnostics;
        let case = crate::experiments::scenarios::Case {
            label: format!("source_ray_{id}"),
            kerr: k,
            initial: *initial,
            config: exp.integration,
        };
        writeln!(
            diagnostic_csv,
            "{}",
            crate::experiments::output::summary_row(id, &case, &out)
        )
        .unwrap();
        rays.push(RayRecord {
            ray_id: id,
            source_uv: [emission.u, emission.v],
            launch_direction: None,
            initial: initial.0,
            final_state: out.samples.last().map(|s| s.state.0),
            end_time: out
                .samples
                .last()
                .map_or(initial.time(), |s| s.state.time()),
            status: out.status,
            stop_reason: format!("{:?}", out.stop_reason),
            failure: out.failure_message,
            errors: [
                d.max_null_abs,
                d.max_energy_relative,
                d.max_lz_relative,
                d.max_q_abs,
                d.max_q_relative,
            ],
            accepted_steps: d.accepted_steps,
            rejected_steps: d.rejected_steps,
            compute_seconds: out.elapsed_seconds,
            original_sample_count: out.samples.len(),
            samples: out
                .samples
                .into_iter()
                .map(|s| TraceSample {
                    affine: s.affine,
                    state: s.state.0,
                })
                .collect(),
        });
    }
    let data = SimulationData {
        experiment: exp.clone(),
        source_available: true,
        rays,
        events,
        diagnostic_csv,
        hit_state_csv,
        seconds: started.elapsed().as_secs_f64(),
        trajectory_note: "all accepted f64 reference samples retained in memory".into(),
    };
    data.validate()?;
    Ok(data)
}

pub fn state(sample: &TraceSample) -> State {
    State(sample.state)
}
