import pathlib,subprocess,os,json,shutil,time,collections
root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex')
clone=root/'target/t31-validation/source'
out=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/repairs/t31/mutants'
env=os.environ.copy();env.update(PATH=str(root/'target/t31-validation/tools')+':'+env['PATH'],RUSTC_WRAPPER='',CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='2',MUTANTS_TIMEOUT='600',CARGO_PROFILE_TEST_OPT_LEVEL='1',CARGO_PROFILE_TEST_DEBUG='0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
for key in ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_COMMON_DIR']: env.pop(key,None)
results=[];stage1=[]
def run(name,package,args):
 directory=out/name;directory.mkdir(parents=True,exist_ok=True)
 argv=['scripts/mutants.sh',package,'--all-features','--copy-vcs','true','--jobs','8']+args
 source=clone/'target/mutants.out'
 if (directory/'metadata.json').exists() and (directory/'raw/outcomes.json').exists():
  metadata=json.loads((directory/'metadata.json').read_text())
  data=json.loads((directory/'raw/outcomes.json').read_text())
  processed=[row for row in data['outcomes'] if row['scenario']!='Baseline']
  baselines=[row for row in data['outcomes'] if row['scenario']=='Baseline']
  if metadata['candidate']!='da2bd13979df88dd987596d7ed726ef99bb89651' or metadata['argv']!=argv or metadata['processed']!=metadata['listed'] or len(processed)!=metadata['processed'] or not baselines or baselines[0]['summary']!='Success':
   raise SystemExit(f'cannot reuse incomplete or different-input shard {name}')
  results.append(metadata);(out/'progress.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps({'reused_completed':name,'processed':len(processed)}),flush=True)
  return processed
 if source.exists() and (directory/'stdout.txt').exists():
  raise SystemExit(f'interrupted {name}: source target/mutants.out and stdout retained; resume missing exact mutant IDs before rerunning pipeline')
 if source.exists(): shutil.rmtree(source)
 started=time.time()
 with (directory/'stdout.txt').open('wb') as stream:result=subprocess.run(argv,cwd=clone,env=env,stdout=stream,stderr=subprocess.STDOUT)
 if not source.joinpath('outcomes.json').exists():
  (directory/'metadata.json').write_text(json.dumps({'name':name,'argv':argv,'exit':result.returncode,'status':'not_started_no_fresh_outcomes'},indent=2)+'\n')
  raise SystemExit('tool did not start a fresh run; previous outcomes cannot be reused')
 shutil.copytree(source,directory/'raw',dirs_exist_ok=True)
 data=json.loads((source/'outcomes.json').read_text());inventory=json.loads((source/'mutants.json').read_text())
 processed=[row for row in data['outcomes'] if row['scenario']!='Baseline']
 baselines=[row for row in data['outcomes'] if row['scenario']=='Baseline']
 metadata={'name':name,'argv':argv,'exit':result.returncode,'seconds':round(time.time()-started,3),'listed':len(inventory),'processed':len(processed),'counts':dict(collections.Counter(row['summary'] for row in processed)),'candidate':'da2bd13979df88dd987596d7ed726ef99bb89651','profile':{'opt-level':1,'debug':0,'debug-assertions':True,'overflow-checks':True},'timeout_seconds':600}
 (directory/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n');results.append(metadata);(out/'progress.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(metadata),flush=True)
 if len(processed)!=len(inventory) or not baselines or baselines[0]['summary']!='Success':raise SystemExit('incomplete shard or failed baseline; preserve evidence and stop')
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
