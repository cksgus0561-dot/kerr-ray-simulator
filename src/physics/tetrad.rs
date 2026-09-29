//! ZAMO local orthonormal frame, future-directed outside r_plus, including ergoregion.
use super::{
    constants::EQUATOR,
    geodesic::State,
    kerr::Kerr,
    metric::{Matrix4, Metric, multiply_vector},
};

/// Columns are e_(hat a)^mu: e_0=(1/alpha,0,0,omega/alpha).
/// alpha=sqrt(Sigma Delta/A), omega=2ar/A, rho_phi=sqrt(g_phi phi).
pub fn zamo_frame(k: Kerr, r: f64, theta: f64) -> Result<Matrix4, String> {
    let g = k.covariant(r, theta)?;
    let alpha = (k.sigma(r, theta) * k.delta(r) / k.area_function(r, theta)).sqrt();
    let omega = -g[0][3] / g[3][3];
    Ok([
        [1.0 / alpha, 0.0, 0.0, 0.0],
        [0.0, 1.0 / g[1][1].sqrt(), 0.0, 0.0],
        [0.0, 0.0, 1.0 / g[2][2].sqrt(), 0.0],
        [omega / alpha, 0.0, 0.0, 1.0 / g[3][3].sqrt()],
    ])
}

/// angle measured locally from OUTWARD radial direction toward +phi.
/// angle=pi is inward radial; local_energy fixes the affine normalization.
pub fn equatorial_ray(
    k: Kerr,
    r: f64,
    phi: f64,
    angle: f64,
    local_energy: f64,
) -> Result<State, String> {
    ray_3d(
        k,
        [0.0, r, EQUATOR, phi],
        [angle.cos(), 0.0, angle.sin()],
        local_energy,
    )
}

/// Unit vector components in the ZAMO (e_r,e_theta,e_phi) space, NOT p_r,p_theta,p_phi.
pub fn ray_3d(k: Kerr, x: [f64; 4], direction: [f64; 3], energy: f64) -> Result<State, String> {
    let norm = direction.iter().map(|v| v * v).sum::<f64>();
    if x.iter().chain(&direction).any(|v| !v.is_finite())
        || !energy.is_finite()
        || energy <= 0.0
        || (norm - 1.0).abs() > 1e-12
    {
        return Err("finite coordinates, unit ZAMO direction and positive energy required".into());
    }
    let local = [
        energy,
        energy * direction[0],
        energy * direction[1],
        energy * direction[2],
    ];
    let contra = multiply_vector(&zamo_frame(k, x[1], x[2])?, &local);
    let cov = multiply_vector(&k.covariant(x[1], x[2])?, &contra);
    let state = State::from_bl(x, cov);
    let scaled = state.null_constraint(k)?.abs() / energy.powi(2);
    if !scaled.is_finite() || scaled > 1e-10 {
        return Err("ZAMO conversion violates null tolerance 1e-10".into());
    }
    Ok(state)
}

pub fn local_energy(k: Kerr, s: State) -> Result<f64, String> {
    let frame = zamo_frame(k, s.radius(), s.theta())?;
    Ok(-(s.0[3] * frame[0][0] + s.0[5] * frame[3][0]))
}

/// Exact b=Lz/E parameterization through a ZAMO direction, for scattering scans.
/// At finite radius a screen offset is not equal to b. Requires positive E.
pub fn ray_with_impact(k: Kerr, r: f64, phi: f64, b: f64, inward: bool) -> Result<State, String> {
    if !b.is_finite() {
        return Err("impact parameter must be finite".into());
    }
    let g = k.covariant(r, EQUATOR)?;
    let alpha = (k.sigma(r, EQUATOR) * k.delta(r) / k.area_function(r, EQUATOR)).sqrt();
    let omega = -g[0][3] / g[3][3];
    let denominator = g[3][3].sqrt() * (1.0 - b * omega);
    let n_phi = b * alpha / denominator;
    if denominator <= 0.0 || !n_phi.is_finite() || n_phi.abs() > 1.0 {
        return Err(
            "impact parameter inaccessible to positive-energy local null ray at this radius".into(),
        );
    }
    let n_r = (1.0 - n_phi * n_phi).sqrt() * if inward { -1.0 } else { 1.0 };
    equatorial_ray(k, r, phi, n_phi.atan2(n_r), 1.0)
}
