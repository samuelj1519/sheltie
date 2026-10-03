import datetime, hashlib, importlib.util, json, os, sys
from pathlib import Path
RUN=Path(__file__).resolve().parents[2]
RAW=RUN/'raw/deliver-1'
B=json.loads((RUN/'run-binding.json').read_text())
STUDY=Path(B['study_dir'])
REPO=Path(B['repo'])
CHECK=Path(B['patch_check_repo'])
PROJECT=json.loads(Path(B['project']).read_text())
INITIAL=PROJECT['initial_head']
INITIAL_TREE=PROJECT['initial_tree']
CANDIDATE='8366454ddbc2a07d25b2e5e949171cde5b5169ae'
TREE='3b9e14dfb50d3b577cc4caaacdf33e96082026fc'
ALLOW=B['allowed_repo_files']
DEADLINE=B['deadline_utc']
spec=importlib.util.spec_from_file_location('frozen_capture', STUDY/'capture.py')
capture=importlib.util.module_from_spec(spec)
spec.loader.exec_module(capture)
records=[]
def now(): return datetime.datetime.now(datetime.timezone.utc).isoformat()
def digest(path):
    data=Path(path).read_bytes()
    return {'path':str(path),'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
def save(name,obj):
    with (RAW/name).open('x') as f: json.dump(obj,f,ensure_ascii=False,indent=2); f.write('\n')
def execute(label,argv,cwd):
    rec,out,err=capture.execute(RAW,label,argv,cwd,120,DEADLINE)
    records.append(str(RAW/(label+'.json')))
    assert rec['exit_code']==0, (label,rec['exit_code'],err.decode(errors='replace'))
    return out
start=now()
environment={n:os.environ.get(n) for n in ['PATH','LANG','LC_ALL','LC_CTYPE','SHELL','GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_CONFIG_COUNT','GIT_CONFIG_GLOBAL','GIT_CONFIG_SYSTEM','GIT_CONFIG_NOSYSTEM','RUSTC_WRAPPER','CARGO_TARGET_DIR','PYTHONDONTWRITEBYTECODE']}
save('environment.json', {'observed_utc':start,'relevant_environment':environment,'all_inner_commands_inherit_this_environment':True})
inputs={
    'change':RUN/'outputs/implement-1/change.md',
    'checks':RUN/'outputs/implement-1/checks.md',
    'review':RUN/'outputs/review-1/review.md',
    'task':Path(B['task']),
    'project':Path(B['project']),
    'method':Path(B['method_dir'])/'instructions/deliver.md',
    'run-binding':RUN/'run-binding.json',
}
input_identity={k:digest(v) for k,v in inputs.items()}
assert inputs['review'].read_text().startswith('建议交付：')
for k in ['change','checks','review']:
    assert TREE in inputs[k].read_text()
    assert INITIAL in inputs[k].read_text()
for k in ['change','review']:
    assert CANDIDATE in inputs[k].read_text()
save('input-identity.json', {'observed_utc':now(),'inputs':input_identity,'review_first_line':inputs['review'].read_text().splitlines()[0]})
save('review-time-observation.json', {'observed_utc':now(),'parent_observation':'coordinator read 19:59:59Z reported then end 19:59:49.535952Z; review worker finalized report task_complete20:00:06.982738Z','final_review_report_end_utc':'2026-10-03T20:00:06.982738+00:00','bound_review_sha256':input_identity['review']['sha256'],'use_final_report_only':True,'earlier_observation_remains_in_original_session_record':True})
headtree=execute('candidate-identity',['git','rev-parse','HEAD','HEAD^{tree}','HEAD^',INITIAL+'^{tree}'],REPO).decode().splitlines()
assert headtree==[CANDIDATE,TREE,INITIAL,INITIAL_TREE],headtree
assert execute('candidate-status',['git','status','--porcelain=v1','--untracked-files=all'],REPO)==b''
changed=execute('candidate-full-paths',['git','diff','--name-only','-z',INITIAL,'HEAD'],REPO)
assert changed.split(b'\0')[:-1]==[p.encode() for p in ALLOW],changed
files={p:digest(REPO/p) for p in ALLOW}
after=json.loads((RUN/'raw/implement-1/candidate-after-commit.json').read_text())
before=json.loads((RUN/'raw/implement-1/candidate-before-checks.json').read_text())
assert after['head']==CANDIDATE and after['tree']==TREE and after['changed_paths']==ALLOW
assert before['index_tree']==TREE,before
for p in ALLOW:
    assert files[p]['sha256']==after['allow_files'][p]['sha256']
    assert files[p]['bytes']==after['allow_files'][p]['bytes']
required=[['scripts/check-docs.sh',*ALLOW],['git','diff','--cached','--check']]
checks=[]
bindings=json.loads((RUN/'raw/implement-1/checks-bindings.json').read_text())
for binding,argv in zip(bindings,required,strict=True):
    rec=json.loads(Path(binding['record']).read_text())
    assert binding['candidate_index_tree']==TREE and binding['head_before_commit']==INITIAL
    assert rec['argv']==argv and rec['cwd']==str(REPO) and rec['exit_code']==0
    assert rec['timed_out'] is False and rec['within_deadline'] is True
    for stream in ['stdout','stderr']:
        assert digest(rec[stream+'_path'])['sha256']==rec[stream+'_sha256']
    checks.append({'record':binding['record'],'argv':argv,'exit_code':0,'tree':TREE,'raw_hashes_verified':True,'start_utc':rec['start_utc'],'end_utc':rec['end_utc']})
save('verified-inputs.json', {'observed_utc':now(),'candidate_head':CANDIDATE,'candidate_tree':TREE,'initial_head':INITIAL,'initial_tree':INITIAL_TREE,'candidate_clean':True,'full_changed_paths':ALLOW,'files':files,'required_checks':checks,'input_identity':input_identity})
patch_bytes=execute('generate-complete-patch',PROJECT['patch_rule']['generate_argv'],REPO)
assert len(patch_bytes)<=8388608 and patch_bytes
outdir=RUN/'outputs/deliver-1'
outdir.mkdir(exist_ok=False)
patch=outdir/'change.patch'
with patch.open('xb') as f: f.write(patch_bytes)
assert digest(patch)['sha256']==digest(RAW/'generate-complete-patch.stdout')['sha256']
existed=CHECK.exists()
if not existed:
    CHECK.parent.mkdir(parents=True,exist_ok=True)
    execute('create-declared-check-repo',['git','clone','--no-hardlinks','--no-checkout',str(REPO),str(CHECK)],REPO)
    execute('checkout-declared-initial',['git','-C',str(CHECK),'-c','core.hooksPath=/dev/null','checkout','--detach',INITIAL],REPO)
assert execute('check-repo-initial-identity',['git','rev-parse','HEAD','HEAD^{tree}'],CHECK).decode().splitlines()==[INITIAL,INITIAL_TREE]
assert execute('check-repo-initial-status',['git','status','--porcelain=v1','--untracked-files=all'],CHECK)==b''
execute('apply-check',['git','apply','--check',str(patch)],CHECK)
execute('apply-index',['git','apply','--index',str(patch)],CHECK)
assert execute('applied-full-tree',['git','write-tree'],CHECK).decode().strip()==TREE
appliedfiles={p:digest(CHECK/p) for p in ALLOW}
for p in ALLOW:
    assert (REPO/p).read_bytes()==(CHECK/p).read_bytes()
    assert appliedfiles[p]['sha256']==files[p]['sha256'] and appliedfiles[p]['bytes']==files[p]['bytes']
assert execute('applied-changed-paths',['git','diff','--cached','--name-only','-z',INITIAL],CHECK).split(b'\0')[:-1]==[p.encode() for p in ALLOW]
assert execute('applied-no-unstaged-drift',['git','diff','--name-only','-z'],CHECK)==b''
assert execute('candidate-final-identity',['git','rev-parse','HEAD','HEAD^{tree}'],REPO).decode().splitlines()==[CANDIDATE,TREE]
assert execute('candidate-final-status',['git','status','--porcelain=v1','--untracked-files=all'],REPO)==b''
for k,v in inputs.items(): assert digest(v)==input_identity[k], ('input drift',k)
end=now()
save('application-verification.json', {'observed_utc':end,'check_repo':str(CHECK),'check_repo_preexisted':existed,'check_repo_initial_head':INITIAL,'check_repo_initial_tree':INITIAL_TREE,'check_repo_initial_clean':True,'candidate_head':CANDIDATE,'candidate_tree':TREE,'applied_full_tree':TREE,'changed_paths':ALLOW,'file_bytes_equal':True,'applied_files':appliedfiles,'candidate_files':files,'patch':digest(patch),'complete_patch_equals_raw_stdout':True,'source_still_same_clean_candidate':True,'inputs_still_match':True,'command_records':records})
save('actor.json', {'actor':'/root/c007_study_coordinator/run01_deliver','model_declaration':'gpt-6.1-sol','reasoning_effort_declaration':'high','inherited':True,'override':False,'helpers':[],'model_session_metadata_ref':'unknown','exact_task_received_utc':'unknown','first_observable_utc':'2026-10-03T20:00:33+00:00','driver_start_utc':start,'driver_end_utc':end,'run_started_at':B['started_at'],'deadline_utc':DEADLINE,'usage':None,'fees':None,'input_identity':input_identity,'command_records':records,'report_finalization_utc':'recorded separately by final-artifacts.json'})
print(json.dumps({'verified':True,'patch':digest(patch),'candidate_head':CANDIDATE,'tree':TREE,'start_utc':start,'end_utc':end,'check_repo_preexisted':existed,'raw_dir':str(RAW)},ensure_ascii=False,indent=2))
