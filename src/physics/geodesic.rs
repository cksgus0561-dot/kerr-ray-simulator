//! Full 3+1 canonical Hamiltonian flow, signature (-,+,+,+). No force/projection.
use super::{
    constants::EQUATOR,
    kerr::Kerr,
    metric::{Matrix4, Metric, Vector4, contract, multiply_vector},
};

/// Eight canonical variables. The first six legacy slots are preserved:
/// (t,r,phi,p_t,p_r,p_phi,theta,p_theta). Use from_bl()/coordinates()/momentum()
/// for the conventional (t,r,theta,phi) ordering at API boundaries.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct State(pub [f64; 8]);
impl State {
    pub fn from_bl(x: Vector4, p: Vector4) -> Self {
        Self([x[0], x[1], x[3], p[0], p[1], p[3], x[2], p[2]])
    }
    pub fn coordinates(self) -> Vector4 {
        [self.0[0], self.0[1], self.0[6], self.0[2]]
    }
    pub fn radius(self) -> f64 {
        self.0[1]
    }
    pub fn theta(self) -> f64 {
        self.0[6]
    }
    pub fn phi(self) -> f64 {
        self.0[2]
    }
    pub fn time(self) -> f64 {
        self.0[0]
    }
    pub fn energy(self) -> f64 {
        -self.0[3]
    }
    pub fn angular_momentum(self) -> f64 {
        self.0[5]
    }
    pub fn momentum(self) -> Vector4 {
        [self.0[3], self.0[4], self.0[7], self.0[5]]
    }
    pub fn is_finite(self) -> bool {
        self.0.iter().all(|x| x.is_finite())
    }
    pub fn null_constraint(self, k: Kerr) -> Result<f64, String> {
        Ok(contract(
            &k.inverse(self.radius(), self.theta())?,
            &self.momentum(),
        ))
    }
    pub fn hamiltonian(self, k: Kerr) -> Result<f64, String> {
        Ok(0.5 * self.null_constraint(k)?)
    }
    /// For a null particle: Q=p_theta^2+cos²theta (Lz²/sin²theta-a²E²).
    /// Evaluate, never project onto, this hidden-symmetry invariant.
    pub fn carter_q(self, k: Kerr) -> f64 {
        let (s, c) = polar_sin_cos(self.theta());
        self.0[7].powi(2)
            + c * c
                * (self.angular_momentum().powi(2) / (s * s)
                    - k.spin().powi(2) * self.energy().powi(2))
    }
}
/// pi/2 is an exact symmetry location; avoid cos(pi/2)'s roundoff residue.
/// This evaluates a known trigonometric value, not an equatorial force/constraint.
pub fn polar_sin_cos(theta: f64) -> (f64, f64) {
    if theta == EQUATOR {
        (1.0, 0.0)
    } else {
        theta.sin_cos()
    }
}

/// Analytic derivatives [partial_r g^mu nu, partial_theta g^mu nu].
/// Only products/quotients are differentiated. No finite differences in evolution.
pub fn inverse_derivatives(k: Kerr, r: f64, theta: f64) -> Result<[Matrix4; 2], String> {
    k.inverse(r, theta)?;
    let a = k.spin();
    let (sn, cs) = polar_sin_cos(theta);
    let s2 = sn * sn;
    let sigma = r * r + a * a * cs * cs;
    let delta = k.delta(r);
    let area = (r * r + a * a).powi(2) - a * a * delta * s2;
    let ds = [2.0 * r, -2.0 * a * a * sn * cs];
    let dd = [2.0 * r - 2.0, 0.0];
    let da = [
        4.0 * r * (r * r + a * a) - a * a * dd[0] * s2,
        -2.0 * a * a * delta * sn * cs,
    ];
    let ds2 = [0.0, 2.0 * sn * cs];
    let denominator = sigma * delta;
    let mut result = [[[0.0; 4]; 4]; 2];
    for mu in 0..2 {
        let dp = ds[mu] * delta + sigma * dd[mu];
        let quotient = |n: f64, np: f64| (np * denominator - n * dp) / denominator.powi(2);
        let g = &mut result[mu];
        g[0][0] = quotient(-area, -da[mu]);
        g[0][3] = quotient(-2.0 * a * r, if mu == 0 { -2.0 * a } else { 0.0 });
        g[3][0] = g[0][3];
        g[1][1] = (dd[mu] * sigma - delta * ds[mu]) / sigma.powi(2);
        g[2][2] = -ds[mu] / sigma.powi(2);
        let n = delta - a * a * s2;
        let np = dd[mu] - a * a * ds2[mu];
        g[3][3] = (np * denominator * s2 - n * (dp * s2 + denominator * ds2[mu]))
            / (denominator * s2).powi(2);
    }
    Ok(result)
}
/// Kept for source compatibility with the original derivative tests.
pub fn inverse_radial_derivative(k: Kerr, r: f64) -> Result<Matrix4, String> {
    Ok(inverse_derivatives(k, r, EQUATOR)?[0])
}
pub fn rhs(k: Kerr, y: &[f64; 8]) -> Result<[f64; 8], String> {
    let s = State(*y);
    if !s.is_finite() {
        return Err("non-finite canonical state".into());
    }
    let v = multiply_vector(&k.inverse(s.radius(), s.theta())?, &s.momentum());
    let dg = inverse_derivatives(k, s.radius(), s.theta())?;
    Ok([
        v[0],
        v[1],
        v[3],
        0.0,
        -0.5 * contract(&dg[0], &s.momentum()),
        0.0,
        v[2],
        -0.5 * contract(&dg[1], &s.momentum()),
    ])
}
