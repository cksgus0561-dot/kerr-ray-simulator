# Seeded generation/storage validation — 2026-09-25

## Executed checks

- `cargo fmt`, final `cargo fmt --check`: PASS.
- `cargo clippy --all-targets -- -D warnings`: PASS.
- `cargo test`: 119 passed, 0 failed, 0 ignored.
- `cargo test --release`: 119 passed, 0 failed, 0 ignored.
- `cargo build --release --bins`: PASS.
- Offline vendored dependencies were used for Cargo commands.
- Original 103 tests retained; only the visualization default-count assertion
  now checks the requested 16384-ray generator. 16 added tests in seeded_run.rs.
  No physics tolerance or existing physical regression expectation was relaxed.

## New tests

Coverage/cell size preservation; one point per cell; fixed ChaCha20 word vector;
independent golden canonical SHA-256; repeated bit-identical inputs; different
seeds; unchanged directions/t_emit; 10000-seed distribution check (10 bins per
axis, each 850..1150, means within 0.015 of 0.5); direct OS seed acquisition and
chi-independent generated inputs; invalid versions/bases; binary round trip;
fresh solver callbacks; all five status codes; non-DET packing; seed/hash/spec
tamper aborts before integration; canonical row mapping; non-axis direction
reproduction; invalid/truncated/trailing binary; numeric sim ordering;
shared config/physics key; legacy JSON selection; worker autosave/no overwrite;
Reproduce without another save; CLI default binary selection and reproduction.

The finite statistical test detects coarse bias; it is not proof of uniformity
over the entire seed space. The implementation uses the OS entropy interface and
an exactly specified 53-bit mapping. No chi/time/index is an entropy input.

## Actual full new run

`results/seeded_validation_20260925/new/simulation_1`:
16384 rays; D15786/C330/E268/NUM0; 395320-byte sim_1.bin and 2854-byte common.json.
No additional raw files. Fresh reproduction: identical all status values and
all u_hit/v_hit/t_hit f64 bits, zero initial-State/canonical mismatch, same source
fingerprint. Hash mismatch tests verify no geodesic callbacks are entered.

Independent Python ChaCha20 + struct + hashlib regenerated all 16384 default
conditions and obtained the same SHA-256:
`f3bb533b9df1650a8b82b788d3b1b8f1660583568e915d932d4aa08d7762115c`.
The script validates counts, exact byte length and detector hit packing, without
running any geodesics or borrowing generated initial conditions from Rust.

## Regression and compatibility

Unchanged Kerr/Schwarzschild, independent initial-state, detector and free-fall
tests passed. Existing 256 and 4096 baseline tests passed. The timed explicit
4096 run returned its original D3502/C330/E264/NUM0 counts. Existing Parquet
read/reproduce tests (including historical IDs, status strings and energy
variants) remain passing. The new default's randomized inputs are a different
experiment and are not expected to have those centered-grid counts.

An additional read-only PyArrow comparison of that already computed 4096 Parquet
against the untouched visualization_4096 CSVs matched all 4096 initial positions,
directions and emission times, all statuses, and all 3502 detected hit triplets
bit-for-bit (zero maximum u/v/t difference). No second legacy simulation was run
for this audit. Evidence: seeded_legacy_4096_audit.json and work/audit_legacy_4096.py.
The 256-ray historical Parquet fixture's fresh reproduction also passed the
unchanged original automated test, retaining D216/C20/E20/NUM0.

## GUI evidence and limitations

Actual release viewer started with `--config examples/seeded.json --grid 2`.
Windows reported a nonzero native handle, title Kerr Geodesic Lab and Responding
true. wgpu chose NVIDIA RTX 3060 Ti / Vulkan / DiscreteGpu. Four calculated rays
reached the scene and produced `results/standard/simulation_2/common.json` plus
`sim_1.bin` (76 bytes, all four CAP). No Save command was sent. stderr was empty.
The test-owned window was closed normally. The built viewer was copied to the
existing bin/kerr-viewer.exe; SHA-256 matched the tested target/release executable.
Existing launcher argument files and their stored-run choice were preserved.

GUI Load Folder/sim selection/Reproduce and config routing are covered through
their shared loader/CLI/worker logic. Their buttons were **not manually clicked**
in this session: no native Windows Computer Use surface was callable here.
No claim of completed manual interaction verification or new rendering FPS.
Cross-OS/CPU execution has not been performed; deterministic known-answer vectors
ran in both Windows debug/release plus independent Python on this host.

Kerr metric, coordinates, tetrad, photon solver, detector intersection, Free-Fall
Grid and legacy result files were not changed. No ML, noise-direction feature,
GPU physics, CPU parallelization or batch executor was introduced.
