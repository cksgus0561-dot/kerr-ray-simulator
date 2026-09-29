//! Continuous arrival events are primary data. Timing is f64 Boyer-Lindquist t.
//! delta_t is t_hit - t_emit, not photon proper time. No rendering FPS participates.
use crate::compute::RayStatus;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct HitEvent {
    pub ray_id: usize,
    pub u_source: f64,
    pub v_source: f64,
    pub t_emit: f64,
    pub u_hit: f64,
    pub v_hit: f64,
    pub t_hit: f64,
    pub delta_t: f64,
    pub status: RayStatus,
}
impl HitEvent {
    pub fn validate(&self) -> Result<(), String> {
        let numbers = [
            self.u_source,
            self.v_source,
            self.t_emit,
            self.u_hit,
            self.v_hit,
            self.t_hit,
            self.delta_t,
        ];
        if numbers.iter().any(|v| !v.is_finite())
            || self.delta_t < 0.0
            || self.status != RayStatus::Detected
            || (self.delta_t - (self.t_hit - self.t_emit)).abs()
                > 1e-12 * self.t_hit.abs().max(self.t_emit.abs()).max(1.0)
        {
            return Err("invalid continuous detector event".into());
        }
        Ok(())
    }
}
