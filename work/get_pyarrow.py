import json,urllib.request,pathlib,zipfile,hashlib
root=pathlib.Path('work/python-parquet');root.mkdir(exist_ok=True)
data=json.load(urllib.request.urlopen('https://pypi.org/pypi/pyarrow/json',timeout=30))
wheel=next(x for x in data['urls'] if 'cp312-cp312-win_amd64.whl' in x['filename'])
p=root/wheel['filename']
if not p.exists(): urllib.request.urlretrieve(wheel['url'],p)
assert hashlib.sha256(p.read_bytes()).hexdigest()==wheel['digests']['sha256']
with zipfile.ZipFile(p) as z:z.extractall(root/'site-packages')
print('Local compatibility-check dependency:',p.name,p.stat().st_size)
