//! 새 실험: 아래 함수처럼 Case를 만들고 select()에 이름을 추가하세요.
use super::{initial_rays, parameter_scan};
use crate::{
    config::{DEFAULT_RAY_COUNT, DEFAULT_SPIN, IntegratorConfig},
    physics::{geodesic::State, kerr::Kerr},
};

#[derive(Clone, Copy, Debug)]
pub struct ExperimentSettings {
    pub spin: f64,
    pub source_radius: f64,
    pub angle_degrees: f64,
    pub ray_count: usize,
    pub impact_min: f64,
    pub impact_max: f64,
    /// BL 좌표 화면에서 빔 광원 y 분포의 반폭. 물리적 충돌매개변수 b와 다릅니다.
    pub beam_half_width: f64,
    pub comparison_impact: f64,
    pub integration: IntegratorConfig,
}
impl Default for ExperimentSettings {
    fn default() -> Self {
        Self {
            spin: DEFAULT_SPIN,
            source_radius: 50.0,
            angle_degrees: 170.0,
            ray_count: DEFAULT_RAY_COUNT,
            impact_min: 4.8,
            impact_max: 5.6,
            beam_half_width: 10.0,
            comparison_impact: 6.0,
            integration: IntegratorConfig::default(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Case {
    pub label: String,
    pub kerr: Kerr,
    pub initial: State,
    pub config: IntegratorConfig,
}
fn case(label: impl Into<String>, kerr: Kerr, initial: State, p: &ExperimentSettings) -> Case {
    Case {
        label: label.into(),
        kerr,
        initial,
        config: p.integration,
    }
}
pub fn single_ray(p: &ExperimentSettings) -> Result<Vec<Case>, String> {
    let k = Kerr::new(p.spin)?;
    Ok(vec![case(
        "single_ray",
        k,
        initial_rays::local_source(k, p.source_radius, 0.0, p.angle_degrees.to_radians())?,
        p,
    )])
}
pub fn parallel_beam(p: &ExperimentSettings) -> Result<Vec<Case>, String> {
    let k = Kerr::new(p.spin)?;
    parameter_scan::linear_values(-p.beam_half_width, p.beam_half_width, p.ray_count)?
        .into_iter()
        .map(|offset| {
            Ok(case(
                format!("beam_y={offset:.8}"),
                k,
                initial_rays::beam_member(k, p.source_radius, offset)?,
                p,
            ))
        })
        .collect()
}
pub fn prograde_vs_retrograde(p: &ExperimentSettings) -> Result<Vec<Case>, String> {
    let k = Kerr::new(p.spin)?;
    [1.0, -1.0]
        .into_iter()
        .map(|sign| {
            Ok(case(
                if sign > 0.0 { "prograde" } else { "retrograde" },
                k,
                initial_rays::incoming_impact(k, p.source_radius, sign * p.comparison_impact)?,
                p,
            ))
        })
        .collect()
}
/// Schwarzschild 기준 포획 경계. Kerr 경계는 parameter_scan::impact_scan을 쓰세요.
pub fn critical_impact_parameter_scan(p: &ExperimentSettings) -> Result<Vec<Case>, String> {
    parameter_scan::impact_scan(
        &ExperimentSettings { spin: 0.0, ..*p },
        p.impact_min,
        p.impact_max,
        p.ray_count,
    )
}
pub fn spin_scan(p: &ExperimentSettings) -> Result<Vec<Case>, String> {
    // 이 목록과 안쪽 실험을 직접 바꿔 새 연구 질문을 정의할 수 있습니다.
    let mut cases = Vec::new();
    for spin in [0.0, 0.2, 0.4, 0.6, 0.8, 0.9, 0.95] {
        cases.extend(prograde_vs_retrograde(&ExperimentSettings { spin, ..*p })?);
    }
    Ok(cases)
}
pub const NAMES: &[&str] = &[
    "single_ray",
    "parallel_beam",
    "prograde_vs_retrograde",
    "critical_impact_parameter_scan",
    "spin_scan",
    "impact_scan",
    "position_scan",
    "angle_scan",
    "three_dimensional_ray",
];
pub fn select(name: &str, p: &ExperimentSettings) -> Result<Vec<Case>, String> {
    p.integration.validate(Kerr::new(p.spin)?)?;
    if p.ray_count == 0
        || p.ray_count > 100_000
        || !p.source_radius.is_finite()
        || p.source_radius <= Kerr::new(p.spin)?.horizon() + p.integration.horizon_epsilon
        || !p.beam_half_width.is_finite()
        || p.beam_half_width < 0.0
        || !p.comparison_impact.is_finite()
        || p.comparison_impact <= 0.0
    {
        return Err(
            "invalid experiment settings (source outside cutoff; ray count 1..100000)".into(),
        );
    }
    let cases = match name {
        "single_ray" => single_ray(p),
        "three_dimensional_ray" => {
            let k = Kerr::new(p.spin)?;
            let initial = crate::physics::tetrad::ray_3d(
                k,
                [0.0, p.source_radius, 1.1, 0.0],
                crate::physics::coordinates::unit([-1.0, 0.15, 0.22])?,
                1.0,
            )?;
            Ok(vec![case("three_dimensional_ray", k, initial, p)])
        }
        "parallel_beam" => parallel_beam(p),
        "prograde_vs_retrograde" => prograde_vs_retrograde(p),
        "critical_impact_parameter_scan" => critical_impact_parameter_scan(p),
        "spin_scan" => spin_scan(p),
        "impact_scan" => parameter_scan::impact_scan(p, p.impact_min, p.impact_max, p.ray_count),
        "position_scan" => {
            parameter_scan::position_scan(p, p.source_radius, p.source_radius * 1.5, p.ray_count)
        }
        "angle_scan" => {
            parameter_scan::angle_scan(p, p.angle_degrees - 5.0, p.angle_degrees + 5.0, p.ray_count)
        }
        _ => Err(format!(
            "unknown experiment {name}; choose {}",
            NAMES.join(", ")
        )),
    }?;
    for c in &cases {
        c.config.validate(c.kerr)?;
        if c.initial.radius() <= c.kerr.horizon() + c.config.horizon_epsilon {
            return Err(format!("{} starts inside the numerical cutoff", c.label));
        }
    }
    Ok(cases)
}
