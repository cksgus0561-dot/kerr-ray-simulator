# Signed chi and near-extremal validation — 2026-09-26

## Conclusion

The input guard now accepts [-0.9999, +0.9999], excluding extremality. This is
**not a validated safe training-data interval**. The signed-spin equations and
reflection symmetry pass, but near-horizon null-constraint failures occur at both
endpoints and become more numerous at tighter ODE tolerances. No metric, inverse,
RHS, stepper, detector, RNG, scheduling, storage schema, default chi or default
tolerance was changed. Failed rays retain NUM; no threshold was relaxed.

## Why the previous domain existed

The actual constant is `src/physics/constants.rs::MAX_SPIN`, not CHI_MAX_SPIN.
The old value was 0.99 and `Kerr::new` required `0.0..=MAX_SPIN`. The original task
specified exactly that initial range and excluded extremality. README, the
analytic-test helper comment and old tests repeat the domain. No measured failure
at 0.99 or derivation of a stability boundary was found. It was a conservative
first-version scope; the negative exclusion was not a Kerr physical restriction.
There is no Git history in this project to establish any additional rationale.

The private Kerr.a field is set to chi without abs/clamping. The only shared
construction guard is Kerr::new; the GUI additionally used 0..=0.99. Both now use
MAX_SPIN symmetrically. The default chi remains 0.6.

## Construction/validation callers

Direct Kerr::new callers audited (all in src):

- simulation/rays.rs: prepare_rays, calculate_rays_with_execution; all seeded
  CLI/worker execution and fresh reproduction reach these methods.
- standard_run/common.rs: Common::config, covering raw binary loading and replay;
  standard_run/parquet_io.rs: legacy chi metadata loading.
- simulation/mod.rs and simulation/archive.rs: legacy/presentation conversion;
  experiments/source_detector.rs: initials, calculation, and rebin validation.
- experiments/scenarios.rs: single_ray, parallel_beam, prograde_vs_retrograde,
  scenario validation and three_dimensional_ray; parameter_scan.rs: parameter scans.
- visualization/geometry.rs, renderer.rs, app.rs: structures, field, trajectory
  conversion/picking; detector_view.rs: detector validation; free_fall.rs: display flow.

Metric evaluation also rejects r<=r_plus and theta outside (0,pi). Integration
config and detector stationary-plane validation are unchanged. Exact +/-1, values
outside the new guard, NaN and infinity remain rejected.

## Signed-a audit and symmetry derivation

- kerr.rs: Sigma, Delta, horizon, ergo, A and diagonal metric depend on a^2;
  g_tphi=-2ar sin^2(theta)/Sigma and g^tphi=-2ar/(Sigma Delta) retain signed a.
- tetrad.rs / field.rs: omega=-g_tphi/g_phiphi retains its sign. No abs on spin.
- coordinates.rs: oblate Cartesian map uses a^2 (appropriately sign-even).
  CartesianSpatial direction is projected into the same ZAMO frame.
- geodesic.rs: analytic derivatives retain the signed off-diagonal terms;
  canonical Hamilton equations, radial potential and Carter Q are consistent.
- detector/intersection.rs: Cartesian plane crossing with the same RHS; no
  positive-spin branches. Capture uses r_plus+epsilon and escape radial motion.
- geometry.rs takes abs(omega) only for the displayed maximum magnitude;
  arrow direction uses signed omega.

Two positive-spin conventions remain outside the standard ray solver:
`physics/validation.rs::photon_radius` intentionally accepts spin magnitude 0..1
for prograde/retrograde analytic formula tests; `experiments/scenarios.rs::
prograde_vs_retrograde` labels positive impact as prograde, a label appropriate
only for positive spin. These helpers were not repurposed or used for this run.

In BL, (a,phi,p_phi)->(-a,-phi,-p_phi) leaves the Hamiltonian invariant. The oblate
Cartesian map therefore reflects (X,Y,Z)->(X,-Y,Z). Reflect launch direction Y
as well. The standard source/detector centers are on the X axis, direction is -X,
and the detector local axes are +Y,+Z: hits must map (u,v,t)->(-u,v,t).
A random stratified realization is not exactly mirror-symmetric. The main +/-
comparisons use identical unreflected inputs; separate mirror runs use exactly
reflected inputs and match by physical input, not by row index or ray ID.

## Evidence and reproduction

- Driver: examples/signed_spin_validation.rs; existing generator/solver/save/load/
  reproduce APIs, one process per run. Analysis: validation/analyze_signed_spin.py.
- Results: validation/signed_spin_20260926/{label}/simulation_1/{common.json,sim_1.bin}.
- Per-ray initial state, identity, stopping state, minimum radius/polar distance,
  steps and invariants are in diagnostics.json outside the standard archives.
- A/B/C change only rtol/atol. Detector root tolerance, horizon cutoff and all other
  physics/numerics remain identical. Default cell count 192x192, 0.5M cell size,
  96x96M launch region, 36,864 rays, center [80,0,0], axes +Y,+Z, fixed direction
  [-1,0,0], t_emit=0. Detector center [-80,0,0], 300x300M, 128x128 pixels.
- CPU wall times below cover the physics progress interval; exclude compile,
  save, diagnostics serialization and GUI. Single measurements, 11 CPU workers.
- Four default runs (+/-0.99 and +/-0.9999) saved, loaded, regenerated seed/hash,
  and recomputed using the unchanged reproduction policy. All PASS; initial-state,
  canonical ID, status and hit mismatches zero; hits bit-identical. This includes
  reproducible NUM outcomes and does not certify numerical accuracy.

## Failure investigation

Every near-extremal failure is NullViolation in compute/cpu.rs, after
Diagnostics::observe -> State::null_constraint -> Kerr::inverse. No nonfinite hit,
step exhaustion, detector-root failure or unexpected sign handling was observed.
All status changes between tolerances are CAP<->NUM; DET and ESC classifications
remain unchanged in the tested sample.

The failed endpoints lie about 0.001..0.002356M outside r_plus (horizon_epsilon=0.001).
These failures are not near the polar axis. An independent 80-digit Decimal
reevaluation of the inverse-metric quadratic form at exactly the same binary64
endpoint states gives |C_null|<1e-5 for every failed endpoint, whereas the actual
f64 solver's diagnostic exceeds 1e-5. This is evaluation evidence, NOT an 80-digit
geodesic integration or authority to reclassify a failed ray.

Example: +0.9999 A, ray 15819: r-r_plus=0.001324359M,
solver max scaled null=1.051998697e-5. At the same endpoint, 80-digit null is
-3.941007567e-6. Replacing only high-precision Delta with the current f64
r*r-2*r+a*a value yields +1.052006021e-5. The absolute sum of cancelling quadratic
terms is 3.648174e6. This specifically implicates cancellation in Kerr::delta,
its inverse-metric denominators and the null contraction near the BL horizon.
The RHS also uses these coefficients. Merely tightening local ODE tolerances
cannot eliminate this conditioning/roundoff issue. No solver repair was attempted.
Full records: failure_diagnosis.json and diagnose_signed_spin.py.

Largest hit differences identify two sensitive trajectories: +0.9999 ray 19203
passes at sampled min r=1.0892358M; ray 18432 for either spin passes within roughly
1.05e-5 radians of a BL pole (sampled min r about22.573M). Their DET values remain
finite. The latter dominates stricter-stage v/time differences. No full-trajectory
investigation outside these flagged cases was performed. Fixed detector root
tolerance 1e-10 also limits what ODE tightening alone can establish.

## Regression and scope

cargo fmt; cargo clippy --all-targets --offline -- -D warnings; cargo test --offline;
cargo test --release --offline; release binary build: PASS. Debug and release each
130 tests (128 existing, two new signed metric/initial-state/RHS tests). The old
negative-input-rejection assertion was updated to the newly authorized boundary;
existing positive physics tolerances were not relaxed.

Schwarzschild, existing positive Kerr, 256/4096 fixtures, seed/bin reproduction,
and serial/parallel full-physics equality tests pass. The pre-change 192x192
chi=0.6 archive was independently reproduced: [ACT,DET,CAP,ESC,NUM]=
[0,36263,330,271,0], status/hash PASS and all hit bit mismatches zero. Its provenance
correctly reports same source/version=false because the domain guard changed.

Only one seed, standard geometry and finite tested chi values are covered. No
claim covers every chi/seed/initial direction, extremality, polar chart crossing,
horizon-interior motion or negative-spin GUI manual interaction. No independent
high-precision geodesic reference was computed. The new input range must not be
interpreted as an approved training-data interval.

## Measured tables

Seed hex: `2f3c83ca05af605c805d7636a6db455b81e6ac28b2a4fb97f1c00b3775361b3b`

Common canonical initial-condition SHA-256: `df9ea08086b696ee706cb5677108d1cccbabc0631586b416d008eeff29004869`

All eight unreflected A/B/C runs have exactly the same input arrays and hash.

A: rtol=1e-12, atol=1e-14; B: 1e-13,1e-15; C: 1e-14,1e-16.

Other fixed numerics: horizon_epsilon=1e-3; escape_radius=400; initial/min/max step=0.05/1e-12/0.5; max_steps=200000; max_affine=1500; null_tolerance=1e-5; detector root=1e-10.

| Run | ACT | DET | CAP | ESC | NUM | Physics seconds |
|---|---:|---:|---:|---:|---:|---:|
| p099_A | 0 | 36279 | 303 | 282 | 0 | 3.3816812 |
| n099_A | 0 | 36269 | 310 | 285 | 0 | 3.3917456 |
| p09999_A | 0 | 36275 | 248 | 289 | 52 | 3.4763897 |
| p09999_B | 0 | 36275 | 235 | 289 | 65 | 4.6691316 |
| p09999_C | 0 | 36275 | 216 | 289 | 84 | 7.2863486 |
| n09999_A | 0 | 36270 | 252 | 287 | 55 | 3.3984327 |
| n09999_B | 0 | 36270 | 245 | 287 | 62 | 4.9594286 |
| n09999_C | 0 | 36270 | 222 | 287 | 85 | 7.7490586 |
| mirror_n099 | 0 | 36279 | 303 | 282 | 0 | 3.6065369 |
| mirror_n09999 | 0 | 36275 | 248 | 289 | 52 | 3.8216300 |

Hit comparisons only use rays detected in BOTH runs. Length/time in M.

| Pair | Common DET | Status changes | max du | mean du | max dv | mean dv | max dt | mean dt |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| p09999_A / p09999_B | 36275 | 21 | 2.024327728e-08 | 6.388570769e-12 | 4.096527562e-09 | 4.242210934e-12 | 1.746900580e-08 | 3.712997233e-11 |
| p09999_B / p09999_C | 36275 | 29 | 2.438362068e-09 | 3.600743513e-12 | 1.302214159e-08 | 3.636670402e-12 | 7.939220836e-08 | 3.528679294e-11 |
| p09999_A / p09999_C | 36275 | 34 | 2.151293188e-08 | 6.771198652e-12 | 1.569250507e-08 | 4.773156203e-12 | 9.567347092e-08 | 4.045145200e-11 |
| n09999_A / n09999_B | 36270 | 15 | 1.047363363e-08 | 6.044664255e-12 | 5.446102236e-09 | 4.244125640e-12 | 3.319888719e-08 | 3.691914623e-11 |
| n09999_B / n09999_C | 36270 | 25 | 7.073777231e-09 | 3.662968032e-12 | 2.022540713e-08 | 3.833834581e-12 | 1.233092632e-07 | 3.652704027e-11 |
| n09999_A / n09999_C | 36270 | 32 | 3.399856396e-09 | 6.110474131e-12 | 1.477930489e-08 | 4.627507529e-12 | 9.011037605e-08 | 3.931413171e-11 |
| p099_A / mirror_n099 | 36279 | 0 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 |
| p09999_A / mirror_n09999 | 36275 | 0 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 | 0.000000000e+00 |

The mean successive differences mostly decrease, but maximum v/time errors increase from A/B to B/C; strict monotone convergence is NOT demonstrated. No detector physical boundary transitions were observed. NUM rises at stricter settings.

Raw full comparisons including changed ray IDs, failure messages and worst-case states: [analysis.json](signed_spin_20260926/analysis.json). Execution logs and individual archive reproduction summaries are in the same directory.
