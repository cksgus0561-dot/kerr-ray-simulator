//! Optional legacy experiment adapter, outside the source-independent simulator.
use super::source_detector::{LaunchDirection, SourcePlane};
use crate::simulation::rays::RayInitialCondition;

/// Express the existing CartesianSpatial, energy=1 source experiment as physical
/// inputs only. Legacy emission ordering/IDs are NOT forwarded to the simulator.
/// LocalZamo and energy!=1 remain supported by the unchanged legacy path.
pub fn from_source_plane(source: &SourcePlane) -> Result<Vec<RayInitialCondition>, String> {
    if source.local_energy != 1.0 {
        return Err("independent rays use fixed local_energy=1".into());
    }
    source
        .emissions()?
        .into_iter()
        .map(|e| {
            let [position_x, position_y, position_z] = source.plane.position(e.u, e.v);
            let LaunchDirection::CartesianSpatial([direction_x, direction_y, direction_z]) =
                e.direction.as_ref().unwrap_or(&source.direction)
            else {
                return Err("adapter requires CartesianSpatial directions".into());
            };
            Ok(RayInitialCondition {
                position_x,
                position_y,
                position_z,
                direction_x: *direction_x,
                direction_y: *direction_y,
                direction_z: *direction_z,
                t_emit: e.t_emit.unwrap_or(source.t_emit),
            })
        })
        .collect()
}
