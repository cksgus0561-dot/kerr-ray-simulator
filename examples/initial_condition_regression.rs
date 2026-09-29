//! One explicit full regression run; not repeated by debug/release unit suites.
//! Only stdout is written. Saved output files are comparison oracles, not inputs.
#[path = "../tests/support/ray_regression.rs"]
mod ray_regression;
use kerr_ray::{experiments::source_detector::status_index, simulation::rays::calculate_rays};
fn main() -> Result<(), String> {
    for side in [16, 64] {
        let baseline = ray_regression::baseline(side);
        let start = std::time::Instant::now();
        let results = calculate_rays(baseline.conditions, &baseline.config, |_, _| true)?;
        let mut counts = [0; 5];
        let mut max_hit_error = [0.0_f64; 3];
        for (id, ray) in results.iter().enumerate() {
            assert_eq!(ray.ray, baseline.prepared[id]);
            let expected = &baseline.expected[&ray.ray.initial().0.map(f64::to_bits)];
            assert_eq!(ray.result.status, expected.status);
            assert_eq!(ray.result.samples[0].state, ray.ray.initial());
            counts[status_index(ray.result.status)] += 1;
            match (ray.detection(), expected.hit) {
                (Some(event), Some(expected)) => {
                    assert_eq!(event.ray_id, id);
                    let actual = [event.hit.uv[0], event.hit.uv[1], event.hit.state.time()];
                    for i in 0..3 {
                        let error = (actual[i] - expected[i]).abs();
                        max_hit_error[i] = max_hit_error[i].max(error);
                        assert!(error <= 1e-9, "ray {id} hit component {i}: {error:e}");
                    }
                    assert_eq!(ray.result.samples.last().unwrap().state, event.hit.state);
                }
                (None, None) => {}
                _ => panic!("detector presence mismatch"),
            }
        }
        assert_eq!(counts, baseline.counts);
        println!(
            "rays={} counts={counts:?} initial_state_error=0 (all f64 bits) max_hit_error_uvt={max_hit_error:?} seconds={:.6}",
            side * side,
            start.elapsed().as_secs_f64()
        );
    }
    Ok(())
}
