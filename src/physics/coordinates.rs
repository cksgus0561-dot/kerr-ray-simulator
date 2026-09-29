//! Cartesian-like BL oblate coordinates (NOT Kerr-Schild and NOT a flat metric).
//! X=sqrt(r²+a²)sin(theta)cos(phi), Y=..., Z=r cos(theta), T=t_BL.
//! Planes fixed in these coordinates are explicitly prescribed detector worldtubes.
use super::{
    geodesic::{State, polar_sin_cos},
    kerr::Kerr,
    metric::Metric,
};
pub type Vec3 = [f64; 3];
pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn scale(a: Vec3, s: f64) -> Vec3 {
    a.map(|v| v * s)
}
pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub fn unit(a: Vec3) -> Result<Vec3, String> {
    let n = dot(a, a).sqrt();
    if !n.is_finite() || n <= 0.0 {
        return Err("finite nonzero vector required".into());
    }
    Ok(scale(a, 1.0 / n))
}
pub fn to_cartesian(k: Kerr, s: State) -> Vec3 {
    bl_to_cartesian(k, s.radius(), s.theta(), s.phi())
}
pub fn bl_to_cartesian(k: Kerr, r: f64, theta: f64, phi: f64) -> Vec3 {
    let (sn, cs) = polar_sin_cos(theta);
    let rho = (r * r + k.spin().powi(2)).sqrt();
    [rho * sn * phi.cos(), rho * sn * phi.sin(), r * cs]
}
/// Inverse of the oblate map, choosing r>0. Axis/horizon singularities are rejected.
pub fn from_cartesian(k: Kerr, x: Vec3) -> Result<[f64; 3], String> {
    if x.iter().any(|v| !v.is_finite()) {
        return Err("non-finite Cartesian coordinate".into());
    }
    let a = k.spin();
    let b = dot(x, x) - a * a;
    let r = (0.5 * (b + b.hypot(2.0 * a * x[2]))).sqrt();
    let th = (x[2] / r).clamp(-1.0, 1.0).acos();
    let phi = x[1].atan2(x[0]);
    k.covariant(r, th)?;
    Ok([r, th, phi])
}
/// Columns dX/dr,dX/dtheta,dX/dphi. Orthogonal in the auxiliary Euclidean map.
pub fn jacobian(k: Kerr, r: f64, theta: f64, phi: f64) -> [Vec3; 3] {
    let (sn, cs) = polar_sin_cos(theta);
    let rho = (r * r + k.spin().powi(2)).sqrt();
    [
        [r / rho * sn * phi.cos(), r / rho * sn * phi.sin(), cs],
        [rho * cs * phi.cos(), rho * cs * phi.sin(), -r * sn],
        [-rho * sn * phi.sin(), rho * sn * phi.cos(), 0.0],
    ]
}
/// A direction tangent to a t_BL slice, expressed in auxiliary Cartesian components,
/// converted to a physical unit spatial direction measured by the ZAMO. Its e_(0)
/// subsequently adds the shift omega to the coordinate photon velocity.
pub fn cartesian_to_local(k: Kerr, q: [f64; 3], direction: Vec3) -> Result<Vec3, String> {
    let [r, th, phi] = q;
    let j = jacobian(k, r, th, phi);
    let g = k.covariant(r, th)?;
    unit(std::array::from_fn(|i| {
        dot(j[i], direction) / dot(j[i], j[i]) * g[i + 1][i + 1].sqrt()
    }))
}
