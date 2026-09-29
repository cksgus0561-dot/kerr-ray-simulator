//! Edit these seven physical scalars per ray; no SourcePlane or input file.
use kerr_ray::{
    config::IntegratorConfig,
    detector::plane::{DetectorPlane, Plane},
    simulation::rays::{RayInitialCondition, RaySimulationConfig, calculate_rays},
};
fn main() -> Result<(), String> {
    let conditions = vec![
        RayInitialCondition {
            position_x: -50.0,
            position_y: 0.0,
            position_z: 0.0,
            direction_x: 1.0,
            direction_y: 0.0,
            direction_z: 0.0,
            t_emit: 0.0,
        },
        RayInitialCondition {
            position_x: -48.3,
            position_y: 2.7,
            position_z: -1.2,
            direction_x: 0.998,
            direction_y: 0.04,
            direction_z: 0.03,
            t_emit: 1.0,
        },
        RayInitialCondition {
            position_x: -53.1,
            position_y: -0.4,
            position_z: 3.8,
            direction_x: 0.994,
            direction_y: -0.08,
            direction_z: 0.02,
            t_emit: 2.0,
        },
    ];
    let config = RaySimulationConfig {
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
            )?,
            resolution: [128, 128],
            root_tolerance: 1e-10,
        },
    };
    for ray in calculate_rays(conditions, &config, |_, _| true)? {
        println!(
            "ray_id={} initial={:?} status={:?} samples={}",
            ray.ray.ray_id(),
            ray.ray.initial(),
            ray.result.status,
            ray.result.samples.len()
        );
        if let Some(event) = ray.detection() {
            println!(
                "  detector ray_id={} uv={:?} t_hit={} delta_t={}",
                event.ray_id,
                event.hit.uv,
                event.hit.state.time(),
                event.delta_t
            );
        }
    }
    Ok(())
}
