# Ray-level CPU execution

The independent-ray entry point `simulation::rays::calculate_rays` now prepares
the existing canonical inputs once, then schedules whole rays on CPU threads.
It still calls `CpuReferenceIntegrator::integrate_with_detector` without any
changes to its equations, tolerances, steps, detector intersection or statuses.

## Execution policy

`simulation::ray_execution::ordered_map` uses scoped Rust standard-library
threads and channels; no new dependency or global mutable integrator is needed.
The default worker count is `max(available_parallelism - 1, 1)`, capped by the
actual input length. This leaves one logical CPU available to the coordinator,
UI and OS, without promising an OS CPU-affinity reservation. An explicit positive
`KERR_RAY_THREADS` value overrides the count for each calculation; 1 selects the
serial path. It is not a physical setting and is not saved in common.json.

```powershell
$env:KERR_RAY_THREADS = '4'
.\target\release\kerr-ray.exe simulate-save --output results/standard
Remove-Item Env:KERR_RAY_THREADS  # restore automatic selection
```

Rust callers can choose `RayExecution::Serial` or `Parallel { threads }` through
`calculate_rays_with_execution`. Serial and parallel paths share the same
per-ray call. Each calculation creates and joins its bounded worker set; the
existing long-lived GUI worker remains the coordinator. Several independent
simulations run simultaneously can each create their own pool; no global thread
budget/affinity manager has been added.

## Ordering and cancellation

- `prepare_rays` and the generator are unchanged. Output index means the
  **canonical prepared index**, not the unsorted caller Vec's index.
- Each job carries that index and its own PreparedRay. Completion writes to
  that result slot. Arrival order never becomes file or detector result order.
- The coordinator gives each worker at most one ray at a time and assigns the
  next ray when that worker completes. No worker independently prefetches jobs.
- Kerr/config/detector references are read-only; each integration owns all its
  mutable state. Communication happens per ray, never per integration step.
- `keep_going` remains FnMut on the calling/coordinator thread and need not be
  Send or Sync. It receives completed count, initially 0 and finally N; polling
  can repeat a count while rays are running. Counts are monotonic.
- Cancellation is checked before the first dispatch, after every completion,
  and on a 10 ms receive timeout. It stops new dispatch and drops partial output.
  At most the already dispatched worker-count rays remain in flight.
- In-flight integrators finish normally before scoped threads join. Their steps
  are not interrupted or modified; cancellation latency can include the slowest
  remaining ray. OS scheduling may add latency to the 10 ms polling interval.
- A worker panic/spawn error aborts the batch; it is not fabricated into a
  NumericalFailure or a successful partial run. Existing generation-ID checks
  still prevent stale results from replacing newer GUI input or auto-saving
  a cancellation already observed before save.

## Arbitrary ray counts and growing the launch region

There are no 128/16384/4096-specific limits in the scheduler. It uses the length
of the prepared Vec for slots, progress and dispatch. Odd/prime-sized inputs,
duplicates and counts smaller than the requested thread count are tested.

To expand the stratified region, change `GeneratorSpec.cell_count` (code or
examples/seeded.json, or existing GUI controls). For example [192,192] with the
unchanged [0.5,0.5] M cell_size covers 96x96 M and produces 36864 rays. No solver
or worker changes are required. The default is [192,192] / 36864 rays.
Existing generator, GUI and physical domain validation limits remain in force.

## Storage and compatibility

Seed generation, PRNG consumption, canonical ordering, initial_conditions_hash,
common.json, binary layout and DET packing were not changed. The standard
calculate/save and reproduce paths already call calculate_rays, so they acquire
parallel execution without a separate serialization path. Runtime worker counts
are not added to saved physical configuration. No GPU/ML/noise features changed.

Legacy experiment/export functions that directly call the reference integrator
(rather than the independent-ray entry point) remain serial. This includes
`simulation::calculate` for historical SourcePlane-only viewer loads and the
older source_to_detector exporter. Standard seeded GUI runs and standard
archive calculation/reproduction use the new parallel path.

## Verification and measurement

`tests/ray_parallel.rs` compares exact initial State, full trajectory sample
bits/affine values, diagnostics, status/stop reason and all hit data for serial
versus several thread counts. Timing diagnostics are intentionally excluded.
It also exercises non-grid positions/directions/t_emit, canonical input
permutations, duplicates, cancellation, explicit NumericalFailure and unchanged
seed/binary reproduction. Internal scheduler tests cover reverse completion
order, bounded cancellation, non-Send callbacks and worker panic cleanup.

The example `ray_parallel_benchmark` runs sequential child processes with the
same release executable, seed and common physics settings, using threads=1 and
the default count. Physics wall time is bounded by first/final progress callbacks
(includes worker creation and scheduling, excludes input preparation). Total
calculate/save time includes generator/preparation/hash/association/validation
and saving. Per-ray trajectory/diagnostic fingerprints are computed afterward
and excluded from both timings. Entire binary results are also compared against
the untouched pre-parallel 16384-ray archive. A larger temporary 192x192 run uses
the same path. Benchmark output is validation evidence, not a new raw format.

```powershell
cargo run --release --example ray_parallel_benchmark -- validation/parallel_measurements_20260926
```

Use a new output directory; it will not overwrite prior measurements. Actual
times and validation results are recorded in BENCHMARK.md and validation logs.
