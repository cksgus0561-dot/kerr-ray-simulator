use kerr_ray::{
    compute::RayStatus,
    config::IntegratorConfig,
    detector::plane::{DetectorPlane, Plane},
    experiments::{
        independent_rays::from_source_plane,
        source_detector::{Emission, LaunchDirection, SourceDetectorExperiment, SourcePattern},
    },
    physics::{coordinates::bl_to_cartesian, kerr::Kerr},
    simulation::rays::{RayInitialCondition, RaySimulationConfig, calculate_rays, prepare_rays},
};
#[path = "support/ray_regression.rs"]
mod ray_regression;
fn config() -> RaySimulationConfig {
    RaySimulationConfig {
        spin: 0.6,
        integration: IntegratorConfig {
            escape_radius: 400.0,
            max_affine: 1500.0,
            ..Default::default()
        },
        detector: DetectorPlane {
            plane: Plane::from_normal_up(
                [80.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                300.0,
                300.0,
            )
            .unwrap(),
            resolution: [128, 128],
            root_tolerance: 1e-10,
        },
    }
}
fn ray() -> RayInitialCondition {
    RayInitialCondition {
        position_x: -48.3,
        position_y: 2.7,
        position_z: -1.2,
        direction_x: 0.998,
        direction_y: 0.04,
        direction_z: 0.03,
        t_emit: 7.0,
    }
}
#[test]
fn arbitrary_positions_directions_and_times_reuse_legacy_conversion() {
    for (position, direction, time) in [
        ([-50.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.0),
        ([-48.3, 2.7, -1.2], [0.998, 0.04, 0.03], 2.0),
        ([-53.1, -0.4, 3.8], [0.994, -0.08, 0.02], 5.0),
    ] {
        let mut exp = SourceDetectorExperiment::default();
        exp.source.plane.center = position;
        exp.source.pattern = SourcePattern::Single;
        exp.source.direction = LaunchDirection::CartesianSpatial(direction);
        exp.source.t_emit = time;
        let original = exp.initials().unwrap()[0].1;
        let new = prepare_rays(from_source_plane(&exp.source).unwrap(), &config()).unwrap();
        assert_eq!(new[0].initial().time(), time);
        for (a, b) in new[0].initial().0.into_iter().zip(original.0) {
            assert!((a - b).abs() <= 2e-13 * b.abs().max(1.0));
        }
        assert!(
            new[0]
                .initial()
                .null_constraint(Kerr::new(0.6).unwrap())
                .unwrap()
                .abs()
                < 1e-12
        );
    }
}
#[test]
fn finite_validation_covers_all_seven_fields() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for index in 0..7 {
            let mut c = ray();
            let fields = [
                &mut c.position_x,
                &mut c.position_y,
                &mut c.position_z,
                &mut c.direction_x,
                &mut c.direction_y,
                &mut c.direction_z,
                &mut c.t_emit,
            ];
            *fields.into_iter().nth(index).unwrap() = bad;
            assert!(prepare_rays(vec![ray(), c], &config()).is_err());
        }
    }
}
#[test]
fn direction_normalization_handles_finite_extremes_and_rejects_zero() {
    for scale in [1e-300, 1.0, 1e300] {
        let mut c = ray();
        c.direction_x = 3.0 * scale;
        c.direction_y = 4.0 * scale;
        c.direction_z = 0.0;
        let result = prepare_rays(vec![c], &config()).unwrap();
        let d = result[0].condition().direction();
        assert!((d[0] - 0.6).abs() < 1e-15 && (d[1] - 0.8).abs() < 1e-15);
    }
    let mut zero = ray();
    zero.direction_x = 0.0;
    zero.direction_y = 0.0;
    zero.direction_z = 0.0;
    assert!(prepare_rays(vec![zero], &config()).is_err());
}
#[test]
fn ids_and_states_are_deterministic_under_permutation_with_duplicates() {
    let a = ray();
    let mut b = a;
    b.position_x -= 3.0;
    let mut c = a;
    c.direction_y = -0.2;
    c.t_emit = 10.0;
    let expected = prepare_rays(vec![a, b, c, a], &config()).unwrap();
    for input in [
        vec![a, b, c, a],
        vec![a, c, b, a],
        vec![c, a, a, b],
        vec![b, a, c, a],
    ] {
        let actual = prepare_rays(input, &config()).unwrap();
        assert_eq!(actual, expected);
        for (id, r) in actual.iter().enumerate() {
            assert_eq!(r.ray_id(), id);
        }
    }
}
#[test]
fn signed_zero_has_one_canonical_representation() {
    let mut a = ray();
    a.position_y = 0.0;
    a.direction_z = 0.0;
    a.t_emit = 0.0;
    let mut b = a;
    b.position_y = -0.0;
    b.direction_z = -0.0;
    b.t_emit = -0.0;
    assert_eq!(
        prepare_rays(vec![a], &config()).unwrap(),
        prepare_rays(vec![b], &config()).unwrap()
    );
    let c = prepare_rays(vec![b], &config()).unwrap()[0].condition();
    assert_eq!(
        [
            c.position_y.to_bits(),
            c.direction_z.to_bits(),
            c.t_emit.to_bits()
        ],
        [0; 3]
    );
}
#[test]
fn canonical_sort_uses_all_keys_in_the_specified_order() {
    let a = ray();
    let mut inputs = vec![a];
    for field in 0..7 {
        let mut b = a;
        let fields = [
            &mut b.position_x,
            &mut b.position_y,
            &mut b.position_z,
            &mut b.direction_x,
            &mut b.direction_y,
            &mut b.direction_z,
            &mut b.t_emit,
        ];
        *fields.into_iter().nth(field).unwrap() += 0.1;
        inputs.push(b);
    }
    let all = prepare_rays(inputs.clone(), &config()).unwrap();
    // Independent tuple comparison after preparation of each singleton.
    let mut singletons: Vec<_> = inputs
        .into_iter()
        .map(|c| prepare_rays(vec![c], &config()).unwrap()[0].condition())
        .collect();
    let key = |c: RayInitialCondition| {
        [
            c.position_x,
            c.position_y,
            c.position_z,
            c.direction_x,
            c.direction_y,
            c.direction_z,
            c.t_emit,
        ]
    };
    singletons.sort_by(|a, b| key(*a).partial_cmp(&key(*b)).unwrap());
    assert_eq!(
        all.iter().map(|r| r.condition()).collect::<Vec<_>>(),
        singletons
    );
}
#[test]
fn per_emission_overrides_survive_the_optional_adapter() {
    let mut exp = SourceDetectorExperiment::default();
    exp.source.pattern = SourcePattern::Explicit(vec![Emission {
        u: 1.0,
        v: 2.0,
        t_emit: Some(9.0),
        direction: Some(LaunchDirection::CartesianSpatial([-1.0, 0.2, 0.1])),
    }]);
    let input = from_source_plane(&exp.source).unwrap();
    assert_eq!(input[0].position(), [80.0, 1.0, 2.0]);
    assert_eq!(input[0].direction(), [-1.0, 0.2, 0.1]);
    assert_eq!(input[0].t_emit, 9.0);
    exp.source.local_energy = 2.0;
    assert!(from_source_plane(&exp.source).is_err());
    exp.source.local_energy = 1.0;
    exp.source.pattern = SourcePattern::Single;
    exp.source.direction = LaunchDirection::LocalZamo([-1.0, 0.0, 0.0]);
    assert!(from_source_plane(&exp.source).is_err());
}
#[test]
fn no_stationary_source_plane_is_required_even_in_ergoregion() {
    let p = bl_to_cartesian(
        Kerr::new(0.6).unwrap(),
        1.95,
        std::f64::consts::FRAC_PI_2,
        0.2,
    );
    let mut c = ray();
    c.position_x = p[0];
    c.position_y = p[1];
    c.position_z = p[2];
    assert!(prepare_rays(vec![c], &config()).is_ok());
}
#[test]
fn initial_domain_and_empty_input_are_checked() {
    assert!(prepare_rays(vec![], &config()).is_err());
    for x in [0.0, 1.0, 500.0] {
        let mut c = ray();
        c.position_x = x;
        c.position_y = 0.0;
        c.position_z = 0.0;
        assert!(prepare_rays(vec![c], &config()).is_err());
    }
}
#[test]
fn identity_reaches_trajectory_status_and_continuous_detection() {
    let mut a = ray();
    a.position_x = 60.0;
    a.direction_x = 1.0;
    a.direction_y = 0.0;
    a.direction_z = 0.0;
    let mut b = a;
    b.position_x = 61.0;
    b.direction_y = 0.03;
    b.t_emit = 15.0;
    let outcomes = calculate_rays(vec![b, a], &config(), |_, _| true).unwrap();
    for (id, out) in outcomes.iter().enumerate() {
        assert_eq!(out.ray.ray_id(), id);
        assert_eq!(out.result.status, RayStatus::Detected);
        assert_eq!(out.result.samples[0].state, out.ray.initial());
        assert_eq!(out.result.samples[0].affine, 0.0);
        let event = out.detection().unwrap();
        assert_eq!(event.ray_id, id);
        assert_eq!(event.t_emit, out.ray.condition().t_emit);
        assert_eq!(event.delta_t, event.hit.state.time() - event.t_emit);
        assert!(event.delta_t > 0.0);
        assert_eq!(out.result.samples.last().unwrap().state, event.hit.state);
    }
}
#[test]
fn cancellation_stops_between_rays_without_reassigning_ids() {
    let mut called = 0;
    assert!(
        calculate_rays(vec![ray()], &config(), |done, total| {
            assert_eq!((done, total), (0, 1));
            called += 1;
            false
        })
        .is_err()
    );
    assert_eq!(called, 1);
}
#[test]
fn regression_256_all_initial_states_match_legacy_and_saved_baseline() {
    check_baseline(16);
}
#[test]
fn regression_4096_all_initial_states_match_legacy_and_saved_baseline() {
    check_baseline(64);
}
fn check_baseline(side: usize) {
    let baseline = ray_regression::baseline(side);
    let mut reversed = baseline.conditions.clone();
    reversed.reverse();
    assert_eq!(
        prepare_rays(reversed, &baseline.config).unwrap(),
        baseline.prepared
    );
    assert_eq!(baseline.counts.iter().sum::<usize>(), side * side);
    for expected in baseline.expected.values() {
        assert_eq!(
            expected.status == RayStatus::Detected,
            expected.hit.is_some()
        );
    }
}
