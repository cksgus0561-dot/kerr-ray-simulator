"""High precision evaluation of problematic saved endpoint states, not a solver."""
import json
import math
import sys
from collections import Counter
from decimal import Decimal as D, getcontext
from pathlib import Path

getcontext().prec = 80
root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("validation/signed_spin_20260926")
report = json.loads((root/"analysis.json").read_text())

def sincos(x):
    s, c, ts, tc = x, D(1), x, D(1)
    for n in range(1, 180):
        ts *= -x*x / D((2*n)*(2*n+1))
        tc *= -x*x / D((2*n-1)*(2*n))
        s, c = s+ts, c+tc
        if abs(ts)+abs(tc) < D("1e-78"):
            break
    return s,c

def residual(a, state, rounded_delta=False):
    # Evaluate the documented inverse Kerr quadratic form with 80 digits,
    # from the exact binary64 endpoint values; no state correction or integration.
    t,r,phi,pt,pr,pp,th,pth = map(D.from_float, state)
    a = D.from_float(a)
    sn,cs = sincos(th)
    sigma = r*r+a*a*cs*cs
    delta = r*r-2*r+a*a
    if rounded_delta:
        rf, af = state[1], float(a)
        delta = D.from_float(rf*rf-2.0*rf+af*af)
    area = (r*r+a*a)**2-a*a*delta*sn*sn
    terms = [-area/(sigma*delta)*pt*pt,
             -4*a*r/(sigma*delta)*pt*pp,
             delta/sigma*pr*pr, pth*pth/sigma,
             (delta-a*a*sn*sn)/(sigma*delta*sn*sn)*pp*pp]
    return float(sum(terms)), float(sum(map(abs,terms))), float(delta)

summary = {}
for label,m in report["runs"].items():
    failures = m["failures"]
    if not failures:
        continue
    rplus = 1+math.sqrt(1-m["chi"]**2)
    samples = []
    for f in failures:
        high, cancellation, delta = residual(m["chi"],f["last_state"])
        with_delta64 = residual(m["chi"],f["last_state"],True)[0]
        samples.append({"ray_id":f["ray_id"],"r_minus_horizon":f["last_state"][1]-rplus,
                        "max_null_f64":f["max_null"],"null_80digits":high,
                        "sum_abs_quadratic_terms":cancellation,"delta":delta,
                        "null_80digits_using_f64_delta":with_delta64,
                        "abs_f64_delta_effect":abs(with_delta64-high)})
    summary[label] = {
        "count":len(failures),
        "r_minus_horizon_range":[min(x["r_minus_horizon"] for x in samples),max(x["r_minus_horizon"] for x in samples)],
        "last_null_80digit_abs_range":[min(abs(x["null_80digits"]) for x in samples),max(abs(x["null_80digits"]) for x in samples)],
        "null_80digit_above_1e_minus5":sum(abs(x["null_80digits"])>1e-5 for x in samples),
        "min_polar_distance":min(f["min_polar_distance"] for f in failures),
        "examples":samples[:3]
    }
    print(label,json.dumps(summary[label]))
for c in report["comparisons"]:
    if c["mirror"]:
        continue
    print(c["left"],c["right"],"transitions",dict(Counter(str(x["status"]) for x in c["changes"])))
    for w in c["worst"]:
        print("worst",w["component"],"ray",w["ray_id"],"minr",w["a"]["min_r"],
              "minpolar",w["a"]["min_polar_distance"],"hit",w["hit_a"],
              "null",w["a"]["max_null"])
(root/"failure_diagnosis.json").write_text(json.dumps(summary,indent=2),encoding="utf-8")
