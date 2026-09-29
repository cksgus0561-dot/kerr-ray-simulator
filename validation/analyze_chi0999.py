"""Reuse the existing archive comparison functions on the new chi=+/-0.999 runs."""
import json
import sys
from functools import cache
from pathlib import Path

# Only import the already-audited function definitions, not the old fixed run list.
source = Path("validation/analyze_signed_spin.py").read_text(encoding="utf-8")
namespace = {"__name__": "archive_comparison"}
exec(compile(source.split("\nlabels = ")[0], "validation/analyze_signed_spin.py", "exec"), namespace)
namespace["load"] = cache(namespace["load"])
load, compare = namespace["load"], namespace["compare"]
root = Path(sys.argv[1])
labels = [f"{sign}0999_{step}" for sign in ["p","n"] for step in "ABC"]
labels += [f"mirror_n0999_{step}" for step in "ABC"]
runs = {label:load(label)[0] for label in labels}
base = load("p0999_A")
old_root = Path("validation/stable_delta_20260926")
old_measure = json.loads((old_root/"p09999_A/measurement.json").read_text())
old_inputs = [d["input"] for d in json.loads((old_root/"p09999_A/diagnostics.json").read_text())]
assert [d["input"] for d in base[3]] == old_inputs
for label in labels[:6]:
    current = load(label)
    assert current[0]["seed_hex"] == old_measure["seed_hex"]
    assert current[0]["initial_conditions_hash"] == old_measure["initial_conditions_hash"]
    assert [d["input"] for d in current[3]] == old_inputs
comparisons = [compare(f"{s}0999_{a}", f"{s}0999_{b}")
               for s in ["p","n"] for a,b in [("A","B"),("B","C"),("A","C")]]
comparisons += [compare(f"p0999_{step}",f"mirror_n0999_{step}",True) for step in "ABC"]
for c in comparisons:
    c["cap_num_changes"] = sum(set(x["status"]) == {2,4} for x in c["changes"])
    c["det_classification_changes"] = sum(1 in x["status"] for x in c["changes"])
result = {"runs":runs,"comparisons":comparisons,"identical_inputs_to_previous_chi09999":True}
(root/"analysis.json").write_text(json.dumps(result,indent=2),encoding="utf-8")
for label,m in runs.items():
    print(label,m["counts"],"seconds",m["physics_seconds"],"max_null",m["max_null"])
for c in comparisons:
    print(c["left"],c["right"],"status",c["status_changes"],"max",c["max_abs_uvt"],"mean",c["mean_abs_uvt"])
