import pathlib,json,hashlib,os,subprocess,time,shutil,collections,fcntl
root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex');clone=root/'target/t31-validation/final-source';out=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/repairs/t31/mutants'
lock=(out/'.pipeline.lock').open('w');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
manifest=json.loads((out/'source-input.json').read_text());old_closure=json.loads((out/'execution-closure.json').read_text());candidate=manifest['candidate']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def verify():
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=clone,text=True).strip()==candidate
 assert not subprocess.check_output(['git','status','--porcelain'],cwd=clone,text=True).strip()
 for row in manifest['source_files']:
  for base in [root,clone]:assert sha(base/row['path'])==row['sha256']
 for row in manifest['frozen_governance_files']:assert sha(clone/row['path'])==row['sha256']
verify()
env=os.environ.copy();env.update(PATH=str(root/'target/t31-validation/tools')+':'+env['PATH'],RUSTC_WRAPPER='',CARGO_TARGET_DIR='target',CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='2',MUTANTS_TIMEOUT='600',CARGO_PROFILE_TEST_OPT_LEVEL='1',CARGO_PROFILE_TEST_DEBUG='0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
for k in ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_COMMON_DIR','GIT_OBJECT_DIRECTORY','GIT_ALTERNATE_OBJECT_DIRECTORIES']:env.pop(k,None)
closure={'candidate':candidate,'source_manifest_sha256':sha(out/'source-input.json'),'same_source_as':old_closure['sha256'],'script_sha256':sha(pathlib.Path(__file__)),'features':old_closure['features'],'profile':old_closure['profile'],'tools':old_closure['tools'],'jobs':4,'nextest_test_threads':2,'timeout_seconds':600,'reason':'Reduce orchestration contention only. Frozen program/tests/config/fixtures and full workspace selection unchanged. Original timeout logs retained; all selected survivors retested.'};closure_id=sha(out/'source-input.json')+'-workspace4x2';closure['sha256']=hashlib.sha256(json.dumps(closure,sort_keys=True).encode()).hexdigest();closure_id=closure['sha256'];(out/'execution-closure-workspace.json').write_text(json.dumps(closure,indent=2)+'\n')
progress=json.loads((out/'progress.json').read_text());chosen=json.loads((out/'sheltie-runtime-stage2-selection.json').read_text());seen=[];stage2=[]
def escape(s):return ''.join('\\'+c if c in r'\.^$|?*+()[]{}' else c for c in s)
for i in range(8):
 part=chosen[i::8];name=f'stage2-sheltie-runtime-{i}-of-8';directory=out/name
 assert not directory.exists();directory.mkdir();verify();source=clone/'target/mutants.out';assert not source.exists()
 argv=['scripts/mutants.sh','sheltie-runtime','--all-features','--copy-vcs','true','--jobs','4','--cargo-test-arg=--test-threads=2','--test-workspace','true','--re','^(?:'+'|'.join(escape(s) for s in part)+')$']
 started=time.time()
 with (directory/'stdout.txt').open('wb') as stream:result=subprocess.run(argv,cwd=clone,env=env,stdout=stream,stderr=subprocess.STDOUT)
 assert source.exists();shutil.move(source,directory/'raw');raw=directory/'raw';data=json.loads((raw/'outcomes.json').read_text());listed=json.loads((raw/'mutants.json').read_text());rows=[r for r in data['outcomes'] if r['scenario']!='Baseline'];baseline=[r for r in data['outcomes'] if r['scenario']=='Baseline'];names=[r['scenario']['Mutant']['name'] for r in rows]
 item={'name':name,'argv':argv,'exit':result.returncode,'seconds':round(time.time()-started,3),'candidate':candidate,'closure_sha256':closure_id,'source_manifest_sha256':sha(out/'source-input.json'),'listed':len(listed),'processed':len(rows),'counts':dict(collections.Counter(r['summary'] for r in rows)),'outcomes_sha256':sha(raw/'outcomes.json'),'mutants_sha256':sha(raw/'mutants.json')};(directory/'metadata.json').write_text(json.dumps(item,indent=2)+'\n');progress.append(item);(out/'progress.json').write_text(json.dumps(progress,indent=2)+'\n');print(json.dumps(item),flush=True)
 assert result.returncode in [0,2] and len(baseline)==1 and baseline[0]['summary']=='Success';assert len(names)==len(set(names))==len(part) and set(names)==set(part)=={r['name'] for r in listed}
 seen.extend(names);stage2.extend(rows)
assert len(seen)==len(set(seen))==len(chosen) and set(seen)==set(chosen)
stage1=[]
for directory in out.glob('stage1-sheltie-runtime-*'):
 stage1.extend(r for r in json.loads((directory/'raw/outcomes.json').read_text())['outcomes'] if r['scenario']!='Baseline')
result=json.loads((out/'results.json').read_text());result['workspace_execution_closure_sha256']=closure_id;result['packages']['sheltie-runtime']={'inventory':len(stage1),'stage1':dict(collections.Counter(r['summary'] for r in stage1)),'stage2':dict(collections.Counter(r['summary'] for r in stage2)),'remaining':[{'name':r['scenario']['Mutant']['name'],'summary':r['summary']} for r in stage2 if r['summary']!='CaughtMutant'],'stage1_unviable':[r['scenario']['Mutant']['name'] for r in stage1 if r['summary']=='Unviable']};result['disposition_complete']=False;(out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print('All 381 selected runtime survivors retested; final disposition still required.',flush=True)
