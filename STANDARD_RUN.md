# Standard simulation archive and actual reproduction

**Legacy Parquet format reference.** The standard default now uses the seed-based
binary format documented in [SEEDED_RUN.md](SEEDED_RUN.md). This page describes
the retained explicit-input/legacy path; its earlier 4096-ray default examples
are historical. Use `--preset regression --grid 64` for those centered inputs.

The new path writes inputs and one result per ray, then can reconstruct inputs
from files and execute the existing CPU f64 Kerr solver again. It never replays
stored trajectories. Legacy visualization/CSV/PNG/APNG exports are unchanged.

## Commands

From the project root, after `. .\env.ps1`:

```powershell
# New 4096-ray calculation and standard archive (default research generator)
cargo run --release -- simulate-save --output results/standard
# Preserved 256-ray source preset
cargo run --release -- simulate-save --preset regression --output results/standard
# Existing experiment/session JSON can configure the input generator and physics
cargo run --release -- simulate-save --config examples/visualization.json --output results/standard
# Read and validate the files; does not integrate
cargo run --release -- inspect-simulation --input results/standard/simulation_1 --sim sim_1.parquet
# Read inputs/config and actually run null geodesics from scratch
cargo run --release -- reproduce --input results/standard/simulation_1 --sim sim_1.parquet
```

`--grid N` on `simulate-save` sets the source generator's N x N sample count.
`--preset regression` resets to the established 16 x 16 preset; do not combine
it with a custom config when intending to preserve that config.
The CLI generator currently adapts the existing CartesianSpatial source preset
with local energy 1.0. The general Rust entry point accepts arbitrary independent
inputs; it does not require a source plane:

```rust,ignore
let recorded = kerr_ray::standard_run::execute(raw_conditions, &ray_config)?;
let saved = kerr_ray::standard_run::save_new(root, &recorded)?;
drop(recorded);
let loaded = kerr_ray::standard_run::load(&saved.directory, "sim_1.parquet")?;
let comparison = kerr_ray::standard_run::reproduce(&loaded)?;
```

`raw_conditions` is `Vec<RayInitialCondition>`; `ray_config` is the existing
`RaySimulationConfig`. This API is separate from the legacy GUI's `SimulationData`.
The GUI now has a Standard simulation archive panel. New GUI calculations auto-save
through the same API. Folder loading is read-only; Reproduce returns the existing
comparison plus its freshly computed trajectories to the viewer. See GUI_ARCHIVE.md.

## Folder and naming

Each `save_new` reserves the first unused positive integer directory with an
atomic `create_dir`; gaps are reused, existing files/directories are never replaced.
Concurrent writers reserve different directories. No zero padding is used.

```
simulation_1/
  common.json
  sim_1.parquet
simulation_2/
  common.json
  sim_1.parquet
```

`load(directory, "sim_N.parquet")` can select a different positive numbered file
with the same common environment. This stage writes only sim_1 per new directory,
not an automatic simulation series. The loader rejects traversal, padded/zero
numbers, missing/incomplete files, unsupported formats and incompatible conventions.
A failed save may leave its new directory reserved; it cannot overwrite a prior run.

## common.json schema (data_format_version = "1")

- `numerics`: `rtol`, `atol`, `initial_step`, `min_step`, `max_step`, `max_steps`,
  `max_affine`, `horizon_epsilon`, `escape_radius`, `null_tolerance`,
  `detector_root_tolerance` — actual values used by this run.
- `detector`: existing Plane fields `center`, `normal`, `e_u`, `e_v` (three-element
  f64 arrays), `width`, `height`. All basis components are preserved directly;
  no normal/up reconstruction changes rounding.
- `convention`: `G=1`, `c=1`, `M=1`, `origin="Kerr mass center"`, `spin_axis="+z"`,
  `handedness="right-handed"`, and exact `ray_coordinate_convention` string below.
- `simulator_version`: current Cargo package version.
- `data_format_version`: "1".
- `build_source_fnv1a64`: existing build-time source/Cargo fingerprint.

Coordinate convention string:

```
Oblate Cartesian-like Boyer-Lindquist: X=sqrt(r^2+a^2)*sin(theta)*cos(phi); Y=sqrt(r^2+a^2)*sin(theta)*sin(phi); Z=r*cos(theta); T=t_BL; CartesianSpatial direction; signature (-,+,+,+); local ZAMO energy=1
```

There is no chi, detector pixel resolution, bin width, source-plane definition,
camera or visualization setting in common.json. JSON uses the existing
serde_json `float_roundtrip` feature. The loader reconstructs the detector with
resolution [1,1] solely to satisfy legacy validation; intersection physics does
not read pixel resolution.

## Parquet schema

| Columns | Physical type | Nullability |
|---|---|---|
| ray_id | INT64 with UINT_64 annotation / uint64 | required |
| position_x, position_y, position_z | DOUBLE / float64 | required |
| direction_x, direction_y, direction_z | DOUBLE / float64 | required |
| t_emit | DOUBLE / float64 | required |
| status | BYTE_ARRAY with UTF8 annotation | required |
| u_hit, v_hit, t_hit | DOUBLE / float64 | optional |

Apache Arrow Rust `parquet` 59.3.0, low-level column API, SNAPPY compression.
The package and dependencies are vendored; no Arrow array dependency is required.
Row groups contain at most 8192 rays. No value is reduced to float32.

File key/value metadata:

- `chi`: one shortest round-trip f64 decimal string (`f64::to_string`).
- `chi_type`: "float64".
- `data_format_version`: "1".

Parquet's standard key/value metadata has UTF-8 string values, not a typed f64
scalar slot. Parsing chi as float64 recovers the original bits; this representation
is readable from Python file/schema metadata without reading common.json. Chi is
not repeated in rows. Tests cover nontrivial chi precision, not only 0.6.

Directions are the ORIGINAL unnormalized f64 inputs. Signed zeros and direction
magnitude are preserved in the row. Original input/result association uses the
unchanged `prepare_rays` for canonical keys and queues of duplicate conditions;
there is no row-index-to-physical-ID shortcut. Rows currently retain input order,
but a shuffled file reproduces the same canonical system.

Energy is the fixed local ZAMO convention 1.0 and is no longer a Parquet column.
New reads use that existing constant. Exact historical schemas with energy remain
readable only when its value is 1.0; another legacy energy fails explicitly.

Statuses: Active, Detected, Captured, Escaped, NumericalFailure. Detected has
three finite hit values. Every other status has three actual Parquet nulls;
partial nulls, sentinels and incompatible status/hit combinations are rejected.

No initial Kerr State, full trajectory, E/Lz/Q, image, histogram or display data
is written. `ray_id` identifies a row's physical input and result; it is not a
physical input feature and encodes no position, direction or grid index.

## Recalculation and comparisons

Load common.json and the chosen Parquet file -> validate versions/conventions,
schema, finite values, energy and nullability -> restore raw conditions and
RaySimulationConfig -> call unchanged calculate_rays -> compare with saved results.
The saved ray_id, status and hit columns are never passed into the solver.

IDs are regenerated by existing normalization, signed-zero treatment, total_cmp
lexicographic sorting and enumeration. `execute` attaches each solver-generated ID
back to the corresponding raw input/result row; `save_new` writes it as required
uint64. IDs must be unique and in 0..N-1. No ID is injected into `prepare_rays` or
`calculate_rays`: their arguments still contain physical conditions only.

Only AFTER integration, reference rows are grouped by canonical physical inputs
and duplicate reference groups are ordered by stored ID for comparison. This
cannot affect the newly computed IDs or physics. An ID moved between distinct
physical groups causes comparison failure. Parquet row indices are not IDs.
Exact canonical duplicates remain separate indistinguishable copies, receiving
consecutive unique IDs. Their individual copy identity has no physical meaning;
the ID set within each duplicate group is compared. Normalization-equivalent
raw directions follow the same rule, while original raw bits remain in each row.
Tests shuffle actual saved rows including duplicates, and deliberately exchange
IDs across distinct physical inputs to verify failed ID comparison with unchanged
fresh State/status/hits.

Historical archives with energy, with or without ray_id, remain readable. Their runtime
`RayRow.ray_id` is None, never an invented saved ID. They still support physics
reproduction, but the report explicitly labels missing stored IDs NOT VERIFIED.
New writes require Some(ID) on every row and cannot save legacy missing-ID rows.
New Parquet files contain a required, non-null uint64 column and no energy column.
The exact old/new schemas are recognized separately; common.json and
its data_format_version remain unchanged.
The reader accepts both equivalent UINT_64/UTF8 converted annotations and modern
INTEGER(64,false)/STRING logical annotations, including PyArrow-written shuffled
files. Names, ordering of columns, flat structure, required/optional flags and
physical types remain checked; signed IDs or nullable IDs are rejected.

Comparison checks ray count, regenerated canonical order/IDs, stored versus
regenerated IDs with checked/mismatch counts, reconstructed
initial State versus solver start (bitwise), every status, all five counts,
hit presence, hit bit equality, maximum absolute/relative u/v/t errors.
Hit acceptance: `abs_error <= 1e-9 + 1e-12 * max(abs(old), abs(new))`.
Relative error is zero when both values are zero. Every bit mismatch is reported
even when within tolerance. Any status/count/identity mismatch or out-of-tolerance
hit makes the command return a failure exit status.

Historical initial States are deliberately absent from this format. Thus the
standalone CLI cannot compare against a historically saved State; it labels its
State comparison as reconstructed-input versus solver-start. The automated A/B
test retains an audit-only State-bit snapshot, drops all original input/config/result
objects after writing, reloads from files, and verifies original versus rebuilt
State bits before fresh integration. The snapshot never enters B's solver inputs.

Same-source/version information is reported. Format/convention incompatibility is
an error; a different source/version is allowed for explicit result comparison and
reported as different, not claimed bitwise reproducible across machines/toolchains.
Wall-clock time itself is a measurement, not a reproducible physical output.

## Verification records

Tests live in `tests/standard_run.rs`; logs and measured example results are in
`validation/standard_run/`. They cover fresh disk-only A/B integration, exact raw
float64/common values, standard nullable schema/chi, shuffled rows/duplicates,
tampered reference results, unique numbering/concurrent saving, unsupported
version/convention/energy, malformed data, truncated files/path validation,
Active/NumericalFailure persistence, and actual user CLI success/failure exits.

Existing physics tests and previous independent-ray tests remain unchanged.
Large 256/4096 runs are saved once and reproduced once for the end-to-end example,
not repeatedly integrated inside both debug and release suites. Their saved rows
are also compared with the established legacy outcomes by physical launch position.

### Historical validation before adding the ray_id column — 2026-09-17

`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and
`cargo test --release` all succeeded. Debug and release each passed 84 tests:
the prior 72 plus 12 new standard-archive tests. No existing test was removed or
relaxed. The existing toolchain emitted its known path-canonicalization warning;
there were no clippy code warnings. SHA-256 audit found 0 changes across the 32
pre-existing physics/detector/compute/visualization/output source files. The
independent initial-condition/canonical-ordering code was not edited.

Actual CLI-created examples:

| Example | rays | Detected | Captured | Escaped | NumericalFailure |
|---|---:|---:|---:|---:|---:|
| results/standard_validation/simulation_1 | 256 | 216 | 20 | 20 | 0 |
| results/standard_validation/simulation_2 | 4096 | 3502 | 330 | 264 | 0 |

Each folder contains common.json and sim_1.parquet. In separate reproduce CLI
processes both returned PASS, zero status/canonical/ID/State-start mismatches,
zero maximum absolute and relative hit errors, and zero hit bit mismatches.
Python also compared every status and hit against the existing source_to_detector
and visualization_4096 CSV oracles: zero differences. No old outputs were used as
new solver inputs or overwritten.

| Measured quantity | 256 rays | 4096 rays |
|---|---:|---:|
| common.json | 1,090 bytes | 1,090 bytes |
| sim_1.parquet (SNAPPY) | 8,861 bytes | 104,175 bytes |
| Initial calculation + raw-input association | 0.2743110 s | 3.7096033 s |
| Save | 0.0094251 s | 0.0089601 s |
| Load + validation | 0.0014851 s | 0.0082311 s |
| Actual reproduction calculation | 0.2550740 s | 3.6991137 s |

These are single measured runs, not repeated benchmark averages. Save timing is
directory/file writing and sync after input validation; load timing includes
schema, input and reconstructed-config validation. Reproduction timing includes
calculate_rays/initial preparation and integration, excluding comparison/reporting.

Python verification actually used PyArrow 25.0.1 and pandas 3.0.1. PyArrow was
placed under the workspace's work/python-parquet for this check; the system Python
installation was not modified. Both readers verified 256/4096 rows and float64
columns. Arrow's real null hit counts were 40/594 per hit column. File metadata chi
was 0.6; these historical files have no ray_id or chi column. pandas' default in-memory representation
may show missing float values as NaN, but the archive uses genuine Parquet nulls,
not NaN sentinels. Evidence: validation/standard_run/python_compatibility.json.

Historical limits before GUI integration: no standard-archive GUI loader/reproduce button; no historical
State stored; no cross-toolchain bitwise guarantee; per-ray energy other than 1
is rejected; interrupted writes may leave an incomplete reserved directory;
no automatic multi-simulation series. Standard archives intentionally have no
trajectory and are not interchangeable with the legacy viewer input folders.

### Validation after adding ray_id — 2026-09-17

Final commands: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`, `cargo test --release`: all PASS. Debug and release each passed
89 tests (previous 84 retained, 5 additional tests). The prior schema assertion
was updated from 12 columns/no ID to 13 columns/required unsigned ID. No physics
test or comparison tolerance was weakened. Logs: `validation/ray_ids/`.

SHA-256 comparison of the 69 pre-existing source/test/Cargo files found changes
only in standard_run/{mod,parquet_io,reproduce}.rs and tests/standard_run.rs.
All 37 protected physics/simulation/detector/compute/visualization/output and
common.rs files remained unchanged. README/documentation and a historical archive
test fixture were also added/updated.

New CLI archives are in `results/ray_id_validation/simulation_1` (256) and
`simulation_2` (4096). Every saved ID matched a freshly regenerated ID, with
zero status/count/initial-State-start mismatches and zero hit absolute/relative
or bit errors. PyArrow 25.0.1 verified uint64, required IDs, no nulls, unique IDs,
0..255 / 0..4095 ranges, all-row input/result association, and exact equality of
every pre-existing column against the original standard_validation archives.

| Rays | Detected | Captured | Escaped | NumericalFailure | IDs checked |
|---|---:|---:|---:|---:|---:|
| 256 | 216 | 20 | 20 | 0 | 256 / 256 |
| 4096 | 3502 | 330 | 264 | 0 | 4096 / 4096 |

| Rays | Previous Parquet | With ray_id | Increase |
|---|---:|---:|---:|
| 256 | 8,861 bytes | 10,370 bytes | 1,509 bytes (17.03%) |
| 4096 | 104,175 bytes | 126,939 bytes | 22,764 bytes (21.85%) |

SNAPPY, float64 data, metadata, nullable hits and common.json structure are
unchanged. Each new common.json remains 1,090 bytes; its existing source
fingerprint naturally records the source used when saving.

Rust tests shuffle duplicate-containing input/result rows before writing. An
independent PyArrow rewrite reverses all 256 rows: fresh reproduction PASS.
Exchanging two IDs across different physical rays produces exactly two stored-ID
mismatches, with zero regenerated-ID/State/status/hit differences and CLI failure.
Duplicating an ID is rejected by load validation. Evidence:
`validation/ray_ids/python_compatibility.json` and `pyarrow_*.txt`.

The examples were written before the final equivalent-annotation reader fix.
Final-reader reproduction of the 4096 file and PyArrow-shuffled 256 file still
has zero differences; `same source/version: false` honestly reflects that reader
edit. Old files/common fingerprints were not rewritten to hide this distinction.

The new historical fixture test initially required release-vs-debug bit equality,
which is not the existing reproduction contract. Its final assertion uses the
unchanged hit tolerance; same-build round-trip tests still require exact bits.
The initial failed log is retained as `debug_initial_cross_profile_assertion.txt`.
Old ID-less archives retain physics reproduction but have zero stored IDs checked
and are explicitly NOT VERIFIED for stored-ID comparison; no ID is invented or
automatically injected. Identical duplicate copies have no independently
recoverable physical identity beyond their canonical group's consecutive ID set.

### Current validation: energy-free archives and GUI — 2026-09-17

The historical sections above describe their then-current schemas. The current
writer has **12 columns including ray_id, excluding energy**. Physics local
energy remains 1.0. PyArrow read the newly written 256 and both 4096 archives:
their complete tables and metadata equal the prior validated tables after
dropping only energy. SNAPPY, required uint64 IDs and nullable float64 hits remain.

Final `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
`cargo test --release` and release viewer build passed. Debug/release each have
**94 passing tests**: 89 prior tests plus five GUI archive integration tests.
No physics tolerance was relaxed. Evidence: `validation/gui_archive/`.

Computer Use actually clicked the GUI calculation button twice. Each completed
4096-ray calculation created a different folder in `results/gui_auto_validation`:
`simulation_1` and `simulation_2`, each containing only common.json and
sim_1.parquet. Both have Detected 3502, Captured 330, Escaped 264, Failure 0.
The 256-ray calculation in `results/standard/simulation_1` has 216/20/20/0.

Actual GUI folder Load and Reproduce of the first 4096 archive passed: 4096 IDs
checked, no ID/status mismatches, and zero maximum u/v/t hit error. Newly computed
paths and detector data were displayed. A copied multi-file test folder was
listed as sim_1, sim_2, sim_3, sim_10. Selected sim_2 reproduced 256 rays with
the same zero errors. The intentionally altered sim_10 (one Captured changed
to Escaped in a test copy) correctly displayed FAIL, one status mismatch,
stored/reproduced Captured 19/20 and Escaped 21/20. Original archives were not
altered. Load/Reproduce did not create new output directories.

During manual verification a scene-switch bug was found: the diagnostics panel
used cached 4096-ray indices after accepting a 256-ray result, before the
viewport refreshed them. app.rs now refreshes the renderer at scene acceptance.
The same 4096-to-256 transition then succeeded in the real GUI. Final 94-test
debug/release runs and clippy were performed after this fix. The earlier 4096
GUI reproduction evidence predates this display-only fix; no heavy 4096 physics
rerun was needed afterward.

Screenshots: autosave_two.png, multi_list.png, reproduce_4096_pass.png,
reproduce_sim2_256_pass.png, reproduce_sim10_expected_fail.png in the evidence
directory. GPU: NVIDIA GeForce RTX 3060 Ti, Vulkan, DiscreteGpu.

See GUI_ARCHIVE.md for invocation paths and limits, including the legacy
LocalZamo/nonunit-energy auto-save exclusion and unavailable archived SourcePlane.
