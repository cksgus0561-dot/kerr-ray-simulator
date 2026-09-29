//! Shared code/JSON/CLI/GUI settings. Physical parameters have exactly one owner.
use crate::experiments::source_detector::{SourceDetectorExperiment, SourcePattern};
use crate::experiments::stratified::GeneratorSpec;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const RESEARCH_RAY_GRID: [usize; 2] = crate::experiments::stratified::DEFAULT_CELL_COUNT;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionConfig {
    /// None is the explicit legacy SourcePlane adapter. Old JSON remains legacy.
    #[serde(default)]
    pub generator: Option<GeneratorSpec>,
    pub experiment: SourceDetectorExperiment,
    pub view: ViewConfig,
    pub camera: CameraConfig,
    /// Export every Nth accepted sample plus both endpoints. Memory retains all samples.
    pub trajectory_export_stride: usize,
}
impl Default for SessionConfig {
    fn default() -> Self {
        let mut config = Self {
            generator: Some(GeneratorSpec::default()),
            experiment: SourceDetectorExperiment::default(),
            view: ViewConfig::default(),
            camera: CameraConfig::default(),
            trajectory_export_stride: 1,
        };
        config.experiment = config.scene_experiment();
        config
    }
}
impl SessionConfig {
    pub fn regression() -> Self {
        Self {
            generator: None,
            experiment: SourceDetectorExperiment::default(),
            ..Self::default()
        }
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let v: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let config: Self = if v.get("experiment").is_some() || v.get("generator").is_some() {
            serde_json::from_value(v).map_err(|e| e.to_string())?
        } else {
            Self {
                generator: None,
                experiment: serde_json::from_value(v).map_err(|e| e.to_string())?,
                ..Self::default()
            }
        };
        config.validate_view()?;
        Ok(config)
    }
    pub fn validate_view(&self) -> Result<(), String> {
        if let Some(generator) = &self.generator {
            generator.validate()?;
        }
        let v = &self.view;
        v.free_fall.validate()?;
        if !(1..=65_536).contains(&v.max_rendered_trajectories)
            || !(2..=35).contains(&v.frame_dragging_density)
            || [
                v.grid_extent,
                v.grid_spacing,
                v.field_extent,
                v.field_time_scale,
                v.trajectory_thickness,
                v.playback_speed,
            ]
            .iter()
            .any(|x| !x.is_finite() || *x <= 0.0)
            || self.trajectory_export_stride == 0
            || !self.camera.distance.is_finite()
            || self.camera.distance <= 0.0
            || self
                .camera
                .target
                .iter()
                .chain([&self.camera.yaw, &self.camera.pitch])
                .any(|x| !x.is_finite())
        {
            return Err("invalid visualization/camera/sampling settings".into());
        }
        Ok(())
    }
    pub fn set_grid(&mut self, n: usize) {
        if let Some(g) = &mut self.generator {
            g.cell_count = [n, n];
        } else {
            self.experiment.source.pattern = SourcePattern::RectangularGrid { nu: n, nv: n };
        }
    }
    /// SourcePlane is only a presentation rectangle for the independent generator.
    /// It is not consulted when generating or preparing seeded rays.
    pub fn scene_experiment(&self) -> SourceDetectorExperiment {
        let mut e = self.experiment.clone();
        if let Some(g) = &self.generator {
            let size = g.extent();
            e.source.plane = crate::detector::plane::Plane {
                center: g.center,
                e_u: g.axis_1,
                e_v: g.axis_2,
                normal: crate::physics::coordinates::cross(g.axis_1, g.axis_2),
                width: size[0],
                height: size[1],
            };
            e.source.pattern = SourcePattern::RectangularGrid {
                nu: g.cell_count[0],
                nv: g.cell_count[1],
            };
            e.source.direction =
                crate::experiments::source_detector::LaunchDirection::CartesianSpatial(
                    g.fixed_direction,
                );
            e.source.t_emit = g.t_emit;
        }
        e
    }
    pub fn physics_key(&self) -> String {
        let mut v: serde_json::Value =
            serde_json::from_str(&physics_key(&self.experiment)).expect("physics json");
        if let Some(g) = &self.generator {
            let obj = v.as_object_mut().unwrap();
            obj.remove("source");
            obj.insert(
                "ray_generator".into(),
                serde_json::to_value(g).expect("generator json"),
            );
        }
        v.to_string()
    }
}

/// Camera/display/output/postprocessing edits must NEVER invalidate physical rays.
pub fn physics_key(exp: &SourceDetectorExperiment) -> String {
    let mut value = serde_json::to_value(exp).expect("serializable experiment");
    let obj = value.as_object_mut().unwrap();
    obj.remove("output");
    obj.remove("postprocess");
    obj.get_mut("detector")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("resolution");
    value.to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraConfig {
    pub target: [f32; 3],
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}
impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            target: [0.0; 3],
            distance: 450.0,
            yaw: -1.10,
            pitch: 0.55,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackMode {
    Static,
    Propagation,
    Detector,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectorMode {
    Accumulated,
    Instantaneous,
    Cumulative,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewConfig {
    pub free_fall: FreeFallConfig,
    /// Set false to restore the exact saved camera instead of fitting on startup.
    pub fit_on_load: bool,
    pub visibility: [bool; 12],
    pub max_rendered_trajectories: usize,
    pub grid_auto: bool,
    pub grid_extent: f32,
    pub grid_spacing: f32,
    pub frame_dragging_density: usize,
    pub field_3d: bool,
    pub field_extent: f32,
    /// Arrows depict omega * dX/dphi multiplied by this display interval in M.
    pub field_time_scale: f32,
    pub trajectory_thickness: f32,
    pub playback_speed: f32,
    pub playback_mode: PlaybackMode,
    pub detector_mode: DetectorMode,
    pub detector_overlay: bool,
    pub detector_panel: bool,
}
impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            free_fall: FreeFallConfig::default(),
            fit_on_load: true,
            visibility: [true; 12],
            max_rendered_trajectories: 256,
            grid_auto: true,
            grid_extent: 200.0,
            grid_spacing: 25.0,
            frame_dragging_density: 15,
            field_3d: false,
            field_extent: 12.0,
            field_time_scale: 12.0,
            trajectory_thickness: 1.35,
            playback_speed: 25.0,
            playback_mode: PlaybackMode::Static,
            detector_mode: DetectorMode::Accumulated,
            detector_overlay: true,
            detector_panel: true,
        }
    }
}

pub const VISIBILITY_NAMES: [&str; 12] = [
    "Cartesian Reference Grid",
    "XYZ axes",
    "Kerr spin axis (+Z)",
    "Event horizon",
    "Ergosurface",
    "SourcePlane",
    "Launch points",
    "DetectorPlane",
    "Trajectories",
    "Detector hit points",
    "Frame dragging",
    "Current ray positions",
];

/// Visualization markers only; excluded from experiment/physics_key and archives.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FreeFallConfig {
    pub visible: bool,
    /// Half-width of the initial Cartesian-like cube, in M.
    pub extent: f64,
    /// Nodes per axis, including the boundary nodes.
    pub density: usize,
    /// BL coordinate time M per wall-clock second, independent of ray playback.
    pub speed: f64,
}
impl Default for FreeFallConfig {
    fn default() -> Self {
        Self {
            visible: false,
            extent: 10.0,
            density: 9,
            speed: 3.0,
        }
    }
}
impl FreeFallConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !self.extent.is_finite()
            || !(3.0..=100.0).contains(&self.extent)
            || !(3..=17).contains(&self.density)
            || !self.speed.is_finite()
            || !(0.01..=100.0).contains(&self.speed)
        {
            return Err("free-fall extent 3..100, density 3..17, speed 0.01..100 required".into());
        }
        Ok(())
    }
}
