//! Pure postprocessing. Half-open physical time bins [start+k*dt,min(end,start+(k+1)*dt)).
//! PNG sampling/playback is independent of t_hit and never feeds back into physics.
use super::{events::HitEvent, plane::DetectorPlane};
/// Resolve arithmetic ambiguity at decimal time edges, without modifying event times.
/// Values within 4*EPSILON*max(1,abs(index)) of an integer are assigned to that edge.
fn snap_integer(x: f64) -> f64 {
    let n = x.round();
    if (x - n).abs() <= 4.0 * f64::EPSILON * x.abs().max(1.0) {
        n
    } else {
        x
    }
}
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum TimeBasis {
    Arrival,
    Travel,
}
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct TimeBins {
    pub width: f64,
    pub start: f64,
    pub end: f64,
    pub basis: TimeBasis,
}
impl TimeBins {
    pub fn count(self) -> Result<usize, String> {
        if [self.width, self.start, self.end]
            .iter()
            .any(|v| !v.is_finite())
            || self.width <= 0.0
            || self.end <= self.start
            || self.start + self.width == self.start
        {
            return Err("time bins need finite width>0, end>start and representable edges".into());
        }
        let n = snap_integer((self.end - self.start) / self.width)
            .ceil()
            .max(1.0);
        if !(1.0..=100_000.0).contains(&n) {
            return Err("time frame count must be 1..100000".into());
        }
        Ok(n as usize)
    }
    pub fn time(self, e: &HitEvent) -> f64 {
        match self.basis {
            TimeBasis::Arrival => e.t_hit,
            TimeBasis::Travel => e.delta_t,
        }
    }
    pub fn index(self, t: f64) -> Option<usize> {
        if !t.is_finite() || t < self.start || t >= self.end {
            return None;
        }
        let n = self.count().ok()?;
        // Half-open bins; repair only f64 arithmetic uncertainty at an integer edge.
        Some((snap_integer((t - self.start) / self.width).floor() as usize).min(n - 1))
    }
}
#[derive(Debug)]
pub struct BinnedEvents {
    pub accumulated: Vec<u64>,
    pub before_window: Vec<u64>,
    /// Sparse per-bin pixel indices, one entry per hit. No dense space*time allocation.
    pub by_bin: Vec<Vec<usize>>,
    pub spatial_count: u64,
    pub window_count: u64,
    pub outside_detector: u64,
    pub after_window: u64,
}
pub fn bin(
    events: &[HitEvent],
    det: &DetectorPlane,
    time: TimeBins,
) -> Result<BinnedEvents, String> {
    let n = time.count()?;
    let pixels = det.resolution[0]
        .checked_mul(det.resolution[1])
        .filter(|n| *n > 0 && *n <= 16_777_216)
        .ok_or("invalid output resolution")?;
    let mut b = BinnedEvents {
        accumulated: vec![0; pixels],
        before_window: vec![0; pixels],
        by_bin: vec![Vec::new(); n],
        spatial_count: 0,
        window_count: 0,
        outside_detector: 0,
        after_window: 0,
    };
    let mut ids = std::collections::HashSet::new();
    for e in events {
        e.validate()?;
        if !ids.insert(e.ray_id) {
            return Err("duplicate ray_id in absorbing detector event list".into());
        }
        let Some([x, y]) = det.pixel([e.u_hit, e.v_hit]) else {
            b.outside_detector += 1;
            continue;
        };
        let pixel = y * det.resolution[0] + x;
        b.accumulated[pixel] += 1;
        b.spatial_count += 1;
        let t = time.time(e);
        if t < time.start {
            b.before_window[pixel] += 1;
        } else if let Some(i) = time.index(t) {
            b.by_bin[i].push(pixel);
            b.window_count += 1;
        } else {
            b.after_window += 1;
        }
    }
    Ok(b)
}
