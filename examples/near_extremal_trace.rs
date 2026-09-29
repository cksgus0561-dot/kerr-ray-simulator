//! Validation-only: re-integrate just the nine previously failed ray/run pairs.
//! No integrator changes, state projection, or new acceptance policy.
use kerr_ray::{
    compute::cpu::CpuReferenceIntegrator,
    physics::{
        geodesic::{inverse_derivatives, rhs},
        kerr::Kerr,
        metric::Metric,
    },
    simulation::rays::prepare_rays,
    standard_run,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let old = Path::new(args.first().ok_or("previous validation root")?);
    let dest = Path::new(args.get(1).ok_or("new validation root")?);
    let analysis: Value = serde_json::from_slice(&fs::read(old.join("analysis.json"))?)?;
    let mut records = Vec::new();
    for label in ["p09999_B", "p09999_C", "n09999_C"] {
        let stored = standard_run::load(&old.join(label).join("simulation_1"), "sim_1.bin")?;
        let cfg = stored.common.config(stored.chi)?;
        let rays = prepare_rays(standard_run::seeded::verify(&stored)?, &cfg)?;
        let k = Kerr::new(stored.chi)?;
        for fail in analysis["runs"][label]["failures"]
            .as_array()
            .ok_or("failures")?
        {
            let id = fail["ray_id"].as_u64().ok_or("id")? as usize;
            let ray = &rays[id];
            let result = CpuReferenceIntegrator.integrate_with_detector(
                k,
                ray.initial(),
                &cfg.integration,
                &cfg.detector,
            );
            let last = result.samples.last().ok_or("empty trajectory")?;
            let expected: [f64; 8] = serde_json::from_value(fail["last_state"].clone())?;
            assert_eq!(last.state.0.map(f64::to_bits), expected.map(f64::to_bits));
            assert_eq!(
                format!("{:?}", result.status),
                fail["status"].as_str().unwrap()
            );
            let mut indices: BTreeSet<_> =
                (result.samples.len().saturating_sub(64)..result.samples.len()).collect();
            indices.insert(0);
            for gap in [
                80.0, 20.0, 5.0, 1.0, 0.2, 0.05, 0.01, 0.005, 0.003, 0.002, 0.0015,
            ] {
                if let Some(i) = result
                    .samples
                    .iter()
                    .position(|s| s.state.radius() - k.horizon() <= gap)
                {
                    indices.insert(i);
                }
            }
            let probes: Vec<_> = indices
                .into_iter()
                .map(|i| {
                    let sample = result.samples[i];
                    let s = sample.state;
                    json!({"sample":i,"affine":sample.affine,"state":s.0,
                        "delta":k.delta(s.radius()),"null":s.null_constraint(k).unwrap(),
                        "covariant":k.covariant(s.radius(),s.theta()).unwrap(),
                        "inverse":k.inverse(s.radius(),s.theta()).unwrap(),
                        "inverse_derivatives":inverse_derivatives(k,s.radius(),s.theta()).unwrap(),
                        "rhs":rhs(k,&s.0).unwrap()
                    })
                })
                .collect();
            records.push(json!({
                "label":label,"ray_id":id,"chi":stored.chi,"initial":ray.initial().0,
                "status":result.status,"stop_reason":format!("{:?}",result.stop_reason),
                "failure":result.failure_message,"integration":cfg.integration,
                "horizon":k.horizon(),"accepted_steps":result.diagnostics.accepted_steps,
                "rejected_steps":result.diagnostics.rejected_steps,
                "probes":probes,"final_state_matches_previous_bits":true
            }));
        }
    }
    fs::write(
        dest.join("focused_traces.json"),
        serde_json::to_vec(&records)?,
    )?;
    println!("{} focused failures reproduced exactly", records.len());
    Ok(())
}
