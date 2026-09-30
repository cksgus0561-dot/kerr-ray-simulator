# Kerr Geodesic Lab User Guide

[한국어](PROGRAM_GUIDE_KO.md) · [README](README.md)

Baseline: v0.3.0, code commit `3eae1ef7981580b1b23a453157bd399a91acf879`. Source, CLI help, the Windows GUI, and small save/reproduction runs were checked on 2026-09-30. Run the commands below in **PowerShell at the project root containing Cargo.toml**. Examples using `simulation_1` assume an unused output root. If folders already exist, substitute the actual number printed by the program.

## 1. Introduction

This program integrates three-dimensional lightlike trajectories (null geodesics) in a fixed Kerr spacetime and finds their intersection positions and arrival times at an absorbing detector plane. Use the GUI to configure experiments and explore trajectories; use the batch CLI for repeated runs at multiple spins. The generated data can support machine-learning research, but the program does not train models.

Units are `G=c=M=1`, with `chi=a/M`. Physics uses an adaptive CPU f64 Dormand–Prince 5(4) integrator; wgpu renders the scene on the GPU. The program does not evolve dynamical gravity or Maxwell fields, compute backreaction, or use a CUDA/GPU ray solver.

## 2. Main features

- Independent three-dimensional ray initial conditions, ZAMO initial momenta, and Kerr geodesic integration.
- Seeded stratified position sampling and CPU parallelism across rays.
- States including DET/CAP/ESC/NUM, detector coordinates `u_hit, v_hit`, and continuous BL arrival time `t_hit`.
- 3D trajectories, detector, horizon and ergosurface, physical-time playback, and camera controls.
- Metric-based frame dragging and a free-fall grid with its own clock.
- Automatic standard archives and genuine recalculation from seed-regenerated initial conditions.
- Accumulated/time-resolved detector images, CSV, PNG/APNG, and separate trajectory exports.

The standard initial configuration is:

| Setting | Default |
|---|---|
| `generator.cell_count` | [192, 192] → 36,864 rays |
| `generator.cell_size` | [0.5, 0.5] M → total 96×96 M |
| `generator.center` | [80, 0, 0] M |
| `generator.axis_1 / axis_2` | [0, 1, 0] / [0, 0, 1] |
| `generator.fixed_direction` | [-1, 0, 0], CartesianSpatial, no direction noise |
| `generator.t_emit` | 0 |
| `experiment.spin` | 0.6 |
| Detector plane | Center [-80, 0, 0] M, normal [1, 0, 0], 300×300 M |
| Detector image | 128×128 pixels |
| Displayed trajectories | Up to 256; physics and detector statistics use every ray |

One uniformly sampled position is drawn independently in each cell. Changing cell counts preserves cell size, so it also changes the launch area. For example, 16×16 means 256 rays in 8×8 M. This is **different from the legacy 256-ray regression preset**, which covers 32×32 M.

## 3. System requirements

The checked environment is Windows MSVC, Rust/cargo 1.98.1, and NVIDIA GeForce RTX 3060 Ti / Vulkan / DiscreteGpu. The project uses Rust edition 2024, eframe/egui 0.36.2, and wgpu 30.0.1. Linux/macOS builds and execution were not checked for this guide; support is not guaranteed here.

- Install Rust and Cargo, Windows C++ build tools, and the Windows SDK. Follow the rustup instructions in the [official Rust installation guide](https://rust-lang.org/tools/install/).
- Initial dependency downloads require internet access. Cargo.lock pins dependencies, not the compiler version.
- The GUI requires a wgpu-compatible GPU and driver. A discrete NVIDIA GPU is preferred when available. CLI calculations do not require a GUI window.
- Korean text uses a system Hangul font. On Windows the program looks for Malgun Gothic; fonts are not redistributed.
- Python is not required to build or run the program. Memory and disk usage grow with ray counts and retained trajectories.

## 4. Installation and build

For a private repository, arrange GitHub access first. After installing Rust, open a new PowerShell window and run:

```powershell
git clone https://github.com/cksgus0561-dot/kerr-ray-simulator.git
cd kerr-ray-simulator
rustc --version
cargo --version
cargo fetch --locked
cargo build --release --locked --bins
```

The executables are `target/release/kerr-ray.exe` and `target/release/kerr-viewer.exe`. Do not assume that a source checkout includes existing simulation data, built executables, or a personal toolchain/cache.

`env.ps1` is a helper for an existing local development environment. Ordinary users can use an installed Rust toolchain on PATH. Use `--offline` only after all dependencies are available locally.

Command help:

```powershell
cargo run --release --locked -- --help
.\target\release\kerr-viewer.exe --help
.\target\release\kerr-ray.exe batch-simulate --help
```


## 5. Launching the program

Standard seeded GUI:

```powershell
.\target\release\kerr-viewer.exe --config .\examples\seeded.json
```

The equivalent Cargo entry point:

```powershell
cargo run --release --locked -- visualize --config .\examples\seeded.json
```

These commands start a new background simulation immediately and save it automatically. For a lightweight GUI check:

```powershell
.\target\release\kerr-viewer.exe --config .\examples\seeded.json --grid 16 --rendered 256
```

`kerr-ray.exe visualize` opens the GUI, `simulate-save` runs one calculation without a window, and `batch-simulate` runs repeated calculations without a window. There is no batch-start button in the GUI.

Without arguments, `kerr-viewer.exe` first reads the argument array in a beside-executable `kerr-viewer.args.json`. If no arguments result, it uses a beside-executable `kerr-viewer.json` configuration. With neither file it starts a default new calculation. Explicit `--config` or `--input` bypasses these launcher settings, so the command above is recommended for a first launch.

To open an existing **visualization export directory**:

```powershell
.\target\release\kerr-viewer.exe --input .\results\my_visualization
```

You must have generated this export first. `--input` reads visualization results containing `run_metadata.json`, events, and optional trajectories. **Open common.json + sim_N.bin directories through the archive UI or the reproduce CLI.** These are different directory formats.

## 6. Using the GUI

### Layout and language

The left panel contains scrollable settings, the center is the 3D viewport, the right panel is **Observation**, and the bottom contains ray playback and status messages. Side panels are resizable. Click a section triangle to expand it. Drag numeric fields or click to type; Shift+drag gives finer control.

Choose **Settings → Language → English / 한국어** to switch immediately. The choice is saved in `kerr-viewer.ui.json` beside the running executable and applied on the next launch. The default is English. Language changes do not alter physics, seed/hash, file schemas, or CLI help. A write failure appears in Settings. If a Hangul font is unavailable, the Korean choice is disabled and an explanation is shown.

### Physical settings and new calculations

Open **Scene → Simulation (background recomputation)**.

| Actual UI label | Meaning / default |
|---|---|
| `chi = a/M` | Default 0.6; validation guard [-0.9999, +0.9999], exact extremal values excluded |
| **Stratified ray generator** | Standard seeded ray-generation settings |
| **Launch region center (M)** | Default [80, 0, 0] |
| **Unit axis 1 (XYZ)** / **Unit axis 2 (XYZ)** | Orthonormal launch-region axes |
| **Cell count 1 / 2** | Default 192 / 192; editable directly. Shortcut buttons: 16, 32, 64, 128, 256 |
| **Cell size (M)** | Default 0.5 / 0.5; multiplied by cell counts to obtain the area |
| **Fixed CartesianSpatial direction (no noise)** | Default [-1, 0, 0]; normalized and passed through the existing ZAMO conversion |
| `t_emit` | Common BL emission time, default 0 |
| **DetectorPlane** | **Center (coordinate M)**, **Normal (XYZ)**, **Local v / up (XYZ)**, **Width / height** |
| **Reference accuracy** | `rtol`, `atol`, `r_escape` |

Axes must have unit length and be orthogonal. Direction must be nonzero. CartesianSpatial direction is not an input of BL `dr/dtheta/dphi` components.

Physical edits automatically submit a new calculation after 450 ms without further input. **Recompute with current physical settings** also starts a new calculation. The standard path draws a fresh seed each time, so pressing this button with unchanged settings does not reproduce the same sampled rays. Pauses between edits can produce multiple calculations and automatic archives.

Current standard numerical defaults:

| Setting | Value |
|---|---|
| `rtol / atol` | 1e-12 / 1e-14 |
| `initial_step / min_step / max_step` | 0.05 / 1e-12 / 0.5 |
| `max_steps / max_affine` | 200000 / 1500 |
| `horizon_epsilon / escape_radius` | 1e-3 / 400 |
| `null_tolerance / detector_root_tolerance` | 1e-5 / 1e-10 |

Numerical fields absent from the GUI can be set in a complete configuration JSON. Very small values may lose visible digits in the GUI's number formatting; check the configuration JSON for exact values. The accepted spin range is not a guarantee of numerical stability for all initial conditions.

Legacy JSON/presets show **SourcePlane (legacy preset)** instead, including ray-grid X/Y and a local ZAMO direction option. This is a different initial-condition path from the standard stratified generator.

### Recalculation versus stopping playback

Ray calculations run on a background worker while the existing scene and camera remain available. Generation IDs prevent stale requests from replacing newer results. Camera, language, rendered-count, line-width, detector-resolution, time-bin, and playback changes do not submit another ray calculation.

**Stop** and **Reset time** return playback to its beginning and pause it. **There is no dedicated cancel-physics button.** A new physical request can supersede an older one, but in-flight rays finish before cancellation is observed. Forcibly closing the window is not a safe save/resume feature.

### 3D display and camera

Under **Scene → Visibility**:

| Control | Display |
|---|---|
| **Cartesian Reference Grid**, **XYZ axes** | Straight position references, not the gravitational field or curved space itself |
| **Kerr spin axis (+Z)** | Kerr center and axis direction |
| **Event horizon**, **Ergosurface** | Surfaces calculated from the current Kerr spin |
| **SourcePlane**, **Launch points** | Launch region and actual initial positions |
| **DetectorPlane**, **Detector hit points** | Detector plane and actual hits |
| **Trajectories**, **Current ray positions** | Calculated paths and positions at the playback time |
| **Frame dragging** | Metric-based angular-velocity arrows |
| **Detector world overlay**, **Detector image panel** | Distribution on the plane / image in the right panel |

In **Display settings (no new physics)**, **Rendered trajectories (physical rays unchanged)** changes only the displayed subset, starting from a default of 256. All physical rays and detector results remain intact. **Line px** defaults to 1.35. Disable **Grid auto-fit** to edit **Extent** and **Spacing**. The grid is capped at 40 intervals per axis; inspect the effective spacing in diagnostics.

Left-drag to orbit; right/middle-drag, or left-drag with **Pan drag** enabled, to pan; use the wheel to zoom. Keyboard controls are arrows to orbit, Shift+arrows to pan, and +/- to zoom. Keyboard camera controls are suppressed while editing text.

- **Fit scene**: fit the displayed scene.
- **Reset camera**: restore the default camera and fit the scene.
- **Kerr center**, **Source**, **Detector**: focus on that object.
- Use **Ray ID → Select / Clear** on the right, or click a displayed ray: highlight it and inspect its status, launch position/direction, hit, steps, and failure reason. Missing hits are not fabricated.

Camera movement does not rebuild trajectory buffers. Changes to the displayed subset/selection or simulation results update the necessary GPU geometry.

### Coordinates and field interpretation

The viewport uses right-handed Cartesian-like BL spatial coordinates, with the Kerr center at the origin and +Z as the spin axis.

`X=sqrt(r²+a²) sin(theta) cos(phi)`, `Y=sqrt(r²+a²) sin(theta) sin(phi)`, `Z=r cos(theta)`, `T=t_BL`.

This is not a Euclidean embedding or Kerr-Schild time. The horizon `r+=1+sqrt(1-a²)` and ergosurface `r=1+sqrt(1-a² cos²(theta))` are mapped into these display coordinates.

Frame dragging uses the actual metric quantity `omega=-g_tphi/g_phiphi`. Arrows represent `omega*(-Y,X,0)*display interval`. Controls are **Field samples/axis** (default 15), **3D field (otherwise z=0 slice)**, **Field extent** (default 12), and **Arrow time scale M** (default 12). This is coordinate angular velocity, not an additional force acting on rays.

### Free-fall grid

Enable **Kerr Free-Fall Grid** under **Kerr Free-Fall Grid (visualization only)**. Defaults are **Extent** 10, **Nodes/axis** 9, and **Grid M / wall s** 3. Once its cache is ready, use **Play grid / Pause grid** and **Reset grid**. Focus on Kerr center for a closer view.

Each marker follows an E=1, Lz=0 rain observer, and surviving neighboring markers are connected by straight segments. It is not a smooth curved grid with separately integrated points throughout every edge. Its `t_BL/M=0..80` clock is independent of photon playback and does not advance while hidden. The rotation axis and a numerical horizon margin of 0.02 M are excluded. **Free-fall metric diagnostics** reports visible nodes and metric checks. These controls do not rerun the photon solver.

### Detector and ray playback

The first menu under **Detector / playback** contains:

| Menu entry | Behavior |
|---|---|
| **Full trajectories** (selected label: **Static**) | Display complete static paths |
| **Ray propagation** (selected label: **Propagation**) | Display paths up to stored BL time, with linear interpolation between samples |
| **Detector playback** (selected label: **Detector**) | Keep full paths while using time-dependent position/detector displays |

The second menu controls the detector filter:

| Menu entry | Behavior |
|---|---|
| **All accumulated hits** (selected label: **Accumulated**) | All hits, regardless of the time slider |
| **Instantaneous time bin** (selected label: **Instantaneous**) | Hits arriving within the current time bin |
| **Cumulative to current time** (selected label: **Cumulative**) | Hits up to the current time |

Select a time bin or cumulative-to-current-time mode to see detector arrivals evolve. Use **Play / Pause**, **Stop**, **Reset time**, and the bottom `t_BL / M` slider. **Physical M / wall s** (default 25) sets presentation speed; FPS is not physical time. The main clock uses absolute `t_hit`; the separate rebin CLI supports `delta_t=t_hit-t_emit` distributions.

`Delta t`, default 1, is the bin width. **Pixels X / Y** defaults to 128 / 128; the GUI allows 1..1024 on each axis. **Counts / white**, default 4, controls brightness scaling. These settings postprocess all stored continuous events without reintegration. Image u increases rightward and v upward. `I(u,v)` is a pixel arrival count, not absolute radiative intensity from a radiation-transfer calculation.

### Automatic saving, loading, and reproduction

Under **Standard simulation archive**:

1. **Auto-save root (new calculations only)** defaults to `results/standard`.
2. A completed new calculation creates `simulation_N/common.json + sim_1.bin` without a separate Save action.
3. Enter that directory under **Simulation folder path**, then click **Load Folder**.
4. Select `sim_N.bin`. This reads the stored summary and leaves the current scene in place.
5. **Reproduce selected simulation** recalculates from the seed, displays fresh paths, and reports **Reproduction PASS/FAIL**. Reproduction does not automatically save another archive.
6. Expand **Full comparison** to inspect the hash, states, hit errors, and source/version comparison.

Legacy Parquet archives are also readable. Old formats without SourcePlane information have limited source display/focus; the program does not invent missing initial conditions.

### Exports and diagnostics

**Export (new filenames only)** defaults to `results/visualization_export`, separately from the automatic archive root.

| Button | Output / purpose |
|---|---|
| **Screenshot** | `screenshot.png`, current view including GUI |
| **Save visualization config** | `visualization.json`, shared SessionConfig |
| **Export current detector image + CSV** | `detector_view.png`, `detector_view.csv` |
| **Save complete run + frame sequences** | Separate visualization export with trajectories, events, PNG/APNG |

Existing files are not overwritten. A complete export requires a **new empty directory**; do not first save screenshots/configuration there and then attempt a complete export into it. **Trajectory export stride** stores every Nth accepted sample and both endpoints. It defaults to 1 and does not alter original in-memory samples or detector results. Frame sequences are **detector images**, not a 3D viewport video recording.

**Observation** shows GPU/backend, FPS, physical/displayed ray counts, state counts, and calculation time. **Conservation / GPU cache** contains null/E/Lz/Q errors, upload counts, effective grid spacing, omega, and p95 frame time. Calculation/export errors appear in the bottom status area; automatic-save errors appear in the archive panel; reproduction errors appear in its panel.

## 7. Running a single simulation

Start the GUI with the command in section 5. After completion, check state counts and the **Auto-saved** message. From the CLI:

```powershell
.\target\release\kerr-ray.exe simulate-save --config .\examples\seeded.json --output .\results\standard
```

Omit `--config` to use current standard defaults. `--output` defaults to `results/standard`. For a lightweight save check:

```powershell
.\target\release\kerr-ray.exe simulate-save --grid 16 --output .\results\single_smoke
```

`--grid N` sets both cell counts to N. `--preset regression` selects the legacy SourcePlane regression path and Parquet storage; it is not the standard seeded BIN experiment.

`simulate-save` has no `--seed` or `--chi` option. Set spin through the GUI or `experiment.spin` in a complete SessionConfig JSON. A reliable way to create a new configuration is **Save visualization config**, followed by editing a copy of that complete JSON. `examples/seeded.json` specifies the generator and fills other fields from defaults. Older JSON without a generator may select the legacy path.

Configuration files define new experiments. An archive's `common.json` is a reproduction record, not a normal `--config` file; do not overwrite it. Code, GUI, and CLI JSON share SessionConfig in [src/session.rs](src/session.rs).

## 8. Running automatic batch simulations

### Required option and chi grid

The exact form is:

`batch-simulate --simulations-per-chi N [--config SESSION_JSON] [--output ROOT] [--chi-indices -76,0,76] [--dry-run]`

| Option | Meaning |
|---|---|
| `--simulations-per-chi N` | Required positive integer. Run N simulations **at each selected chi**. No default |
| `--config SESSION_JSON` | Seeded configuration sharing generator/detector/numerics. Standard defaults if omitted |
| `--output ROOT` | Default `results/standard` |
| `--chi-indices` | Strictly increasing, unique integers in -76..76, comma-separated without spaces |
| `--dry-run` | Print the plan only: no seeds, ray calculations, or output data |

The full grid has 153 indices, `i=-76..76`.

`s_i = i * atanh(0.999) / 76`, `chi_i = tanh(s_i)`.

The code fixes the endpoints at exactly -0.999 and +0.999 and includes zero and sign symmetry. Spacing is not uniform in chi itself. `--chi-indices 0` selects chi=0; `-76,0,76` selects -0.999, 0, +0.999. In batch mode this grid determines spin instead of the configuration's single `experiment.spin`.

For the full grid, N=1 gives **153 simulations** and N=10 gives **1530 simulations**. For a subset, the total is `number of selected indices × N`. Inspect the plan first.

```powershell
.\target\release\kerr-ray.exe batch-simulate --simulations-per-chi 1 --dry-run
.\target\release\kerr-ray.exe batch-simulate --simulations-per-chi 10 --dry-run
```

To actually calculate the full grid:

```powershell
.\target\release\kerr-ray.exe batch-simulate --config .\examples\seeded.json --simulations-per-chi 1 --output .\results\batch
```

For N=10, replace `--simulations-per-chi 1` with `--simulations-per-chi 10`. The guide verification checked these totals with dry-run; it did not execute the full production runs.

### Small executable example

This creates a temporary 16×16 configuration without editing the original example. It calculates **256 rays twice at chi=0**, using new seeds.

```powershell
New-Item -ItemType Directory -Force .\work\guide-smoke | Out-Null
$smokeConfig = Get-Content .\examples\seeded.json -Raw | ConvertFrom-Json
$smokeConfig.generator.cell_count = @(16, 16)
$smokeJson = $smokeConfig | ConvertTo-Json -Depth 20
$smokePath = Join-Path (Get-Location).Path 'work/guide-smoke/session.json'
[System.IO.File]::WriteAllText($smokePath, $smokeJson, [System.Text.UTF8Encoding]::new($false))
.\target\release\kerr-ray.exe batch-simulate --config .\work\guide-smoke\session.json --simulations-per-chi 2 --chi-indices 0 --output .\results\batch_smoke
```

Batch has no `--grid` option, so the smaller cell count goes in JSON. With the unchanged 0.5 M cell size, this example covers 8×8 M.

One entire batch occupies one new `simulation_N` directory. It writes `common.json` once, then processes chi indices in ascending order, finishing all repetitions at each chi before proceeding. Files are numbered `sim_1.bin, sim_2.bin, ...`. Simulations run sequentially; rays within each simulation run in parallel on the CPU.

Check the final `batch saved: ...; completed 2 / 2` message and use the printed directory.

```powershell
.\target\release\kerr-ray.exe inspect-simulation --input .\results\batch_smoke\simulation_1 --sim sim_2.bin
.\target\release\kerr-ray.exe reproduce --input .\results\batch_smoke\simulation_1 --sim sim_2.bin
```

**Actually checked:** an equivalent small configuration generated common.json and two correctly numbered BIN files with independent seeds, exited normally, and reproduced sim_2.bin with PASS. Recorded ACT/DET/CAP/ESC/NUM counts were [0,0,252,4,0] and [0,1,251,4,0]. Later executions can have different counts because they use new seeds.

## 9. Output files

### Standard archives

Default output paths are relative to the current working directory. A single run writes one simulation file in a new directory; a batch writes several files in one new directory.

```text
results/standard/
  simulation_1/
    common.json
    sim_1.bin
  simulation_2/
    common.json
    sim_1.bin

results/batch/
  simulation_1/
    common.json
    sim_1.bin
    sim_2.bin
    ...
```

The program atomically reserves the first unused positive `simulation_N` number. Existing directories/files are not overwritten. Gaps in numbering can be reused.

**common.json** contains:

- `numerics`: integration errors/steps, affine/step limits, capture/escape/null settings, detector-root tolerance.
- `detector`: center, normal, local axes, physical width/height. Image pixel resolution is not included.
- `convention`: units, coordinates, spin axis, signature, and local ZAMO energy=1.
- `simulator_version`, `build_source_fnv1a64`, `data_format_version="2-seeded-bin"`.
- `ray_generator`: generator/RNG versions and rules, region/cells/direction/emission time.
- `binary_format`: byte order, layout, status codes, hit columns, and hashing rules.
- For a batch, `batch`: chi sampling rules/range/full point count, `simulations_per_chi`, and selected `chi_indices`.

Each BIN stores its own chi. Language, camera, rendered count, and detector pixel/bin settings are not standard reproduction records.

**sim_N.bin** is raw little-endian binary. N is the total ray count and K is the DET count.

| Starting offset (bytes) | Field | Type / size |
|---|---|---|
| 0 | chi | f64 / 8 |
| 8 | seed | uint8[32] / 32 |
| 40 | initial_conditions_hash | SHA-256 / 32 |
| 72 | status | uint8[N] / N |
| 72+N | DET hit triples | K×[u_hit,v_hit,t_hit], f64 / 24K |

Total size is `72 + N + 24K` bytes. There is no separate header/trailer, padding, or compression. Statuses follow canonical ray order; hit triples follow the DET rays in that order. Non-DET rays have no hit triple. **Current BIN files do not store ray_id, raw initial positions/directions, an energy column, full trajectories, or images.** Initial conditions and IDs are regenerated from seed plus generator. Distinguish this from the older Parquet schema.

### Visualization exports

A complete visualization export is separate from a standard BIN archive:

```text
my_visualization/
  visualization.json
  run_started.json
  run_metadata.json
  scene_metadata.json
  ray_diagnostics.csv
  detector_hit_states.csv
  detector_events.csv
  trajectory_samples.csv.gz
  detector_accumulated.csv
  detector_accumulated.png
  detector_before_window.csv
  detector_time_bins.csv
  frame_index.csv
  derived_metadata.json
  detector_instantaneous.apng
  detector_cumulative.apng
  detector_frames/
    instantaneous/frame_000000.png
    cumulative/frame_000000.png
    ...
```

`detector_events.csv` contains ray_id, source u/v, t_emit, u_hit, v_hit, t_hit, delta_t, and status. `trajectory_samples.csv.gz` has columns `ray_id, affine, t, r, theta, phi, p_t, p_r, p_theta, p_phi`. Lossless gzip holds f64 BL samples, which are converted to Cartesian-like coordinates for display.

To change only time bins and pixel resolution, use this **event export directory**:

```powershell
.\target\release\kerr-ray.exe rebin --input .\results\my_visualization --output .\results\rebinned --dt 0.1 --resolution 256x256 --fps 30
```

This does not reintegrate geodesics. Use a new output directory. A standard BIN directory is not valid input to this rebin command; reproduce it in the GUI and make a complete export first if needed.

## 10. Seeds and reproduction

Each new standard simulation gets an independent 256-bit seed from the OS. Seeds are not derived from chi or run numbers. ChaCha20 (`rand_chacha 0.9.0`) regenerates the same sampled cell positions from the same seed and generator settings.

`prepare_rays` validates finite values, normalizes directions and signed zero, then deterministically sorts by position XYZ, direction XYZ, and t_emit. It assigns internal ray_id=0..N-1 afterward. Historical ray IDs are not injected from BIN files.

`initial_conditions_hash` is SHA-256 of the canonical sequence of position XYZ, normalized direction XYZ, and t_emit encoded as little-endian f64 values. It is checked during saving/loading/reproduction. A mismatch causes an error **before geodesic integration starts**.

To reproduce:

1. Keep the original `common.json` and desired `sim_N.bin` together.
2. Use inspect to check loading, initial-condition hashing, and stored state counts.
3. Use reproduce to regenerate inputs and rerun Kerr integration.
4. Check `result: PASS` and the detailed differences.

```powershell
.\target\release\kerr-ray.exe inspect-simulation --input .\results\standard\simulation_1 --sim sim_1.bin
.\target\release\kerr-ray.exe reproduce --input .\results\standard\simulation_1 --sim sim_1.bin
```

Substitute the actual output directory number. Omitting `--sim` selects the first listed file, but specifying it avoids ambiguity. The current user-facing route for running the same seed is **reproduce**. Running `simulate-save` again draws a new seed.

PASS checks ray count, canonical ordering/IDs, reconstructed initial State versus solver start, states/hit presence/counts, and hit tolerances. The hit criterion is
`abs(error) <= 1e-9 + 1e-12 * max(abs(original),abs(new))`.
Hit bit mismatches and source/version equality are also reported, but are not themselves PASS requirements. Do not interpret PASS as a guarantee of bitwise equality on every environment. The full historical initial Kerr State is not separately stored in the archive either.

## 11. CPU parallelism

The default worker count is Rust's `available_parallelism()` minus 1, with a minimum of 1. Actual thread count is also limited by the number of rays. Override it with a positive integer in `KERR_RAY_THREADS`. A value of 1 selects serial calculation; zero and invalid values are errors.

After generating the small configuration/results in section 8:

```powershell
$env:KERR_RAY_THREADS = "4"
.\target\release\kerr-ray.exe batch-simulate --config .\work\guide-smoke\session.json --simulations-per-chi 2 --chi-indices 0 --output .\results\batch_threads
$env:KERR_RAY_THREADS = "1"
.\target\release\kerr-ray.exe reproduce --input .\results\batch_smoke\simulation_1 --sim sim_2.bin
Remove-Item Env:KERR_RAY_THREADS -ErrorAction SilentlyContinue
```

The environment variable affects programs launched from that PowerShell session. Results are assembled in canonical ray order rather than completion order. The GPU renders the scene; this variable does not enable GPU physics. Batch mode does not execute several simulations simultaneously.

## 12. Status codes

| BIN code | Short name / internal enum | Meaning | Trajectory color |
|---|---|---|---|
| 0 | ACT / Active | No capture, escape, or detection yet; can remain after the affine limit | Gray |
| 1 | DET / Detected | First valid intersection inside the absorbing detector boundary | Cyan |
| 2 | CAP / Captured | Reached the numerical cutoff r <= r+ + horizon_epsilon | Orange |
| 3 | ESC / Escaped | r >= escape_radius with outward motion | Purple |
| 4 | NUM / NumericalFailure | NaN, null-constraint violation, step limits, or integration failure | Red |

Exceeding the maximum step count is not ESC. CAP does not mean the solver integrated inside the horizon. Inspect NUM diagnostics rather than treating failures as valid physical outcomes. Batch can record NUM outcomes, so a successful command is not proof that every ray completed numerically. Selected rays use a highlight color.

## 13. Troubleshooting and cautions

| Symptom | Check |
|---|---|
| cargo missing / linker error | Rust PATH, a new PowerShell window, Windows C++ tools and SDK |
| GPU initialization fails | Compatible driver and adapter messages in the console; NVIDIA selection depends on available devices |
| Korean unavailable / language not retained | System Hangul font and read/write access to UI preferences beside the executable |
| Automatic save fails | Working directory and write access to **Auto-save root (new calculations only)**. The calculated result is retained in memory, but saving did not succeed |
| File/directory already exists | Use a new empty export directory. Standard archives automatically reserve a new simulation_N |
| Cannot open BIN through --input | Use **Load Folder → Reproduce selected simulation**, or CLI reproduce |
| TRAJECTORY UNAVAILABLE | Old event-only visualization results preserve detector data but do not invent paths |
| Detector image does not change during playback | Select a time bin or cumulative-to-current-time mode instead of all accumulated hits |
| Too few rays visible | Distinguish rendered subset from physical count; check visibility and playback time |
| Hash mismatch / reproduction FAIL | Check common.json/BIN pairing, generator/source versions, corruption, and detailed comparisons; do not relax criteria to obtain PASS |
| Missing batch argument | `--simulations-per-chi N` is mandatory and positive; indices are integers in -76..76, not chi floats |
| Unexpected startup settings | Use explicit --config; inspect launcher sidecars and whether the JSON selects a legacy path |

BL coordinates are singular at the horizon, and the solver stops outside it. Neither the free-fall grid nor the straight coordinate grid depicts a Euclidean shape of curved space.

Forced termination/Ctrl+C or an I/O failure can leave completed files and incomplete files/directories. A directory's existence does not establish batch completion. Check the completion log and inspect validation. There is no automatic batch resume; starting again under a different root uses new seeds.

Validation commands:

```powershell
cargo fmt --check
cargo check --locked
cargo test --locked --no-fail-fast
cargo clippy --locked -- -D warnings
cargo build --release --locked --bins
```

At this baseline, **146 of 148 tests passed; 2 existing failures remain**. The following old bitwise regressions in [tests/ray_initial_conditions.rs](tests/ray_initial_conditions.rs) were not changed for this documentation task:

- `regression_256_all_initial_states_match_legacy_and_saved_baseline`
- `regression_4096_all_initial_states_match_legacy_and_saved_baseline`

fmt/check/clippy/release build passed. Verification covered CLI help, small single/batch calculations and reproduction, actual GUI language switching, automatic saving, archive selection, reproduction PASS, and free-fall grid display/time evolution. The language sidecar's write/read implementation was also checked. This is not a new validation of every GUI input, the full 153-point production dataset, other OSes, or all physical initial conditions.

## 14. Quick start

**GUI**

1. Install Rust and Windows build tools as described in section 3.
2. Clone and build release executables using section 4.
3. Launch the GUI with `--config .\examples\seeded.json` from section 5.
4. Select a language under **Settings → Language**.
5. Let the automatic default calculation finish; check 36,864 physical rays, up to 256 rendered trajectories, and the automatic-save message.
6. Select **Ray propagation**, **Cumulative to current time**, then **Reset time → Play**.
7. To recalculate the same sampled experiment, use reproduce from section 10.

**Batch**

1. Use section 8's dry-run commands to confirm N=1 → 153 and N=10 → 1530.
2. Run its small 16×16 / chi index 0 / 2-repetition example first.
3. Check `common.json, sim_1.bin, sim_2.bin` and reproduce sim_2.bin.
4. Only execute the standard generator/full-grid command when you intend to generate the full dataset.

Further references: [SEEDED_RUN.md](SEEDED_RUN.md), [BATCH_SIMULATION.md](BATCH_SIMULATION.md), [CPU_PARALLEL.md](CPU_PARALLEL.md), and [VISUALIZATION.md](VISUALIZATION.md). Settings and execution paths are defined in [src/session.rs](src/session.rs), [src/experiments/stratified.rs](src/experiments/stratified.rs), [src/standard_run/cli.rs](src/standard_run/cli.rs), [src/standard_run/batch_cli.rs](src/standard_run/batch_cli.rs), [src/standard_run/seeded.rs](src/standard_run/seeded.rs), and [src/visualization/ui.rs](src/visualization/ui.rs).
