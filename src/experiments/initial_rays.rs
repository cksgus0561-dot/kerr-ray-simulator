//! 초기조건 편집 예제. 각도는 좌표 운동량이 아니라 ZAMO가 측정하는 방향입니다.
use crate::physics::{geodesic::State, kerr::Kerr, tetrad};

/// 광원의 BL 위치 (r,phi)와 국소 발사각. 0=바깥 방향, pi=안쪽 방향.
pub fn local_source(k: Kerr, r: f64, phi: f64, angle: f64) -> Result<State, String> {
    tetrad::equatorial_ray(k, r, phi, angle, 1.0)
}
/// 정확한 b=Lz/E로 산란 실험을 정의. 유한 거리의 화면 높이와 b는 다릅니다.
pub fn incoming_impact(k: Kerr, r: f64, b: f64) -> Result<State, String> {
    tetrad::ray_with_impact(k, r, 0.0, b, true)
}
/// 먼 거리 빔: BL 좌표 화면에서 x=source_x, y=offset에 광원을 배치.
/// 각 ZAMO의 e_r,e_phi 방향을 평평한 극좌표처럼 맞춰 -x로 발사합니다.
/// 곡률이 있는 서로 다른 지점의 '평행'은 유일하지 않습니다.
/// 이 정의는 무한히 먼 거리에서 평행 빔으로 접근합니다.
pub fn beam_member(k: Kerr, source_x: f64, offset: f64) -> Result<State, String> {
    let r = source_x.hypot(offset);
    let phi = offset.atan2(source_x);
    local_source(k, r, phi, std::f64::consts::PI - phi)
}
