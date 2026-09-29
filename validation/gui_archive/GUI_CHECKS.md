# Actual Computer Use verification — 2026-09-17

Release kerr-viewer.exe; NVIDIA GeForce RTX 3060 Ti / Vulkan / DiscreteGpu.
Windows UI was operated through the computer-use skill's @oai/sky API.

| Check | Result | Evidence |
|---|---|---|
| Click Recompute twice, 4096 physical rays | PASS | autosave_two.png; two folders under results/gui_auto_validation |
| Exactly common.json + sim_1.parquet per new folder | PASS | filesystem inspection; parquet_verification.json |
| Required uint64 IDs, no energy, original f64 data unchanged | PASS | PyArrow table/metadata equality after dropping old energy |
| Load single-file simulation folder | PASS | reproduce_4096_pass.png |
| Actual 4096 reproduction and paths | PASS | reproduce_4096_pass.png; IDs/status/hit errors zero |
| Numeric multi-file list and selection | PASS | multi_list.png: sim_1, sim_2, sim_3, sim_10 |
| Select sim_2, reproduce 256 from previous 4096 scene | PASS after fix | reproduce_sim2_256_pass.png |
| Select deliberately altered sim_10 | PASS (expected reproduction FAIL) | reproduce_sim10_expected_fail.png; exactly one status mismatch |
| Load/Reproduce do not create archives | PASS | output directories unchanged after the UI actions |
| Fit scene does not create archive | PASS | clicked in GUI; output directories unchanged |
| Every other display-only control individually exercised this task | NOT TESTED | Existing physics-key separation retained; automated display-key test passes |
| Invalid archive / save failure recovery | Automated PASS | tests/gui_archive.rs; not independently injected through real GUI |

The test sim_10 was a COPY of the valid 256 archive with the first Captured
status changed to Escaped. Its unchanged inputs recompute to the true 216/20/20/0
counts. The expected FAIL demonstrates that stored outcomes do not drive physics.

## Discovered and fixed UI bug

Initially, accepting 256 rays after 4096 rays terminated the viewer. The
diagnostics panel used stale cached renderer indices before prepare ran for the
new scene. app.rs now prepares the renderer on scene acceptance before panels
can read those indices. The exact GUI transition was repeated successfully.
Final fmt/clippy/debug/release/build checks were run after the fix; 94 tests pass
in each profile. Physics, detector, compute, output, canonical rays.rs and
common.rs hash checks: 24/24 protected files unchanged.

The two actual 4096 auto-saves and its GUI reproduction were checked before
this display-only fix. The smaller-scene regression and expected FAIL were
checked with the final binary. No claim of a post-fix heavy 4096 rerun is made.

## Recorded artifacts

- Debug/release totals: test_summary.json (94 each).
- Final command logs: clippy_final.txt, debug_final.txt, release_final.txt,
  viewer_build_final.txt. Earlier run logs are retained for audit.
- Complete all-row schema/data comparison: parquet_verification.json.
- Protected code audit: source_audit.json vs source_before.json.
- Screenshots listed above preserve actual observed GUI states.
