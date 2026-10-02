import argparse,hashlib,json,os,pathlib,subprocess,time
ap=argparse.ArgumentParser();ap.add_argument('--selection',required=True);ap.add_argument('--oracle',required=True);ap.add_argument('--name',required=True);args=ap.parse_args()
root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex');source=root/'target/m1-validation/supplement-source';base=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/m1-2026-10-01/mutants';out=base/args.name;out.mkdir(exist_ok=False)
manifest=json.loads((base/'source-input.json').read_text());inventory={m['name']:m for m in json.loads((base/'inventory.json').read_text())};selection=json.loads(pathlib.Path(args.selection).read_text());oracle=pathlib.Path(args.oracle).resolve()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
originals={e['path']:(source/e['path']).read_bytes() for e in manifest['source_files']}
for e in manifest['source_files']:assert sha(source/e['path'])==e['sha256'],e['path']
assert not subprocess.check_output(['git','status','--porcelain'],cwd=source,text=True).strip()
env=os.environ.copy();env.update(RUSTC_WRAPPER='',CARGO_TARGET_DIR=str(root/'target/m1-validation/supplement-target'),CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='1',CARGO_PROFILE_DEV_OPT_LEVEL='1',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_DEV_DEBUG_ASSERTIONS='true',CARGO_PROFILE_DEV_OVERFLOW_CHECKS='true')
for k in ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_COMMON_DIR','GIT_OBJECT_DIRECTORY','GIT_ALTERNATE_OBJECT_DIRECTORIES']:env.pop(k,None)
results=[]
closure={'candidate':manifest['candidate'],'source_manifest_sha256':sha(base/'source-input.json'),'inventory_sha256':sha(base/'inventory.json'),'runner_sha256':sha(pathlib.Path(__file__)),'oracle_sha256':sha(oracle),'selection':selection,'scope':'Separate supplemental real-CLI detection. Does not rewrite formal mutation outcomes. Each generated mutation alone is applied to an ordinary isolated frozen clone, then restored. Baseline executes same oracle/flags.','environment':{k:v for k,v in env.items() if k.startswith('CARGO_') or k=='RUSTC_WRAPPER'},'rustc':subprocess.check_output(['rustc','--version'],text=True).strip()};(out/'closure.json').write_text(json.dumps(closure,indent=2)+'\n')
def build_and_run(folder):
 folder.mkdir();argv=['cargo','build','--offline','--locked','--all-features','--bin','sheltie','--message-format=json'];start=time.time()
 with (folder/'build.stdout.jsonl').open('wb') as stdout,(folder/'build.stderr.txt').open('wb') as stderr:r=subprocess.run(argv,cwd=source,env=env,stdout=stdout,stderr=stderr)
 record={'build_argv':argv,'build_exit':r.returncode,'seconds':time.time()-start};assert r.returncode==0,record
 executables=[]
 for line in (folder/'build.stdout.jsonl').read_text().splitlines():
  d=json.loads(line)
  if d.get('reason')=='compiler-artifact' and d.get('target',{}).get('name')=='sheltie' and d.get('executable'):executables.append(d['executable'])
 assert len(executables)==1,executables
 binary=pathlib.Path(executables[0]);record['binary']=str(binary);record['binary_sha256']=sha(binary)
 argv=['python3',str(oracle),'--binary',str(binary),'--source',str(source),'--output',str(folder/'oracle')]
 with (folder/'oracle.stdout.txt').open('wb') as stdout,(folder/'oracle.stderr.txt').open('wb') as stderr:r=subprocess.run(argv,cwd=source,env=env,stdout=stdout,stderr=stderr)
 record.update(oracle_argv=argv,oracle_exit=r.returncode,raw_sha256={str(p.relative_to(folder)):sha(p) for p in folder.rglob('*') if p.is_file()});(folder/'metadata.json').write_text(json.dumps(record,indent=2)+'\n');return record
try:
 baseline=build_and_run(out/'baseline');assert baseline['oracle_exit']==0,'unmutated supplemental baseline failed'
 for i,name in enumerate(selection):
  m=inventory[name];path=source/m['file'];assert path.read_bytes()==originals[m['file']]
  lines=m['diff'].splitlines(keepends=True);patch='--- a/'+m['file']+'\n+++ b/'+m['file']+'\n'+''.join(lines[2:]);folder=out/('mutant-'+str(i));(out/('mutant-'+str(i)+'.diff')).write_text(m['diff'])
  subprocess.run(['git','apply','--check','-'],input=patch,text=True,cwd=source,check=True);subprocess.run(['git','apply','-'],input=patch,text=True,cwd=source,check=True)
  changed={e['path']:sha(source/e['path']) for e in manifest['source_files'] if sha(source/e['path'])!=e['sha256']};assert set(changed)=={m['file']},changed
  try:
   record=build_and_run(folder);record.update(id=name,diff_sha256=hashlib.sha256(m['diff'].encode()).hexdigest(),changed_source_sha256=changed,detected=record['oracle_exit']!=0);results.append(record)
   print(json.dumps({'id':name,'supplemental_detected':record['detected'],'oracle_exit':record['oracle_exit']}),flush=True)
  finally:path.write_bytes(originals[m['file']])
 for e in manifest['source_files']:assert sha(source/e['path'])==e['sha256'],e['path']
 assert not subprocess.check_output(['git','status','--porcelain'],cwd=source,text=True).strip()
 (out/'results.json').write_text(json.dumps({'complete':True,'baseline':baseline,'results':results,'formal_outcomes_unchanged':True},indent=2)+'\n')
finally:
 for file,data in originals.items():
  if (source/file).read_bytes()!=data:(source/file).write_bytes(data)
 (out/'partial-results.json').write_text(json.dumps(results,indent=2)+'\n')
