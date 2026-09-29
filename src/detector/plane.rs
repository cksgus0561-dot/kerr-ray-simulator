//! Stationary finite rectangular worldtubes in Cartesian-like BL coordinates.
//! Euclidean basis/size describe the coordinate surface, not proper lengths/areas.
use crate::physics::{
    coordinates::{Vec3, add, cross, dot, scale, sub, unit},
    kerr::Kerr,
};
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Plane {
    pub center: Vec3,
    pub normal: Vec3,
    pub e_u: Vec3,
    pub e_v: Vec3,
    pub width: f64,
    pub height: f64,
}
impl Plane {
    pub fn from_normal_up(
        center: Vec3,
        normal: Vec3,
        up: Vec3,
        width: f64,
        height: f64,
    ) -> Result<Self, String> {
        let normal = unit(normal)?;
        let e_u = unit(cross(up, normal))?;
        let e_v = cross(normal, e_u);
        let plane = Self {
            center,
            normal,
            e_u,
            e_v,
            width,
            height,
        };
        plane.validate()?;
        Ok(plane)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self
            .center
            .iter()
            .chain(&self.normal)
            .chain(&self.e_u)
            .chain(&self.e_v)
            .any(|v| !v.is_finite())
            || !self.width.is_finite()
            || !self.height.is_finite()
            || self.width <= 0.0
            || self.height <= 0.0
        {
            return Err("plane needs finite values and positive sizes".into());
        }
        let basis = [self.e_u, self.e_v, self.normal];
        for i in 0..3 {
            for j in 0..3 {
                if (dot(basis[i], basis[j]) - if i == j { 1.0 } else { 0.0 }).abs() > 1e-12 {
                    return Err("plane e_u, e_v, normal must be orthonormal within 1e-12".into());
                }
            }
        }
        if dot(cross(self.e_u, self.e_v), self.normal) < 1.0 - 1e-12 {
            return Err("plane basis must be right handed".into());
        }
        Ok(())
    }
    /// Conservative physical domain: all fixed-coordinate detector points at r>2.
    /// This guarantees the static worldlines are timelike, outside any ergoregion.
    pub fn validate_stationary(&self, k: Kerr) -> Result<(), String> {
        self.validate()?;
        let u = (-dot(self.center, self.e_u)).clamp(-self.width / 2.0, self.width / 2.0);
        let v = (-dot(self.center, self.e_v)).clamp(-self.height / 2.0, self.height / 2.0);
        let nearest = self.position(u, v);
        if dot(nearest, nearest) <= 4.0 + k.spin().powi(2) {
            return Err(
                "stationary plane must lie entirely outside r=2 (static timelike region)".into(),
            );
        }
        Ok(())
    }
    pub fn position(&self, u: f64, v: f64) -> Vec3 {
        add(self.center, add(scale(self.e_u, u), scale(self.e_v, v)))
    }
    pub fn signed_distance(&self, x: Vec3) -> f64 {
        dot(sub(x, self.center), self.normal)
    }
    pub fn uv(&self, x: Vec3) -> [f64; 2] {
        let d = sub(x, self.center);
        [dot(d, self.e_u), dot(d, self.e_v)]
    }
    /// Both physical rectangle edges are included. No numerical size inflation.
    pub fn contains(&self, uv: [f64; 2]) -> bool {
        uv.iter().all(|x| x.is_finite())
            && uv[0].abs() <= self.width / 2.0
            && uv[1].abs() <= self.height / 2.0
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DetectorPlane {
    pub plane: Plane,
    pub resolution: [usize; 2],
    /// Absolute coordinate signed-distance tolerance for the step-internal root.
    pub root_tolerance: f64,
}
impl DetectorPlane {
    pub fn validate(&self, k: Kerr) -> Result<(), String> {
        self.plane.validate_stationary(k)?;
        if !self.root_tolerance.is_finite()
            || self.root_tolerance <= 0.0
            || self.resolution.contains(&0)
            || self.resolution[0]
                .checked_mul(self.resolution[1])
                .is_none_or(|n| n > 16_777_216)
        {
            return Err("invalid detector root tolerance/resolution".into());
        }
        Ok(())
    }
    /// u increases right, v increases upward; row 0 is +v at the top of PNG/array.
    pub fn pixel(&self, uv: [f64; 2]) -> Option<[usize; 2]> {
        if !self.plane.contains(uv) {
            return None;
        }
        let [nx, ny] = self.resolution;
        if nx == 0 || ny == 0 {
            return None;
        }
        let x = (((uv[0] / self.plane.width + 0.5) * nx as f64).floor() as usize).min(nx - 1);
        let y = (((0.5 - uv[1] / self.plane.height) * ny as f64).floor() as usize).min(ny - 1);
        Some([x, y])
    }
}
