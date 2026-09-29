//! Plane crossings are located in affine parameter by re-integrating a bracket.
//! No frame numbers or endpoint-nearness heuristic. Never alter coordinates/momenta.
use super::plane::DetectorPlane;
use crate::{
    config::IntegratorConfig,
    physics::{
        coordinates::to_cartesian,
        geodesic::{State, rhs},
        integrator::{dopri54, error_norm},
        kerr::Kerr,
    },
};
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Intersection {
    pub affine: f64,
    pub state: State,
    pub uv: [f64; 2],
    pub signed_residual: f64,
    pub iterations: usize,
}

pub fn crossing(
    k: Kerr,
    start: State,
    end: State,
    step: f64,
    det: &DetectorPlane,
    cfg: &IntegratorConfig,
) -> Result<Option<Intersection>, String> {
    let plane = &det.plane;
    let d0 = plane.signed_distance(to_cartesian(k, start));
    let d1 = plane.signed_distance(to_cartesian(k, end));
    if !d0.is_finite() || !d1.is_finite() {
        return Err("non-finite detector distance".into());
    }
    // A tangent or a segment lying in the plane is not a transverse crossing.
    if d0 == 0.0 && d1 == 0.0 {
        return Ok(None);
    }
    if d0 != 0.0 && d1 != 0.0 && d0.is_sign_positive() == d1.is_sign_positive() {
        return Ok(None);
    }
    let mut lo = 0.0;
    let mut hi = step;
    let mut state = end;
    let mut distance = d1;
    let mut iterations = 0;
    if d0 == 0.0 {
        hi = 0.0;
        state = start;
        distance = 0.0;
    } else {
        for _ in 0..64 {
            if distance.abs() <= det.root_tolerance {
                break;
            }
            let mid = 0.5 * (lo + hi);
            let t = dopri54(&start.0, mid, |y| rhs(k, y))?;
            let s = State(t.value);
            let d = plane.signed_distance(to_cartesian(k, s));
            iterations += 1;
            if d == 0.0 || d.is_sign_positive() != d0.is_sign_positive() {
                hi = mid;
                state = s;
                distance = d;
            } else {
                lo = mid;
            }
        }
    }
    if distance.abs() > det.root_tolerance {
        return Err("detector root did not converge within coordinate tolerance".into());
    }
    if hi > 0.0 {
        let trial = dopri54(&start.0, hi, |y| rhs(k, y))?;
        if error_norm(&start.0, &trial, cfg.rtol, cfg.atol) > 1.0 {
            return Err("detector root state exceeds ODE tolerance".into());
        }
    }
    let uv = plane.uv(to_cartesian(k, state));
    Ok(plane.contains(uv).then_some(Intersection {
        affine: hi,
        state,
        uv,
        signed_residual: distance,
        iterations,
    }))
}
