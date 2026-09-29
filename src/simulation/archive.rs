//! Adapters between the existing standard CPU API and the legacy viewer data.
use super::{RayRecord, SimulationData, TraceSample, rays::RayOutcome};
use crate::{
    compute::RayStatus,
    detector::events::HitEvent,
    experiments::source_detector::SourceDetectorExperiment,
    physics::{
        coordinates::{dot, sub},
        kerr::Kerr,
    },
};

pub fn scene(
    outcomes: Vec<RayOutcome>,
    exp: SourceDetectorExperiment,
    source_available: bool,
    seconds: f64,
) -> Result<SimulationData, String> {
    let k = Kerr::new(exp.spin)?;
    let mut rays = Vec::with_capacity(outcomes.len());
    let mut events = Vec::new();
    use std::fmt::Write;
    let mut diagnostic_csv = format!("{}\n", crate::experiments::output::SUMMARY_HEADER);
    let mut hit_state_csv = String::from(
        "ray_id,affine,t,r,theta,phi,pt,pr,ptheta,pphi,C_null,Q,root_signed_residual\n",
    );
    for outcome in outcomes {
        let id = outcome.ray.ray_id();
        let initial = outcome.ray.initial();
        // UV is only meaningful when the generating SourcePlane is available.
        // The legacy event schema requires finite placeholders otherwise; these
        // zeros are never interpreted as source measurements or replay inputs.
        let uv = if source_available {
            let d = sub(outcome.ray.condition().position(), exp.source.plane.center);
            [dot(d, exp.source.plane.e_u), dot(d, exp.source.plane.e_v)]
        } else {
            [0.0; 2]
        };
        let out = outcome.result;
        if let Some(hit) = &out.detection {
            let s = hit.state;
            writeln!(hit_state_csv,"{id},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",hit.affine,s.time(),s.radius(),s.theta(),s.phi(),s.0[3],s.0[4],s.0[7],s.0[5],s.null_constraint(k)?,s.carter_q(k),hit.signed_residual).unwrap();
            events.push(HitEvent {
                ray_id: id,
                u_source: uv[0],
                v_source: uv[1],
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
            initial,
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
            launch_direction: Some(outcome.ray.condition().direction()),
            source_uv: [uv[0], uv[1]],
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
        experiment: exp,
        source_available,
        rays,
        events,
        seconds,
        trajectory_note: if source_available {
            "Canonical CPU trajectories; standard auto-save enabled".into()
        } else {
            "Fresh reproduction trajectories. SourcePlane/UV were not stored and are unavailable."
                .into()
        },
        diagnostic_csv,
        hit_state_csv,
    };
    data.validate()?;
    Ok(data)
}
