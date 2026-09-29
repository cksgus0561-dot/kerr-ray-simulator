# Source-independent 3D ray initialization

## Public interface

`simulation::rays::RayInitialCondition` contains exactly seven `f64` fields:
`position_x`, `position_y`, `position_z`, `direction_x`, `direction_y`,
`direction_z`, `t_emit`. It has no ID, energy, source plane, grid, or file loader.

The origin is Kerr's center, +Z is its spin axis, and the coordinates are
right-handed with G=c=M=1. Positions and directions use the existing oblate
Cartesian-like BL convention, not Kerr-Schild coordinates and not BL derivative
components. `t_emit` is the individual ray's BL coordinate emission time.

```rust,ignore
prepare_rays(Vec<RayInitialCondition>, &RaySimulationConfig)
    -> Result<Vec<PreparedRay>, String>
calculate_rays(Vec<RayInitialCondition>, &RaySimulationConfig, keep_going)
    -> Result<Vec<RayOutcome>, String>
```

`RaySimulationConfig` contains only `spin`, the existing `IntegratorConfig`, and
the existing `DetectorPlane`. A caller can build it without constructing a
`SourcePlane` or `SourceDetectorExperiment`. `calculate_rays` calls preparation
itself; callers cannot inject a `PreparedRay` or a historical ID into execution.
The callback receives completed/total counts and can cancel between rays.

An executable, editable example is `examples/arbitrary_rays.rs`:

```powershell
. .\env.ps1
cargo run --release --example arbitrary_rays
```

It uses three positions with different X coordinates, distinct global directions
and emission times. It writes a console summary only; it does not
create any result or initial-condition file.

## Canonical identity

1. Reject an empty input list and any nonfinite value in any of the seven fields.
2. Reject zero directions. Normalize finite nonzero directions, scaling by their
   largest absolute component first to avoid norm overflow/underflow.
3. Canonicalize signed zeros in all seven fields to +0.
4. Sort lexicographically by position X/Y/Z, normalized direction X/Y/Z, then
   emission time, using `f64::total_cmp` without strings or fuzzy comparisons.
5. Enumerate the sorted inputs as IDs 0..N-1, then generate their initial States.

Duplicates are retained. Identical canonical inputs receive consecutive IDs
and identical initial States; individual duplicate copies have no physical
identity distinguishing them. No original index is encoded in the ID or used
as a tie-breaker. ID and initial State fields in `PreparedRay` are private and
readable through accessors. The same prepared record accompanies its entire
`PhysicsResult`, including trajectory samples, status and intersection.

This guarantees ordering for the same finite input set, irrespective of Vec
order, in the same numerical environment. It does not claim cross-platform
bitwise identity of transcendental functions or the solver. Old source-grid
IDs are intentionally not preserved; comparisons must match physical inputs.

## Unchanged physical pipeline

The new path calls the existing `from_cartesian`, `cartesian_to_local`, and
`tetrad::ray_3d` functions, with local ZAMO energy fixed at 1.0. It then invokes
the unchanged `CpuReferenceIntegrator::integrate_with_detector`.

No metric, coordinate map, tetrad, ODE, detector intersection, null check or
outcome classification is altered. Initial rays must be outside the capture
cutoff and inside escape radius, in the existing valid BL chart. They are not
subject to the legacy *stationary source plane* r>2 restriction; the existing
ZAMO construction permits valid exterior ergoregion initial points.

`RayOutcome::detection()` returns a borrowed in-memory view carrying the same ID,
`t_emit`, `delta_t`, and the existing `Intersection` (continuous time and hit UV).
It does not create a new detector model or save format. No fake `u_source` or
`v_source` is invented for independent rays.

## Remaining legacy source-plane code

The existing `SourceDetectorExperiment`, `simulation::calculate`, GUI worker,
visualization data, legacy `HitEvent` with source UV, and saved-result reader are
preserved. The current GUI still runs that legacy plane experiment. It is not
an arbitrary-ray editor. Its existing behavior and file schema were deliberately
not migrated in this initialization-only change.

`experiments::independent_rays::from_source_plane` is an optional adapter for
legacy CartesianSpatial experiments at local energy 1. It emits seven-scalar
physical inputs only and does not pass IDs. Per-emission direction/time overrides
are respected. LocalZamo source presets and nonunit-energy legacy experiments
remain on their unchanged legacy path; the adapter reports an error for them.
The new simulation module and the arbitrary-ray example do not use this adapter.

## Validation

`tests/ray_initial_conditions.rs` covers arbitrary positions/directions/times,
legacy conversion agreement (relative/absolute scale 2e-13), finite validation,
normalization, duplicates, signed zeros, permutations, canonical key order,
identity through trajectories/status/detections, cancellation and initial domain.

The 256/4096 initial-state regressions freshly generate the established source
presets in code and convert them through the optional adapter. All eight State
components are compared bit-for-bit against the old initializer and existing
diagnostic CSVs. They read the project's existing results/source_to_detector and
results/visualization_4096 CSVs, not the large trajectory archive. Missing baseline
CSVs cause failure rather than silent skipping. These two suite tests audit saved
outcome counts; they do not reintegrate thousands of rays on each debug/release run.

`examples/initial_condition_regression.rs` performs the separate full execution
comparison once: fresh canonical input -> unchanged solver -> corresponding saved
status and u_hit/v_hit/t_hit. Inputs are generated in code, never reconstructed from
old States or historical IDs. The old IDs are used only to join the old diagnostic
and detector CSVs as a comparison oracle. Initial State tolerance is zero (bits);
each hit-coordinate/time absolute tolerance is 1e-9.

### Executed verification (2026-09-16)

- `cargo fmt`: exit 0.
- `cargo clippy --all-targets -- -D warnings`: exit 0.
- `cargo test`: 72 passed, 0 failed (existing 59 + new 13).
- `cargo test --release`: 72 passed, 0 failed.
- `cargo run --release --example initial_condition_regression`: exit 0.
- `cargo run --release --example arbitrary_rays`: exit 0; one Detected and two
  Captured, with input-derived canonical IDs and individual emission times.
- SHA-256 audit of all 32 existing files in physics, detector, compute,
  visualization and output: no files changed.

| Fresh calculation | Detected | Captured | Escaped | Failure | Maximum initial State error | Maximum hit (u,v,t) error |
|---|---:|---:|---:|---:|---|---|
| 256 rays | 216 | 20 | 20 | 0 | 0, all f64 bits match | (0,0,0) |
| 4096 rays | 3502 | 330 | 264 | 0 | 0, all f64 bits match | (0,0,0) |

Each large preset was integrated only once through the new API for this full
comparison, outside the repeated debug/release suite. Recorded execution times
including assertions were 0.244058 s and 3.542344 s; these are regression-run
observations, not a repeated performance benchmark. Existing result directories
were read-only comparison data. Logs are in `validation/initial_conditions/`:
`debug_tests.txt`, `release_tests.txt`, `full_regression.txt`, `arbitrary_rays.txt`,
and `protected_code_audit.json`.

The toolchain emitted its pre-existing `could not canonicalize path
<user-profile-directory>` environment warning; all commands above returned success.

## Future input persistence boundary

A later storage/reproduction layer can supply the seven-field list and complete
`RaySimulationConfig` to `prepare_rays` / `calculate_rays`. It must not supply IDs,
old States, trajectories, or old detector outputs as solver input. No initial
condition persistence, replay, random noise, GPU solver or ML is implemented here.
