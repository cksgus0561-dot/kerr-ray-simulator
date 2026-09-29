//! 학생이 실험 수치 설정을 바꾸는 곳. 모든 길이는 M 단위, 시간은 M/c 단위.
use crate::physics::kerr::Kerr;

pub const DEFAULT_SPIN: f64 = 0.6;
pub const DEFAULT_RAY_COUNT: usize = 41;
pub const PREVIEW_RAY_COUNT: usize = 9; // Reserved for the future asynchronous UI.

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct IntegratorConfig {
    /// Numerical exterior cutoff, NOT a modification of the event horizon.
    pub horizon_epsilon: f64,
    /// Escape requires r >= this radius AND dr/dlambda > 0.
    pub escape_radius: f64,
    /// Per-component local error: atol + rtol*max(|old|,|new|).
    pub rtol: f64,
    pub atol: f64,
    /// Affine parameter step limits; energy normalization changes affine scale.
    pub initial_step: f64,
    pub min_step: f64,
    pub max_step: f64,
    /// Includes rejected attempts. Exhaustion is NumericalFailure, never Escaped.
    pub max_steps: usize,
    /// Intentional finite experiment duration: returns Active + AffineLimit.
    pub max_affine: f64,
    /// Limit |g^mu nu p_mu p_nu| / (initial ZAMO photon energy)^2.
    /// This makes the null check invariant under a constant affine rescaling.
    pub null_tolerance: f64,
}

impl Default for IntegratorConfig {
    fn default() -> Self {
        Self {
            horizon_epsilon: 1e-3,
            escape_radius: 100.0,
            rtol: 1e-12,
            atol: 1e-14,
            initial_step: 0.05,
            min_step: 1e-12,
            max_step: 0.5,
            max_steps: 200_000,
            max_affine: 1000.0,
            null_tolerance: 1e-5,
        }
    }
}

impl IntegratorConfig {
    pub fn validate(self, k: Kerr) -> Result<(), String> {
        let positive = [
            self.horizon_epsilon,
            self.escape_radius,
            self.rtol,
            self.atol,
            self.initial_step,
            self.min_step,
            self.max_step,
            self.max_affine,
            self.null_tolerance,
        ];
        if positive.iter().any(|v| !v.is_finite() || *v <= 0.0)
            || self.min_step > self.initial_step
            || self.initial_step > self.max_step
            || self.escape_radius <= k.horizon() + self.horizon_epsilon
            || k.horizon() + self.horizon_epsilon <= k.horizon()
            || self.max_steps == 0
        {
            return Err("invalid integration settings: positive finite values, ordered steps, exterior escape radius required".into());
        }
        Ok(())
    }
}
