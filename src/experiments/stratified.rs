//! Versioned initial-condition generation only; no SourcePlane or solver dependency.
//! The arithmetic order below is part of v1. Do not introduce FMA/fast-math or
//! extra random calls without changing the version and preserving this decoder.
use crate::simulation::rays::RayInitialCondition;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use serde::{Deserialize, Serialize};

pub const VERSION: &str = "stratified-cartesian-v1";
pub const DEFAULT_CELL_COUNT: [usize; 2] = [192, 192];
pub const RNG: &str =
    "ChaCha20 (rand_chacha 0.9.0), 256-bit key, stream=0, word_pos=0; next_u64 low-u32 first";
pub const UNIFORM: &str =
    "(next_u64() >> 11) as f64 * 2^-53; two calls per cell: axis_1 then axis_2";
pub const ORDER: &str =
    "axis_2 outer j=0..n2, axis_1 inner i=0..n1; results in prepare_rays canonical order";

/// Coordinate lengths in M, in the existing oblate Cartesian-like convention.
/// Unit orthogonal basis vectors are validated, never silently re-normalized.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GeneratorSpec {
    pub ray_generator_version: String,
    pub rng_algorithm: String,
    pub seed_bits: u32,
    pub uniform_f64_rule: String,
    pub cell_order: String,
    pub center: [f64; 3],
    pub axis_1: [f64; 3],
    pub axis_2: [f64; 3],
    pub cell_size: [f64; 2],
    pub cell_count: [usize; 2],
    pub fixed_direction: [f64; 3],
    pub t_emit: f64,
    pub position_sampling: String,
    pub direction_noise: String,
}
impl Default for GeneratorSpec {
    fn default() -> Self {
        Self {
            ray_generator_version: VERSION.into(),
            rng_algorithm: RNG.into(),
            seed_bits: 256,
            uniform_f64_rule: UNIFORM.into(),
            cell_order: ORDER.into(),
            center: [80.0, 0.0, 0.0],
            axis_1: [0.0, 1.0, 0.0],
            axis_2: [0.0, 0.0, 1.0],
            cell_size: [0.5, 0.5],
            cell_count: DEFAULT_CELL_COUNT,
            fixed_direction: [-1.0, 0.0, 0.0],
            t_emit: 0.0,
            position_sampling: "one uniform point per cell".into(),
            direction_noise: "none".into(),
        }
    }
}
impl GeneratorSpec {
    pub fn count(&self) -> Result<usize, String> {
        let n = self.cell_count[0]
            .checked_mul(self.cell_count[1])
            .ok_or("cell count overflow")?;
        if n == 0 || n > 1_048_576 {
            return Err("generator requires 1..1048576 cells".into());
        }
        Ok(n)
    }
    pub fn extent(&self) -> [f64; 2] {
        [
            self.cell_size[0] * self.cell_count[0] as f64,
            self.cell_size[1] * self.cell_count[1] as f64,
        ]
    }
    pub fn validate(&self) -> Result<(), String> {
        let defaults = Self::default();
        if self.ray_generator_version != VERSION
            || self.rng_algorithm != RNG
            || self.seed_bits != 256
            || self.uniform_f64_rule != UNIFORM
            || self.cell_order != ORDER
            || self.position_sampling != defaults.position_sampling
            || self.direction_noise != "none"
        {
            return Err("unsupported ray generator specification/version".into());
        }
        self.count()?;
        if self
            .center
            .iter()
            .chain(&self.axis_1)
            .chain(&self.axis_2)
            .chain(&self.cell_size)
            .chain(&self.fixed_direction)
            .chain([&self.t_emit])
            .any(|v| !v.is_finite())
            || self.cell_size.iter().any(|s| *s <= 0.0)
            || self.extent().iter().any(|s| !s.is_finite())
            || self.fixed_direction.iter().all(|v| *v == 0.0)
        {
            return Err("invalid finite generator geometry/direction/time".into());
        }
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        if (dot(self.axis_1, self.axis_1) - 1.0).abs() > 1e-12
            || (dot(self.axis_2, self.axis_2) - 1.0).abs() > 1e-12
            || dot(self.axis_1, self.axis_2).abs() > 1e-12
        {
            return Err("generator basis must be orthonormal within 1e-12".into());
        }
        Ok(())
    }
    pub fn generate(&self, seed: [u8; 32]) -> Result<Vec<RayInitialCondition>, String> {
        self.validate()?;
        let mut rng = ChaCha20Rng::from_seed(seed);
        let mut rays = Vec::with_capacity(self.count()?);
        for j in 0..self.cell_count[1] {
            for i in 0..self.cell_count[0] {
                let u = uniform(&mut rng);
                let v = uniform(&mut rng);
                // Fixed binary64 operations: origin + random displacement in each
                // in-plane coordinate, then center + (axis1*s1 + axis2*s2).
                let s1 = (i as f64 - self.cell_count[0] as f64 * 0.5) * self.cell_size[0]
                    + u * self.cell_size[0];
                let s2 = (j as f64 - self.cell_count[1] as f64 * 0.5) * self.cell_size[1]
                    + v * self.cell_size[1];
                let p: [f64; 3] = std::array::from_fn(|k| {
                    self.center[k] + (self.axis_1[k] * s1 + self.axis_2[k] * s2)
                });
                rays.push(RayInitialCondition {
                    position_x: p[0],
                    position_y: p[1],
                    position_z: p[2],
                    direction_x: self.fixed_direction[0],
                    direction_y: self.fixed_direction[1],
                    direction_z: self.fixed_direction[2],
                    t_emit: self.t_emit,
                });
            }
        }
        if rays
            .iter()
            .any(|r| r.position().iter().any(|x| !x.is_finite()))
        {
            return Err("generator coordinate overflow".into());
        }
        Ok(rays)
    }
}
/// No chi, timestamp, simulation index or caller-provided entropy is involved.
pub fn master_seed() -> Result<[u8; 32], String> {
    let mut seed = [0; 32];
    getrandom::fill(&mut seed).map_err(|e| format!("OS seed generation failed: {e}"))?;
    Ok(seed)
}
pub fn uniform(rng: &mut ChaCha20Rng) -> f64 {
    (rng.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
}
