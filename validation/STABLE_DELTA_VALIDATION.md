# Stable Kerr Delta — 2026-09-26

## Status

Delta evaluation accuracy is improved without changing its mathematical value,
physical model, integration algorithm or classification thresholds. Default
+/-0.9999 runs now have NUM=0. Tighter runs still have a few genuine endpoint null
violations. **The complete regression suite is NOT green:** two historical
bitwise-initial-state tests fail, and replaying the old 192x192 archive exceeds
its existing hit-comparison tolerance for one ray (three components). Neither
expected results nor tolerances were changed to hide this.

## Production change and call graph

Only src/physics/kerr.rs::Kerr::delta has changed production behavior.
No other polynomial Delta evaluation was found in src. All of these already
consume that method directly or indirectly:

- covariant metric g_rr, inverse metric and area_function;
- inverse_derivatives and Hamiltonian RHS in geodesic.rs;
- ZAMO lapse and initial momentum conversion in tetrad.rs;
- null_constraint -> inverse -> metric::contract -> Diagnostics::observe;
- radial potential validation and detector root re-integration using the same RHS;
- horizon exterior stepping and capture: horizon() and horizon_epsilon are unchanged;
  candidate states and null diagnostics now use the stable Delta.

The direct r*r - 2*r + a*a subtracts order-one rounded terms near the horizon
leaving Delta about 3e-5 at failed endpoints. Absolute error about1e-16 becomes
relative error about5e-12, amplified by inverse metric denominators and a null
quadratic form whose cancelling terms total millions.

New evaluation, natural units M=1:

```rust
let a2 = self.a * self.a;
let a2_error = self.a.mul_add(self.a, -a2);
r.mul_add(r - 2.0, a2) + a2_error
```

This is the polynomial r(r-2)+a^2 = r^2-2r+a^2, with a compensated rounded a^2
product. FMA rounds the product and sum together; the second FMA recovers the
product residual. Near the exterior horizon r-2 is exact by Sterbenz's lemma.
No chi threshold/branch, special ray handling, projection, epsilon change, NUM
reclassification, or null_tolerance relaxation exists. Signed a is squared in
the same manner for positive, negative and zero spin. Horizon/ergosphere formulas
and analytic derivatives are unchanged.

Rounded-root factorization was measured, not assumed superior. On the 403 old
failed states, maximum relative Delta errors were: direct5.48134e-12;
factorized with old horizon roots1.78455e-12; factorized with FMA discriminant
6.88585e-14; compensated polynomial1.92368e-16. Root rounding leaves a residual
near the root; compensation gives the best measured result without adding sqrt.

## High-precision audit

examples/delta_accuracy.rs exports actual Rust f64 evaluations. Independent
validation/analyze_delta_accuracy.py uses Decimal(80), exact binary64 input
values, and independently evaluated sin/cos series/metric. It does NOT integrate
rays or project states. 403 previous failed endpoints from six tolerance/sign
runs plus56 grid cases (a=0,+/-.6,+/-.99,+/-.9999, r_plus+1e-12 to r_plus+1e6).

The 459-case tests/fixtures/stable_delta.json contains independently computed
Delta references. The new test permits 2 epsilon relative error; existing physics
checks and all historical data remain unchanged. Extreme near-root and far-field
grid Delta max relative error is1.52013e-16. This does not claim correct rounding
for every representable input or arbitrary overflow-range coordinates.

## Runs and unchanged settings

Same 192x192 cells /36,864 rays;0.5M cells;96x96M region; center[80,0,0]; axes+Y,+Z;
direction[-1,0,0]; t_emit=0. Same 256-bit seed as the previous signed audit.
All ordinary runs retain identical canonical input arrays/hash and standard
300x300M detector at[-80,0,0]. Mirror runs deliberately reflect Y for symmetry.

A: rtol1e-12,atol1e-14. B:1e-13,1e-15. C:1e-14,1e-16.
Unchanged horizon_epsilon1e-3, null_tolerance1e-5, detector root1e-10,
escape_radius400, max_affine1500, max_steps200000, initial/min/max step0.05/1e-12/0.5.
Times are single physics wall measurements with11 workers, excluding save and
analysis. All DET hit components are finite.

## Remaining NUM and convergence

Default A: +0.9999 [0,36275,300,289,0], -0.9999 [0,36270,307,287,0]. All107 default
old NUM rays (52 positive,55 negative) were freshly integrated to CAP with the
unchanged classification rules. They were not relabeled from saved results.

Positive B/C have1/5 NUM; negative B/C have0/3 NUM. All remaining failures are
NullViolation. Unlike the old diagnostic artifacts, their same endpoint states
also violate null_tolerance under80-digit evaluation: roughly1.0037e-5..1.0281e-5.
Thus the improved Delta now diagnoses actual constraint drift in these f64
integrated states. Tightening the ODE tolerance still does not monotonically
improve invariant accuracy. No high-precision geodesic solve has been done to
separate accumulated step/RHS roundoff contributions further.

All A/B/C status transitions are CAP->NUM. DET and ESC labels stay unchanged.
Mean hit differences decrease between successive tolerance levels, but maximum
v/time differences do not. Near-polar ray18432 and near-horizon scattering ray
19203 remain sensitive. Fixed detector root tolerance also limits conclusions
from ODE tolerance tightening alone. Complete monotone convergence is NOT claimed.

## Symmetry and reproduction

With chi reversal and exact launch reflection Y->-Y (direction_Y likewise),
hits transform u->-u, v/t unchanged. Both .99 and .9999 paired36,864-ray runs have
zero status mismatch and exactly zero reflected hit difference.

New +.9999 A, -.9999 A and .6 default archives: save/load/fresh seed regeneration,
initial hash and Kerr recomputation PASS; status and hits bit-identical.
The original seed/RNG, canonical ordering, binary schema and replay thresholds
are unchanged. Source fingerprint correctly distinguishes the changed arithmetic.

## Regression failures are retained

cargo fmt and clippy --all-targets --offline -- -D warnings PASS. Release build
PASS. cargo test --offline --no-fail-fast and release equivalent: each131 tests,
129 passed,2 failed. Failures:

- regression_256_all_initial_states_match_legacy_and_saved_baseline
- regression_4096_all_initial_states_match_legacy_and_saved_baseline

Both fail at tests/support/ray_regression.rs:98, bitwise equality with historical
CSV momentum components. No test assertion or stored expected value was edited.
Separate fresh integration (examples/legacy_delta_regression.rs) confirms physical
status counts and detector-hit presence unchanged for all256/4096 rays. New and
old initial coordinate components are identical; momentum rounding differs by
up to7.106e-15. 76/256 and1209/4096 initial states differ bitwise. Maximum hit
absolute differences [u,v,t]: 256[5.771e-11,6.099e-11,1.058e-10];
4096[1.697e-10,1.614e-10,2.374e-10]. This evidence supplements, not overrides,
the failed bitwise tests.

Original192x192 chi=.6 status counts remain[0,36263,330,271,0]. Cross-version
reproduction FAILS the existing abs1e-9+rel1e-12 policy for ray18432's u/v/t:
[1.381112008e-8,5.471063602e-9,3.330600862e-8]. No status/hash mismatch. This is
not falsely reported as a successful historical replay. New-version replay PASS.

Existing Schwarzschild tests, analytic positive-Kerr tests, signed tests,
serial/parallel equality, and new-version archive/replay tests pass. No full
GUI interaction audit, multi-seed sweep, exact-extremal case, interior integration,
or independent high-precision geodesic reference was performed.

## Files and reproducibility

Production: src/physics/kerr.rs. New numerical test: tests/stable_delta.rs and
fixtures/stable_delta.json. Validation-only: examples/delta_accuracy.rs,
examples/legacy_delta_regression.rs, validation/analyze_delta_accuracy.py,
validation/compare_delta_versions.py; diagnose_signed_spin.py now accepts a
results directory. Existing signed_spin_validation driver and analysis reused.
README and this report document limitations. Old result archives are untouched.

All new evidence is under validation/stable_delta_20260926: before/after.json,
accuracy_summary.json, analysis.json, failure_diagnosis.json, cross_version.json,
legacy_outcome_regression.json, debug_all.txt, release_all.txt, clippy.txt,
build.txt and per-run standard simulation_1 folders/reproduction summaries.

## Measured tables

| Metric (403 old endpoints) | Before | After |
|---|---:|---:|
| actual_delta: max_rel | 5.481341867e-12 | 1.923681975e-16 |
| covariant: max_rel_nonzero | 5.481081752e-12 | 7.148178181e-16 |
| inverse: max_rel_nonzero | 5.481673162e-12 | 6.307919353e-16 |
| null: max_abs_error | 1.764184189e-05 | 1.194581176e-09 |
| null: mean_abs_error | 7.878789657e-06 | 2.688105631e-10 |

| Run | ACT | DET | CAP | ESC | NUM | Physics seconds |
|---|---:|---:|---:|---:|---:|---:|
| p099_A | 0 | 36279 | 303 | 282 | 0 | 3.9752293 |
| n099_A | 0 | 36269 | 310 | 285 | 0 | 3.7179986 |
| p09999_A | 0 | 36275 | 300 | 289 | 0 | 3.6623856 |
| p09999_B | 0 | 36275 | 299 | 289 | 1 | 5.1582242 |
| p09999_C | 0 | 36275 | 295 | 289 | 5 | 8.1868495 |
| n09999_A | 0 | 36270 | 307 | 287 | 0 | 3.6564296 |
| n09999_B | 0 | 36270 | 307 | 287 | 0 | 5.1518550 |
| n09999_C | 0 | 36270 | 304 | 287 | 3 | 7.7781793 |
| mirror_n099 | 0 | 36279 | 303 | 282 | 0 | 3.7400764 |
| mirror_n09999 | 0 | 36275 | 300 | 289 | 0 | 3.7260790 |

Hit comparisons include only rays detected in both runs.

| Pair | Common DET | Status changes | max du | mean du | max dv | mean dv | max dt | mean dt |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| p09999_A / p09999_B | 36275 | 1 | 1.940816219e-08 | 6.381366791e-12 | 3.967095097e-09 | 4.223610965e-12 | 1.673703309e-08 | 3.702591244e-11 |
| p09999_B / p09999_C | 36275 | 4 | 1.485341905e-08 | 3.943831019e-12 | 1.612611555e-08 | 3.709824170e-12 | 9.830392855e-08 | 3.586299717e-11 |
| p09999_A / p09999_C | 36275 | 5 | 2.179768899e-08 | 6.791655116e-12 | 1.848039410e-08 | 4.790580766e-12 | 1.126657310e-07 | 4.053184920e-11 |
| n09999_A / n09999_B | 36270 | 0 | 1.174790265e-09 | 5.834802117e-12 | 1.476390565e-09 | 4.181307670e-12 | 9.004423873e-09 | 3.652227824e-11 |
| n09999_B / n09999_C | 36270 | 3 | 3.394119541e-09 | 3.585015047e-12 | 1.994880705e-08 | 3.777186643e-12 | 1.216353667e-07 | 3.617092024e-11 |
| n09999_A / n09999_C | 36270 | 3 | 2.219329276e-09 | 6.115836736e-12 | 1.847241649e-08 | 4.719131049e-12 | 1.126309428e-07 | 3.992307251e-11 |
| p099_A / mirror_n099 | 36279 | 0 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 |
| p09999_A / mirror_n09999 | 36275 | 0 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 |

Seed: 2f3c83ca05af605c805d7636a6db455b81e6ac28b2a4fb97f1c00b3775361b3b
Initial-condition hash: df9ea08086b696ee706cb5677108d1cccbabc0631586b416d008eeff29004869
