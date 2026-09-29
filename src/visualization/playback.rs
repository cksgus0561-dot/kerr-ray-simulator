//! Physical time is independent of the presentation frame counter.
use crate::{
    physics::{coordinates::bl_to_cartesian, kerr::Kerr},
    simulation::RayRecord,
};
#[derive(Clone, Debug)]
pub struct Playback {
    pub time: f64,
    pub playing: bool,
    pub range: [f64; 2],
}
impl Default for Playback {
    fn default() -> Self {
        Self {
            time: 0.0,
            playing: false,
            range: [0.0, 1.0],
        }
    }
}
impl Playback {
    pub fn tick(&mut self, wall_seconds: f64, speed: f64) {
        if self.playing {
            self.time = (self.time + wall_seconds.min(0.25) * speed).min(self.range[1]);
            if self.time >= self.range[1] {
                self.playing = false;
            }
        }
    }
    pub fn reset(&mut self) {
        self.time = self.range[0];
        self.playing = false;
    }
}
/// Display interpolation in Cartesian-like BL coordinates between stored samples.
/// Does not integrate, project, refit, or modify the physical trajectory.
pub fn position_at(ray: &RayRecord, k: Kerr, time: f64) -> Option<[f64; 3]> {
    let samples = &ray.samples;
    let first = samples.first()?;
    let last = samples.last()?;
    if time < first.state[0] || time > last.state[0] {
        return None;
    }
    let i = samples.partition_point(|s| s.state[0] < time);
    let xyz = |p: [f64; 8]| bl_to_cartesian(k, p[1], p[6], p[2]);
    if i == 0 {
        return Some(xyz(first.state));
    }
    let a = &samples[i - 1];
    let b = &samples[i.min(samples.len() - 1)];
    let d = b.state[0] - a.state[0];
    let f = if d > 0.0 {
        (time - a.state[0]) / d
    } else {
        0.0
    };
    let (x, y) = (xyz(a.state), xyz(b.state));
    Some(std::array::from_fn(|j| x[j] + f * (y[j] - x[j])))
}
