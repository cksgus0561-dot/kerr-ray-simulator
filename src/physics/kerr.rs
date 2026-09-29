//! Exact Kerr metric, (-,+,+,+), G=c=M=1.
//! BL is singular at Delta=0 and on the polar axis. This API is exterior only.
use super::{
    constants::MAX_SPIN,
    metric::{Matrix4, Metric},
};

#[derive(Clone, Copy, Debug)]
pub struct Kerr {
    a: f64,
}

impl Kerr {
    pub fn new(chi: f64) -> Result<Self, String> {
        if !chi.is_finite() || !(-MAX_SPIN..=MAX_SPIN).contains(&chi) {
            return Err(format!(
                "chi must be finite and in [-{MAX_SPIN},{MAX_SPIN}]; extremal Kerr is excluded"
            ));
        }
        Ok(Self { a: chi })
    }
    pub fn spin(self) -> f64 {
        self.a
    }
    pub fn sigma(self, r: f64, theta: f64) -> f64 {
        r * r + self.a * self.a * theta.cos().powi(2)
    }
    pub fn delta(self, r: f64) -> f64 {
        // Delta = r(r-2) + a^2, G=c=M=1. Near the outer horizon,
        // r-2 is exact by Sterbenz's lemma. Fuse the cancelling product/sum,
        // then restore the rounding residual of a^2 using a second FMA.
        // Rounded horizon roots would instead inject root error into Delta.
        // This is the same polynomial for every signed a; no cutoff/threshold
        // or null-constraint correction is involved.
        let a2 = self.a * self.a;
        let a2_error = self.a.mul_add(self.a, -a2);
        r.mul_add(r - 2.0, a2) + a2_error
    }
    pub fn horizon(self) -> f64 {
        1.0 + (1.0 - self.a * self.a).sqrt()
    }
    pub fn ergosphere(self, theta: f64) -> f64 {
        1.0 + (1.0 - self.a * self.a * theta.cos().powi(2)).sqrt()
    }
    pub fn area_function(self, r: f64, theta: f64) -> f64 {
        (r * r + self.a * self.a).powi(2) - self.a * self.a * self.delta(r) * theta.sin().powi(2)
    }
    fn check(self, r: f64, theta: f64) -> Result<(), String> {
        if !r.is_finite()
            || r <= self.horizon()
            || !theta.is_finite()
            || theta <= 0.0
            || theta >= std::f64::consts::PI
        {
            return Err("BL metric requires r > r_plus and 0 < theta < pi".into());
        }
        Ok(())
    }
}

impl Metric for Kerr {
    fn covariant(&self, r: f64, theta: f64) -> Result<Matrix4, String> {
        self.check(r, theta)?;
        let s = self.sigma(r, theta);
        let d = self.delta(r);
        let sn2 = theta.sin().powi(2);
        let mut g = [[0.0; 4]; 4];
        g[0][0] = -1.0 + 2.0 * r / s;
        // ds^2 contains 2*g_tphi dt dphi: do not lose this factor of two.
        g[0][3] = -2.0 * self.a * r * sn2 / s;
        g[3][0] = g[0][3];
        g[1][1] = s / d;
        g[2][2] = s;
        g[3][3] = self.area_function(r, theta) * sn2 / s;
        Ok(g)
    }
    fn inverse(&self, r: f64, theta: f64) -> Result<Matrix4, String> {
        self.check(r, theta)?;
        let s = self.sigma(r, theta);
        let d = self.delta(r);
        let sn2 = theta.sin().powi(2);
        let mut g = [[0.0; 4]; 4];
        g[0][0] = -self.area_function(r, theta) / (s * d);
        g[0][3] = -2.0 * self.a * r / (s * d);
        g[3][0] = g[0][3];
        g[1][1] = d / s;
        g[2][2] = 1.0 / s;
        g[3][3] = (d - self.a * self.a * sn2) / (s * d * sn2);
        Ok(g)
    }
}
