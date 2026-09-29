//! Existing CSVs are comparison oracles only. They NEVER supply new ray IDs,
//! conditions, Kerr states, or integration outputs. No trajectory file is read.
use kerr_ray::{
    compute::RayStatus,
    experiments::{
        independent_rays::from_source_plane,
        source_detector::{SourceDetectorExperiment, SourcePattern, status_index},
    },
    simulation::rays::{PreparedRay, RayInitialCondition, RaySimulationConfig, prepare_rays},
};
use std::{collections::HashMap, path::PathBuf};

pub struct Expected {
    pub status: RayStatus,
    pub hit: Option<[f64; 3]>,
}
pub struct Baseline {
    pub conditions: Vec<RayInitialCondition>,
    pub config: RaySimulationConfig,
    pub prepared: Vec<PreparedRay>,
    pub expected: HashMap<[u64; 8], Expected>,
    pub counts: [usize; 5],
}
pub fn baseline(side: usize) -> Baseline {
    // Fresh code-generated physical inputs, independent of all saved records.
    let mut exp = SourceDetectorExperiment::default();
    exp.source.pattern = SourcePattern::RectangularGrid { nu: side, nv: side };
    let conditions = from_source_plane(&exp.source).unwrap();
    let config = RaySimulationConfig {
        spin: exp.spin,
        integration: exp.integration,
        detector: exp.detector.clone(),
    };
    let prepared = prepare_rays(conditions.clone(), &config).unwrap();
    let legacy = exp.initials().unwrap();
    let position_key = |p: [f64; 3]| p.map(f64::to_bits);
    let by_position: HashMap<_, _> = legacy
        .iter()
        .map(|(emission, state)| {
            (
                position_key(exp.source.plane.position(emission.u, emission.v)),
                *state,
            )
        })
        .collect();
    for (id, ray) in prepared.iter().enumerate() {
        assert_eq!(ray.ray_id(), id);
        assert_eq!(ray.condition().direction(), [-1.0, 0.0, 0.0]);
        assert_eq!(
            ray.initial().0.map(f64::to_bits),
            by_position[&position_key(ray.condition().position())]
                .0
                .map(f64::to_bits)
        );
    }
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("results")
        .join(if side == 16 {
            "source_to_detector"
        } else {
            "visualization_4096"
        });
    let mut events = csv::Reader::from_path(directory.join("detector_events.csv")).unwrap();
    let hits: HashMap<usize, [f64; 3]> = events
        .records()
        .map(|row| {
            let row = row.unwrap();
            (
                row[0].parse().unwrap(),
                [
                    row[4].parse().unwrap(),
                    row[5].parse().unwrap(),
                    row[6].parse().unwrap(),
                ],
            )
        })
        .collect();
    let mut reader = csv::Reader::from_path(directory.join("ray_diagnostics.csv")).unwrap();
    let header = reader.headers().unwrap().clone();
    let index = |name: &str| header.iter().position(|h| h == name).unwrap();
    let mut expected = HashMap::new();
    let mut counts = [0; 5];
    for row in reader.records() {
        let row = row.unwrap();
        let old_id: usize = row[index("ray_id")].parse().unwrap();
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
        .map(|name| row[index(name)].parse::<f64>().unwrap());
        // Compare saved states to independently regenerated legacy inputs.
        assert_eq!(
            state.map(f64::to_bits),
            legacy[old_id].1.0.map(f64::to_bits)
        );
        let status = match &row[index("status")] {
            "Detected" => RayStatus::Detected,
            "Captured" => RayStatus::Captured,
            "Escaped" => RayStatus::Escaped,
            "NumericalFailure" => RayStatus::NumericalFailure,
            "Active" => RayStatus::Active,
            other => panic!("unknown status {other}"),
        };
        counts[status_index(status)] += 1;
        let hit = hits.get(&old_id).copied();
        assert_eq!(hit.is_some(), status == RayStatus::Detected);
        assert!(
            expected
                .insert(state.map(f64::to_bits), Expected { status, hit })
                .is_none()
        );
    }
    assert_eq!(expected.len(), side * side);
    assert_eq!(
        counts,
        if side == 16 {
            [0, 216, 20, 20, 0]
        } else {
            [0, 3502, 330, 264, 0]
        }
    );
    for ray in &prepared {
        assert!(expected.contains_key(&ray.initial().0.map(f64::to_bits)));
    }
    Baseline {
        conditions,
        config,
        prepared,
        expected,
        counts,
    }
}
