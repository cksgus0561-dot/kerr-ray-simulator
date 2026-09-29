//! E=1, Lz=0, Q_timelike=0 inward rain congruence, parameterized by t_BL.
//! Marker connections show a material lattice, NOT moving space or an embedding.
//! Metric, inverse and oblate coordinate mapping are exclusively the core APIs.
use crate::{
    physics::{
        coordinates::{bl_to_cartesian, from_cartesian},
        kerr::Kerr,
        metric::{Metric, contract, multiply_vector},
    },
    session::FreeFallConfig,
};

/// Numerical display termination, not a changed event horizon. Axis is excluded.
pub const HORIZON_MARGIN: f64 = 0.02;
pub const DURATION: f64 = 80.0;
pub const SAMPLE_DT: f64 = 0.25;
pub const INTEGRATION_DT: f64 = 0.05;

#[derive(Clone, Copy, Debug)]
pub struct Flow {
    /// Covector and contravariant 4-velocity in (t,r,theta,phi) order.
    pub momentum: [f64; 4],
    pub velocity: [f64; 4],
    pub coordinate_velocity: [f64; 3],
    /// Diagonal spatial metric on t_BL=const; not observer rest-space lengths.
    pub spatial_diagonal: [f64; 3],
    pub normalization_error: f64,
}
pub fn flow(k: Kerr, q: [f64; 3]) -> Result<Flow, String> {
    if q.iter().any(|v| !v.is_finite())
        || q[0] <= k.horizon() + HORIZON_MARGIN
        || q[1].sin().abs() < 1e-8
    {
        return Err("free-fall marker outside supported BL exterior chart".into());
    }
    let inv = k.inverse(q[0], q[1])?;
    // BL has g^rt=g^rtheta=g^rphi=0. Unit rest mass: g^ab p_a p_b=-1.
    let pr2 = (-1.0 - inv[0][0]) / inv[1][1];
    if !pr2.is_finite() || pr2 <= 0.0 {
        return Err("invalid inward timelike normalization".into());
    }
    let p = [-1.0, -pr2.sqrt(), 0.0, 0.0];
    let u = multiply_vector(&inv, &p);
    let g = k.covariant(q[0], q[1])?;
    let error = (contract(&g, &u) + 1.0).abs();
    if u.iter().any(|v| !v.is_finite())
        || u[0] <= 0.0
        || u[1] >= 0.0
        || !error.is_finite()
        || error > 1e-8
    {
        return Err("invalid free-fall 4-velocity / normalization".into());
    }
    Ok(Flow {
        momentum: p,
        velocity: u,
        coordinate_velocity: [u[1] / u[0], u[2] / u[0], u[3] / u[0]],
        spatial_diagonal: [g[1][1], g[2][2], g[3][3]],
        normalization_error: error,
    })
}

/// RK4 advection of the fixed vector field; no new general timelike solver.
/// None removes a marker when any trial reaches the numerical horizon margin.
pub fn advect(k: Kerr, q: [f64; 3], dt: f64) -> Result<Option<[f64; 3]>, String> {
    if !dt.is_finite() || dt <= 0.0 {
        return Err("positive finite advection step required".into());
    }
    let eval = |x: [f64; 3]| -> Result<Option<[f64; 3]>, String> {
        if x[0] <= k.horizon() + HORIZON_MARGIN {
            return Ok(None);
        }
        Ok(Some(flow(k, x)?.coordinate_velocity))
    };
    let add = |v: [f64; 3], f: f64| std::array::from_fn(|i| q[i] + dt * f * v[i]);
    let Some(a) = eval(q)? else { return Ok(None) };
    let Some(b) = eval(add(a, 0.5))? else {
        return Ok(None);
    };
    let Some(c) = eval(add(b, 0.5))? else {
        return Ok(None);
    };
    let Some(d) = eval(add(c, 1.0))? else {
        return Ok(None);
    };
    let next = std::array::from_fn(|i| q[i] + dt * (a[i] + 2.0 * b[i] + 2.0 * c[i] + d[i]) / 6.0);
    Ok(eval(next)?.map(|_| next))
}

pub struct Cache {
    pub kerr: Kerr,
    /// Uniform t_BL samples, retained in f64 BL coordinates. phi is unwrapped.
    pub paths: Vec<Vec<[f64; 3]>>,
    /// Original nearest-neighbor lattice edges, never reconnect after loss.
    pub edges: Vec<[usize; 2]>,
    pub excluded_initial: usize,
    pub max_normalization_error: f64,
    pub seconds: f64,
}
impl Cache {
    pub fn build(
        chi: f64,
        cfg: &FreeFallConfig,
        keep_going: impl Fn() -> bool,
    ) -> Result<Self, String> {
        cfg.validate()?;
        let started = std::time::Instant::now();
        let k = Kerr::new(chi)?;
        let n = cfg.density;
        let mut paths = Vec::with_capacity(n * n * n);
        let mut edges = Vec::with_capacity(3 * n * n * (n - 1));
        let mut excluded_initial = 0;
        let mut maximum: f64 = 0.0;
        for z in 0..n {
            for y in 0..n {
                for x in 0..n {
                    if !keep_going() {
                        return Err("free-fall cache cancelled".into());
                    }
                    let id = paths.len();
                    for (v, offset) in [(x, 1), (y, n), (z, n * n)] {
                        if v + 1 < n {
                            edges.push([id, id + offset]);
                        }
                    }
                    let xyz = [x, y, z]
                        .map(|i| -cfg.extent + 2.0 * cfg.extent * i as f64 / (n - 1) as f64);
                    let mut path = Vec::new();
                    if let Ok(mut q) = from_cartesian(k, xyz)
                        && q[0] > k.horizon() + HORIZON_MARGIN
                        && q[1].sin().abs() >= 1e-8
                    {
                        path.push(q);
                        'samples: for _ in 1..=(DURATION / SAMPLE_DT) as usize {
                            if !keep_going() {
                                return Err("free-fall cache cancelled".into());
                            }
                            let substeps = (SAMPLE_DT / INTEGRATION_DT).ceil() as usize;
                            for _ in 0..substeps {
                                let Some(next) = advect(k, q, SAMPLE_DT / substeps as f64)? else {
                                    break 'samples;
                                };
                                q = next;
                            }
                            maximum = maximum.max(flow(k, q)?.normalization_error);
                            path.push(q);
                        }
                    } else {
                        excluded_initial += 1;
                    }
                    paths.push(path);
                }
            }
        }
        Ok(Self {
            kerr: k,
            paths,
            edges,
            excluded_initial,
            max_normalization_error: maximum,
            seconds: started.elapsed().as_secs_f64(),
        })
    }
    /// Interpolate BL samples at a common display time, then apply the original map.
    /// Markers are removed after their final valid sample; never respawned.
    pub fn position(&self, id: usize, time: f64) -> Option<[f64; 3]> {
        let q = self.coordinate(id, time)?;
        let xyz = bl_to_cartesian(self.kerr, q[0], q[1], q[2]);
        xyz.iter().all(|v| v.is_finite()).then_some(xyz)
    }
    pub fn coordinate(&self, id: usize, time: f64) -> Option<[f64; 3]> {
        let path = self.paths.get(id)?;
        if path.is_empty()
            || !time.is_finite()
            || time < 0.0
            || time > (path.len() - 1) as f64 * SAMPLE_DT
        {
            return None;
        }
        let index = time / SAMPLE_DT;
        let i = index.floor() as usize;
        let a = path[i];
        let b = path[(i + 1).min(path.len() - 1)];
        Some(std::array::from_fn(|j| {
            a[j] + (index - i as f64) * (b[j] - a[j])
        }))
    }
}
