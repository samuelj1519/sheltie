import hashlib,json,os,subprocess,tarfile
from pathlib import Path
r=Path('/private/tmp/sheltie-first-baseline-20261006');assets=r/'v0.3.0';results=[]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def call(binary,home,*args):
 cmd=[str(binary),'--home',str(home),'--json',*args];out=subprocess.run(cmd,capture_output=True,text=True)
 results.append({'argv':cmd,'exit':out.returncode,'stdout':out.stdout,'stderr':out.stderr})
 (r/'public-cli-evidence.json').write_text(json.dumps(results,indent=2)+'\n')
 assert out.returncode==0,(cmd,out.stdout,out.stderr)
 data=json.loads(out.stdout);assert data['ok'] is True,data;return data
release=json.loads((r/'new-release.json').read_text());assert release['tag_name']=='v0.3.0' and not release['draft'] and not release['prerelease']
asset_hashes={}
for a in release['assets']:
 p=assets/a['name'];assert p.stat().st_size==a['size'],p
 actual=sha(p);assert a['digest']=='sha256:'+actual,(a['name'],a['digest'],actual);asset_hashes[a['name']]=actual
manifest=json.loads((assets/'dist-manifest.json').read_text());assert manifest['announcement_tag']=='v0.3.0',manifest.get('announcement_tag')
archives=[a for a in manifest['artifacts'].values() if a.get('kind')=='executable-zip'];assert len(archives)==1,archives
a=archives[0];assert a['target_triples']==['aarch64-apple-darwin'];assert a['checksums']['sha256']==asset_hashes[a['name']]
extracted=r/'public-extracted';extracted.mkdir()
with tarfile.open(assets/a['name']) as t:t.extractall(extracted,filter='data')
bins=list(extracted.rglob('sheltie'));assert len(bins)==1,bins;public=bins[0];public_hash=sha(public)
architecture=subprocess.check_output(['file',str(public)],text=True).strip();assert 'Mach-O' in architecture and 'arm64' in architecture,architecture
installed_by_script=r/'installer-bin/sheltie';assert sha(installed_by_script)==public_hash
probe=r/'public-version-only';version=call(public,probe,'self','version');assert not probe.exists();assert version['data']['version']=='0.3.0' and version['data']['schema_version']==4
fresh=r/'public-fresh-home';call(public,fresh,'self','install');engine=fresh/'bin/sheltie';assert sha(engine)==public_hash
call(engine,fresh,'workbook','add',str(Path.cwd()/'examples/two-step'))
start=call(engine,fresh,'work','start','--workbook','two-step','--flow','default','--input','topic=Verify the first supported public baseline');work=start['data']['work_id']
for node in ['outline','summary']:
 begin=call(engine,fresh,'attempt','begin',work,'--node',node);d=begin['data'];assert Path(d['brief_path']).is_file()
 for name,path in d['outputs'].items():
  p=Path(path);assert p.is_relative_to(fresh);p.write_text('Public release validation output for '+name+'\n')
 call(engine,fresh,'attempt','submit',work,'--attempt',d['attempt'],'--summary','Declared output written')
status=call(engine,fresh,'work','status',work);assert status['data']['status']['kind']=='succeeded' and status['next']==[],status
old_items=[json.loads(l) for l in (r/'updater-build.json').read_text().splitlines()];paths={v['executable'] for v in old_items if v.get('executable') and v['target']['name']=='sheltie'};assert len(paths)==1;old=Path(paths.pop());old_hash=sha(old)
update_home=r/'public-update-home';old_version=call(old,update_home,'self','version');assert old_version['data']['version']=='0.3.0-rc.1' and old_version['data']['schema_version']==4
call(old,update_home,'self','install');updater=update_home/'bin/sheltie';call(updater,update_home,'workbook','add',str(Path.cwd()/'examples/two-step'));store_before=sha(update_home/'store.db');assert sha(updater)==old_hash
update=call(updater,update_home,'self','update','--version','0.3.0');assert update['data']=={'from':'0.3.0-rc.1','to':'0.3.0','up_to_date':False},update
assert sha(updater)==public_hash and sha(update_home/'bin/sheltie.prev')==old_hash;assert sha(update_home/'store.db')==store_before
assert call(updater,update_home,'self','version')['data']['version']=='0.3.0'
call(updater,update_home,'self','rollback');assert sha(updater)==old_hash and not (update_home/'bin/sheltie.prev').exists();assert sha(update_home/'store.db')==store_before
assert call(updater,update_home,'self','version')['data']['version']=='0.3.0-rc.1'
summary={'result':'PASS','release_source_commit':'063029a9d9a41ef97ee8be48c12187312e599d1c','public_release_id':release['id'],'artifact_sha256':asset_hashes,'binary_sha256':public_hash,'architecture':architecture,'fresh_work_id':work,'update_from':'unpublished schema-4 source candidate 6d0f37a7c27c8cc265b0d799365e6a48dd66102c / 0.3.0-rc.1','update_to':'public v0.3.0','rollback_original_binary_sha256':old_hash,'unchanged_update_store_sha256':store_before,'old_release_migration_tested':False,'host_configuration_modified':False}
(r/'public-verification.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
