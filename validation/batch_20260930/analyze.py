"""Read-only audit of the three CLI validation runs and existing test logs."""
import hashlib
import json
import math
from pathlib import Path
import re
import struct

audit = Path(__file__).resolve().parent
project = audit.parent.parent
directory = audit / "runs" / "simulation_1"
common = json.loads((directory / "common.json").read_text(encoding="utf-8-sig"))
g = common["ray_generator"]
assert g["cell_count"] == [192, 192]
assert g["cell_size"] == [0.5, 0.5]
assert g["fixed_direction"] == [-1.0, 0.0, 0.0]
assert g["direction_noise"] == "none"
assert common["batch"]["chi_indices"] == [-76, 0, 76]
assert common["batch"]["simulations_per_chi"] == 1
assert common["batch"]["chi_sampling"]["total_chi_count"] == 153
assert common["numerics"]["rtol"] == 1e-12
assert common["numerics"]["atol"] == 1e-14
assert common["numerics"]["null_tolerance"] == 1e-5
n = math.prod(g["cell_count"])
results = []
for number, expected_chi in enumerate([-0.999, 0.0, 0.999], 1):
    path = directory / f"sim_{number}.bin"
    raw = path.read_bytes()
    chi = struct.unpack_from("<d", raw)[0]
    assert chi == expected_chi
    statuses = raw[72:72+n]
    assert len(statuses) == n and all(s in range(5) for s in statuses)
    counts = [statuses.count(s) for s in range(5)]
    assert len(raw) == 72 + n + 24 * counts[1]
    hits = list(struct.iter_unpack("<ddd", raw[72+n:]))
    assert len(hits) == counts[1] and all(math.isfinite(x) for h in hits for x in h)
    assert all(abs(u) <= common["detector"]["width"]/2 and
               abs(v) <= common["detector"]["height"]/2 and t >= g["t_emit"]
               for u, v, t in hits)
    reproduction = (audit / f"reproduce_{number}.log").read_text(encoding="utf-8-sig")
    for evidence in ["Initial conditions SHA-256: PASS", "canonical mismatch: 0",
                     "status mismatch: 0", "hit bit mismatches [u,v,t]: [0, 0, 0]",
                     "hit max abs error [u,v,t]: [0.0, 0.0, 0.0]", "result: PASS"]:
        assert evidence in reproduction, (number, evidence)
    results.append(dict(file=path.name, chi=chi, rays=n, counts_ACT_DET_CAP_ESC_NUM=counts,
                        finite_hits=len(hits), seed=raw[8:40].hex(),
                        initial_conditions_hash=raw[40:72].hex(), bytes=len(raw),
                        sha256=hashlib.sha256(raw).hexdigest(), reproduction="PASS",
                        hit_bit_mismatches=[0, 0, 0]))
assert len({r["seed"] for r in results}) == 3
assert len({r["initial_conditions_hash"] for r in results}) == 3

tests = {}
for mode in ["debug", "release"]:
    text = (audit / f"tests_{mode}.log").read_text(encoding="utf-8-sig")
    counts = re.findall(r"test result: .*? (\d+) passed; (\d+) failed", text)
    tests[mode] = dict(passed=sum(int(a) for a, _ in counts), failed=sum(int(b) for _, b in counts))
    failures = re.findall(r"^test (\S+) \.\.\. FAILED$", text, re.M)
    assert sorted(failures) == sorted([
        "regression_256_all_initial_states_match_legacy_and_saved_baseline",
        "regression_4096_all_initial_states_match_legacy_and_saved_baseline"])
    tests[mode]["failures"] = failures

before = json.loads((audit / "before_hashes.json").read_text(encoding="utf-8-sig"))
changed = [item["path"] for item in before if
           hashlib.sha256((project / item["path"]).read_bytes()).hexdigest().upper() != item["sha256"]]
expected_changes = {"src/main.rs", "src/experiments/mod.rs", "src/standard_run/mod.rs",
                    "src/standard_run/common.rs", "src/standard_run/seeded.rs", "README.md"}
assert {p.replace("\\", "/") for p in changed} == expected_changes

grid_log = (audit / "chi_grid.log").read_text(encoding="utf-8-sig")
grid = [float(v) for v in re.findall(r"^chi index -?\d+: (.*)$", grid_log, re.M)]
assert len(grid) == 153 and grid[0] == -0.999 and grid[76] == 0.0 and grid[-1] == 0.999
assert all(a < b for a, b in zip(grid, grid[1:]))
assert all(grid[76-i] == -grid[76+i] for i in range(1, 77))
sampling = dict(count=len(grid), first=grid[0], center=grid[76], last=grid[-1],
                sign_symmetry_max_error=0.0, endpoints_error=0.0,
                center_gap=grid[77]-grid[76], endpoint_gap=grid[-1]-grid[-2])
report = dict(runs=results, tests=tests, changed_existing_files=changed, sampling=sampling,
              standard_generation_runs=3, standard_reproduction_runs=3,
              total_archive_bytes=sum(p.stat().st_size for p in directory.iterdir()))
(audit / "audit_results.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
print(json.dumps(report, indent=2))
