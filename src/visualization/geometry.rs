//! Coordinate geometry only. No geodesic forces or duplicate Kerr equations.
use crate::{
    compute::RayStatus,
    detector::plane::Plane,
    physics::{
        coordinates::{bl_to_cartesian, from_cartesian},
        field::frame_dragging,
        kerr::Kerr,
    },
    session::ViewConfig,
    simulation::SimulationData,
};
use glam::Vec3;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Segment {
    pub a: [f32; 4],
    pub b: [f32; 4],
    pub color: [f32; 4],
    pub meta: [u32; 4],
}
impl Segment {
    pub fn line(a: Vec3, b: Vec3, color: [f32; 4], kind: u32, id: u32) -> Self {
        Self {
            a: [a.x, a.y, a.z, 0.0],
            b: [b.x, b.y, b.z, 0.0],
            color,
            meta: [kind, id, 0, 0],
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SurfaceVertex {
    pub position: [f32; 3],
    pub kind: u32,
    pub color: [f32; 4],
}

pub fn vector(v: [f64; 3]) -> Vec3 {
    Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32)
}
pub fn plane_corners(p: &Plane) -> [Vec3; 4] {
    [[-1., -1.], [1., -1.], [1., 1.], [-1., 1.]]
        .map(|s| vector(p.position(s[0] * p.width / 2.0, s[1] * p.height / 2.0)))
}
pub fn ray_color(s: RayStatus) -> [f32; 4] {
    match s {
        RayStatus::Detected => [0.20, 0.78, 1.0, 0.72],
        RayStatus::Captured => [1.0, 0.49, 0.13, 0.85],
        RayStatus::Escaped => [0.76, 0.47, 1.0, 0.65],
        RayStatus::NumericalFailure => [1.0, 0.08, 0.18, 1.0],
        RayStatus::Active => [0.7, 0.74, 0.8, 0.6],
    }
}

/// Morton ordering preserves 2D coverage; evenly spaced ranks work for rectangular
/// grids and do not pick a contiguous stripe of the source. Returns array indices.
pub fn subset(data: &SimulationData, max: usize, selected: Option<usize>) -> Vec<usize> {
    fn morton(x: u32, y: u32) -> u64 {
        let mut key = 0;
        for b in 0..16 {
            key |= ((x as u64 >> b) & 1) << (2 * b);
            key |= ((y as u64 >> b) & 1) << (2 * b + 1);
        }
        key
    }
    let p = &data.experiment.source.plane;
    let mut ids: Vec<_> = data
        .rays
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let x = ((r.source_uv[0] / p.width + 0.5).clamp(0.0, 1.0) * 65535.0) as u32;
            let y = ((r.source_uv[1] / p.height + 0.5).clamp(0.0, 1.0) * 65535.0) as u32;
            (morton(x, y), i)
        })
        .collect();
    ids.sort_unstable();
    let count = max.min(ids.len());
    if count == 0 {
        return Vec::new();
    }
    let mut result: Vec<_> = (0..count)
        .map(|i| ids[((2 * i + 1) * ids.len() / (2 * count)).min(ids.len() - 1)].1)
        .collect();
    if let Some(id) = selected.and_then(|id| data.rays.iter().position(|r| r.ray_id == id))
        && !result.contains(&id)
    {
        result[count - 1] = id;
    }
    result
}
pub fn bounds(data: &SimulationData, indices: &[usize]) -> (Vec3, Vec3) {
    let k = Kerr::new(data.experiment.spin).unwrap();
    let mut low = Vec3::ZERO;
    let mut high = Vec3::ZERO;
    let mut include = |v: Vec3| {
        low = low.min(v);
        high = high.max(v);
    };
    for p in [
        &data.experiment.source.plane,
        &data.experiment.detector.plane,
    ] {
        if std::ptr::eq(p, &data.experiment.source.plane) && !data.source_available {
            continue;
        }
        for c in plane_corners(p) {
            include(c);
        }
    }
    for &i in indices {
        for s in &data.rays[i].samples {
            include(vector(bl_to_cartesian(
                k, s.state[1], s.state[6], s.state[2],
            )));
        }
    }
    let margin = (high - low) * 0.06;
    (low - margin, high + margin)
}
pub fn nice_spacing(extent: f32) -> f32 {
    let raw = (extent * 2.0 / 12.0).max(0.001);
    let p = 10_f32.powf(raw.log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .find(|v| v * p >= raw)
        .unwrap()
        * p
}
pub fn grid(extent: f32, spacing: f32) -> Vec<Segment> {
    // Grid is an auxiliary straight coordinate lattice, never a warped gravity grid.
    let mut out = Vec::new();
    let n = (extent / spacing).floor().clamp(0.0, 20.0) as i32;
    for i in -n..=n {
        for j in -n..=n {
            for axis in 0..3 {
                let mut a = Vec3::ZERO;
                let mut b = Vec3::ZERO;
                a[axis] = -extent;
                b[axis] = extent;
                a[(axis + 1) % 3] = i as f32 * spacing;
                b[(axis + 1) % 3] = a[(axis + 1) % 3];
                a[(axis + 2) % 3] = j as f32 * spacing;
                b[(axis + 2) % 3] = a[(axis + 2) % 3];
                out.push(Segment::line(a, b, [0.25, 0.35, 0.44, 0.12], 0, u32::MAX));
            }
        }
    }
    out
}
fn arrow(out: &mut Vec<Segment>, a: Vec3, b: Vec3, color: [f32; 4], kind: u32) {
    let d = b - a;
    let len = d.length();
    if len <= 1e-8 {
        return;
    }
    out.push(Segment::line(a, b, color, kind, u32::MAX));
    let u = d / len;
    let side = u
        .cross(if u.z.abs() < 0.9 { Vec3::Z } else { Vec3::Y })
        .normalize();
    let base = b - d * 0.25;
    for sign in [-1.0, 1.0] {
        out.push(Segment::line(
            b,
            base + side * len * 0.12 * sign,
            color,
            kind,
            u32::MAX,
        ));
    }
}
fn plane(out: &mut Vec<Segment>, p: &Plane, kind: u32, color: [f32; 4]) {
    let c = plane_corners(p);
    for i in 0..4 {
        out.push(Segment::line(c[i], c[(i + 1) % 4], color, kind, u32::MAX));
    }
    let center = vector(p.center);
    let s = (p.width.min(p.height) * 0.2) as f32;
    arrow(
        out,
        center,
        center + vector(p.e_u) * s,
        [1.0, 0.35, 0.35, 0.95],
        kind,
    );
    arrow(
        out,
        center,
        center + vector(p.e_v) * s,
        [0.35, 1.0, 0.4, 0.95],
        kind,
    );
    arrow(
        out,
        center,
        center + vector(p.normal) * s,
        [0.4, 0.6, 1.0, 0.95],
        kind,
    );
}
pub fn static_geometry(
    data: &SimulationData,
    extent: f32,
    spacing: f32,
) -> (Vec<Segment>, Vec<SurfaceVertex>) {
    let k = Kerr::new(data.experiment.spin).unwrap();
    let mut lines = grid(extent, spacing);
    for (axis, color) in [
        (Vec3::X, [1., 0.25, 0.3, 0.8]),
        (Vec3::Y, [0.3, 0.9, 0.45, 0.8]),
        (Vec3::Z, [0.35, 0.55, 1., 0.8]),
    ] {
        arrow(&mut lines, -axis * extent, axis * extent, color, 1);
    }
    arrow(
        &mut lines,
        Vec3::new(0., 0., -8.),
        Vec3::new(0., 0., 12.),
        [1., 0.82, 0.34, 1.],
        2,
    );
    lines.push(Segment::line(
        Vec3::ZERO,
        Vec3::ZERO,
        [1., 0.85, 0.4, 1.],
        2,
        u32::MAX,
    ));
    if data.source_available {
        plane(
            &mut lines,
            &data.experiment.source.plane,
            5,
            [0.2, 1., 0.72, 0.8],
        );
    }
    plane(
        &mut lines,
        &data.experiment.detector.plane,
        7,
        [0.35, 0.68, 1., 0.85],
    );
    for ray in &data.rays {
        let p = vector(bl_to_cartesian(
            k,
            ray.initial[1],
            ray.initial[6],
            ray.initial[2],
        ));
        lines.push(Segment::line(
            p,
            p,
            [0.2, 1., 0.65, 0.65],
            6,
            ray.ray_id as u32,
        ));
    }
    let mut vertices = Vec::new();
    for i in 0..32 {
        for j in 0..64 {
            let point = |ii: usize, jj: usize, ergo: bool| {
                let th = std::f64::consts::PI * ii as f64 / 32.;
                let ph = std::f64::consts::TAU * jj as f64 / 64.;
                vector(bl_to_cartesian(
                    k,
                    if ergo { k.ergosphere(th) } else { k.horizon() },
                    th,
                    ph,
                ))
            };
            let p = [
                point(i, j, false),
                point(i + 1, j, false),
                point(i + 1, j + 1, false),
                point(i, j + 1, false),
            ];
            for v in [p[0], p[1], p[2], p[0], p[2], p[3]] {
                let light = (v.normalize().dot(Vec3::new(0.4, -0.6, 0.7).normalize()) * 0.5 + 0.5)
                    * 0.65
                    + 0.15;
                vertices.push(SurfaceVertex {
                    position: v.to_array(),
                    kind: 3,
                    color: [0.11 * light, 0.15 * light, 0.23 * light, 1.],
                });
            }
            if i % 2 == 0 {
                lines.push(Segment::line(
                    point(i, j, true),
                    point(i, j + 1, true),
                    [1., 0.64, 0.19, 0.40],
                    4,
                    u32::MAX,
                ));
            }
            if j % 4 == 0 {
                lines.push(Segment::line(
                    point(i, j, true),
                    point(i + 1, j, true),
                    [1., 0.64, 0.19, 0.40],
                    4,
                    u32::MAX,
                ));
            }
        }
    }
    (lines, vertices)
}
pub fn ray_segments(data: &SimulationData, indices: &[usize]) -> Vec<Segment> {
    let k = Kerr::new(data.experiment.spin).unwrap();
    let mut out = Vec::new();
    for &i in indices {
        let ray = &data.rays[i];
        for pair in ray.samples.windows(2) {
            let a = &pair[0];
            let b = &pair[1];
            let xyz = |s: &[f64; 8]| vector(bl_to_cartesian(k, s[1], s[6], s[2]));
            let mut segment = Segment::line(
                xyz(&a.state),
                xyz(&b.state),
                ray_color(ray.status),
                8,
                ray.ray_id as u32,
            );
            segment.a[3] = a.state[0] as f32;
            segment.b[3] = b.state[0] as f32;
            out.push(segment);
        }
    }
    out
}
pub fn hit_segments(
    data: &SimulationData,
    bins: crate::detector::binning::TimeBins,
) -> Vec<Segment> {
    let mut events: Vec<_> = data.events.iter().collect();
    events.sort_by(|a, b| a.t_hit.total_cmp(&b.t_hit));
    events
        .into_iter()
        .enumerate()
        .map(|(rank, e)| {
            let p = vector(data.experiment.detector.plane.position(e.u_hit, e.v_hit));
            let mut s = Segment::line(p, p, [0.95, 0.94, 0.45, 0.95], 9, e.ray_id as u32);
            s.a[3] = e.t_hit as f32;
            s.b[3] = e.t_hit as f32;
            // CPU f64 classifies bins and ranks. GPU comparisons use exact integers.
            s.meta[2] = rank as u32;
            s.meta[3] = bins.index(e.t_hit).map_or(u32::MAX, |i| i as u32);
            s
        })
        .collect()
}
pub fn field_segments(k: Kerr, v: &ViewConfig) -> (Vec<Segment>, f64) {
    let n = v.frame_dragging_density;
    let mut out = Vec::new();
    let mut maximum = 0.0_f64;
    let z_count = if v.field_3d { n } else { 1 };
    for z in 0..z_count {
        for y in 0..n {
            for x in 0..n {
                let sample =
                    |i: usize| ((i as f64 / (n - 1) as f64) * 2. - 1.) * v.field_extent as f64;
                let p = [
                    sample(x),
                    sample(y),
                    if z_count == 1 { 0.0 } else { sample(z) },
                ];
                let Ok(q) = from_cartesian(k, p) else {
                    continue;
                };
                let Ok(omega) = frame_dragging(k, q[0], q[1]) else {
                    continue;
                };
                maximum = maximum.max(omega.abs());
                let a = vector(p);
                let delta =
                    Vec3::new(-p[1] as f32, p[0] as f32, 0.) * omega as f32 * v.field_time_scale;
                arrow(&mut out, a, a + delta, [0.26, 0.95, 0.72, 0.85], 10);
            }
        }
    }
    (out, maximum)
}
