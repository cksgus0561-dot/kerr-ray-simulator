# CPU ray parallelism validation — 2026-09-26

## Changed implementation

- src/simulation/ray_execution.rs: scoped standard-library worker set, indexed
  result collection, bounded dispatch, coordinator callback and error/cancel cleanup.
- src/simulation/rays.rs: existing calculate_rays delegates to execution policy;
  explicit calculate_rays_with_execution permits serial/reference comparisons.
- src/simulation/mod.rs: exports scheduling module.
- tests/ray_parallel.rs: five physics/order/cancellation/archive integration tests.
- Internal ray_execution tests: four scheduling/order/cancellation/panic checks.
- examples/ray_parallel_benchmark.rs: sequential same-seed release benchmark.
- CPU_PARALLEL.md, README, SEEDED_RUN and BENCHMARK: usage and measured evidence.
- bin/kerr-viewer.exe: updated from the verified release build; matching SHA-256
  5399C07BC30F4CFB95DE1DE4C975C0E0C62B8C57AB5DFC305D746317232F236C.

No physics, integrator, detector intersection, generator, hash, binary format,
thread-count metadata, GUI controls, dependency or tolerance changes. The existing
standard save/reproduce paths acquire parallelism through their unchanged calls
to calculate_rays. Legacy direct-integrator exporters remain serial.

## Executed validation

- cargo fmt; final cargo fmt --check: PASS.
- cargo clippy --all-targets -- -D warnings: PASS.
- cargo test: **128 passed, 0 failed**.
- cargo test --release: **128 passed, 0 failed**.
- cargo build --release --bins --example ray_parallel_benchmark: PASS.
- Cargo used the existing offline vendor source. No dependencies were added.
- Previous 119 tests preserved; 9 new checks added. No old assertions weakened.

New physics tests use arbitrary lengths 1,2,17,63 and thread counts 1,2,4,8;
non-coplanar positions, differing directions/t_emit; reversed inputs; exact
duplicates; deliberately failing numerical initial conditions; pre-dispatch and
mid-flight cancellation; zero-thread rejection; binary save/load/reproduce.
They directly compare f64 bits of all sample states and affine values, initial
States, hit coordinates/time/intersection state and root metadata. Diagnostics,
status, stop reason and failure strings agree; elapsed timing is excluded.

Scheduling tests force out-of-order completion, bound cancelled dispatch to the
in-flight worker count, use a non-Send callback to cancel while all workers wait
at a barrier, and verify an intentional worker panic returns an error without
deadlock. Existing latest-generation worker and autosave tests also pass.

## Full-size comparison and scaling

See parallel_measurements_20260926/summary.json and BENCHMARK.md for timings.
Same release executable, same input seed/settings: 16384 serial and parallel
results matched each ray's full-physics fingerprint. Entire sim_1.bin files
matched the saved **pre-change** 16384 binary, covering hash, canonical order,
every status and packed u_hit/v_hit/t_hit bit. Counts D15786/C330/E268/NUM0.

A 192x192 (36864) temporary generator setting used the same scheduler unchanged:
D36263/C330/E271/NUM0. The default cell count was not changed. Input length, not
any benchmark-specific count, determines scheduling and output allocation.

CLI reproduce against the pre-change archive: **PASS**, SHA-256 initial hash
matched, zero status/initial-state/canonical mismatches and zero hit bit errors.
Source fingerprint is different because the implementation changed; this does
not suppress or relax any reproduction comparison.

## Limits / not tested

- One timing sample per mode on this machine; no cross-OS/CPU test or confidence interval.
- Peak RAM and GUI responsiveness during large calculations were not measured.
  Complete trajectories remain in memory as before; larger inputs need more RAM.
- No GUI button interaction was performed for this scheduling-only change.
- Cancellation joins already dispatched per-ray solves; it cannot interrupt an
  adaptive integration step or return before the slowest in-flight ray finishes.
- No global CPU budget across multiple simultaneous application instances;
  KERR_RAY_THREADS can limit each calculation. CPU affinity is not pinned.
- The measured larger count is 36864, not 65536 or the generator's maximum.
