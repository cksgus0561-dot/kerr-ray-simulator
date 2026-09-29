use super::{RecordedRun, Result, counts, grouped, key};
use crate::simulation::rays::{RayOutcome, calculate_rays, prepare_rays};
use std::{collections::VecDeque, time::Instant};
/// Fixed comparison policy; it does not change solver tolerances.
pub const HIT_ATOL: f64 = 1e-9;
pub const HIT_RTOL: f64 = 1e-12;
#[derive(Debug)]
pub struct Comparison {
    pub initial_conditions_hash_verified: bool,
    pub rays: usize,
    pub expected_rays: usize,
    pub canonical_mismatches: usize,
    pub id_mismatches: usize,
    pub stored_ids_checked: usize,
    pub stored_id_mismatches: usize,
    pub initial_state_bit_mismatches: usize,
    pub status_mismatches: usize,
    pub hit_presence_mismatches: usize,
    pub hit_tolerance_mismatches: usize,
    pub hit_bit_mismatches: [usize; 3],
    pub hit_max_abs: [f64; 3],
    pub hit_max_rel: [f64; 3],
    pub expected_counts: [usize; 5],
    pub actual_counts: [usize; 5],
    pub calculation_seconds: f64,
    pub same_code: bool,
}
impl Comparison {
    pub fn passed(&self) -> bool {
        self.rays == self.expected_rays
            && self.canonical_mismatches == 0
            && self.id_mismatches == 0
            && self.stored_id_mismatches == 0
            && self.initial_state_bit_mismatches == 0
            && self.status_mismatches == 0
            && self.hit_presence_mismatches == 0
            && self.hit_tolerance_mismatches == 0
            && self.actual_counts == self.expected_counts
    }
    pub fn summary(&self) -> String {
        let detail = format!(
            "Reproduction check\nrays: {} / {}\ncanonical mismatch: {}\nregenerated ray_id mismatch: {}\nstored ray_id checked: {} / {} (missing IDs in legacy archives are NOT VERIFIED)\nstored vs regenerated ray_id mismatch: {}\nreconstructed initial State vs solver start (bit mismatch): {}\nhistorical initial State: not stored; original-vs-loaded covered by round-trip tests\nstatus mismatch: {}\nhit presence mismatch: {}\ncounts [Active,Detected,Captured,Escaped,NumericalFailure]: {:?} / {:?}\nhit max abs error [u,v,t]: {:?}\nhit max rel error [u,v,t]: {:?}\nhit bit mismatches [u,v,t]: {:?}\nhit tolerance: abs <= {HIT_ATOL} + {HIT_RTOL} * max(abs(original),abs(new))\nhit tolerance mismatches: {}\nsame source/version: {}\nreproduction calculation seconds: {:.9}\nresult: {}",
            self.rays,
            self.expected_rays,
            self.canonical_mismatches,
            self.id_mismatches,
            self.stored_ids_checked,
            self.expected_rays,
            self.stored_id_mismatches,
            self.initial_state_bit_mismatches,
            self.status_mismatches,
            self.hit_presence_mismatches,
            self.actual_counts,
            self.expected_counts,
            self.hit_max_abs,
            self.hit_max_rel,
            self.hit_bit_mismatches,
            self.hit_tolerance_mismatches,
            self.same_code,
            self.calculation_seconds,
            if self.passed() { "PASS" } else { "FAIL" }
        );
        if self.initial_conditions_hash_verified {
            let detail=detail.replace(
                &format!("stored ray_id checked: 0 / {} (missing IDs in legacy archives are NOT VERIFIED)",self.expected_rays),
                "stored ray_id: not applicable (seeded format stores no IDs)");
            format!(
                "Initial conditions SHA-256: PASS (seed regenerated before integration; no stored IDs)\n{detail}"
            )
        } else {
            detail
        }
    }
}
/// Results are reference data only. Only loaded raw inputs/config enter the solver.
pub fn reproduce(saved: &RecordedRun) -> Result<Comparison> {
    reproduce_with_outcomes(saved, |_, _| true).map(|(report, _)| report)
}
/// Preserve the existing comparison and return the same fresh solve for display.
pub fn reproduce_with_outcomes(
    saved: &RecordedRun,
    keep_going: impl FnMut(usize, usize) -> bool,
) -> Result<(Comparison, Vec<RayOutcome>)> {
    saved.validate()?;
    let config = saved.common.config(saved.chi)?;
    let inputs: Vec<_> = if saved.seed_metadata.is_some() {
        super::seeded::verify(saved)?
    } else {
        saved.rows.iter().map(|r| r.input).collect()
    };
    let prepared = prepare_rays(inputs.clone(), &config)?;
    let mut groups = grouped(&saved.rows, &config)?;
    let timer = Instant::now();
    let outcomes = calculate_rays(inputs, &config, keep_going)?;
    let calculation_seconds = timer.elapsed().as_secs_f64();
    // Only after fresh integration: associate indistinguishable duplicate copies
    // with reference IDs in ascending order. These IDs never order solver inputs
    // or set PreparedRay IDs. Swapping IDs across different physical groups fails.
    for group in groups.values_mut() {
        group
            .make_contiguous()
            .sort_by_key(|&i| saved.rows[i].ray_id);
    }
    let mut report = Comparison {
        initial_conditions_hash_verified: saved.seed_metadata.is_some(),
        rays: outcomes.len(),
        expected_rays: saved.rows.len(),
        canonical_mismatches: 0,
        id_mismatches: 0,
        stored_ids_checked: 0,
        stored_id_mismatches: 0,
        initial_state_bit_mismatches: 0,
        status_mismatches: 0,
        hit_presence_mismatches: 0,
        hit_tolerance_mismatches: 0,
        hit_bit_mismatches: [0; 3],
        hit_max_abs: [0.0; 3],
        hit_max_rel: [0.0; 3],
        expected_counts: counts(&saved.rows),
        actual_counts: [0; 5],
        calculation_seconds,
        same_code: saved.common.simulator_version == env!("CARGO_PKG_VERSION")
            && saved.common.build_source_fnv1a64 == env!("KERR_SOURCE_FINGERPRINT"),
    };
    for (i, out) in outcomes.iter().enumerate() {
        report.id_mismatches += usize::from(out.ray.ray_id() != i || prepared[i].ray_id() != i);
        report.canonical_mismatches +=
            usize::from(key(out.ray.condition()) != key(prepared[i].condition()));
        let initial = prepared[i].initial().0.map(f64::to_bits);
        report.initial_state_bit_mismatches += usize::from(
            out.ray.initial().0.map(f64::to_bits) != initial
                || out.result.samples[0].state.0.map(f64::to_bits) != initial,
        );
        let row_index = groups
            .get_mut(&key(out.ray.condition()))
            .and_then(VecDeque::pop_front)
            .ok_or("cannot associate loaded physical input with computed ray")?;
        let expected = &saved.rows[row_index];
        if let Some(id) = expected.ray_id {
            report.stored_ids_checked += 1;
            report.stored_id_mismatches += usize::from(id != out.ray.ray_id() as u64);
        }
        report.actual_counts
            [crate::experiments::source_detector::status_index(out.result.status)] += 1;
        report.status_mismatches += usize::from(expected.status != out.result.status);
        match (expected.hit, out.detection()) {
            (Some(expected), Some(actual)) => {
                report.id_mismatches += usize::from(actual.ray_id != out.ray.ray_id());
                let actual = [actual.hit.uv[0], actual.hit.uv[1], actual.hit.state.time()];
                for j in 0..3 {
                    let abs = (actual[j] - expected[j]).abs();
                    let scale = actual[j].abs().max(expected[j].abs());
                    let rel = if scale == 0.0 { 0.0 } else { abs / scale };
                    report.hit_max_abs[j] = report.hit_max_abs[j].max(abs);
                    report.hit_max_rel[j] = report.hit_max_rel[j].max(rel);
                    report.hit_bit_mismatches[j] +=
                        usize::from(actual[j].to_bits() != expected[j].to_bits());
                    report.hit_tolerance_mismatches +=
                        usize::from(!abs.is_finite() || abs > HIT_ATOL + HIT_RTOL * scale);
                }
            }
            (None, None) => {}
            _ => report.hit_presence_mismatches += 1,
        }
    }
    Ok((report, outcomes))
}
