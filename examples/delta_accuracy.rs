//! Export actual f64 evaluations for an independent Decimal reference audit.
use kerr_ray::physics::{geodesic::State, kerr::Kerr, metric::Metric};
use serde::Deserialize;
use serde_json::json;
use std::{fs, path::Path};

#[derive(Deserialize)]
struct Case {
    label: String,
    chi: f64,
    state: [f64; 8],
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let cases: Vec<Case> = serde_json::from_slice(&fs::read(args.first().ok_or("cases.json")?)?)?;
    let records: Vec<_> = cases
        .into_iter()
        .map(|c| {
            let a = c.chi;
            let k = Kerr::new(a).unwrap();
            let s = State(c.state);
            let r = s.radius();
            let a2 = a * a;
            let a2_error = a.mul_add(a, -a2);
            let compensated = r.mul_add(r - 2.0, a2) + a2_error;
            let old = r * r - 2.0 * r + a * a;
            let root = (1.0 - a * a).sqrt();
            let factored = (r - (1.0 + root)) * (r - (1.0 - root));
            // More accurate discriminant still leaves rounded root/subtraction errors.
            let root_fma = (-a).mul_add(a, 1.0).sqrt();
            let factored_fma = (r - (1.0 + root_fma)) * (r - (1.0 - root_fma));
            json!({"label":c.label,"chi":a,"state":c.state,
                "old_delta":old,"factored_delta":factored,"factored_fma_delta":factored_fma,
                "compensated_delta":compensated,"actual_delta":k.delta(r),
                "covariant":k.covariant(r,s.theta()).unwrap(),
                "inverse":k.inverse(r,s.theta()).unwrap(),
                "null":s.null_constraint(k).unwrap()
            })
        })
        .collect();
    let output = Path::new(args.get(1).ok_or("output.json")?);
    fs::write(output, serde_json::to_vec(&records)?)?;
    println!("{} cases -> {}", records.len(), output.display());
    Ok(())
}
