//! Independent analytic comparison quantities; never used to bend/evolve rays.
use super::geodesic::State;
use super::kerr::Kerr;

/// Formula helper permits the limiting a=1 solely for analytic limit tests.
/// This helper takes a nonnegative spin magnitude; it does not evolve rays.
/// Kerr::new separately excludes exact extremality for actual integration.
pub fn photon_radius(a: f64, prograde: bool) -> Result<f64, String> {
    if !a.is_finite() || !(0.0..=1.0).contains(&a) {
        return Err("formula needs 0<=a<=1".into());
    }
    let argument = if prograde { -a } else { a };
    Ok(2.0 * (1.0 + ((2.0 / 3.0) * argument.acos()).cos()))
}
pub fn critical_schwarzschild_impact() -> f64 {
    3.0 * 3.0_f64.sqrt()
}
/// Equatorial null radial potential: Sigma^2 (dr/dlambda)^2 = R.
pub fn radial_potential(k: Kerr, s: State) -> f64 {
    let r = s.radius();
    let a = k.spin();
    let e = s.energy();
    let l = s.angular_momentum();
    (e * (r * r + a * a) - a * l).powi(2) - k.delta(r) * ((l - a * e).powi(2) + s.carter_q(k))
}

#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    pub max_null_abs: f64,
    pub max_null_energy_scaled: f64,
    pub max_energy_relative: f64,
    pub max_lz_relative: f64,
    pub max_energy_abs: f64,
    pub max_lz_abs: f64,
    pub max_q_abs: f64,
    pub max_q_relative: f64,
    pub accepted_steps: usize,
    pub rejected_steps: usize,
    pub event_iterations: usize,
}
impl Diagnostics {
    pub fn observe(
        &mut self,
        k: Kerr,
        initial: State,
        s: State,
        energy_scale: f64,
    ) -> Result<(), String> {
        let c = s.null_constraint(k)?.abs();
        if !c.is_finite() {
            return Err("non-finite null constraint".into());
        }
        if !s.carter_q(k).is_finite() || !initial.carter_q(k).is_finite() {
            return Err("non-finite Carter constant".into());
        }
        self.max_null_abs = self.max_null_abs.max(c);
        self.max_null_energy_scaled = self.max_null_energy_scaled.max(c / energy_scale.powi(2));
        let de = (s.energy() - initial.energy()).abs();
        let dl = (s.angular_momentum() - initial.angular_momentum()).abs();
        self.max_energy_abs = self.max_energy_abs.max(de);
        self.max_lz_abs = self.max_lz_abs.max(dl);
        // Zero conserved quantity has no relative error: use absolute error in energy units.
        let relative_or_scaled = |change: f64, initial: f64| {
            change
                / if initial.abs() > energy_scale * 1e-14 {
                    initial.abs()
                } else {
                    energy_scale
                }
        };
        self.max_energy_relative = self
            .max_energy_relative
            .max(relative_or_scaled(de, initial.energy()));
        self.max_lz_relative = self
            .max_lz_relative
            .max(relative_or_scaled(dl, initial.angular_momentum()));
        self.max_q_abs = self
            .max_q_abs
            .max((s.carter_q(k) - initial.carter_q(k)).abs());
        self.max_q_relative = self.max_q_relative.max(
            (s.carter_q(k) - initial.carter_q(k)).abs()
                / initial.carter_q(k).abs().max(energy_scale.powi(2)),
        );
        Ok(())
    }
}
