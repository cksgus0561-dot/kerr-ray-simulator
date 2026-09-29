# GUI standard archives

## Current seeded default

New default GUI calculations use **192x192 stratified rays**, 0.5M cells, and
write **common.json + sim_1.bin** in a fresh simulation_N folder. The Simulation
panel's **Stratified ray generator** controls edit the shared GeneratorSpec.
SourcePlane controls remain for explicit legacy presets. See [SEEDED_RUN.md](SEEDED_RUN.md).

Load Folder and sim selection dispatch by common.json version and sort numbered
binary files numerically. Load validates the seed-generated initial hash without
integration; Reproduce checks again, executes fresh physics on the existing worker,
and displays hash/status/hit comparisons. Binary stores neither IDs nor inputs.
The sections below describe the preserved legacy Parquet path; its IO, generation
checks and non-overwriting behavior are shared with the new format.

## Save a new calculation

The **Standard simulation archive** panel has an **Auto-save root** field
(default `results/standard`, relative to the process working directory).
Starting without `--input`, pressing **Recompute with current physical settings**,
or changing physical settings submits `Request::CalculateAndSave` to the existing
generation-controlled CPU worker. The fixed-energy Cartesian source adapter feeds
`standard_run::execute_with_outcomes`; its actual outcomes are shared with the
viewer. `standard_run::save_new` reserves a new unpadded `simulation_N` directory
using the existing collision prevention logic. It writes only `common.json` and
`sim_1.parquet`. Existing folders are never overwritten.

Display settings, folder loading and reproduction do not call this save path.
Superseded calculations are cancelled before saving when the generation check
observes the newer request. If a save is already in progress it may finish, but
its stale completion cannot replace the current scene. Save errors are displayed
and the calculated scene remains available. Interrupted writes can leave an
incomplete reserved directory; its name will not be reused.

## Load a folder and reproduce

1. Enter the simulation **folder path**, then press **Load Folder**. This is a
   path-entry control, not a native Windows folder picker.
2. The IO worker reads `common.json`, lists regular files named
   `sim_<positive integer>.parquet` and sorts numerically. Zero-padded, zero,
   negative, suffixed and directory names are excluded.
3. Select a simulation. Its actual schema and contents are validated. Missing or
   invalid common data and invalid Parquet data produce a visible Load ERROR.
4. Press **Reproduce selected simulation**. The existing reproduction API builds
   fresh initial conditions, regenerates canonical IDs and executes the existing
   Kerr CPU solver. Stored IDs/status/hits are comparison data, not solver input.
5. The panel displays PASS/FAIL, ray counts, ID/status mismatch counts, maximum
   u/v/t errors and stored/reproduced status counts. Full comparison text expands.
   The viewer receives the newly computed trajectories and detector events.

Loading alone does not replace the existing scene with invented trajectories.
The selected archive has no stored trajectories: its summary explicitly says
to use Reproduce. Neither Load nor Reproduce creates a new archive directory.

## Schema and compatibility

New Parquet files have twelve columns: required `ray_id: uint64`, required
float64 `position_x/y/z`, `direction_x/y/z`, `t_emit`, required UTF-8 `status`,
and nullable float64 `u_hit/v_hit/t_hit`. There is no energy column. Internal
local energy remains exactly 1.0. SNAPPY, chi metadata, common.json and canonical
ID generation are unchanged. The reader also accepts the existing historical
energy-bearing schemas (including the oldest ID-less version); stored energy
must satisfy the existing energy=1 convention. Missing historical IDs are shown
as unverified, not invented as comparison evidence.

## Scope and limitations

- The verified independent-ray archive API supports CartesianSpatial launch
  directions and fixed local energy 1. Legacy LocalZamo/nonunit-energy source
  experiments still calculate through their original path, but report that
  automatic standard saving is unsupported and retain their scene.
- SourcePlane and source UV are not stored in standard archives. Reproduction
  hides the unavailable source plane/focus instead of inventing it. Actual
  launch positions and directions come from the recorded independent rays.
  Legacy event adapters use finite UV placeholders, explicitly unavailable as
  physical source coordinates. Source controls define a *new* experiment.
- No trajectory was added to Parquet. Existing optional CSV/PNG/APNG/trajectory
  visualization export and the three CLI archive commands remain available.
- No cross-toolchain bitwise guarantee is added; comparison uses the existing
  tolerances and source/build provenance reporting.

## Implementation map

- `standard_run/parquet_io.rs`: energy-free writer and historical readers.
- `standard_run/mod.rs`: existing reservation/save API, folder listing and
  execution returning actual solver outcomes.
- `standard_run/reproduce.rs`: existing comparison with outcomes returned.
- `simulation/worker.rs`: calculate-and-save and reproduce requests.
- `simulation/archive.rs`: outcomes-to-viewer adapter; no new physics.
- `visualization/archive_ui.rs`: folder IO, selection and comparison panel.
- `visualization/app.rs`: UI requests and safe scene replacement. Renderer
  indices are refreshed before diagnostics read a newly received smaller scene.
- `tests/gui_archive.rs`: five additional worker/archive integration tests.

Verification logs, PyArrow checks and Computer Use screenshots are in
`validation/gui_archive/`. See STANDARD_RUN.md for the current validation record.
