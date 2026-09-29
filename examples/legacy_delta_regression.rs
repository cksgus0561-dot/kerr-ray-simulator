//! Measure legacy physical outcomes without changing historical bitwise tests.
use kerr_ray::{
    experiments::{
        independent_rays::from_source_plane,
        source_detector::{SourceDetectorExperiment, SourcePattern, status_index},
    },
    simulation::rays::{RaySimulationConfig, calculate_rays},
};
use serde_json::json;
use std::{collections::HashMap, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(std::env::args().nth(1).ok_or("output directory")?);
    let mut reports = Vec::new();
    for side in [16, 64] {
        let mut e = SourceDetectorExperiment::default();
        e.source.pattern = SourcePattern::RectangularGrid { nu: side, nv: side };
        let cfg = RaySimulationConfig {
            spin: e.spin,
            integration: e.integration,
            detector: e.detector,
        };
        let dir = PathBuf::from("results").join(if side == 16 {
            "source_to_detector"
        } else {
            "visualization_4096"
        });
        let mut expected_hits = HashMap::new();
        for r in csv::Reader::from_path(dir.join("detector_events.csv"))?.records() {
            let r = r?;
            expected_hits.insert(
                r[0].parse::<usize>()?,
                [r[4].parse::<f64>()?, r[5].parse()?, r[6].parse()?],
            );
        }
        let coord_key = |s: [f64; 8]| [s[0], s[1], s[2], s[6]].map(f64::to_bits);
        let mut expected = HashMap::new();
        let mut reader = csv::Reader::from_path(dir.join("ray_diagnostics.csv"))?;
        let header = reader.headers()?.clone();
        let index = |name: &str| header.iter().position(|h| h == name).unwrap();
        for r in reader.records() {
            let r = r?;
            let state = [
                "initial_t",
                "initial_r",
                "initial_phi",
                "initial_pt",
                "initial_pr",
                "initial_pphi",
                "initial_theta",
                "initial_ptheta",
            ]
            .map(|name| r[index(name)].parse::<f64>().unwrap());
            expected.insert(
                coord_key(state),
                (
                    state,
                    r[index("status")].to_string(),
                    r[index("ray_id")].parse::<usize>()?,
                ),
            );
        }
        let outcomes = calculate_rays(from_source_plane(&e.source)?, &cfg, |_, _| true)?;
        let mut counts = [0; 5];
        let mut statuses = 0;
        let mut state_bits = 0;
        let mut state_max_abs = [0.0_f64; 8];
        let mut hit_max = [0.0_f64; 3];
        let mut hit_sum = [0.0_f64; 3];
        let mut detected = 0;
        let mut hit_presence = 0;
        for out in outcomes {
            let initial = out.ray.initial().0;
            let (old, status, id) = &expected[&coord_key(initial)];
            state_bits += usize::from(initial.map(f64::to_bits) != old.map(f64::to_bits));
            for j in 0..8 {
                state_max_abs[j] = state_max_abs[j].max((initial[j] - old[j]).abs());
            }
            counts[status_index(out.result.status)] += 1;
            statuses += usize::from(format!("{:?}", out.result.status) != *status);
            match (out.result.detection, expected_hits.get(id)) {
                (Some(hit), Some(old)) => {
                    detected += 1;
                    let new = [hit.uv[0], hit.uv[1], hit.state.time()];
                    for j in 0..3 {
                        let delta = (new[j] - old[j]).abs();
                        hit_max[j] = hit_max[j].max(delta);
                        hit_sum[j] += delta;
                    }
                }
                (None, None) => {}
                _ => hit_presence += 1,
            }
        }
        let report = json!({"rays":side*side,"counts":counts,"status_mismatches":statuses,
            "initial_state_bit_mismatch_rays":state_bits,"initial_state_max_abs":state_max_abs,
            "hit_presence_mismatches":hit_presence,"common_detected":detected,
            "hit_max_abs":hit_max,"hit_mean_abs":hit_sum.map(|v|v/detected as f64)});
        println!("{report}");
        reports.push(report);
    }
    fs::write(
        root.join("legacy_outcome_regression.json"),
        serde_json::to_vec_pretty(&reports)?,
    )?;
    Ok(())
}
