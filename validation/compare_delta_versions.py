"""Compare immutable previous archives to the stable-Delta results."""
import json
import struct
from pathlib import Path

root=Path("validation/stable_delta_20260926")
def read(path,n=36864):
    data=path.read_bytes()
    status=data[72:72+n]
    offset=72+n
    hits=[]
    for code in status:
        h=struct.unpack_from("<ddd",data,offset) if code==1 else None
        if h: offset+=24
        hits.append(h)
    assert offset==len(data)
    return data[8:72],status,hits

report={}
for label in ["p09999_A","p09999_B","p09999_C","n09999_A","n09999_B","n09999_C","default006"]:
    old=(Path("validation/parallel_measurements_20260926/larger") if label=="default006"
         else Path("validation/signed_spin_20260926")/label)
    previous=read(old/"simulation_1/sim_1.bin")
    current=read(root/label/"simulation_1/sim_1.bin")
    assert previous[0]==current[0] # seed and canonical initial-condition hash
    mismatches=[]
    for i,(a,b) in enumerate(zip(previous[2],current[2])):
        if a is not None and b is not None:
            errors=[abs(x-y) for x,y in zip(a,b)]
            bad=[j for j in range(3) if errors[j] > 1e-9+1e-12*max(abs(a[j]),abs(b[j]))]
            if bad: mismatches.append({"ray_id":i,"components":bad,"abs_uvt":errors})
    report[label]={
        "status_changes":sum(a!=b for a,b in zip(previous[1],current[1])),
        "old_num":list(previous[1]).count(4),"new_num":list(current[1]).count(4),
        "old_num_now_cap":sum(a==4 and b==2 for a,b in zip(previous[1],current[1])),
        "cross_version_hit_policy_mismatches":mismatches
    }
(root/"cross_version.json").write_text(json.dumps(report,indent=2),encoding="utf-8")
print(json.dumps(report,indent=2))
