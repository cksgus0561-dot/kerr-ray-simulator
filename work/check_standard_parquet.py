import pathlib,sys,json,csv,struct,collections
root=pathlib.Path.cwd()
sys.path.insert(0,str(root/'work/python-parquet/site-packages'))
import pyarrow as pa
import pyarrow.parquet as pq
import pandas as pd
reports=[]
for folder,side,old in [('simulation_1',16,'source_to_detector'),('simulation_2',64,'visualization_4096')]:
 path=root/'results/standard_validation'/folder/'sim_1.parquet'
 table=pq.read_table(path); md=table.schema.metadata
 names=['position_x','position_y','position_z','direction_x','direction_y','direction_z','energy','t_emit','status','u_hit','v_hit','t_hit']
 assert table.column_names==names
 assert all(pa.types.is_float64(table.schema.field(n).type) for n in names if n!='status')
 assert pa.types.is_string(table.schema.field('status').type)
 assert all(table.schema.field(n).nullable==(n in ['u_hit','v_hit','t_hit']) for n in names)
 assert float(md[b'chi'])==0.6 and md[b'chi_type']==b'float64'
 assert table.num_rows==side*side
 rows=table.to_pylist(); counts=collections.Counter(r['status'] for r in rows)
 with open(root/'results'/old/'ray_diagnostics.csv',newline='') as f: baseline=list(csv.DictReader(f))
 with open(root/'results'/old/'detector_events.csv',newline='') as f: events={int(r['ray_id']):r for r in csv.DictReader(f)}
 # Physical initial positions determine correspondence; historical IDs only join old CSVs.
 legacy={((u+.5)/side*32-16,(v+.5)/side*32-16):v*side+u for v in range(side) for u in range(side)}
 max_errors=[0.,0.,0.]; hit_bit_mismatches=[0,0,0]
 for r in rows:
  idx=legacy[r['position_y'],r['position_z']]
  assert r['position_x']==80. and [r[x] for x in ['direction_x','direction_y','direction_z']]==[-1.,0.,0.]
  assert r['energy']==1. and r['t_emit']==0.
  assert r['status']==baseline[idx]['status']
  for j,key in enumerate(['u_hit','v_hit','t_hit']):
   if r['status']=='Detected':
    expected=float(events[idx][key]); max_errors[j]=max(max_errors[j],abs(r[key]-expected))
    hit_bit_mismatches[j]+=struct.pack('d',r[key])!=struct.pack('d',expected)
   else: assert r[key] is None
 df=pd.read_parquet(path,engine='pyarrow')
 assert len(df)==side*side and all(str(df[n].dtype)=='float64' for n in names if n!='status')
 reports.append({'folder':folder,'rows':len(rows),'counts':dict(counts),'schema':str(table.schema),
  'chi':float(md[b'chi']),'hit_max_abs_vs_existing':max_errors,'hit_bit_mismatches_vs_existing':hit_bit_mismatches,
  'null_hits':table.column('u_hit').null_count,'pandas_dtypes':{n:str(t) for n,t in df.dtypes.items()},
  'compression':pq.ParquetFile(path).metadata.row_group(0).column(0).compression})
result={'pyarrow':pa.__version__,'pandas':pd.__version__,'reports':reports}
(root/'validation/standard_run/python_compatibility.json').write_text(json.dumps(result,indent=2))
print(json.dumps(result,indent=2))
