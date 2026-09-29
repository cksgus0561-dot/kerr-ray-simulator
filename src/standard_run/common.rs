//! Shared physical/numerical environment, independent of display and ray results.
use crate::{
    config::IntegratorConfig,
    detector::plane::{DetectorPlane, Plane},
    physics::kerr::Kerr,
    simulation::rays::RaySimulationConfig,
};
use serde::{Deserialize, Serialize};

pub const FORMAT_VERSION: &str = "1";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Convention {
    #[serde(rename = "G")]
    pub g: f64,
    pub c: f64,
    #[serde(rename = "M")]
    pub m: f64,
    pub origin: String,
    pub spin_axis: String,
    pub handedness: String,
    pub ray_coordinate_convention: String,
}
impl Default for Convention {
    fn default() -> Self {
        Self { g:1.0,c:1.0,m:1.0,
        origin:"Kerr mass center".into(),spin_axis:"+z".into(),handedness:"right-handed".into(),
        ray_coordinate_convention:"Oblate Cartesian-like Boyer-Lindquist: X=sqrt(r^2+a^2)*sin(theta)*cos(phi); Y=sqrt(r^2+a^2)*sin(theta)*sin(phi); Z=r*cos(theta); T=t_BL; CartesianSpatial direction; signature (-,+,+,+); local ZAMO energy=1".into() }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Numerics {
    pub rtol: f64,
    pub atol: f64,
    pub initial_step: f64,
    pub min_step: f64,
    pub max_step: f64,
    pub max_steps: usize,
    pub max_affine: f64,
    pub horizon_epsilon: f64,
    pub escape_radius: f64,
    pub null_tolerance: f64,
    pub detector_root_tolerance: f64,
}
impl Numerics {
    fn integration(&self) -> IntegratorConfig {
        IntegratorConfig {
            rtol: self.rtol,
            atol: self.atol,
            initial_step: self.initial_step,
            min_step: self.min_step,
            max_step: self.max_step,
            max_steps: self.max_steps,
            max_affine: self.max_affine,
            horizon_epsilon: self.horizon_epsilon,
            escape_radius: self.escape_radius,
            null_tolerance: self.null_tolerance,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Common {
    pub numerics: Numerics,
    /// The actual Plane fields are retained verbatim; never rebuild its basis.
    pub detector: Plane,
    pub convention: Convention,
    pub simulator_version: String,
    pub data_format_version: String,
    pub build_source_fnv1a64: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ray_generator: Option<crate::experiments::stratified::GeneratorSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_format: Option<super::seeded::BinaryFormat>,
    /// Absent in single-run archives; records the batch plan, not solver inputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch: Option<super::batch::BatchSpecification>,
}
impl Common {
    pub fn from_config(c: &RaySimulationConfig) -> Self {
        let n = c.integration;
        Self {
            numerics: Numerics {
                rtol: n.rtol,
                atol: n.atol,
                initial_step: n.initial_step,
                min_step: n.min_step,
                max_step: n.max_step,
                max_steps: n.max_steps,
                max_affine: n.max_affine,
                horizon_epsilon: n.horizon_epsilon,
                escape_radius: n.escape_radius,
                null_tolerance: n.null_tolerance,
                detector_root_tolerance: c.detector.root_tolerance,
            },
            detector: c.detector.plane.clone(),
            convention: Convention::default(),
            simulator_version: env!("CARGO_PKG_VERSION").into(),
            data_format_version: FORMAT_VERSION.into(),
            build_source_fnv1a64: env!("KERR_SOURCE_FINGERPRINT").into(),
            ray_generator: None,
            binary_format: None,
            batch: None,
        }
    }
    pub fn config(&self, chi: f64) -> Result<RaySimulationConfig, String> {
        self.validate_format()?;
        if self.convention != Convention::default() {
            return Err("unsupported physical/coordinate convention".into());
        }
        if self.simulator_version.is_empty() || self.build_source_fnv1a64.is_empty() {
            return Err("missing simulator provenance".into());
        }
        let cfg = RaySimulationConfig {
            spin: chi,
            integration: self.numerics.integration(),
            detector: DetectorPlane {
                plane: self.detector.clone(),
                root_tolerance: self.numerics.detector_root_tolerance,
                // Required by legacy validation only, unused by intersection physics.
                // Never persisted: no pixel-dependent quantity participates in replay.
                resolution: [1, 1],
            },
        };
        let k = Kerr::new(chi)?;
        cfg.integration.validate(k)?;
        cfg.detector.validate(k)?;
        Ok(cfg)
    }
    pub fn validate_format(&self) -> Result<(), String> {
        if let Some(batch) = &self.batch {
            batch.validate()?;
            if self.data_format_version != super::seeded::FORMAT_VERSION {
                return Err("batch requires seeded binary format".into());
            }
        }
        match self.data_format_version.as_str() {
            FORMAT_VERSION if self.ray_generator.is_none() && self.binary_format.is_none() => {
                Ok(())
            }
            super::seeded::FORMAT_VERSION => {
                self.ray_generator
                    .as_ref()
                    .ok_or("missing ray_generator")?
                    .validate()?;
                if self.binary_format.as_ref() != Some(&super::seeded::BinaryFormat::default()) {
                    return Err("unsupported binary format specification".into());
                }
                Ok(())
            }
            _ => Err("unsupported data_format_version".into()),
        }
    }
}
