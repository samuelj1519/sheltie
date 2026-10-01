import hashlib,json,pathlib,tarfile
base=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C002-v0.2.0-reliability/evidence/m1-2026-10-01/mutants')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
for stage in sorted(base.glob('stage*')):
 if not (stage/'metadata.json').exists() or not (stage/'raw-manifest.json').exists():continue
 if (stage/'raw.tar.gz').exists():continue
 raw=stage/'raw';expected=json.loads((stage/'raw-manifest.json').read_text())
 files={str(p.relative_to(raw)):p for p in raw.rglob('*') if p.is_file()}
 assert set(files)==set(expected),stage
 assert all(not p.is_symlink() for p in raw.rglob('*'))
 for key,p in files.items():assert expected[key]=={'sha256':sha(p),'bytes':p.stat().st_size},key
 temp=stage/'raw.tar.gz.part';assert not temp.exists()
 with tarfile.open(temp,'w:gz',compresslevel=6) as t:
  for key,p in sorted(files.items()):t.add(p,arcname='raw/'+key,recursive=False)
 observed={}
 with tarfile.open(temp,'r:gz') as t:
  for m in t:
   assert m.isfile() and m.name.startswith('raw/'),m.name
   key=m.name[4:];assert key not in observed
   data=t.extractfile(m).read();observed[key]={'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
 assert observed==expected,stage
 temp.rename(stage/'raw.tar.gz')
 kept=[key for key in files if '/' not in key and key!='debug.log']
 record={'archive':'raw.tar.gz','archive_sha256':sha(stage/'raw.tar.gz'),'raw_manifest_sha256':sha(stage/'raw-manifest.json'),'members':len(expected),'original_bytes':sum(x['bytes'] for x in expected.values()),'archive_bytes':(stage/'raw.tar.gz').stat().st_size,'verified_all_members':True,'loose_index_files':sorted(kept),'scope':'Only complete phases with immutable metadata. Original raw-manifest remains unchanged; logs and diffs are losslessly archived.'}
 (stage/'archive-manifest.json').write_text(json.dumps(record,indent=2)+'\n')
 for key,p in files.items():
  assert expected[key]=={'sha256':sha(p),'bytes':p.stat().st_size}
  if key not in kept:p.unlink()
 for p in sorted([p for p in raw.rglob('*') if p.is_dir()],key=lambda p:len(p.parts),reverse=True):
  if not any(p.iterdir()):p.rmdir()
 print(json.dumps({'phase':stage.name,**record}),flush=True)
