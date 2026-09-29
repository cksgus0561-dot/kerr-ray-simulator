"""Add exact official crates to the existing offline directory source."""
import hashlib, io, json, pathlib, tarfile, urllib.request
root = pathlib.Path(__file__).resolve().parents[1]
crates = {'rand_chacha':'0.9.0','rand_core':'0.9.3','ppv-lite86':'0.2.21',
          'sha2':'0.10.9','digest':'0.10.7','block-buffer':'0.10.4',
          'crypto-common':'0.1.7','generic-array':'0.14.7','typenum':'1.19.0',
          'cpufeatures':'0.2.17'}
for name, version in crates.items():
    dirname = f'{name}-{version}'
    dest = root / 'vendor_viz' / dirname
    if dest.exists():
        continue
    raw = urllib.request.urlopen(f'https://static.crates.io/crates/{name}/{dirname}.crate').read()
    api = json.load(urllib.request.urlopen(f'https://crates.io/api/v1/crates/{name}/{version}'))
    checksum = hashlib.sha256(raw).hexdigest()
    assert checksum == api['version']['checksum'], dirname
    with tarfile.open(fileobj=io.BytesIO(raw)) as archive:
        archive.extractall(root / 'vendor_viz', filter='data')
    files = {p.relative_to(dest).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()
             for p in dest.rglob('*') if p.is_file()}
    (dest / '.cargo-checksum.json').write_text(json.dumps({'files':files,'package':checksum}))
    print(dirname, checksum, flush=True)
