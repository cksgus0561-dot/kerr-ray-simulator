pub mod cpu;
use crate::{
    config::IntegratorConfig,
    physics::{geodesic::State, kerr::Kerr, validation::Diagnostics},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RayStatus {
    Active,
    Detected,
    Captured,
    Escaped,
    NumericalFailure,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StopReason {
    DetectorHit,
    HorizonCutoff,
    EscapeRadius,
    AffineLimit,
    StepLimit,
    StepUnderflow,
    InvalidInitialState,
    NullViolation,
    NonFinite,
    IntegratorFailure,
}

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub affine: f64,
    pub state: State,
}

#[derive(Debug)]
pub struct PhysicsResult {
    pub samples: Vec<Sample>,
    pub status: RayStatus,
    pub stop_reason: StopReason,
    pub failure_message: Option<String>,
    pub diagnostics: Diagnostics,
    pub elapsed_seconds: f64,
    pub detection: Option<crate::detector::intersection::Intersection>,
}

/// Backend-neutral contract. GPU preview must keep these statuses and diagnostics.
pub trait RayIntegrator {
    fn integrate(&self, kerr: Kerr, initial: State, config: &IntegratorConfig) -> PhysicsResult;
}
