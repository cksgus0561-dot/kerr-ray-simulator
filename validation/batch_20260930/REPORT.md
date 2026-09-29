# Batch generation validation — 2026-09-30

Implemented and verified only in the existing project workspace. The separate
migration workspace was not read, copied, built, edited or otherwise used. No physical or RNG
settings changed. `before_hashes.json` and `audit_results.json` confirm that all
previous physics, detector, generator, simulation/parallelism, reproduction,
configuration and test files retain their original SHA-256 values.

## Source changes

New code: `src/experiments/chi_sampling.rs`, `src/standard_run/batch.rs`,
`src/standard_run/batch_cli.rs`, `tests/batch.rs`.
Existing edits: `src/experiments/mod.rs`, `src/standard_run/mod.rs`, `src/main.rs`
(module/CLI registration); `src/standard_run/common.rs` (optional batch metadata);
`src/standard_run/seeded.rs` (share the unchanged BIN encoding with the batch writer,
validate batch file number/chi on load). Documentation: README.md,
BATCH_SIMULATION.md, this report and the audit script/logs.

## Chi sampling

- Actual full grid count: **153**; first=-0.999, center=+0.0, last=+0.999.
- Strictly increasing, all entries inside the required interval.
- Nonzero pairs are exact f64 sign reversals; sign symmetry error **0**.
- Boundary error relative to the requested f64 endpoint constants: **0**.
- Formula check for all 153 entries: absolute difference <= 2*f64::EPSILON
  (4.440892098500626e-16). Endpoints are assigned rather than round-tripped.
- Center positive gap: approximately 0.04996101528870298.
- Last positive gap: approximately 0.00010511865232343265.
- Positive gaps strictly decrease toward the endpoint.

`chi_grid.log` is a full dry-run listing; no simulations or seeds were produced
by that dry run. `simulations_per_chi` has no default. The code accepts explicit
N and a validated subset for verification; the full 153*N execution order is
unit-tested without calculating 153 simulations.

## Actual standard-size batch

Command (release executable):

```powershell
.\target\release\kerr-ray.exe batch-simulate --simulations-per-chi 1 --chi-indices '-76,0,76' --output validation/batch_20260930/runs
```

Exactly **3 newly generated standard simulations**, each 192x192=36,864 rays,
96M x 96M, 0.5M cells. No config override or new tolerance was used.
OS-generated seeds were distinct; the existing ChaCha20 version and fixed
direction were used. Counts are from the log and independently decoded BIN bytes.

| File | chi | ACT | DET | CAP | ESC | NUM | Calculation/association seconds | BIN bytes |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| sim_1.bin | -0.999 | 0 | 36280 | 305 | 279 | 0 | 3.553086800 | 907656 |
| sim_2.bin | 0 | 0 | 36267 | 343 | 254 | 0 | 3.913379100 | 907344 |
| sim_3.bin | +0.999 | 0 | 36265 | 310 | 289 | 0 | 3.877701600 | 907296 |

The different signs use different seeds, so these counts are **not** a signed
symmetry experiment. This task establishes batch execution/reproduction, not new
near-extremal solver accuracy claims.

Output: `runs/simulation_1/common.json` and `sim_1.bin`..`sim_3.bin`.
The four files total **2,725,473 bytes** (~2.60 MiB) and are retained as compact
verification evidence. No trajectory arrays or large image exports were saved.
`audit_results.json` records each seed, initial hash, BIN SHA-256 and finite-hit
count. All DET hits are finite, within the detector and at/after emission time.

## Reproduction and compatibility

All **3** saved simulations were independently recalculated using the existing
`reproduce --input ... --sim sim_N.bin` command, without generating new seeds.

For each: initial hash PASS; canonical/order mismatch 0; reconstructed initial
state bit mismatch 0; status mismatch 0; count mismatch 0; hit-presence mismatch 0;
hit maximum absolute differences [0,0,0]; hit bit mismatches [0,0,0]; same source
and version true; **reproduction PASS**. Calculation times were respectively
3.552351200, 3.664327700 and 3.592309700 seconds. Full evidence is in
`reproduce_1.log`..`reproduce_3.log`.

The standard binary layout is unchanged, and still has no ray IDs or raw ray
inputs. `common.json` adds optional `batch.chi_sampling`,
`batch.simulations_per_chi`, and `batch.chi_indices`. Single-run saves omit `batch`.
New tests compare batch bytes with a fresh existing single-run calculation using
the same seed: exact BIN equality. Existing single CLI/GUI worker save/load/replay
tests pass, including old-format loaders. No full-size fourth single-run simulation
was added merely to repeat the standard calculations.

## Build and tests

- cargo check --offline --locked: **PASS**.
- cargo clippy --offline --locked --all-targets -- -D warnings: **PASS**.
- cargo fmt --check: **PASS** (`fmt_check.log`).
- cargo build --release --offline --locked --bin kerr-ray: **PASS**.
- New batch tests: **9/9 PASS**, debug and release.
- Full debug suite: **138 PASS / 2 FAIL / 140 total**.
- Full release suite: **138 PASS / 2 FAIL / 140 total**.

Both failures are the previously documented historical initial-state bitwise
regressions, at unchanged `tests/support/ray_regression.rs:98`:

1. `regression_256_all_initial_states_match_legacy_and_saved_baseline`
2. `regression_4096_all_initial_states_match_legacy_and_saved_baseline`

They match the pre-task record in `validation/STABLE_DELTA_VALIDATION.md`.
The prior 131 tests therefore retain 129 passing/2 failing; all 9 added tests pass.
No existing test was deleted, relaxed or marked ignored. The old archive replay
discrepancy remains an existing unresolved issue and was not recalculated here.
The existing test suites also perform small helper simulations; these are distinct
from the 3 standard-size batch runs and their 3 reproduction calculations above.

## Limits of this verification

The full production dataset was deliberately **not generated**. The 3 standard
runs plus tiny test fixtures do not establish accuracy for every seed or spin.
No GUI redesign, D: migration, GPU computation or ML work was performed.
Failures/interruptions leave already completed files in the reserved batch folder;
automatic resume and crash recovery are not implemented or claimed. Existing
numerical-failure classifications are retained unchanged.
