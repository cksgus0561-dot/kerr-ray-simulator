use kerr_ray::physics::kerr::Kerr;

#[test]
fn delta_matches_80_digit_reference_at_old_failures_and_across_domain() {
    // References use exact input f64 values and Decimal(80), not the new formula.
    // First 403 cases are previously failed endpoints; 56 span a=0,+/-.6,
    // +/-.99,+/-.9999 and r_plus+1e-12 through r_plus+1e6.
    let cases: Vec<[f64; 3]> =
        serde_json::from_str(include_str!("fixtures/stable_delta.json")).unwrap();
    assert_eq!(cases.len(), 459);
    for [chi, r, expected] in cases {
        let actual = Kerr::new(chi).unwrap().delta(r);
        assert!(actual > 0.0);
        assert!(
            (actual - expected).abs() <= 2.0 * f64::EPSILON * expected.abs(),
            "chi={chi}, r={r:.17e}, actual={actual:.17e}, reference={expected:.17e}"
        );
        assert_eq!(actual, Kerr::new(-chi).unwrap().delta(r));
    }
}
