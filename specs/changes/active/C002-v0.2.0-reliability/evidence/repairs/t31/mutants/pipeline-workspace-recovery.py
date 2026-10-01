import collections, fcntl, hashlib, json, os, pathlib, shutil, subprocess, threading, time

root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex')
clone=root/'target/t31-validation/final-source'
out=pathlib.Path(__file__).resolve().parent
lock=(out/'.pipeline.lock').open('w')
fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
manifest=json.loads((out/'source-input.json').read_text())
execution=json.loads((out/'execution-closure-workspace.json').read_text())
chosen=json.loads((out/'sheltie-runtime-stage2-selection.json').read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def verify():
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=clone,text=True).strip()==manifest['candidate']
    assert not subprocess.check_output(['git','status','--porcelain'],cwd=clone,text=True).strip()
    for row in manifest['source_files']:
        for base in [root,clone]:assert sha(base/row['path'])==row['sha256']
    for row in manifest['frozen_governance_files']:assert sha(clone/row['path'])==row['sha256']
verify()
assert execution['source_manifest_sha256']==sha(out/'source-input.json')
assert execution['jobs']==4 and execution['nextest_test_threads']==2 and execution['timeout_seconds']==600
env=os.environ.copy()
env.update(PATH=str(root/'target/t31-validation/tools')+':'+env['PATH'],RUSTC_WRAPPER='',CARGO_TARGET_DIR='target',CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='2',MUTANTS_TIMEOUT='600',CARGO_PROFILE_TEST_OPT_LEVEL='1',CARGO_PROFILE_TEST_DEBUG='0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
for key in ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_COMMON_DIR','GIT_OBJECT_DIRECTORY','GIT_ALTERNATE_OBJECT_DIRECTORIES']:env.pop(key,None)
progress=json.loads((out/'progress.json').read_text())
source=clone/'target/mutants.out'
state={'pid':os.getpid(),'status':'running','stage':'recover interrupted prefix','execution_closure_sha256':execution['sha256'],'recovery_script_sha256':sha(pathlib.Path(__file__))}
def monitor():
    while state['status']=='running':
        record=dict(state);record['heartbeat_unix']=time.time()
        try:
            rows=json.loads((source/'outcomes.json').read_text())['outcomes']
            rows=[r for r in rows if r['scenario']!='Baseline']
            record['live_completed']=len(rows);record['live_counts']=dict(collections.Counter(r['summary'] for r in rows))
        except (FileNotFoundError,json.JSONDecodeError):pass
        temporary=out/'monitor-status.json.tmp';temporary.write_text(json.dumps(record,indent=2)+'\n');temporary.replace(out/'monitor-status.json')
        time.sleep(20)
threading.Thread(target=monitor,daemon=True).start()
def save(name,argv,exit_code,seconds,interrupted=False):
    directory=out/name;raw=directory/'raw'
    assert directory.is_dir() and not raw.exists() and source.is_dir()
    shutil.move(source,raw)
    data=json.loads((raw/'outcomes.json').read_text());listed=json.loads((raw/'mutants.json').read_text())
    baseline=[r for r in data['outcomes'] if r['scenario']=='Baseline'];assert len(baseline)==1 and baseline[0]['summary']=='Success'
    rows=[r for r in data['outcomes'] if r['scenario']!='Baseline'];names=[r['scenario']['Mutant']['name'] for r in rows]
    assert len(names)==len(set(names)) and set(names)<={r['name'] for r in listed}
    if not interrupted:assert exit_code in [0,2] and len(rows)==len(listed)
    else:
        assert exit_code is None and data['end_time'] is None
        for row in rows:
            assert row['summary'] in ['CaughtMutant','MissedMutant']
            phases={p['phase']:p for p in row['phase_results']}
            assert phases['Build']['process_status']=='Success'
            test=phases['Test'];assert '--workspace' in test['argv'] and '--all-features' in test['argv'] and '--test-threads=2' in test['argv']
            assert test['process_status']=='Success' if row['summary']=='MissedMutant' else test['process_status']=={'Failure':100}
    item={'name':name,'argv':argv,'exit':exit_code,'interrupted':interrupted,'seconds':seconds,'candidate':manifest['candidate'],'closure_sha256':execution['sha256'],'source_manifest_sha256':sha(out/'source-input.json'),'recovery_script_sha256':state['recovery_script_sha256'],'listed':len(listed),'processed':len(rows),'counts':dict(collections.Counter(r['summary'] for r in rows)),'outcomes_sha256':sha(raw/'outcomes.json'),'mutants_sha256':sha(raw/'mutants.json')}
    if interrupted:item['reason']='Original tool session and owned processes no longer exist. Retain only terminal entries, not unfinished mutants or a fabricated overall exit.'
    (directory/'metadata.json').write_text(json.dumps(item,indent=2)+'\n');progress.append(item);(out/'progress.json').write_text(json.dumps(progress,indent=2)+'\n')
    print(json.dumps(item),flush=True)
def completed():
    result=[]
    for directory in out.glob('stage2-sheltie-runtime-*'):
        if (directory/'metadata.json').exists():
            result.extend(r['scenario']['Mutant']['name'] for r in json.loads((directory/'raw/outcomes.json').read_text())['outcomes'] if r['scenario']!='Baseline')
    assert len(result)==len(set(result)) and set(result)<=set(chosen)
    return set(result)
def escape(s):return ''.join('\\'+c if c in r'\.^$|?*+()[]{}' else c for c in s)
try:
    if source.exists():
        directory=out/'stage2-sheltie-runtime-3-of-8';assert not (directory/'metadata.json').exists()
        save(directory.name,['see original stdout and per-entry recorded argv'],None,None,True)
    for i in range(8):
        part=[n for n in chosen[i::8] if n not in completed()]
        if not part:continue
        name=f'stage2-sheltie-runtime-{i}-of-8'
        if (out/name).exists():name+='-continuation'
        directory=out/name;assert not directory.exists();directory.mkdir();verify()
        argv=['scripts/mutants.sh','sheltie-runtime','--all-features','--copy-vcs','true','--jobs','4','--cargo-test-arg=--test-threads=2','--test-workspace','true','--re','^(?:'+'|'.join(escape(n) for n in part)+')$']
        state['stage']=name;state['selected']=len(part);started=time.time()
        with (directory/'stdout.txt').open('wb') as stream:
            process=subprocess.Popen(argv,cwd=clone,env=env,stdout=stream,stderr=subprocess.STDOUT)
            state['child_pid']=process.pid;returncode=process.wait()
        save(name,argv,returncode,round(time.time()-started,3))
    assert completed()==set(chosen)
    stages=[]
    for directory in out.glob('stage2-sheltie-runtime-*'):
        stages.extend(r for r in json.loads((directory/'raw/outcomes.json').read_text())['outcomes'] if r['scenario']!='Baseline')
    result=json.loads((out/'results.json').read_text());result['workspace_execution_closure_sha256']=execution['sha256'];result['packages']['sheltie-runtime']['stage2']=dict(collections.Counter(r['summary'] for r in stages));result['packages']['sheltie-runtime']['remaining']=[{'name':r['scenario']['Mutant']['name'],'summary':r['summary']} for r in stages if r['summary']!='CaughtMutant'];result['disposition_complete']=False;(out/'results.json').write_text(json.dumps(result,indent=2)+'\n');state['status']='complete'
except BaseException:
    state['status']='failed'
    raise
finally:
    state['heartbeat_unix']=time.time();(out/'monitor-status.json').write_text(json.dumps(state,indent=2)+'\n')
