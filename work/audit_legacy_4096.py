"""Compare the already timed legacy run with the untouched 4096-ray CSV oracle."""
import csv, json, pathlib, struct, sys
sys.path.insert(0,str(pathlib.Path('work/python-parquet/site-packages').resolve()))
import pyarrow.parquet as pq
base=pathlib.Path('results/visualization_4096')
with (base/'ray_diagnostics.csv').open() as f:
    statuses={int(r['ray_id']):r['status'] for r in csv.DictReader(f)}
with (base/'detector_events.csv').open() as f:
    hits={int(r['ray_id']):[float(r[x]) for x in ['u_hit','v_hit','t_hit']] for r in csv.DictReader(f)}
rows=pq.read_table('results/seeded_validation_20260925/legacy/simulation_1/sim_1.parquet').to_pylist()
mapping={'ACT':'Active','DET':'Detected','CAP':'Captured','ESC':'Escaped','NUM':'NumericalFailure'}
status_errors=0; bit_errors=[0,0,0]; maximum=[0.,0.,0.]; seen=set()
for r in rows:
    i=round((r['position_y']+15.75)/.5); j=round((r['position_z']+15.75)/.5)
    assert r['position_x']==80. and r['position_y']==(i+.5)*.5-16. and r['position_z']==(j+.5)*.5-16.
    assert (r['direction_x'],r['direction_y'],r['direction_z'],r['t_emit'])==(-1.,0.,0.,0.)
    old_id=j*64+i; assert old_id not in seen;seen.add(old_id)
    status_errors+=mapping[r['status']]!=statuses[old_id]
    expected=hits.get(old_id)
    if expected is None: assert all(r[k] is None for k in ['u_hit','v_hit','t_hit'])
    else:
        for k,(a,b) in enumerate(zip(expected,[r['u_hit'],r['v_hit'],r['t_hit']])):
            maximum[k]=max(maximum[k],abs(a-b)); bit_errors[k]+=struct.pack('<d',a)!=struct.pack('<d',b)
assert len(seen)==4096 and status_errors==0 and bit_errors==[0,0,0]
print(json.dumps(dict(rays=4096,position_direction_time_matches=4096,status_mismatches=status_errors,
    detected_hits_compared=len(hits),hit_bit_mismatches=bit_errors,hit_max_abs=maximum,result='PASS'),indent=2))
