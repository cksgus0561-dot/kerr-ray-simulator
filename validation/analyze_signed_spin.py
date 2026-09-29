"""Read validation artifacts only; no physics model or acceptance policy."""
import json
import math
import struct
import sys
from collections import Counter
from pathlib import Path

root = Path(sys.argv[1])

def load(label):
    directory = root / label
    measure = json.loads((directory / "measurement.json").read_text())
    data = (directory / "simulation_1/sim_1.bin").read_bytes()
    n = measure["rays"]
    chi, = struct.unpack_from("<d", data)
    assert chi == measure["chi"]
    statuses = data[72:72+n]
    offset = 72+n
    hits = []
    for status in statuses:
        hit = struct.unpack_from("<ddd", data, offset) if status == 1 else None
        if hit:
            offset += 24
            assert all(map(math.isfinite, hit))
        hits.append(hit)
    assert offset == len(data)
    diagnostics = json.loads((directory / "diagnostics.json").read_text())
    assert all(d["ray_id"] == i for i, d in enumerate(diagnostics))
    return measure, statuses, hits, diagnostics

def compare(left, right, mirror=False):
    a, b = load(left), load(right)
    if mirror:
        index = {tuple(d["input"]): i for i, d in enumerate(b[3])}
        mapping = []
        for d in a[3]:
            p = d["input"].copy()
            p[1], p[4] = -p[1], -p[4]
            mapping.append(index[tuple(p)])
    else:
        assert [d["input"] for d in a[3]] == [d["input"] for d in b[3]]
        assert a[0]["initial_conditions_hash"] == b[0]["initial_conditions_hash"]
        mapping = list(range(len(a[1])))
    changes, differences = [], []
    for i, j in enumerate(mapping):
        if a[1][i] != b[1][j]:
            changes.append({"ray_id": i, "other_id": j,
                            "status": [a[1][i], b[1][j]],
                            "a": a[3][i], "b": b[3][j]})
        if a[2][i] is not None and b[2][j] is not None:
            h = list(b[2][j])
            if mirror:
                h[0] = -h[0]
            errors = [abs(x-y) for x,y in zip(a[2][i], h)]
            differences.append((i,j,errors))
    maximum = [max(d[2][k] for d in differences) for k in range(3)]
    mean = [math.fsum(d[2][k] for d in differences)/len(differences) for k in range(3)]
    worst = []
    for k in range(3):
        i,j,delta = max(differences, key=lambda d:d[2][k])
        worst.append({"component":k, "ray_id":i, "other_id":j, "delta":delta,
                      "hit_a":a[2][i], "hit_b":b[2][j], "a":a[3][i],"b":b[3][j]})
    return {"left":left, "right":right, "mirror":mirror, "common_detected":len(differences),
            "status_changes":len(changes),"changes":changes,
            "max_abs_uvt":maximum, "mean_abs_uvt":mean, "worst":worst}

labels = ["p099_A","n099_A","p09999_A","p09999_B","p09999_C",
          "n09999_A","n09999_B","n09999_C","mirror_n099","mirror_n09999"]
runs = {}
for label in labels:
    a = load(label)
    failures = [d for d in a[3] if d["status"] == "NumericalFailure"]
    runs[label] = {**a[0], "failures":failures,
                   "failure_reasons":dict(Counter(d["reason"] for d in failures))}
comparisons = [compare(f"{s}_{a}",f"{s}_{b}")
               for s in ["p09999","n09999"] for a,b in [("A","B"),("B","C"),("A","C")]]
comparisons += [compare("p099_A","mirror_n099",True),
                compare("p09999_A","mirror_n09999",True)]
for label in labels[:8]:
    assert runs[label]["seed_hex"] == runs[labels[0]]["seed_hex"]
    assert runs[label]["initial_conditions_hash"] == runs[labels[0]]["initial_conditions_hash"]
result = {"runs":runs,"comparisons":comparisons}
(root/"analysis.json").write_text(json.dumps(result,indent=2),encoding="utf-8")
for name,m in runs.items():
    print(name,m["counts"],"physics",round(m["physics_seconds"],4),"failures",m["failure_reasons"])
for c in comparisons:
    print(c["left"],c["right"],"status",c["status_changes"],"DET",c["common_detected"],
          "max",c["max_abs_uvt"],"mean",c["mean_abs_uvt"])
