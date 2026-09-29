//! Source-independent initial conditions and CPU execution. No file formats or UI.
//! Positions and directions use the existing oblate Cartesian-like BL map:
//! origin at Kerr, +Z spin, right-handed, lengths in M; time is t_BL/M.
use crate::{
    compute::{PhysicsResult, cpu::CpuReferenceIntegrator},
    config::IntegratorConfig,
    detector::{intersection::Intersection, plane::DetectorPlane},
    physics::{
        coordinates::{cartesian_to_local, from_cartesian, unit},
        geodesic::State,
        kerr::Kerr,
        tetrad::ray_3d,
    },
};

/// Physical input only. IDs are assigned internally, never supplied by callers.
/// Energy is fixed to the existing local ZAMO energy convention, 1.0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayInitialCondition {
    pub position_x: f64,
    pub position_y: f64,
    pub position_z: f64,
    pub direction_x: f64,
    pub direction_y: f64,
    pub direction_z: f64,
    pub t_emit: f64,
}
impl RayInitialCondition {
    pub fn position(self) -> [f64; 3] {
        [self.position_x, self.position_y, self.position_z]
    }
    pub fn direction(self) -> [f64; 3] {
        [self.direction_x, self.direction_y, self.direction_z]
    }
    fn key(self) -> [f64; 7] {
        [
            self.position_x,
            self.position_y,
            self.position_z,
            self.direction_x,
            self.direction_y,
            self.direction_z,
            self.t_emit,
        ]
    }
    fn normalized(self) -> Result<Self, String> {
        // Scale first to accept finite nonzero directions even near f64 limits.
        // This is input normalization, not a change to the ZAMO conversion.
        let scale = self
            .direction()
            .into_iter()
            .map(f64::abs)
            .fold(0.0, f64::max);
        if scale == 0.0 {
            return Err("ray direction must be nonzero".into());
        }
        let direction = unit(self.direction().map(|v| v / scale))?;
        let zero = |v: f64| if v == 0.0 { 0.0 } else { v };
        Ok(Self {
            position_x: zero(self.position_x),
            position_y: zero(self.position_y),
            position_z: zero(self.position_z),
            direction_x: zero(direction[0]),
            direction_y: zero(direction[1]),
            direction_z: zero(direction[2]),
            t_emit: zero(self.t_emit),
        })
    }
}

/// No source plane, source UV, grid, IDs, or visualization settings are required.
#[derive(Clone, Debug)]
pub struct RaySimulationConfig {
    pub spin: f64,
    pub integration: IntegratorConfig,
    pub detector: DetectorPlane,
}

/// Identity and initial state cannot be changed after preparation.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedRay {
    ray_id: usize,
    condition: RayInitialCondition,
    initial: State,
}
impl PreparedRay {
    pub fn ray_id(&self) -> usize {
        self.ray_id
    }
    pub fn condition(&self) -> RayInitialCondition {
        self.condition
    }
    pub fn initial(&self) -> State {
        self.initial
    }
}

/// Validate every scalar, normalize directions, then sort the seven physical
/// scalars with total_cmp. +/-0 are canonicalized to +0. Exact duplicates remain
/// as indistinguishable copies with consecutive IDs; no hidden input-order key.
pub fn prepare_rays(
    conditions: Vec<RayInitialCondition>,
    config: &RaySimulationConfig,
) -> Result<Vec<PreparedRay>, String> {
    let k = Kerr::new(config.spin)?;
    config.integration.validate(k)?;
    config.detector.validate(k)?;
    if conditions.is_empty() {
        return Err("at least one ray initial condition is required".into());
    }
    if conditions
        .iter()
        .any(|c| c.key().iter().any(|v| !v.is_finite()))
    {
        return Err("every ray position, direction and t_emit must be finite".into());
    }
    let mut conditions = conditions
        .into_iter()
        .map(RayInitialCondition::normalized)
        .collect::<Result<Vec<_>, _>>()?;
    conditions.sort_by(|a, b| {
        a.key()
            .into_iter()
            .zip(b.key())
            .map(|(a, b)| a.total_cmp(&b))
            .find(|order| !order.is_eq())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    conditions
        .into_iter()
        .enumerate()
        .map(|(ray_id, condition)| {
            // Reuse the verified CartesianSpatial -> local ZAMO -> Kerr pipeline.
            let q = from_cartesian(k, condition.position())?;
            let n = cartesian_to_local(k, q, condition.direction())?;
            let initial = ray_3d(k, [condition.t_emit, q[0], q[1], q[2]], n, 1.0)?;
            if initial.radius() <= k.horizon() + config.integration.horizon_epsilon
                || initial.radius() >= config.integration.escape_radius
            {
                return Err("ray must start between capture cutoff and escape radius".into());
            }
            Ok(PreparedRay {
                ray_id,
                condition,
                initial,
            })
        })
        .collect()
}

/// Existing PhysicsResult owns this ray's full trajectory, status and detection.
/// Its identity is attached before integration, independent of completion order.
#[derive(Debug)]
pub struct RayOutcome {
    pub ray: PreparedRay,
    pub result: PhysicsResult,
}
/// In-memory view of the existing intersection, not a new saved event format.
/// No fictitious source UV is supplied to the legacy HitEvent schema.
pub struct RayDetection<'a> {
    pub ray_id: usize,
    pub t_emit: f64,
    pub delta_t: f64,
    pub hit: &'a Intersection,
}
impl RayOutcome {
    pub fn detection(&self) -> Option<RayDetection<'_>> {
        self.result.detection.as_ref().map(|hit| RayDetection {
            ray_id: self.ray.ray_id,
            t_emit: self.ray.condition.t_emit,
            delta_t: hit.state.time() - self.ray.condition.t_emit,
            hit,
        })
    }
}

/// New 3D entry point. The callback permits existing worker-style cancellation
/// between rays. No RNG, file loading, legacy IDs, or source plane participates.
pub fn calculate_rays(
    conditions: Vec<RayInitialCondition>,
    config: &RaySimulationConfig,
    keep_going: impl FnMut(usize, usize) -> bool,
) -> Result<Vec<RayOutcome>, String> {
    calculate_rays_with_execution(
        conditions,
        config,
        super::ray_execution::RayExecution::from_environment()?,
        keep_going,
    )
}

/// Explicit serial reference or thread limit for comparisons and Rust experiments.
/// Results remain in prepare_rays canonical order, independent of completion order.
pub fn calculate_rays_with_execution(
    conditions: Vec<RayInitialCondition>,
    config: &RaySimulationConfig,
    execution: super::ray_execution::RayExecution,
    keep_going: impl FnMut(usize, usize) -> bool,
) -> Result<Vec<RayOutcome>, String> {
    let rays = prepare_rays(conditions, config)?;
    let k = Kerr::new(config.spin)?;
    super::ray_execution::ordered_map(
        rays,
        execution,
        |ray| {
            let result = CpuReferenceIntegrator.integrate_with_detector(
                k,
                ray.initial,
                &config.integration,
                &config.detector,
            );
            RayOutcome { ray, result }
        },
        keep_going,
    )
}
