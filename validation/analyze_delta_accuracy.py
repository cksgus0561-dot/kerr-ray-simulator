"""80-digit reference from exact binary64 inputs, outside the simulator."""
import json
import sys
from decimal import Decimal as D, getcontext
from pathlib import Path

getcontext().prec = 80
root = Path(sys.argv[1])

def sincos(x):
    s,c,ts,tc = x,D(1),x,D(1)
    for n in range(1,180):
        ts *= -x*x/D((2*n)*(2*n+1))
        tc *= -x*x/D((2*n-1)*(2*n))
        s,c=s+ts,c+tc
        if abs(ts)+abs(tc)<D("1e-78"):
            break
    return s,c

def reference(case):
    a=D.from_float(case["chi"])
    t,r,phi,pt,pr,pp,th,pth=map(D.from_float,case["state"])
    sn,cs=sincos(th)
    sigma=r*r+a*a*cs*cs
    delta=r*r-2*r+a*a
    area=(r*r+a*a)**2-a*a*delta*sn*sn
    g=[[D(0) for _ in range(4)] for _ in range(4)]
    inv=[[D(0) for _ in range(4)] for _ in range(4)]
    g[0][0]=-1+2*r/sigma
    g[0][3]=g[3][0]=-2*a*r*sn*sn/sigma
    g[1][1]=sigma/delta
    g[2][2]=sigma
    g[3][3]=area*sn*sn/sigma
    inv[0][0]=-area/(sigma*delta)
    inv[0][3]=inv[3][0]=-2*a*r/(sigma*delta)
    inv[1][1]=delta/sigma
    inv[2][2]=1/sigma
    inv[3][3]=(delta-a*a*sn*sn)/(sigma*delta*sn*sn)
    p=[pt,pr,pth,pp]
    null=sum(inv[i][j]*p[i]*p[j] for i in range(4) for j in range(4))
    return delta,g,inv,null

summary={}
for phase in ["before","after"]:
    path=root/(phase+".json")
    if not path.exists():
        continue
    cases=json.loads(path.read_text())
    result={}
    for group in ["failed_endpoints","grid"]:
        selected=[c for c in cases if c["label"].startswith("grid_")== (group=="grid")]
        refs=[reference(c) for c in selected]
        metrics={}
        for name in ["old_delta","factored_delta","factored_fma_delta","compensated_delta","actual_delta"]:
            errors=[abs(D.from_float(c[name])-ref[0]) for c,ref in zip(selected,refs)]
            relative=[x/abs(ref[0]) for x,ref in zip(errors,refs)]
            metrics[name]={"max_abs":float(max(errors)),"max_rel":float(max(relative)),
                           "mean_abs":float(sum(errors)/len(errors))}
        for name,column in [("covariant",1),("inverse",2)]:
            entries=[(D.from_float(c[name][i][j]),ref[column][i][j])
                     for c,ref in zip(selected,refs) for i in range(4) for j in range(4)]
            metrics[name]={"max_abs":float(max(abs(x-y) for x,y in entries)),
                           "max_rel_nonzero":float(max(abs((x-y)/y) for x,y in entries if y))}
        null_errors=[abs(D.from_float(c["null"])-ref[3]) for c,ref in zip(selected,refs)]
        metrics["null"]={"max_abs_error":float(max(null_errors)),
                         "mean_abs_error":float(sum(null_errors)/len(null_errors))}
        result[group]={"count":len(selected),"metrics":metrics}
    summary[phase]=result
(root/"accuracy_summary.json").write_text(json.dumps(summary,indent=2),encoding="utf-8")
print(json.dumps(summary,indent=2))
