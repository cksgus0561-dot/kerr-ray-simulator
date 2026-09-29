# Near-extremal residual failures and chi = +/-0.999 — 2026-09-26

## Scope and result

Validation only. All 62 files under `src` have identical SHA-256 hashes before
and after this task. The compensated `Kerr::delta()` is retained. No solver,
threshold, generator, RNG, storage, scheduler, or default parameter was changed.
Validation helpers and evidence were added; no training spin limit was set.

The six new +/-0.999 runs have NUM=0 and no tolerance-dependent status changes.
Their maximum observed energy-scaled null residual is 4.233676009e-6, below the
unchanged 1e-5 threshold. This supports +/-0.999 as a candidate for the tested
configuration, not as an established limit or a guarantee over other seeds.

The previous +/-0.9999 full-run results were reused rather than recalculated.
Only their nine failed run/ray pairs (eight distinct initial conditions) were
re-integrated. All nine terminal states matched their saved f64 bit patterns.
Their 80-digit terminal null residuals still exceed the threshold. This is
residual drift of the f64-integrated state, not the former Delta evaluation bug.

## Common inputs and method

- 192 x 192 = 36,864 rays; cell 0.5 x 0.5 M; extent 96 x 96 M.
- Generator center [80,0,0], axis_1 +Y, axis_2 +Z, fixed direction [-1,0,0],
  t_emit=0, local ZAMO energy=1. No direction noise.
- One seed-based uniform point per cell; unchanged ChaCha20 generator.
- Detector center [-80,0,0], normal +X, e_u +Y, e_v +Z, width/height 300 M;
  existing image resolution 128 x 128. No detector changes.
- Seed: `2f3c83ca05af605c805d7636a6db455b81e6ac28b2a4fb97f1c00b3775361b3b`
- Initial-condition SHA-256:
  `df9ea08086b696ee706cb5677108d1cccbabc0631586b416d008eeff29004869`
- All six unreflected +/-0.999 canonical input arrays and hashes equal the
  previous +/-0.9999 inputs. Generator metadata and source fingerprint match.
- A: rtol=1e-12, atol=1e-14; B: 1e-13,1e-15; C: 1e-14,1e-16.
- Unchanged: null_tolerance=1e-5, horizon_epsilon=1e-3 M,
  detector_root_tolerance=1e-10, escape_radius=400 M, max_affine=1500,
  max_steps=200000, initial/min/max step=0.05/1e-12/0.5.
- Times are single CPU physics wall measurements, 11 workers, excluding archive
  saving and analysis. Prior +/-0.9999 times are retained from that audit.
- Every DET hit is finite. Status counts below sum to 36,864 in every row.
- Hit errors compare matching canonical conditions detected in both runs.
  Units: u,v in M; t in Boyer-Lindquist time/M. These are differences between
  two f64 solves, not errors against an exact geodesic reference.

## Counts and physics times

| chi / tolerance | ACT | DET | CAP | ESC | NUM | seconds | max scaled null |
| --- | --- | --- | --- | --- | --- | --- | --- |
| +0.9999 A | 0 | 36275 | 300 | 289 | 0 | 3.6623856 | 6.877584383e-06 |
| +0.9999 B | 0 | 36275 | 299 | 289 | 1 | 5.1582242 | 1.003756188e-05 |
| +0.9999 C | 0 | 36275 | 295 | 289 | 5 | 8.1868495 | 1.017469913e-05 |
| -0.9999 A | 0 | 36270 | 307 | 287 | 0 | 3.6564296 | 7.169554010e-06 |
| -0.9999 B | 0 | 36270 | 307 | 287 | 0 | 5.1518550 | 6.175599992e-06 |
| -0.9999 C | 0 | 36270 | 304 | 287 | 3 | 7.7781793 | 1.028133556e-05 |
| +0.999 A | 0 | 36275 | 302 | 287 | 0 | 3.4647131 | 1.403037459e-06 |
| +0.999 B | 0 | 36275 | 302 | 287 | 0 | 4.6533803 | 2.859684173e-06 |
| +0.999 C | 0 | 36275 | 302 | 287 | 0 | 7.1283194 | 4.233676009e-06 |
| -0.999 A | 0 | 36270 | 307 | 287 | 0 | 3.3687339 | 2.289307304e-06 |
| -0.999 B | 0 | 36270 | 307 | 287 | 0 | 4.6651506 | 1.844775397e-06 |
| -0.999 C | 0 | 36270 | 307 | 287 | 0 | 7.0933293 | 2.493092325e-06 |

## Status transitions

| comparison | all status changes | CAP/NUM changes | DET/other changes |
| --- | --- | --- | --- |
| +0.9999 A -> B | 1 | 1 | 0 |
| +0.9999 B -> C | 4 | 4 | 0 |
| +0.9999 A -> C | 5 | 5 | 0 |
| -0.9999 A -> B | 0 | 0 | 0 |
| -0.9999 B -> C | 3 | 3 | 0 |
| -0.9999 A -> C | 3 | 3 | 0 |
| +0.999 A -> B | 0 | 0 | 0 |
| +0.999 B -> C | 0 | 0 | 0 |
| +0.999 A -> C | 0 | 0 | 0 |
| -0.999 A -> B | 0 | 0 | 0 |
| -0.999 B -> C | 0 | 0 | 0 |
| -0.999 A -> C | 0 | 0 | 0 |

All nonzero transitions above are CAP -> NUM; none are NUM -> CAP.

## Hit differences: maximum and mean absolute values

| comparison | common DET | max du | mean du | max dv | mean dv | max dt | mean dt |
| --- | --- | --- | --- | --- | --- | --- | --- |
| +0.9999 A -> B | 36275 | 1.940816219e-08 | 6.381366791e-12 | 3.967095097e-09 | 4.223610965e-12 | 1.673703309e-08 | 3.702591244e-11 |
| +0.9999 B -> C | 36275 | 1.485341905e-08 | 3.943831019e-12 | 1.612611555e-08 | 3.709824170e-12 | 9.830392855e-08 | 3.586299717e-11 |
| +0.9999 A -> C | 36275 | 2.179768899e-08 | 6.791655116e-12 | 1.848039410e-08 | 4.790580766e-12 | 1.126657310e-07 | 4.053184920e-11 |
| -0.9999 A -> B | 36270 | 1.174790265e-09 | 5.834802117e-12 | 1.476390565e-09 | 4.181307670e-12 | 9.004423873e-09 | 3.652227824e-11 |
| -0.9999 B -> C | 36270 | 3.394119541e-09 | 3.585015047e-12 | 1.994880705e-08 | 3.777186643e-12 | 1.216353667e-07 | 3.617092024e-11 |
| -0.9999 A -> C | 36270 | 2.219329276e-09 | 6.115836736e-12 | 1.847241649e-08 | 4.719131049e-12 | 1.126309428e-07 | 3.992307251e-11 |
| +0.999 A -> B | 36275 | 2.511560848e-09 | 5.770790930e-12 | 6.724292234e-10 | 4.051280864e-12 | 1.863469379e-09 | 3.616982078e-11 |
| +0.999 B -> C | 36275 | 2.846294811e-08 | 4.286041867e-12 | 6.787274742e-09 | 3.441280540e-12 | 4.135640097e-08 | 3.412730707e-11 |
| +0.999 A -> C | 36275 | 3.097450896e-08 | 6.827866006e-12 | 7.094262955e-09 | 4.385768298e-12 | 4.321987035e-08 | 3.815970321e-11 |
| -0.999 A -> B | 36270 | 6.844005696e-09 | 6.034314951e-12 | 1.756468464e-07 | 9.069295405e-12 | 1.453867640e-07 | 4.094641783e-11 |
| -0.999 B -> C | 36270 | 2.433410029e-09 | 3.565078420e-12 | 1.906896330e-08 | 4.147435056e-12 | 8.067812018e-08 | 3.560494034e-11 |
| -0.999 A -> C | 36270 | 9.277415725e-09 | 6.394123535e-12 | 1.947158097e-07 | 1.011947203e-11 | 1.611633138e-07 | 4.447645683e-11 |

### Convergence interpretation

For +/-0.999, successive mean differences decrease from A/B to B/C in all three
components. Negative-spin maxima also decrease. Positive-spin maxima increase,
so complete monotonic convergence is NOT established. No DET or other status
changes occur at this spin magnitude, and no NUM appears in any tolerance run.
The maximum observed null residual itself does not monotonically decrease.

Positive-spin B/C hit maxima come from near-polar ray 18432 (minimum polar
angular distance about 1.0485e-5 rad, minimum r about 22.573 M). Negative-spin
A/B maximum v/t differences come from ray 18725 (minimum r about 2.728 M).
These are diagnostics already retained during the new runs; no whole-dataset
trajectory analysis or altered numerical thresholds were introduced.

The negative +/-0.999 A/B maximum v/t differences exceed those at +/-0.9999.
Thus +/-0.999 is better here in classification stability and null margin, but
not uniformly better in every maximum detector-hit difference. Detector root
tolerance is fixed; these tests cannot isolate its contribution to the error
floor from integration and f64 coordinate conditioning.

## Focused +/-0.9999 endpoint audit

`examples/near_extremal_trace.rs` regenerates conditions from the original seed,
uses the unchanged CPU integrator for the nine failed ray/run pairs, and exports
initial/radius checkpoint samples and the final 64 accepted samples. Terminal
states and statuses are asserted identical to the previous saved diagnostics.
No failed ray was relabeled or projected onto the null constraint.

`validation/analyze_focused_traces.py` evaluates the original analytic Kerr
metric, inverse, derivatives, Hamiltonian null and RHS with Python Decimal at
80 significant digits, exact binary64 inputs, and independent sine/cosine
Taylor series. It does not perform high-precision geodesic integration. Reference
values below are rounded only for display; the evaluation itself uses 80 digits.
Metric relative errors are maxima over nonzero components.

The table reports signed raw C_null. Actual code uses
abs(C_null)/initial_ZAMO_energy^2, with initial ZAMO energy 1 here up to roundoff;
this negligible normalization does not affect any of the threshold comparisons.

| chi / tol | ray_id | r-r+ / M | Delta f64 | C_null f64 | C_null 80-digit | accepted/rejected |
| --- | --- | --- | --- | --- | --- | --- |
| +0.9999 B | 15819 | 0.001025978153 | 3.007095006e-05 | 1.003756188e-05 | 1.003746398e-05 | 5122/0 |
| +0.9999 C | 15819 | 0.001283268370 | 3.794218094e-05 | -1.004384831e-05 | -1.004319807e-05 | 7977/174 |
| +0.9999 C | 15963 | 0.001334367067 | 3.952119198e-05 | -1.014093868e-05 | -1.014075903e-05 | 8243/232 |
| +0.9999 C | 15990 | 0.001346049579 | 3.988292907e-05 | -1.013139263e-05 | -1.013149196e-05 | 8733/571 |
| +0.9999 C | 15998 | 0.001282048478 | 3.790454866e-05 | -1.007295214e-05 | -1.007288065e-05 | 9795/773 |
| +0.9999 C | 16048 | 0.001434253501 | 4.262288397e-05 | -1.017469913e-05 | -1.017433529e-05 | 9159/492 |
| -0.9999 C | 20858 | 0.001092745783 | 3.210083879e-05 | -1.028133556e-05 | -1.028100200e-05 | 8468/221 |
| -0.9999 C | 20873 | 0.001125142962 | 3.308899980e-05 | -1.017400064e-05 | -1.017397521e-05 | 8439/229 |
| -0.9999 C | 20985 | 0.001090805004 | 3.204170885e-05 | -1.023383811e-05 | -1.023305142e-05 | 5920/0 |

| chi / tol | ray_id | Delta relative error | g max relative error | inverse max relative error | null evaluation abs error | RHS max relative error |
| --- | --- | --- | --- | --- | --- | --- |
| +0.9999 B | 15819 | 3.089169718e-17 | 1.233784908e-16 | 9.149102566e-17 | 9.790184317e-11 | 1.157747027e-11 |
| +0.9999 C | 15819 | 1.595062247e-16 | 3.055294136e-16 | 2.237339573e-16 | 6.502379562e-10 | 1.123948455e-11 |
| +0.9999 C | 15963 | 7.493499573e-17 | 2.397232040e-16 | 2.590577057e-16 | 1.796587977e-10 | 2.156839907e-11 |
| +0.9999 C | 15990 | 8.738195122e-17 | 1.894955214e-16 | 2.003372563e-16 | 9.933223371e-11 | 1.749840705e-11 |
| +0.9999 C | 15998 | 7.400858154e-19 | 2.115521862e-16 | 2.491759622e-16 | 7.149042093e-11 | 5.014289577e-12 |
| +0.9999 C | 16048 | 4.681332723e-17 | 7.446856305e-17 | 1.350923614e-16 | 3.638330382e-10 | 4.569759220e-12 |
| -0.9999 C | 20858 | 1.866140133e-16 | 3.237351201e-16 | 2.412010119e-16 | 3.335598273e-10 | 2.371784464e-11 |
| -0.9999 C | 20873 | 4.500673748e-17 | 8.030687690e-17 | 2.535007733e-16 | 2.542290363e-11 | 1.368869970e-11 |
| -0.9999 C | 20985 | 1.698918144e-16 | 3.787566133e-16 | 2.695189392e-16 | 7.866955386e-10 | 7.228496768e-12 |

### What is established, and what remains unresolved

- All endpoints lie outside the unchanged capture cutoff: r-r+ is
  0.001025978 to 0.001434254 M, while capture uses 0.001 M. All stop for
  NullViolation before reaching that cutoff.
- All nine immediately preceding accepted states are below the threshold even
  in 80-digit evaluation; the next accepted states cross it. Initial high-
  precision null residuals are of order 1e-16.
- Terminal Delta relative error <=1.8662e-16, metric <=3.7876e-16 and inverse
  <=2.6952e-16. Terminal null evaluation error <=7.8670e-10, much smaller than
  the approximately 1e-5 residual of the integrated state. Evaluating the same
  state more accurately does NOT remove the failures.
- The near-horizon BL state is highly conditioned: the first-order estimate
  for combined half-ULP changes of all stored state components is 2.33e-7 to
  5.34e-7 in C_null. This estimate is not a rigorous error interval. It is less
  than the full accumulated residual but comparable to its margin above the
  threshold, so it can affect which side of the threshold a near-boundary
  state occupies. It does not explain the entire drift by endpoint rounding.
- RHS component evaluation error is measurable (terminal maximum relative error
  up to 2.372e-11; across the final sampled states up to 4.819e-11). A small
  difference of large terms can be less accurate than individual metric
  components. These measurements alone do not identify the dominant accumulated
  source of trajectory error.
- The integrator controls a local error norm of state components. It then
  checks the conserved null condition independently, before classifying capture.
  Acceptance of a Runge-Kutta step is not a guarantee of global null conservation.
  No threshold/order/capture bug has been established by these observations.
- Confirmed: constraint drift in the f64-integrated state plus high BL horizon
  sensitivity. Accumulated step rounding, RHS cancellation and truncation error
  have NOT been quantitatively separated. Smaller accepted steps and more
  operations can increase roundoff exposure, but high-precision integration
  would be required to establish the contribution of each mechanism.

At the same r-r+=0.001 M cutoff, Delta is approximately 9.0420e-5 for |chi|=.999
and 2.9284e-5 for |chi|=.9999 (about 3.09 times smaller). This supports the
conditioning explanation, not a proof of stability for all input rays.

## Signed symmetry at +/-0.999

The existing transformation is chi -> -chi, launch Y -> -Y and direction_Y ->
-direction_Y. The fixed +X-normal detector with e_u=+Y, e_v=+Z maps hits as
(u,v,t) -> (-u,v,t). It follows the existing coordinate convention; no new
symmetry rule or coordinate convention was introduced.

For A, B and C, each 36,864-ray reflected run has zero status mismatches and
exactly zero reflected u/v/t differences (maximum and mean). Canonical IDs are
mapped by reflected initial conditions rather than assumed equal across the
reflection. Unreflected same-seed positive and negative counts need not agree:
the finite stratified random point set is not itself reflection-symmetric.

## Standard save/load/reproduction at +/-0.999

Both signs at A used the unchanged common.json + sim_1.bin path. Reproduction
regenerated conditions from the stored seed, verified the initial-condition
hash, canonicalized and freshly integrated all 36,864 rays. Both PASS:

- hash PASS; canonical/ray ID mismatch 0;
- reconstructed initial State vs solver start bit mismatch 0;
- status mismatch 0; count and hit-presence mismatch 0;
- hit u/v/t bit mismatch 0; maximum absolute and relative difference 0;
- existing tolerance abs <= 1e-9 + 1e-12 * max(abs(old),abs(new)) unchanged;
- same source/version true; replay time +0.999 3.359696 s, -0.999 3.403243 s.

The format does not store historical initial Kerr States. Reproduction's State
check compares reconstructed State with solver start, not an absent historical
State record. B/C separate reproduction was not run; their standard archives
were saved/read for the comparisons and symmetry tests.

## Limitations and validation status

- Only this seed, generator, direction, detector and numerical configuration
  were validated. No multi-seed or general phase-space guarantee is claimed.
- No independent high-precision trajectory integration was performed.
- Strict +/-0.9999 NUM failures remain; no solver or threshold was changed.
- Current observations support +/-0.999 as a data-range candidate for this
  setup, particularly for status stability, not as a newly certified upper
  limit. Detector-hit maxima are not uniformly monotone with tighter tolerance.
- The previous stable-Delta work has known unresolved regressions: two historical
  bitwise initial-State tests (129/131 passed), and one ray's old-version
  192x192 replay hits exceeding its unchanged comparison tolerance. They were
  documented in STABLE_DELTA_VALIDATION.md; they were not repaired, relaxed,
  hidden or rerun in this validation-only task. New-version +/-0.999 replay
  passes as reported above.
- The focused Rust helper built and ran successfully. Its rustfmt check passed.
  An optional targeted release Clippy attempt could not start because Cargo's
  package fingerprint filesystem scan returned Windows access denied (os error
  5); this is not reported as a lint PASS. No full regression suite was rerun.

## Evidence and repeatability

Production source hash verification: chi0999_comparison_20260926/source_hash_verification.json
(62 source files checked, 0 changed). New validation-only helpers:
examples/near_extremal_trace.rs, validation/analyze_chi0999.py,
validation/analyze_focused_traces.py.

New evidence: validation/chi0999_comparison_20260926/ contains per-run standard
archives, measurement/diagnostic JSON, reproduction logs, analysis.json,
focused_traces.json and focused_precision.json. Reused full-run evidence:
validation/stable_delta_20260926/. Helpers use existing simulation APIs, not
alternate physics, archived trajectories as input or altered success criteria.

## Full terminal state records

Each array uses the existing State order:
(t, r, phi, p_t, p_r, p_phi, theta, p_theta). Full preceding sample records and
metric/RHS evaluations are in focused_traces.json and focused_precision.json.

### +0.9999 B, ray 15819

Stop: NullViolation; previous 80-digit C_null: 9.736900561e-06.

```text
[363.6692067213763, 1.0151677602190166, 111.90217532177743, -0.9874411693062543, -290259.7071420387, -6.724296177158009, 1.7565786247663397, 0.12682841411091922]
```

### +0.9999 C, ray 15819

Stop: NullViolation; previous 80-digit C_null: -9.743030992e-06.

```text
[348.2316098807056, 1.0154250504355722, 104.30064047147516, -0.9874411693062543, -230057.24270587225, -6.724296177158009, 1.756574882551887, 0.12708254639950387]
```

### +0.9999 C, ray 15963

Stop: NullViolation; previous 80-digit C_null: -9.876729233e-06.

```text
[330.0774945606263, 1.0154761491327755, 105.25326584648722, -0.9874361620262291, -211154.5960155638, -6.340368135765219, 1.5059467455554216, 0.02954183006859517]
```

### +0.9999 C, ray 15990

Stop: NullViolation; previous 80-digit C_null: -9.924142859e-06.

```text
[329.037572480921, 1.0154878316452576, 105.028164386304, -0.987435777431995, -207920.50913312024, -6.287738150532856, 1.4906909655513683, -0.008593719216943404]
```

### +0.9999 C, ray 15998

Stop: NullViolation; previous 80-digit C_null: -9.921011042e-06.

```text
[332.5318461629369, 1.0154238305442886, 106.65008188675861, -0.9874359355777748, -218447.23391543853, -6.2755238781630736, 1.6856826062122738, -0.010647036303225067]
```

### +0.9999 C, ray 16048

Stop: NullViolation; previous 80-digit C_null: -9.957773319e-06.

```text
[324.9192955065397, 1.0155760355665329, 102.85323189913451, -0.9874361205690775, -190793.62538461116, -6.127250071757077, 1.798296856991502, -0.05170847275252551]
```

### -0.9999 C, ray 20858

Stop: NullViolation; previous 80-digit C_null: -9.888776340e-06.

```text
[343.92954983724144, 1.0152345278493171, -112.00923706215266, -0.9874363323490942, -256627.1645264868, 6.233684053264486, 1.3908277327707546, 0.11065895991346145]
```

### -0.9999 C, ray 20873

Stop: NullViolation; previous 80-digit C_null: -9.798000186e-06.

```text
[341.9126630657195, 1.015266925027705, -111.01551718129873, -0.9874363076605577, -250013.28999862124, 6.268368187799368, 1.416017075762926, 0.09392650958617388]
```

### -0.9999 C, ray 20985

Stop: NullViolation; previous 80-digit C_null: -9.859832045e-06.

```text
[347.89433432961494, 1.0152325870700054, -111.4854918344919, -0.9874387395999473, -267702.76210000506, 6.573427584093429, 1.492763261509344, 0.5858241843557108]
```
