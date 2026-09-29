import pathlib,tomllib,shutil
root=pathlib.Path.cwd(); dest=root/'vendor_viz'
def key(p):
 d=tomllib.loads((p/'Cargo.toml').read_text(encoding='utf-8'))['package'];return d['name'],d['version']
existing={key(p) for p in dest.iterdir() if (p/'Cargo.toml').exists()}
added=[]
for p in (root/'../../work/parquet-deps/vendor').resolve().iterdir():
 if key(p) not in existing:
  shutil.copytree(p,dest/p.name);added.append(p.name)
print('Added vendor packages:',added)
