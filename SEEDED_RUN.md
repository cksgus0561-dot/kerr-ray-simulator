# Seeded standard runs — format 2-seeded-bin

CPU scheduling update: the independent-ray calculation now supports bounded
ray-level CPU parallelism; see [CPU_PARALLEL.md](CPU_PARALLEL.md). The generator,
seed/hash rules and raw format documented below are unchanged.

## Scope and defaults

The default research generator now emits 192 × 192 = 36,864 independent
`RayInitialCondition` values. The old research preset covered 32 × 32 M with
64 × 64 cell centers (spacing 0.5 M on both axes). The new rectangle covers
96 × 96 M, keeping the same 0.5 × 0.5 M cells and one ray per cell.
Position is randomized **inside each cell**, not direction. Thus these new
randomized runs do not have the old preset's fixed outcome counts.

Default center = (80,0,0); axis 1 = +y; axis 2 = +z; fixed direction = (-1,0,0);
t_emit = 0; local ZAMO energy remains 1. Coordinate lengths are BL oblate
Cartesian-like coordinate lengths in M, not proper distances.

`experiments::stratified::GeneratorSpec` owns generation. It knows neither
SourcePlane nor chi. `SessionConfig.generator = Some(spec)` selects this path.
SourcePlane is derived only as the viewer's presentation rectangle. `None`
selects the existing legacy adapter, including unchanged 256/4096 regression
inputs. Historical JSON without a generator is intentionally interpreted as legacy.

## Deterministic generator v1

- A new normal calculation calls `getrandom::fill(&mut [u8;32])`. It uses the
  OS random source directly (Windows ProcessPrng); no time, chi or numbering
  participates, and no seed bits are truncated. OS failure aborts the request.
- `rand_chacha = 0.9.0`, `ChaCha20Rng::from_seed`: 256-bit key, initial 64-bit
  stream identifier and block counter both zero. `next_u64` consumes the low
  u32 first. Dependencies are pinned, vendored and checksummed.
- For every cell consume exactly two u64 words, axis 1 then axis 2:
  `u = (word >> 11) as f64 * 2^-53`. No modulo, additional calls or noise.
- Traverse axis 2 (`j`) outermost, axis 1 (`i`) innermost.
- Fixed binary64 operations, no fused multiply-add or fast-math:
  `s1 = (i - n1*0.5)*d1 + u1*d1`, similarly for s2;
  `p[k] = center[k] + (axis1[k]*s1 + axis2[k]*s2)`.
- Basis vectors are validated orthonormal to 1e-12, never silently rebuilt.
  Existing direction normalization, signed-zero canonicalization, seven-field
  `total_cmp` ordering and CartesianSpatial → ZAMO → Kerr conversion are reused.
- No RNG enters `calculate_rays` or the integrator. Identical duplicates, if any,
  remain distinct entries under the existing canonical rule.

These operations and consumption rules belong to `stratified-cartesian-v1`.
Changing them requires a new generator version and a retained old decoder.
Binary64 rounding at representational boundaries is not a physical perturbation;
uniformity means equal probability over the 2^53 discrete uniform values.
Cross-platform reproducibility is protected by fixed-word known-answer vectors
and an initial-condition fingerprint. Only the actual tested platform(s) are
claimed in the validation report; geodesic hit comparison keeps its tolerances.

## Single-run files

Each GUI Calculate or default CLI simulate-save reserves a fresh, unused
`simulation_N` by atomic directory creation and writes exactly:

```
common.json
sim_1.bin
```

There is no additional format.json. Failed writes may leave an incomplete reserved
directory; future runs never overwrite it. Load/Reproduce do not save a new run.
The loader accepts numbered `sim_N.bin` files with numeric sorting (1,2,3,10),
for future batch archives. No batch calculation was introduced here.

### common.json

Preserves `numerics` (rtol, atol, initial/min/max step, max_steps, max_affine,
horizon_epsilon, escape_radius, null_tolerance, detector_root_tolerance), the
actual detector Plane basis/center/size, convention, simulator_version and
build_source_fnv1a64. Adds `ray_generator` and `binary_format` objects.

`ray_generator` records version, RNG algorithm, 256 seed bits, uniform rule,
cell order, center, axis_1/axis_2, cell_size/cell_count, fixed_direction,
constant t_emit, one-point-per-cell rule and direction_noise="none".
`binary_format` fixes all types, layout, endian, status mapping, hit column
meanings, SHA-256 and the exact hashed sequence. Unknown specifications fail
explicitly. No detector resolution or display/postprocessing state is included.

### sim_1.bin — exact bytes, no padding

| Offset | Data | Bytes |
|---|---|---:|
| 0 | chi, IEEE-754 float64 little-endian | 8 |
| 8 | seed, original uint8[32] | 32 |
| 40 | initial_conditions_hash, SHA-256 digest uint8[32] | 32 |
| 72 | status[N], uint8 in canonical order | N |
| 72+N | detected_hits[K][3], float64 little-endian: u_hit,v_hit,t_hit | 24K |

N = cell_count[0] × cell_count[1]; K = number of DET codes.
Codes: 0 ACT, 1 DET, 2 CAP, 3 ESC, 4 NUM.
Consume one hit triplet whenever the status traversal encounters DET.
Other rays occupy **no hit bytes**. Total length = **72 + N + 24K**.
Length errors, invalid status codes, invalid hits and unsupported specs fail.

No ray_id, per-ray input, energy column, trajectory, internal State, conservation
diagnostic, image, pixel resolution, camera, grid or playback data is written.
The loader reconstructs temporary in-memory rows for the existing comparison API;
these rows are not part of the file schema.

## Hash and fresh reproduction

Generate inputs from seed/spec → existing prepare_rays canonical ordering →
SHA-256 of each canonical ray's seven f64 fields, each as eight little-endian
IEEE-754 bytes: position_x/y/z, normalized direction_x/y/z, t_emit.
No IDs, count fields, text conversion or separators enter the hash.

Loading validates this fingerprint. Reproduce regenerates from seed/spec and
checks it **again before integration**, then calls the existing solver and
compares status and continuous hits. Cached row inputs and stored results cannot
seed the solver. A seed bit flip or changed geometry produces an explicit initial
conditions hash mismatch, without entering the geodesic calculation.

Existing comparison remains `abs_error <= 1e-9 + 1e-12 * max(abs(old),abs(new))`,
with bit differences, count/status differences and initial-State differences
reported as before. No saved IDs are checked in binary format because none exist.
Initial hash is not a checksum of detector results; changed results are detected
by fresh reproduction.

## Commands and GUI

From the project directory after `. .\env.ps1`:

```powershell
cargo run --release -- simulate-save --output results/standard
cargo run --release -- simulate-save --config examples/seeded.json --output results/standard
cargo run --release -- inspect-simulation --input results/standard/simulation_1
cargo run --release -- reproduce --input results/standard/simulation_1 --sim sim_1.bin
cargo run --release --bin kerr-viewer
```

The bundled launcher can retain its old stored-run arguments. To select the new
generator explicitly with that launcher, use
`./Open-Viewer.ps1 -Config examples/seeded.json` (or `bin/kerr-viewer.exe --config examples/seeded.json`).
Its executable has been updated; its saved startup argument file is preserved.

`--grid N` changes cell count while preserving cell size in the seeded path.
GUI: Simulation → Stratified ray generator edits the shared center, basis, cell
counts/sizes, fixed direction and t_emit. Calculate uses the existing worker and
automatic save. Standard simulation archive → Load Folder → sim selection →
Reproduce uses the same binary/legacy dispatcher. Load alone gives a summary;
only Reproduce computes new trajectories. Camera/display edits do not change the
generator physics key; Free-Fall Grid code and behavior are unchanged.

Legacy `--preset regression` (16²) and historical JSON preserve centered inputs
and the existing Parquet writer/reader for compatibility. For the old 4096 case,
use `--preset regression --grid 64`. New default writes are binary. Explicit
arbitrary-input Rust `standard_run::execute/save_new` still supports Parquet since
arbitrary inputs are not in general regenerable by this stratified generator.
Old Parquet energy/status/ID variants remain readable and reproducible.

## Where to modify

- `src/experiments/stratified.rs`: generator specification and versioned sampling.
- `src/session.rs`: shared settings and legacy/presentation adapter.
- `src/standard_run/seeded.rs`: binary encoding/decoding and hash verification.
- `src/standard_run/common.rs`, `mod.rs`, `reproduce.rs`, `cli.rs`: common data,
  format routing and existing reproduction/comparison integration.
- `src/simulation/worker.rs`: Calculate/autosave/Reproduce worker routing.
- `src/visualization/ui.rs`, `app.rs`, `archive_ui.rs`, `cli.rs`: shared controls.
- `tests/seeded_run.rs`: generator, binary, hash, CLI and worker checks.
- `tests/visualization.rs`: requested default-count assertion (other assertions unchanged).
- `Cargo.toml`, `Cargo.lock`, added `vendor_viz` crates: pinned RNG/hash dependencies.
- `examples/seeded.json`, README/GUI_ARCHIVE/STANDARD_RUN/BENCHMARK and validation
  records: current configuration, compatibility, measured evidence.
- `bin/kerr-viewer.exe`: rebuilt viewer distributed through the existing launcher.

Physics, detector intersection and Free-Fall Grid implementations are unchanged.
