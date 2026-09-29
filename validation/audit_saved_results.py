"""Read-only audit of the shipped datasets. Requires Python, numpy and Pillow.

Run from any directory: python validation/audit_saved_results.py
Does not integrate geodesics or change dataset files. Writes only audit evidence
in validation/. These Python packages are not Rust runtime dependencies.
"""
from pathlib import Path
from collections import Counter
import csv
import hashlib
import json
import math
import sys
import zipfile
import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
VALIDATION = ROOT / "validation"


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def rows(path):
    with path.open(newline="", encoding="utf-8-sig") as file:
        return list(csv.DictReader(file))


def fnv(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f"{value:016x}"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def saved_manifest():
    return {
        p.relative_to(ROOT).as_posix(): sha(p)
        for name in ["source_to_detector", "source_to_detector_dt01", "benchmark_detector"]
        for p in sorted((ROOT / "results" / name).rglob("*")) if p.is_file()
    }


before = saved_manifest()
manifest_path = VALIDATION / "saved_results_sha256.json"
if manifest_path.exists():
    assert before == read_json(manifest_path), "Saved dataset changed since final audit"
else:
    manifest_path.write_text(json.dumps(before, indent=2) + "\n", encoding="utf-8")

baseline = {}
for name, expected in read_json(ROOT / "tests/fixtures/baseline_test_hashes.json").items():
    baseline[name] = sha(ROOT / "tests" / name) == expected
with zipfile.ZipFile(ROOT.parent / "kerr-ray-baseline-m1-m3.zip") as archive:
    name = next(n for n in archive.namelist() if n.endswith("src/physics/integrator.rs"))
    baseline["src/physics/integrator.rs"] = (
        archive.read(name) == (ROOT / "src/physics/integrator.rs").read_bytes()
    )
assert all(baseline.values()), baseline

source = ROOT / "results/source_to_detector"
metadata = read_json(source / "run_metadata.json")
raw = (source / "detector_events.csv").read_bytes()
assert fnv(raw) == metadata["events_fnv1a64"]
events = rows(source / "detector_events.csv")
diagnostics = rows(source / "ray_diagnostics.csv")
hits = rows(source / "detector_hit_states.csv")
assert len(events) == len(hits) == 216
assert len(diagnostics) == metadata["ray_count"] == 256
ids = {int(e["ray_id"]) for e in events}
assert len(ids) == len(events)
counts = Counter(d["status"] for d in diagnostics)
assert all(counts[k] == v for k, v in metadata["outcomes"].items())
assert ids == {int(d["ray_id"]) for d in diagnostics if d["status"] == "Detected"}
for e in events:
    assert e["status"] == "Detected"
    assert all(math.isfinite(float(v)) for k, v in e.items() if k != "status")
    assert float(e["t_emit"]) == 0.0
    assert float(e["delta_t"]) == float(e["t_hit"]) - float(e["t_emit"])
    assert float(e["delta_t"]) >= 0

error_columns = {"null_abs": "max_null_abs", "E_relative": "max_E_relative",
                 "Lz_relative": "max_Lz_relative", "Q_abs": "max_Q_abs",
                 "Q_relative": "max_Q_relative"}
errors = {key: max(float(d[col]) for d in diagnostics) for key, col in error_columns.items()}
assert errors == metadata["max_errors"]
by_id = {int(e["ray_id"]): e for e in events}
plane = metadata["experiment"]["detector"]["plane"]
spin = metadata["experiment"]["spin"]
max_residual = 0.0
for hit in hits:
    e = by_id[int(hit["ray_id"])]
    assert all(math.isfinite(float(v)) for v in hit.values())
    assert float(hit["t"]) == float(e["t_hit"])
    r, theta, phi = (float(hit[k]) for k in ["r", "theta", "phi"])
    rho = math.sqrt(r*r + spin*spin)
    x = [rho*math.sin(theta)*math.cos(phi), rho*math.sin(theta)*math.sin(phi), r*math.cos(theta)]
    relative = [v-c for v, c in zip(x, plane["center"])]
    dot = lambda basis: sum(a*b for a, b in zip(relative, basis))
    distance = dot(plane["normal"])
    max_residual = max(max_residual, abs(distance))
    assert abs(distance) <= metadata["experiment"]["detector"]["root_tolerance"] + 2e-13
    assert abs(distance - float(hit["root_signed_residual"])) < 2e-13
    assert abs(dot(plane["e_u"]) - float(e["u_hit"])) < 2e-13
    assert abs(dot(plane["e_v"]) - float(e["v_hit"])) < 2e-13


def audit_derived(folder):
    derived = read_json(folder / "derived_metadata.json")
    assert derived["source_events_fnv1a64"] == fnv(raw)
    nx, ny = derived["resolution"]
    n = derived["frame_count"]
    config = derived["time_bins"]
    dt, start, end = (config[k] for k in ["width", "start", "end"])
    accumulated = np.zeros((ny, nx), dtype=np.uint64)
    expected = np.zeros((n, ny, nx), dtype=np.uint64)
    before_window = np.zeros_like(accumulated)
    after = 0
    for e in events:
        u, v = float(e["u_hit"]), float(e["v_hit"])
        assert abs(u) <= plane["width"] / 2 and abs(v) <= plane["height"] / 2
        x = min(nx-1, math.floor((u/plane["width"]+.5)*nx))
        y = min(ny-1, math.floor((.5-v/plane["height"])*ny))
        accumulated[y, x] += 1
        t = float(e["t_hit"] if config["basis"] == "Arrival" else e["delta_t"])
        if t < start:
            before_window[y, x] += 1
        elif t >= end:
            after += 1
        else:
            index = (t-start)/dt
            nearest = round(index)
            if abs(index-nearest) <= 4*sys.float_info.epsilon*max(1, abs(index)):
                index = nearest
            expected[min(n-1, math.floor(index)), y, x] += 1
    actual = np.loadtxt(folder / "detector_accumulated.csv", delimiter=",", dtype=np.uint64, ndmin=2)
    assert np.array_equal(accumulated, actual)
    actual_before = np.loadtxt(folder / "detector_before_window.csv", delimiter=",", dtype=np.uint64, ndmin=2)
    assert np.array_equal(before_window, actual_before)
    sparse = np.zeros_like(expected)
    for entry in rows(folder / "detector_time_bins.csv"):
        f, x, y, count = (int(entry[k]) for k in ["frame", "x_pixel", "y_pixel", "count"])
        assert sparse[f, y, x] == 0 and count > 0
        sparse[f, y, x] = count
    assert np.array_equal(sparse, expected)
    assert int(expected.sum()) == derived["analysis_window_count"]
    assert int(before_window.sum()) == derived["before_window_count"]
    assert after == derived["after_window_count"]
    assert int(accumulated.sum()) == derived["inside_detector_count"] == len(events)
    assert int(expected.sum()) + int(before_window.sum()) + after == len(events)
    white = derived["postprocess"]["counts_per_white"]
    encode = lambda data: (np.minimum(data, white)*65535//white).astype(np.uint16)
    with Image.open(folder / "detector_accumulated.png") as img:
        assert np.array_equal(np.asarray(img), encode(accumulated))
    index_rows = rows(folder / "frame_index.csv")
    assert len(index_rows) == n
    cumulative = before_window.copy()
    fps = derived["postprocess"]["playback_fps"]
    for mode in ["instantaneous", "cumulative"]:
        frames_dir = folder / "detector_frames" / mode
        assert len(list(frames_dir.glob("frame_*.png"))) == n
        cumulative = before_window.copy()
        with Image.open(folder / f"detector_{mode}.apng") as animation:
            assert animation.n_frames == n
            for i in range(n):
                cumulative += expected[i]
                frame_row = index_rows[i]
                assert int(frame_row["frame"]) == i
                assert math.isclose(float(frame_row["time_start"]), start+i*dt, abs_tol=1e-12)
                assert math.isclose(float(frame_row["time_end"]), min(end, start+(i+1)*dt), abs_tol=1e-12)
                assert int(frame_row["new_count"]) == int(expected[i].sum())
                assert int(frame_row["cumulative_count"]) == int(cumulative.sum())
                pixels = encode(expected[i] if mode == "instantaneous" else cumulative)
                with Image.open(frames_dir / f"frame_{i:06}.png") as image:
                    assert image.size == (nx, ny)
                    assert np.array_equal(np.asarray(image), pixels)
                animation.seek(i)
                assert abs(animation.info["duration"] - 1000/fps) < 1e-9
                assert np.array_equal(np.asarray(animation), pixels)
    return {"events": len(events), "accumulated_count": int(accumulated.sum()),
            "time_bin_count_sum": int(expected.sum()), "frames_per_mode": n,
            "checked_png_frames": 2*n, "checked_apng_frames": 2*n,
            "resolution": [nx, ny], "physical_dt": dt, "playback_fps": fps,
            "all_numeric_and_image_pixels_match_raw_events": True}


derived_reports = {name: audit_derived(ROOT / "results" / name)
                   for name in ["source_to_detector", "source_to_detector_dt01"]}
assert saved_manifest() == before, "Audit changed a saved result"
report = {"baseline_files_unchanged": baseline, "saved_files_unchanged": len(before),
          "geodesics_reintegrated": False, "raw_events_fnv1a64": fnv(raw),
          "raw_events_sha256": hashlib.sha256(raw).hexdigest(),
          "ray_count": 256, "outcomes": metadata["outcomes"], "max_errors": errors,
          "max_recomputed_detector_signed_residual": max_residual,
          "datasets": derived_reports}
(VALIDATION / "saved_results_audit.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
print(json.dumps(report, indent=2))
