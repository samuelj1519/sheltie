import pathlib,subprocess,os,json,shutil,time,collections,hashlib,fcntl
root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex')
clone=root/'target/t31-validation/source'
out=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/repairs/t31/mutants'
env=os.environ.copy();env.update(PATH=str(root/'target/t31-validation/tools')+':'+env['PATH'],RUSTC_WRAPPER='',CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='2',MUTANTS_TIMEOUT='600',CARGO_PROFILE_TEST_OPT_LEVEL='1',CARGO_PROFILE_TEST_DEBUG='0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
for key in ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_COMMON_DIR']: env.pop(key,None)
candidate='da2bd13979df88dd987596d7ed726ef99bb89651'
lock_path=out/'.pipeline.lock'
lock_fd=os.open(lock_path,os.O_CREAT|os.O_RDWR,0o600)
try:
 fcntl.flock(lock_fd,fcntl.LOCK_EX|fcntl.LOCK_NB)
except BlockingIOError:
 raise SystemExit('another durable mutation pipeline holds the run lock')

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def checked_input():
 manifest=out.parent/'candidate-input.txt'
 lines=manifest.read_text().splitlines()
 if f'temporary_mutation_candidate: {candidate}' not in lines:
  raise SystemExit('candidate-input.txt identifies a different clone commit')
 actual=subprocess.check_output(['git','rev-parse','HEAD'],cwd=clone,text=True).strip()
 if actual!=candidate:raise SystemExit(f'clone HEAD changed: {actual}')
 dirty=subprocess.check_output(['git','status','--porcelain','--untracked-files=all'],cwd=clone,text=True).strip()
 if dirty:raise SystemExit('clone source tree is dirty; stop before mutation')
 count=0
 for line in lines:
  if '  ' not in line:continue
  expected,name=line.split('  ',1)
  for base in (root,clone):
   path=base/name
   if not path.is_file() or sha(path)!=expected:raise SystemExit(f'input mismatch: {path}')
  count+=1
 if count!=147:raise SystemExit(f'expected 147 source/config/fixture files, found {count}')
 return sha(manifest)

input_sha=checked_input()
which=lambda name: pathlib.Path(shutil.which(name,path=env['PATH']) or '')
tools_info={name:{'path':str(which(name)),'sha256':sha(which(name))} for name in ['cargo','cargo-mutants','cargo-nextest']}
rustc_version=subprocess.check_output(['rustc','--version'],env=env,text=True).strip()
cargo_version=subprocess.check_output(['cargo','--version'],env=env,text=True).strip()
mutants_version=subprocess.check_output(['cargo','mutants','--version'],env=env,text=True).strip()
nextest_version=subprocess.check_output(['cargo','nextest','--version'],env=env,text=True).splitlines()[0]
config=pathlib.Path.home()/'.cargo/config.toml'
closure={'candidate':candidate,'pipeline_sha256':sha(pathlib.Path(__file__)),'clone':str(clone.resolve()),'input_manifest_sha256':input_sha,'source_files':147,'rustc_version':rustc_version,'cargo_version':cargo_version,'mutants_version':mutants_version,'nextest_version':nextest_version,'tools':tools_info,'global_cargo_config_sha256':sha(config) if config.exists() else None,'cargo_home':env.get('CARGO_HOME',str(pathlib.Path.home()/'.cargo')),'execution_env':{key:env.get(key) for key in ['CARGO_NET_OFFLINE','CARGO_BUILD_JOBS','CARGO_PROFILE_TEST_OPT_LEVEL','CARGO_PROFILE_TEST_DEBUG','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS','CARGO_PROFILE_TEST_OVERFLOW_CHECKS','RUSTC_WRAPPER','NEXTEST_SUCCESS_OUTPUT','MUTANTS_TIMEOUT']},'rustflags_sha256':hashlib.sha256(env['RUSTFLAGS'].encode()).hexdigest() if env.get('RUSTFLAGS') else None,'rustdocflags_sha256':hashlib.sha256(env['RUSTDOCFLAGS'].encode()).hexdigest() if env.get('RUSTDOCFLAGS') else None,'test_tool':'nextest','copy_vcs':True,'jobs':8,'test_workspace_stage1':False,'test_workspace_stage2':True}
closure_id=hashlib.sha256(json.dumps(closure,sort_keys=True,separators=(',',':')).encode()).hexdigest()
closure_file=out/'execution-closure.json'
if closure_file.exists():
 existing=json.loads(closure_file.read_text())
 if existing.get('sha256')!=closure_id or existing.get('pipeline_sha256')!=closure['pipeline_sha256']:
  raise SystemExit('existing execution closure differs; preserve it and do not reuse prior pieces')
else:
 closure_file.write_text(json.dumps({'sha256':closure_id,**closure},indent=2)+'\n')
results=[];stage1=[]
closure_profile={'opt-level':1,'debug':0,'debug-assertions':True,'overflow-checks':True}
def run(name,package,args):
 checked_input()
 directory=out/name
 argv=['scripts/mutants.sh',package,'--all-features','--copy-vcs','true','--jobs','8']+args
 source=clone/'target/mutants.out'
 meta=directory/'metadata.json'
 raw=directory/'raw'
 if meta.exists() and (raw/'outcomes.json').exists() and (raw/'mutants.json').exists():
  metadata=json.loads(meta.read_text())
  data=json.loads((raw/'outcomes.json').read_text())
  inventory=json.loads((raw/'mutants.json').read_text())
  processed=[row for row in data['outcomes'] if row['scenario']!='Baseline']
  baselines=[row for row in data['outcomes'] if row['scenario']=='Baseline']
  names=[row['scenario']['Mutant']['name'] for row in processed]
  expected={item['name'] for item in inventory}
  manifest_file=directory/'raw-manifest.json'
  if manifest_file.exists():
   original=json.loads(manifest_file.read_text())
   if original.get('outcomes.json')!=sha(raw/'outcomes.json') or original.get('mutants.json')!=sha(raw/'mutants.json'):
    raise SystemExit(f'{name}: archived raw manifest differs from outcomes/inventory')
  if (metadata.get('candidate')!=candidate or metadata.get('argv')!=argv or metadata.get('profile')!=closure_profile or metadata.get('timeout_seconds')!=600 or metadata.get('closure_sha256')!=closure_id or metadata.get('exit') not in (0,2) or metadata.get('listed')!=len(inventory) or metadata.get('processed')!=len(processed) or metadata.get('counts')!=dict(collections.Counter(row['summary'] for row in processed)) or metadata.get('outcomes_sha256')!=sha(raw/'outcomes.json') or metadata.get('mutants_sha256')!=sha(raw/'mutants.json') or any(row['summary'] not in {'CaughtMutant','MissedMutant','Timeout','Unviable'} for row in processed) or len(names)!=len(expected) or len(set(names))!=len(names) or set(names)!=expected or len(baselines)!=1 or baselines[0]['summary']!='Success'):
   raise SystemExit(f'{name}: prior output lacks the same complete input and execution closure')
  results.append(metadata)
  (out/'progress.json').write_text(json.dumps(results,indent=2)+'\n')
  print(json.dumps({'reused_completed':name,'processed':len(processed)}),flush=True)
  return processed
 if directory.exists():
  raise SystemExit(f'{name}: partial stdout/raw/metadata exists; preserve it and resume missing exact IDs before running again')
 if source.exists():
  if not results:
   raise SystemExit('unclaimed target/mutants.out exists before first shard; do not delete it')
  previous=out/results[-1]['name']/'raw'
  if not (previous/'outcomes.json').exists() or sha(previous/'outcomes.json')!=sha(source/'outcomes.json') or sha(previous/'mutants.json')!=sha(source/'mutants.json'):
   raise SystemExit('target/mutants.out is not the fully archived previous shard; preserve it')
  shutil.rmtree(source)
 directory.mkdir(parents=True)
 started=time.time()
 with (directory/'stdout.txt').open('wb') as stream:
  result=subprocess.run(argv,cwd=clone,env=env,stdout=stream,stderr=subprocess.STDOUT)
 if not (source/'outcomes.json').exists() or not (source/'mutants.json').exists():
  raise SystemExit(f'{name}: no fresh outcomes; stdout retained, do not overwrite')
 shutil.copytree(source,raw)
 data=json.loads((raw/'outcomes.json').read_text())
 inventory=json.loads((raw/'mutants.json').read_text())
 processed=[row for row in data['outcomes'] if row['scenario']!='Baseline']
 baselines=[row for row in data['outcomes'] if row['scenario']=='Baseline']
 names=[row['scenario']['Mutant']['name'] for row in processed]
 expected={item['name'] for item in inventory}
 accepted={'CaughtMutant','MissedMutant','Timeout','Unviable'}
 if (result.returncode not in (0,2) or len(baselines)!=1 or baselines[0]['summary']!='Success' or len(names)!=len(expected) or len(set(names))!=len(names) or set(names)!=expected or any(row['summary'] not in accepted for row in processed)):
  (directory/'incomplete.json').write_text(json.dumps({'name':name,'argv':argv,'exit':result.returncode,'listed':len(inventory),'processed':len(processed),'closure_sha256':closure_id},indent=2)+'\n')
  raise SystemExit(f'{name}: incomplete, abnormal exit/summary, or failed baseline; raw retained')
 metadata={'name':name,'argv':argv,'exit':result.returncode,'seconds':round(time.time()-started,3),'listed':len(inventory),'processed':len(processed),'counts':dict(collections.Counter(row['summary'] for row in processed)),'candidate':candidate,'profile':closure_profile,'timeout_seconds':600,'closure_sha256':closure_id,'outcomes_sha256':sha(raw/'outcomes.json'),'mutants_sha256':sha(raw/'mutants.json')}
 meta.write_text(json.dumps(metadata,indent=2)+'\n')
 results.append(metadata)
 (out/'progress.json').write_text(json.dumps(results,indent=2)+'\n')
 print(json.dumps(metadata),flush=True)
 return processed
# Exact Rust-regex literals; every missed/timeout, no sampling.
def escape(value):return ''.join('\\'+ch if ch in r'\.^$|?*+()[]{}' else ch for ch in value)
full_inventory=json.loads((out/'inventory.json').read_text())
for package in ['sheltie-core','sheltie-runtime']:
 package_rows=[]
 shards=4 if package=='sheltie-core' else 24
 for shard in range(shards):
  package_rows.extend(run(f'stage1-{package}-{shard}-of-{shards}',package,['--test-workspace','false','--shard',f'{shard}/{shards}']))
 stage1.extend(package_rows)
 expected={item['name'] for item in full_inventory if item['package']==package}
 actual=[row['scenario']['Mutant']['name'] for row in package_rows]
 if len(actual)!=len(expected) or len(set(actual))!=len(actual) or set(actual)!=expected:
  raise SystemExit(f'{package}: stage1 inventory has missing, duplicate, or extra mutants')
 (out/f'{package}-stage1-inventory-check.json').write_text(json.dumps({'candidate':'da2bd13979df88dd987596d7ed726ef99bb89651','expected':len(expected),'processed':len(actual),'unique':len(set(actual)),'missing':sorted(expected-set(actual)),'extra':sorted(set(actual)-expected)},indent=2)+'\n')
 candidates=[row['scenario']['Mutant']['name'] for row in package_rows if row['summary'] in ['MissedMutant','Timeout']]
 (out/f'{package}-stage2-selection.json').write_text(json.dumps(candidates,indent=2)+'\n')
 if not candidates: continue
 chunks=1 if package=='sheltie-core' else 8
 stage2=[]
 for index in range(chunks):
  chosen=candidates[index::chunks]
  if not chosen:continue
  name=f'stage2-{package}-{index}-of-{chunks}'
  stage2.extend(run(name,package,['--test-workspace','true','--re','^(?:'+'|'.join(escape(item) for item in chosen)+')$']))
 observed=[row['scenario']['Mutant']['name'] for row in stage2]
 if len(observed)!=len(candidates) or len(set(observed))!=len(observed) or set(observed)!=set(candidates):
  raise SystemExit(f'{package}: stage2 missed or duplicated a survivor/timeout')
 (out/f'{package}-stage2-selection-check.json').write_text(json.dumps({'candidate':'da2bd13979df88dd987596d7ed726ef99bb89651','selected':len(candidates),'processed':len(observed),'unique':len(set(observed)),'missing':sorted(set(candidates)-set(observed)),'extra':sorted(set(observed)-set(candidates))},indent=2)+'\n')
print('all stages executed; survivor disposition and independent review still required',flush=True)
