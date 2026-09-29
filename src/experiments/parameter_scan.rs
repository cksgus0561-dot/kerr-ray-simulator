//! 작은 함수로 범위를 정의합니다. GUI와 데이터 분석에 의존하지 않습니다.
use super::{
    initial_rays,
    scenarios::{Case, ExperimentSettings},
};
use crate::physics::kerr::Kerr;
pub fn linear_values(start: f64, end: f64, count: usize) -> Result<Vec<f64>, String> {
    if !start.is_finite() || !end.is_finite() || end < start || count == 0 || count > 100_000 {
        return Err("scan needs finite ordered endpoints and count 1..100000".into());
    }
    Ok((0..count)
        .map(|i| {
            if count == 1 {
                start
            } else {
                start + (end - start) * i as f64 / (count - 1) as f64
            }
        })
        .collect())
}
pub fn impact_scan(
    p: &ExperimentSettings,
    min: f64,
    max: f64,
    count: usize,
) -> Result<Vec<Case>, String> {
    let k = Kerr::new(p.spin)?;
    linear_values(min, max, count)?
        .into_iter()
        .map(|b| {
            Ok(Case {
                label: format!("impact_b={b:.10}"),
                kerr: k,
                initial: initial_rays::incoming_impact(k, p.source_radius, b)?,
                config: p.integration,
            })
        })
        .collect()
}
pub fn position_scan(
    p: &ExperimentSettings,
    min: f64,
    max: f64,
    count: usize,
) -> Result<Vec<Case>, String> {
    let k = Kerr::new(p.spin)?;
    linear_values(min, max, count)?
        .into_iter()
        .map(|r| {
            Ok(Case {
                label: format!("source_r={r:.8}"),
                kerr: k,
                initial: initial_rays::local_source(k, r, 0.0, p.angle_degrees.to_radians())?,
                config: p.integration,
            })
        })
        .collect()
}
pub fn angle_scan(
    p: &ExperimentSettings,
    min_deg: f64,
    max_deg: f64,
    count: usize,
) -> Result<Vec<Case>, String> {
    let k = Kerr::new(p.spin)?;
    linear_values(min_deg, max_deg, count)?
        .into_iter()
        .map(|angle| {
            Ok(Case {
                label: format!("angle_deg={angle:.8}"),
                kerr: k,
                initial: initial_rays::local_source(k, p.source_radius, 0.0, angle.to_radians())?,
                config: p.integration,
            })
        })
        .collect()
}
