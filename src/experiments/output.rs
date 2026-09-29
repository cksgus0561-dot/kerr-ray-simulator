//! CSV I/O is outside physics. Summary rows retain settings and initial canonical data.
use super::{observables as obs, scenarios::Case};
use crate::{compute::PhysicsResult, physics::geodesic::State};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
pub const SUMMARY_HEADER: &str = "ray_id,label,spin,status,stop_reason,failure_message,captured,escaped,sampled_min_r,scattering_angle_bl_rad,delta_phi_rad,total_abs_phi_rad,revolutions,affine_length,coordinate_time,seconds,max_null_abs,max_null_energy_scaled,max_E_relative,max_Lz_relative,max_E_abs,max_Lz_abs,max_Q_abs,accepted_steps,rejected_steps,event_iterations,initial_t,initial_r,initial_phi,initial_pt,initial_pr,initial_pphi,initial_b,rtol,atol,epsilon,r_escape,max_affine,initial_step,min_step,max_step,max_steps,null_tolerance,initial_theta,initial_ptheta,initial_Q,max_Q_relative";
fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
pub fn summary_row(id: usize, case: &Case, result: &PhysicsResult) -> String {
    let d = &result.diagnostics;
    let c = &case.config;
    let mut fields = vec![
        id.to_string(),
        quoted(&case.label),
        case.kerr.spin().to_string(),
        format!("{:?}", result.status),
        format!("{:?}", result.stop_reason),
        quoted(result.failure_message.as_deref().unwrap_or("")),
        obs::captured(result).to_string(),
        obs::escaped(result).to_string(),
        format!("{:.16e}", obs::minimum_radius(result)),
        obs::final_scattering_angle(case.kerr, result)
            .map(|v| format!("{v:.16e}"))
            .unwrap_or_default(),
    ];
    for v in [
        obs::accumulated_azimuth(result),
        obs::azimuth_travel(result),
        obs::revolutions(result),
        obs::affine_length(result),
        obs::coordinate_time_elapsed(result),
        result.elapsed_seconds,
        d.max_null_abs,
        d.max_null_energy_scaled,
        d.max_energy_relative,
        d.max_lz_relative,
        d.max_energy_abs,
        d.max_lz_abs,
        d.max_q_abs,
    ] {
        fields.push(format!("{v:.16e}"));
    }
    for v in [d.accepted_steps, d.rejected_steps, d.event_iterations] {
        fields.push(v.to_string());
    }
    for v in &case.initial.0[..6] {
        fields.push(format!("{v:.16e}"));
    }
    fields.push(if case.initial.energy() == 0.0 {
        String::new()
    } else {
        format!(
            "{:.16e}",
            case.initial.angular_momentum() / case.initial.energy()
        )
    });
    for v in [
        c.rtol,
        c.atol,
        c.horizon_epsilon,
        c.escape_radius,
        c.max_affine,
        c.initial_step,
        c.min_step,
        c.max_step,
    ] {
        fields.push(format!("{v:.16e}"));
    }
    fields.push(c.max_steps.to_string());
    fields.push(format!("{:.16e}", c.null_tolerance));
    for v in [
        case.initial.theta(),
        case.initial.0[7],
        case.initial.carter_q(case.kerr),
        d.max_q_relative,
    ] {
        fields.push(format!("{v:.16e}"));
    }
    fields.join(",")
}
pub fn write_trajectory(path: &Path, case: &Case, result: &PhysicsResult) -> std::io::Result<()> {
    let mut w = BufWriter::new(File::create(path)?);
    writeln!(
        w,
        "affine,t,r,theta,phi,pt,pr,ptheta,pphi,x_bl,y_bl,C_null,E,Lz,Q"
    )?;
    for sample in &result.samples {
        let s: State = sample.state;
        let values = [
            sample.affine,
            s.0[0],
            s.radius(),
            s.theta(),
            s.phi(),
            s.0[3],
            s.0[4],
            s.0[7],
            s.0[5],
            s.radius() * s.phi().cos(),
            s.radius() * s.phi().sin(),
            s.null_constraint(case.kerr).unwrap_or(f64::NAN),
            s.energy(),
            s.angular_momentum(),
            s.carter_q(case.kerr),
        ];
        writeln!(
            w,
            "{}",
            values
                .iter()
                .map(|v| format!("{v:.16e}"))
                .collect::<Vec<_>>()
                .join(",")
        )?;
    }
    w.flush()
}
