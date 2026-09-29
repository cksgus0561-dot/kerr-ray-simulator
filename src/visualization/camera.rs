use crate::session::CameraConfig;
use glam::{Mat4, Vec3};
pub fn eye(c: &CameraConfig) -> Vec3 {
    Vec3::from_array(c.target)
        + c.distance
            * Vec3::new(
                c.pitch.cos() * c.yaw.cos(),
                c.pitch.cos() * c.yaw.sin(),
                c.pitch.sin(),
            )
}
pub fn matrix(c: &CameraConfig, aspect: f32) -> Mat4 {
    Mat4::perspective_rh(
        45_f32.to_radians(),
        aspect.max(0.05),
        (c.distance * 0.0002).max(0.001),
        c.distance * 20.0 + 10_000.0,
    ) * Mat4::look_at_rh(eye(c), Vec3::from_array(c.target), Vec3::Z)
}
pub fn orbit(c: &mut CameraConfig, delta: [f32; 2]) {
    c.yaw -= delta[0] * 0.006;
    c.pitch = (c.pitch + delta[1] * 0.006).clamp(-1.54, 1.54);
}
pub fn pan(c: &mut CameraConfig, delta: [f32; 2], viewport_height: f32) {
    let forward = (Vec3::from_array(c.target) - eye(c)).normalize();
    let right = forward.cross(Vec3::Z).normalize();
    let up = right.cross(forward);
    let scale = 2.0 * c.distance * (22.5_f32.to_radians()).tan() / viewport_height.max(1.0);
    c.target =
        (Vec3::from_array(c.target) + scale * (-delta[0] * right + delta[1] * up)).to_array();
}
pub fn zoom(c: &mut CameraConfig, delta: f32) {
    c.distance = (c.distance * (-delta * 0.0015).exp()).clamp(0.2, 1e7);
}
pub fn fit(c: &mut CameraConfig, min: Vec3, max: Vec3, aspect: f32) {
    c.target = ((min + max) * 0.5).to_array();
    let radius = (max - min).length() * 0.5;
    let half_angle = (22.5_f32.to_radians().tan() * aspect.clamp(0.1, 1.0)).atan();
    c.distance = (radius / half_angle.sin() * 1.10).max(5.0);
}
