//! Quantities derived from the existing Kerr metric, shared with visualization.
use super::{kerr::Kerr, metric::Metric};
/// ZAMO angular velocity dphi/dt in Boyer-Lindquist coordinates (1/M).
/// It is not a local linear velocity and must not be interpreted as a force.
pub fn frame_dragging(k: Kerr, r: f64, theta: f64) -> Result<f64, String> {
    let g = k.covariant(r, theta)?;
    let omega = -g[0][3] / g[3][3];
    if omega.is_finite() {
        Ok(omega)
    } else {
        Err("non-finite ZAMO angular velocity".into())
    }
}
