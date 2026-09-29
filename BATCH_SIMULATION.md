# Automatic seeded simulation batches

## Run from the existing C: project

```powershell
. .\env.ps1
cargo fetch --locked # First setup: download the pinned dependencies from crates.io.
cargo run --release --offline -- batch-simulate --help
# Inspect the full 153-point plan only: no seeds, rays or output files are created.
cargo run --release --offline -- batch-simulate --simulations-per-chi 1 --dry-run
# Small validation: 3 simulations, each with the unchanged standard 36,864 rays.
cargo run --release --offline -- batch-simulate --simulations-per-chi 1 --chi-indices -76,0,76 --output results/batch_check
# Recalculate one selected saved simulation from its seed/settings.
cargo run --release --offline -- reproduce --input results/batch_check/simulation_1 --sim sim_2.bin
```

`--simulations-per-chi N` is required: there is **no production default**.
The `1` above is an explicit example, not a selected dataset size.
Omitting `--chi-indices` selects the full grid; omitting `--dry-run` performs
real calculations. The implementation validation does not generate that full dataset.
`--config` optionally reads the existing `SessionConfig` JSON, without new GUI
or physics settings. `--output` defaults to `results/standard`.

## Sampling and execution

`src/experiments/chi_sampling.rs`: for integer i=-76..76,
`s_i = i * atanh(0.999) / 76`, `chi_i = tanh(s_i)`.
The 153 values are strictly increasing. Endpoints are assigned exactly to the
f64 representations of -0.999 and +0.999; the center is +0.0. Negative entries
are sign reversals of the positive magnitudes, guaranteeing exact sign symmetry.
This is a sampling rule, not a change to the Kerr equations or allowed solver range.

`src/standard_run/batch.rs`: `BatchSpecification::full(N)` and `execute` provide
the code interface. `chi_indices` can select a strictly increasing, duplicate-free
subset of the fixed grid. File order is ascending chi, then repetition 1..N.

Simulations run sequentially; each reuses `master_seed()` (32 bytes from the OS),
`seeded::execute_with_outcomes`, and the existing ray-level CPU parallel executor.
Seeds are not derived from chi, repetition or file number. The versioned ChaCha20
generator and canonical ordering are unchanged. Each simulation releases its
trajectories before the next calculation; no trajectory export is added.
The CLI reports all five status counts, seed, initial hash, time and BIN size.
NUM rays remain NUM; errors are not hidden by changing numerical thresholds.

## Archive and reproduction

Each batch atomically reserves a new unused `simulation_N` directory:

```text
simulation_N/
  common.json
  sim_1.bin
  sim_2.bin
  ...
```

One common.json keeps the existing numerics, detector, convention, provenance,
generator and binary specification. The optional `batch` object adds:

- `chi_sampling`: chi_min=-0.999, chi_max=0.999, transform/formula,
  positive_interval_count=76, total_chi_count=153.
- `simulations_per_chi`: explicitly supplied N.
- `chi_indices`: actual selected indices, including a validation subset if used.

BIN layout is unchanged: little-endian f64 chi, 32-byte seed, 32-byte initial hash,
N ray status bytes, then three f64 hit values for each DET ray in canonical order.
No ray IDs, raw initial-condition arrays, pixels or trajectory samples are added.
The single-run and batch writers use the same binary encoder.

Loading/reproduction uses the existing seed regeneration, SHA-256, canonical
ordering, Kerr integration and comparison policy. Batch file numbering/spin is
checked against its plan. Existing single-run archives omit `batch` and still load.
The current loader understands both old and new common files; older executables
with `deny_unknown_fields` cannot read the new batch metadata.

Directories and BIN files use create-new semantics: existing results are never
overwritten. An execution/I/O failure stops the batch and preserves its reserved
folder and already saved simulations. The plan is not proof of completion: check
the completion message and numbered files. An interrupted/truncated BIN is
rejected by the existing length/hash checks. Automatic resume is not implemented.

## Verification scope

See `validation/batch_20260930/REPORT.md` for actual test results, the three
standard-size runs, reproduction comparisons and hashes. The known historical
bitwise/archive regression failures following the stable Delta change remain
separate; no test tolerance or expected physical result is changed for this feature.
