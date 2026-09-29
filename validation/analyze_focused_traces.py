"""80-digit endpoint/RHS evaluation and local f64 quantization sensitivity.

No high-precision geodesic integration, altered thresholds or state corrections.
"""
import json
import math
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
        if abs(ts)+abs(tc)<D("1e-78"): break
    return s,c

def reference(chi,state):
    a=D.from_float(chi)
    t,r,phi,pt,pr,pp,th,pth = map(D.from_float,state)
    sn,cs=sincos(th)
    s2=sn*sn
    sigma=r*r+a*a*cs*cs
    delta=r*r-2*r+a*a
    area=(r*r+a*a)**2-a*a*delta*s2
    g=[[D(0) for _ in range(4)] for _ in range(4)]
    inv=[[D(0) for _ in range(4)] for _ in range(4)]
    g[0][0]=-1+2*r/sigma
    g[0][3]=g[3][0]=-2*a*r*s2/sigma
    g[1][1]=sigma/delta
    g[2][2]=sigma
    g[3][3]=area*s2/sigma
    inv[0][0]=-area/(sigma*delta)
    inv[0][3]=inv[3][0]=-2*a*r/(sigma*delta)
    inv[1][1]=delta/sigma
    inv[2][2]=1/sigma
    inv[3][3]=(delta-a*a*s2)/(sigma*delta*s2)
    ds=[2*r,-2*a*a*sn*cs]; dd=[2*r-2,D(0)]
    darea=[4*r*(r*r+a*a)-a*a*dd[0]*s2,-2*a*a*delta*sn*cs]
    ds2=[D(0),2*sn*cs]; den=sigma*delta
    derivatives=[]
    for mu in range(2):
        dp=ds[mu]*delta+sigma*dd[mu]
        quotient=lambda n,np:(np*den-n*dp)/(den*den)
        dg=[[D(0) for _ in range(4)] for _ in range(4)]
        dg[0][0]=quotient(-area,-darea[mu])
        dg[0][3]=dg[3][0]=quotient(-2*a*r,-2*a if mu==0 else D(0))
        dg[1][1]=(dd[mu]*sigma-delta*ds[mu])/(sigma*sigma)
        dg[2][2]=-ds[mu]/(sigma*sigma)
        n=delta-a*a*s2; np=dd[mu]-a*a*ds2[mu]
        dg[3][3]=(np*den*s2-n*(dp*s2+den*ds2[mu]))/(den*s2)**2
        derivatives.append(dg)
    p=[pt,pr,pth,pp]
    contract=lambda matrix:sum(matrix[i][j]*p[i]*p[j] for i in range(4) for j in range(4))
    velocity=[sum(inv[i][j]*p[j] for j in range(4)) for i in range(4)]
    dc=[contract(dg) for dg in derivatives]
    rhs=[velocity[0],velocity[1],velocity[3],D(0),-dc[0]/2,D(0),velocity[2],-dc[1]/2]
    null=contract(inv)
    # First-order sensitivity to at most half one ULP in each stored state
    # component (the fixed input chi is not a state variable). Not a rigorous interval.
    grad=[D(0),dc[0],D(0),2*velocity[0],2*velocity[1],2*velocity[3],dc[1],2*velocity[2]]
    quant=sum(abs(d)*D.from_float(math.ulp(x))/2 for d,x in zip(grad,state))
    return delta,g,inv,null,rhs,quant

def matrix_errors(actual,ref):
    entries=[(D.from_float(actual[i][j]),ref[i][j]) for i in range(4) for j in range(4)]
    return {"max_abs":float(max(abs(x-y) for x,y in entries)),
            "max_rel_nonzero":float(max(abs((x-y)/y) for x,y in entries if y))}

traces=json.loads((root/"focused_traces.json").read_text())
summary=[]
for trace in traces:
    probes=[]
    for p in trace["probes"]:
        delta,g,inv,null,rhs,quant = reference(trace["chi"],p["state"])
        probes.append({
            "sample":p["sample"],"affine":p["affine"],"state":p["state"],
            "r_minus_horizon":p["state"][1]-trace["horizon"],
            "delta_f64":p["delta"],"delta_reference":float(delta),
            "delta_relative_error":float(abs(D.from_float(p["delta"])-delta)/abs(delta)),
            "null_f64":p["null"],"null_reference":float(null),
            "null_evaluation_error":float(abs(D.from_float(p["null"])-null)),
            "metric_error":matrix_errors(p["covariant"],g),
            "inverse_error":matrix_errors(p["inverse"],inv),
            "rhs_abs_error":[float(abs(D.from_float(x)-y)) for x,y in zip(p["rhs"],rhs)],
            "rhs_max_relative_nonzero":float(max(abs((D.from_float(x)-y)/y) for x,y in zip(p["rhs"],rhs) if y)),
            "null_half_ulp_state_sensitivity":float(quant),
        })
    tail=[p for p in probes if p["sample"]>=trace["accepted_steps"]-63]
    item={k:v for k,v in trace.items() if k!="probes"}
    item.update({
        "terminal":probes[-1],"previous":probes[-2],
        "initial_null_reference":probes[0]["null_reference"],
        "tail_max_null_evaluation_error":max(p["null_evaluation_error"] for p in tail),
        "tail_max_rhs_relative_error":max(p["rhs_max_relative_nonzero"] for p in tail),
        "probes":probes,
    })
    summary.append(item)
    p=probes[-1]
    print(trace["label"],trace["ray_id"],"gap",p["r_minus_horizon"],
          "null",p["null_f64"],p["null_reference"],"delta",p["delta_f64"],
          "delta_rel",p["delta_relative_error"],"metric_rel",p["metric_error"]["max_rel_nonzero"],
          "inverse_rel",p["inverse_error"]["max_rel_nonzero"],"RHS_rel",p["rhs_max_relative_nonzero"],
          "half_ulp_null",p["null_half_ulp_state_sensitivity"],
          "steps",trace["accepted_steps"],trace["rejected_steps"])
(root/"focused_precision.json").write_text(json.dumps(summary,indent=2),encoding="utf-8")
