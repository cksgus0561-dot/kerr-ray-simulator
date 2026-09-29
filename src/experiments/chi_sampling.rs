//! Sampling coordinates for the dataset, independent of the Kerr solver.
use serde::{Deserialize, Serialize};

pub const CHI_LIMIT: f64 = 0.999;
pub const POSITIVE_INTERVAL_COUNT: i32 = 76;
pub const TOTAL_CHI_COUNT: usize = 153;

/// Persist the formula, not a hard-coded table of spins.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChiSampling {
    pub chi_min: f64,
    pub chi_max: f64,
    pub transform: String,
    pub positive_interval_count: i32,
    pub total_chi_count: usize,
}
impl Default for ChiSampling {
    fn default() -> Self {
        Self {
            chi_min: -CHI_LIMIT,
            chi_max: CHI_LIMIT,
            transform: "s=atanh(chi); s_i=i*atanh(0.999)/76; chi_i=tanh(s_i)".into(),
            positive_interval_count: POSITIVE_INTERVAL_COUNT,
            total_chi_count: TOTAL_CHI_COUNT,
        }
    }
}
impl ChiSampling {
    pub fn validate(&self) -> Result<(), String> {
        if self != &Self::default() {
            return Err("unsupported chi sampling specification".into());
        }
        Ok(())
    }
}

/// Evaluate each positive magnitude once and obtain the negative side by sign
/// reversal. The analytical endpoints and center are assigned exactly in f64;
/// this avoids unnecessary tanh(atanh(x)) rounding at the two boundaries.
pub fn chi_at(index: i32) -> Result<f64, String> {
    if !(-POSITIVE_INTERVAL_COUNT..=POSITIVE_INTERVAL_COUNT).contains(&index) {
        return Err("chi index must be in -76..=76".into());
    }
    let magnitude = match index.abs() {
        0 => return Ok(0.0),
        POSITIVE_INTERVAL_COUNT => CHI_LIMIT,
        i => (f64::from(i) * CHI_LIMIT.atanh() / f64::from(POSITIVE_INTERVAL_COUNT)).tanh(),
    };
    Ok(if index < 0 { -magnitude } else { magnitude })
}

pub fn chi_grid() -> Vec<f64> {
    (-POSITIVE_INTERVAL_COUNT..=POSITIVE_INTERVAL_COUNT)
        .map(|i| chi_at(i).expect("fixed valid index"))
        .collect()
}
