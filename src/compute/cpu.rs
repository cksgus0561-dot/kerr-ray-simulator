//! Sequential f64 reference engine. Parallel scheduling belongs above this layer.
use super::{PhysicsResult, RayIntegrator, RayStatus, Sample, StopReason};
use crate::{
    config::IntegratorConfig,
    physics::{
        geodesic::{State, rhs},
        integrator::{dopri54, error_norm},
        kerr::Kerr,
        tetrad,
        validation::Diagnostics,
    },
};
use std::time::Instant;

pub struct CpuReferenceIntegrator;
impl RayIntegrator for CpuReferenceIntegrator {
    fn integrate(&self, k: Kerr, initial: State, cfg: &IntegratorConfig) -> PhysicsResult {
        self.integrate_optional_detector(k, initial, cfg, None)
    }
}
impl CpuReferenceIntegrator {
    pub fn integrate_with_detector(
        &self,
        k: Kerr,
        initial: State,
        cfg: &IntegratorConfig,
        det: &crate::detector::plane::DetectorPlane,
    ) -> PhysicsResult {
        self.integrate_optional_detector(k, initial, cfg, Some(det))
    }
    fn integrate_optional_detector(
        &self,
        k: Kerr,
        initial: State,
        cfg: &IntegratorConfig,
        det: Option<&crate::detector::plane::DetectorPlane>,
    ) -> PhysicsResult {
        let start = Instant::now();
        let mut out = PhysicsResult {
            samples: vec![Sample {
                affine: 0.0,
                state: initial,
            }],
            status: RayStatus::Active,
            stop_reason: StopReason::AffineLimit,
            failure_message: None,
            diagnostics: Diagnostics::default(),
            elapsed_seconds: 0.0,
            detection: None,
        };
        if let Some(d) = det
            && let Err(e) = d.validate(k)
        {
            fail(&mut out, StopReason::InvalidInitialState, e);
            return out;
        }
        run(k, initial, *cfg, det, &mut out);
        out.elapsed_seconds = start.elapsed().as_secs_f64();
        out
    }
}

fn fail(out: &mut PhysicsResult, reason: StopReason, message: impl Into<String>) {
    out.status = RayStatus::NumericalFailure;
    out.stop_reason = reason;
    out.failure_message = Some(message.into());
}

fn run(
    k: Kerr,
    initial: State,
    cfg: IntegratorConfig,
    det: Option<&crate::detector::plane::DetectorPlane>,
    out: &mut PhysicsResult,
) {
    if let Err(e) = cfg.validate(k) {
        fail(out, StopReason::InvalidInitialState, e);
        return;
    }
    if !initial.is_finite() || initial.radius() <= k.horizon() {
        fail(
            out,
            StopReason::InvalidInitialState,
            "initial state must be finite and outside BL horizon",
        );
        return;
    }
    let energy = match tetrad::local_energy(k, initial) {
        Ok(v) if v.is_finite() && v > 0.0 && v.powi(2).is_finite() && v.powi(2) > 0.0 => v,
        _ => {
            fail(
                out,
                StopReason::InvalidInitialState,
                "ray must be future-directed with positive ZAMO energy",
            );
            return;
        }
    };
    if let Err(e) = out.diagnostics.observe(k, initial, initial, energy) {
        fail(out, StopReason::NonFinite, e);
        return;
    }
    if out.diagnostics.max_null_energy_scaled > cfg.null_tolerance {
        fail(
            out,
            StopReason::NullViolation,
            "initial state violates null constraint",
        );
        return;
    }
    let capture = k.horizon() + cfg.horizon_epsilon;
    let mut s = initial;
    let mut affine = 0.0;
    let mut h = cfg.initial_step;
    for _ in 0..cfg.max_steps {
        let v = match rhs(k, &s.0) {
            Ok(v) => v,
            Err(e) => {
                fail(out, StopReason::IntegratorFailure, e);
                return;
            }
        };
        if s.radius() <= capture {
            out.status = RayStatus::Captured;
            out.stop_reason = StopReason::HorizonCutoff;
            return;
        }
        if s.radius() >= cfg.escape_radius && v[1] > 0.0 {
            out.status = RayStatus::Escaped;
            out.stop_reason = StopReason::EscapeRadius;
            return;
        }
        if affine >= cfg.max_affine {
            return;
        }
        h = h.min(cfg.max_step).min(cfg.max_affine - affine);
        // Limit inward steps using distance to the true coordinate singularity.
        // Rejected stages never become part of the reported trajectory.
        if v[1] < 0.0 {
            h = h.min(0.2 * (s.radius() - k.horizon()) / (-v[1]));
        }
        // BL polar axis is also a coordinate singularity; never silently jump charts.
        if v[6] != 0.0 {
            let distance = if v[6] > 0.0 {
                std::f64::consts::PI - s.theta()
            } else {
                s.theta()
            };
            h = h.min(0.2 * distance / v[6].abs());
        }
        if h < cfg.min_step || affine + h == affine {
            fail(
                out,
                StopReason::StepUnderflow,
                "required step below minimum or affine precision",
            );
            return;
        }
        let mut trial = match dopri54(&s.0, h, |y| rhs(k, y)) {
            Ok(t) => t,
            Err(_) => {
                out.diagnostics.rejected_steps += 1;
                h *= 0.25;
                continue;
            }
        };
        let mut norm = error_norm(&s.0, &trial, cfg.rtol, cfg.atol);
        if !norm.is_finite() {
            fail(out, StopReason::NonFinite, "non-finite error estimate");
            return;
        }
        if norm > 1.0 {
            out.diagnostics.rejected_steps += 1;
            h *= (0.9 * norm.powf(-0.2)).clamp(0.1, 0.5);
            continue;
        }
        let end = State(trial.value);
        let event = if end.radius() <= capture {
            Some((capture, true))
        } else if s.radius() < cfg.escape_radius && end.radius() >= cfg.escape_radius {
            Some((cfg.escape_radius, false))
        } else {
            None
        };
        if let Some((target, inward)) = event {
            // Locate the FIRST bracketed crossing by re-integrating from this step's start.
            // Keep the crossed side of the bracket; never overwrite r or interpolate p.
            let mut lo = 0.0;
            let mut hi = h;
            for _ in 0..48 {
                let mid = 0.5 * (lo + hi);
                let candidate = match dopri54(&s.0, mid, |y| rhs(k, y)) {
                    Ok(t) => t,
                    Err(e) => {
                        fail(out, StopReason::IntegratorFailure, e);
                        return;
                    }
                };
                out.diagnostics.event_iterations += 1;
                let crossed = if inward {
                    candidate.value[1] <= target
                } else {
                    candidate.value[1] >= target
                };
                if crossed {
                    hi = mid;
                    trial = candidate;
                } else {
                    lo = mid;
                }
                if (trial.value[1] - target).abs() < 1e-12 * target.max(1.0) {
                    break;
                }
            }
            h = hi;
            norm = error_norm(&s.0, &trial, cfg.rtol, cfg.atol);
            if !norm.is_finite() || norm > 1.0 {
                out.diagnostics.rejected_steps += 1;
                h *= 0.5;
                continue;
            }
        }
        let mut detected = None;
        if let Some(d) = det {
            match crate::detector::intersection::crossing(k, s, State(trial.value), h, d, &cfg) {
                Ok(Some(mut hit)) => {
                    h = hit.affine;
                    trial.value = hit.state.0;
                    hit.affine += affine;
                    out.diagnostics.event_iterations += hit.iterations;
                    detected = Some(hit);
                }
                Ok(None) => {}
                Err(e) => {
                    fail(out, StopReason::IntegratorFailure, e);
                    return;
                }
            }
        }
        s = State(trial.value);
        affine += h;
        if h > 0.0 {
            out.diagnostics.accepted_steps += 1;
            out.samples.push(Sample { affine, state: s });
        }
        if let Err(e) = out.diagnostics.observe(k, initial, s, energy) {
            fail(out, StopReason::NonFinite, e);
            return;
        }
        if out.diagnostics.max_null_energy_scaled > cfg.null_tolerance {
            fail(
                out,
                StopReason::NullViolation,
                "accepted state exceeds energy-scaled null tolerance",
            );
            return;
        }
        if let Some(hit) = detected {
            out.status = RayStatus::Detected;
            out.stop_reason = StopReason::DetectorHit;
            out.detection = Some(hit);
            return;
        }
        // Classify immediately so an event on the final allowed step is not lost.
        if s.radius() <= capture {
            out.status = RayStatus::Captured;
            out.stop_reason = StopReason::HorizonCutoff;
            return;
        }
        if s.radius() >= cfg.escape_radius && rhs(k, &s.0).is_ok_and(|v| v[1] > 0.0) {
            out.status = RayStatus::Escaped;
            out.stop_reason = StopReason::EscapeRadius;
            return;
        }
        if affine >= cfg.max_affine {
            return;
        }
        h *= if norm == 0.0 {
            5.0
        } else {
            (0.9 * norm.powf(-0.2)).clamp(0.2, 5.0)
        };
    }
    fail(
        out,
        StopReason::StepLimit,
        "maximum attempted steps reached; escape has NOT been established",
    );
}
