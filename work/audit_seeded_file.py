"""Independent format/hash audit for default-axis files; no geodesic calculation."""
import hashlib, json, pathlib, struct, sys

def words(seed):
    key = list(struct.unpack('<8I', seed))
    block = 0
    while True:
        state = [0x61707865,0x3320646e,0x79622d32,0x6b206574] + key + [block & 0xffffffff, block >> 32, 0, 0]
        x = state.copy()
        def rotate(v,n): return ((v << n) | (v >> (32-n))) & 0xffffffff
        def quarter(a,b,c,d):
            x[a]=(x[a]+x[b])&0xffffffff; x[d]=rotate(x[d]^x[a],16)
            x[c]=(x[c]+x[d])&0xffffffff; x[b]=rotate(x[b]^x[c],12)
            x[a]=(x[a]+x[b])&0xffffffff; x[d]=rotate(x[d]^x[a],8)
            x[c]=(x[c]+x[d])&0xffffffff; x[b]=rotate(x[b]^x[c],7)
        for _ in range(10):
            for q in [(0,4,8,12),(1,5,9,13),(2,6,10,14),(3,7,11,15),(0,5,10,15),(1,6,11,12),(2,7,8,13),(3,4,9,14)]: quarter(*q)
        out=[(a+b)&0xffffffff for a,b in zip(x,state)]
        for i in range(0,16,2): yield out[i] | out[i+1]<<32
        block+=1

directory=pathlib.Path(sys.argv[1])
common=json.loads((directory/'common.json').read_text())
raw=(directory/'sim_1.bin').read_bytes()
g=common['ray_generator']; n1,n2=g['cell_count']; N=n1*n2
chi,=struct.unpack_from('<d',raw); seed=raw[8:40]; stored_hash=raw[40:72]
assert g['fixed_direction']==[-1.,0.,0.], 'audit normalization restricted to default axis direction'
rng=words(seed); rays=[]
for j in range(n2):
    for i in range(n1):
        u=(next(rng)>>11)*2**-53; v=(next(rng)>>11)*2**-53
        s1=(i-n1*.5)*g['cell_size'][0]+u*g['cell_size'][0]
        s2=(j-n2*.5)*g['cell_size'][1]+v*g['cell_size'][1]
        p=[g['center'][k]+(g['axis_1'][k]*s1+g['axis_2'][k]*s2) for k in range(3)]
        rays.append(tuple(0. if x==0. else x for x in p+g['fixed_direction']+[g['t_emit']]))
h=hashlib.sha256(b''.join(struct.pack('<7d',*r) for r in sorted(rays))).digest()
assert h==stored_hash, 'independent Python canonical hash mismatch'
status=raw[72:72+N]; assert len(status)==N and all(x<=4 for x in status)
K=status.count(1); assert len(raw)==72+N+24*K
hits=list(struct.iter_unpack('<3d',raw[72+N:])); assert len(hits)==K
report=dict(directory=str(directory),chi=chi,physical_rays=N,counts=[status.count(i) for i in range(5)],
    seed_hex=seed.hex(),initial_conditions_sha256=h.hex(),independent_python_hash='PASS',
    binary_bytes=len(raw),expected_binary_bytes=72+N+24*K,common_bytes=(directory/'common.json').stat().st_size,
    files=sorted(p.name for p in directory.iterdir()))
print(json.dumps(report,indent=2))
