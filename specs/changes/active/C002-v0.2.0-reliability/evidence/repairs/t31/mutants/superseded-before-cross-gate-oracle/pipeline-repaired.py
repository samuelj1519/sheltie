import collections, fcntl, hashlib, json, os, pathlib, shutil, subprocess, time
root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex')
clone=root/'target/t31-validation/repaired-source'
out=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/repairs/t31/mutants'
env=os.environ.copy()
env.update(PATH=str(root/'target/t31-validation/tools')+':'+env['PATH'],RUSTC_WRAPPER='',CARGO_TARGET_DIR='target',CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='2',MUTANTS_TIMEOUT='600',CARGO_PROFILE_TEST_OPT_LEVEL='1',CARGO_PROFILE_TEST_DEBUG='0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
for key in ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_COMMON_DIR','GIT_OBJECT_DIRECTORY','GIT_ALTERNATE_OBJECT_DIRECTORIES']:env.pop(key,None)
lock=(out/'.pipeline.lock').open('w')
fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
manifest=json.loads((out/'source-input.json').read_text())
candidate=manifest['candidate']
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def verify():
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=clone,text=True).strip()==candidate
 assert not subprocess.check_output(['git','status','--porcelain'],cwd=clone,text=True).strip()
 for entry in manifest['source_files']:
  for base in [root,clone]:assert sha(base/entry['path'])==entry['sha256'],entry['path']
verify()
closure={'candidate':candidate,'manifest_sha256':sha(out/'source-input.json'),'pipeline_sha256':sha(pathlib.Path(__file__)),'source_files':len(manifest['source_files']),'features':'all-features','profile':{'opt-level':1,'debug':0,'debug-assertions':True,'overflow-checks':True},'jobs':8,'timeout_seconds':600,'target':'relative target in each mutant source','tools':{name:subprocess.check_output(argv,cwd=clone,env=env,text=True).splitlines()[0] for name,argv in [('rustc',['rustc','--version']),('cargo',['cargo','--version']),('nextest',['cargo','nextest','--version']),('mutants',['cargo','mutants','--version'])]}}
closure_id=hashlib.sha256(json.dumps(closure,sort_keys=True).encode()).hexdigest()
closure['sha256']=closure_id
(out/'execution-closure.json').write_text(json.dumps(closure,indent=2)+'\n')
results=[]
def record(item):
 results.append(item);(out/'progress.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(item),flush=True)
argv=['cargo','nextest','run','--all-features','--no-tests=pass']
started=time.time()
with (out/'baseline.stdout.txt').open('wb') as stream:result=subprocess.run(argv,cwd=clone,env=env,stdout=stream,stderr=subprocess.STDOUT)
record({'name':'baseline','argv':argv,'exit':result.returncode,'seconds':round(time.time()-started,3),'candidate':candidate,'closure_sha256':closure_id})
if result.returncode:raise SystemExit(result.returncode)
argv=['cargo','mutants','-p','sheltie-core','-p','sheltie-runtime','--list','--json','--exclude','crates/*/src/testkit.rs']
with (out/'inventory.json').open('wb') as stream:subprocess.run(argv,cwd=clone,env=env,stdout=stream,stderr=subprocess.PIPE,check=True)
inventory=json.loads((out/'inventory.json').read_text())
(out/'inventory-metadata.json').write_text(json.dumps({'argv':argv,'count':len(inventory),'packages':dict(collections.Counter(row['package'] for row in inventory)),'sha256':sha(out/'inventory.json'),'candidate':candidate},indent=2)+'\n')
def run(name,package,args):
 verify();directory=out/name
 if directory.exists():raise RuntimeError(f'{name}: existing result retained; do not overwrite')
 directory.mkdir()
 source=clone/'target/mutants.out'
 if source.exists():raise RuntimeError('unclaimed mutation output; retain and stop')
 argv=['scripts/mutants.sh',package,'--all-features','--copy-vcs','true','--jobs','8']+args
 started=time.time()
 with (directory/'stdout.txt').open('wb') as stream:result=subprocess.run(argv,cwd=clone,env=env,stdout=stream,stderr=subprocess.STDOUT)
 if not source.exists():raise RuntimeError(f'{name}: no output')
 shutil.move(source,directory/'raw')
 raw=directory/'raw';data=json.loads((raw/'outcomes.json').read_text());listed=json.loads((raw/'mutants.json').read_text())
 rows=[r for r in data['outcomes'] if r['scenario']!='Baseline'];baseline=[r for r in data['outcomes'] if r['scenario']=='Baseline']
 names=[r['scenario']['Mutant']['name'] for r in rows];expected={r['name'] for r in listed}
 item={'name':name,'argv':argv,'exit':result.returncode,'seconds':round(time.time()-started,3),'candidate':candidate,'closure_sha256':closure_id,'listed':len(listed),'processed':len(rows),'counts':dict(collections.Counter(r['summary'] for r in rows)),'outcomes_sha256':sha(raw/'outcomes.json'),'mutants_sha256':sha(raw/'mutants.json')}
 (directory/'metadata.json').write_text(json.dumps(item,indent=2)+'\n');record(item)
 assert result.returncode in [0,2] and len(baseline)==1 and baseline[0]['summary']=='Success',name
 assert len(names)==len(expected) and len(set(names))==len(names) and set(names)==expected,name
 assert all(r['summary'] in ['CaughtMutant','MissedMutant','Timeout','Unviable'] for r in rows),name
 return rows
summaries={}
def escape(name):return ''.join('\\'+c if c in r'\.^$|?*+()[]{}' else c for c in name)
for package,shards in [('sheltie-core',4),('sheltie-runtime',8)]:
 stage1=[]
 for i in range(shards):stage1.extend(run(f'stage1-{package}-{i}-of-{shards}',package,['--test-workspace','false','--shard',f'{i}/{shards}']))
 expected={r['name'] for r in inventory if r['package']==package};names=[r['scenario']['Mutant']['name'] for r in stage1]
 assert len(names)==len(set(names))==len(expected) and set(names)==expected
 chosen=[r['scenario']['Mutant']['name'] for r in stage1 if r['summary'] in ['MissedMutant','Timeout']]
 (out/f'{package}-stage2-selection.json').write_text(json.dumps(chosen,indent=2)+'\n')
 stage2=[];chunks=1 if package=='sheltie-core' else 8
 for i in range(chunks):
  part=chosen[i::chunks]
  if part:stage2.extend(run(f'stage2-{package}-{i}-of-{chunks}',package,['--test-workspace','true','--re','^(?:'+'|'.join(escape(name) for name in part)+')$']))
 names2=[r['scenario']['Mutant']['name'] for r in stage2]
 assert len(names2)==len(set(names2))==len(chosen) and set(names2)==set(chosen)
 summaries[package]={'inventory':len(expected),'stage1':dict(collections.Counter(r['summary'] for r in stage1)),'stage2':dict(collections.Counter(r['summary'] for r in stage2)),'remaining':[{'name':r['scenario']['Mutant']['name'],'summary':r['summary']} for r in stage2 if r['summary']!='CaughtMutant'],'stage1_unviable':[r['scenario']['Mutant']['name'] for r in stage1 if r['summary']=='Unviable']}
 (out/'results.json').write_text(json.dumps({'candidate':candidate,'closure_sha256':closure_id,'packages':summaries,'disposition_complete':False},indent=2)+'\n')
print('All inventory and second-stage selections processed; disposition and independent review still required.',flush=True)
