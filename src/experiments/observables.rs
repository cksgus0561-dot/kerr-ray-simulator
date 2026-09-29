//! 새 관측량은 PhysicsResult를 읽는 함수를 추가하면 됩니다. 물리 상태를 바꾸지 않습니다.
use crate::{
    compute::{PhysicsResult, RayStatus},
    physics::{
        geodesic::{State, rhs},
        kerr::Kerr,
    },
};
pub fn captured(result: &PhysicsResult) -> bool {
    result.status == RayStatus::Captured
}
pub fn escaped(result: &PhysicsResult) -> bool {
    result.status == RayStatus::Escaped
}
/// Accepted samples' minimum; finite sampling can overestimate the exact periapsis.
pub fn minimum_radius(result: &PhysicsResult) -> f64 {
    result
        .samples
        .iter()
        .map(|s| s.state.radius())
        .fold(f64::INFINITY, f64::min)
}
pub fn accumulated_azimuth(result: &PhysicsResult) -> f64 {
    result.samples.last().unwrap().state.phi() - result.samples[0].state.phi()
}
/// Total sampled |dphi|, so reversals count positively. Coordinate-dependent.
pub fn azimuth_travel(result: &PhysicsResult) -> f64 {
    result
        .samples
        .windows(2)
        .map(|s| (s[1].state.phi() - s[0].state.phi()).abs())
        .sum()
}
pub fn revolutions(result: &PhysicsResult) -> f64 {
    azimuth_travel(result) / std::f64::consts::TAU
}
pub fn affine_length(result: &PhysicsResult) -> f64 {
    result.samples.last().unwrap().affine
}
/// Direction in x=r cos(phi), y=r sin(phi). Finite-radius coordinate diagnostic.
/// This is not a local angle at infinity or an embedding.
pub fn coordinate_heading(k: Kerr, s: State) -> Option<f64> {
    let v = rhs(k, &s.0).ok()?;
    Some(s.phi() + (s.radius() * v[2]).atan2(v[1]))
}
/// Principal signed change (-pi,pi], only for Escaped rays.
/// Keep accumulated_azimuth alongside it to retain multiple winding information.
pub fn final_scattering_angle(k: Kerr, result: &PhysicsResult) -> Option<f64> {
    if !escaped(result) {
        return None;
    }
    let delta = coordinate_heading(k, result.samples.last()?.state)?
        - coordinate_heading(k, result.samples.first()?.state)?;
    Some(delta.sin().atan2(delta.cos()))
}
// Example of a one-function extension:
pub fn coordinate_time_elapsed(result: &PhysicsResult) -> f64 {
    result.samples.last().unwrap().state.0[0] - result.samples[0].state.0[0]
}
