import datetime,hashlib,importlib.util,json,os
from pathlib import Path
RUN=Path(__file__).resolve().parents[2]
B=json.loads((RUN/'run-binding.json').read_text())
R=json.loads((RUN/'begin-deliver-1-reply.json').read_text())['data']
P=json.loads(Path(B['project']).read_text())
RAW=Path(__file__).resolve().parent
REPO=Path(B['repo']);CHECK=Path(B['patch_check_repo']);DEAD=B['deadline_utc']
CAND='957efb7076861d1dc54e780d841e0b0bb767bccd';TREE='02966cae1d3fe7a601c26d4bd6f69c66e323a8c4'
ALLOW=P['allow_files'];BASE=P['initial_head'];BT=P['initial_tree']
spec=importlib.util.spec_from_file_location('frozen_capture',Path(B['study_dir'])/'capture.py')
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def sha(data):return hashlib.sha256(data).hexdigest()
def identity(path):
 path=Path(path);data=path.read_bytes();return dict(path=str(path),sha256=sha(data),bytes=len(data))
def save(name,value):
 with (RAW/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
records=[]
def run(label,argv,cwd=REPO):
 r,out,err=c.execute(RAW,label,argv,str(cwd),120,DEAD);records.append(str(RAW/(label+'.json')))
 assert r['exit_code']==0,(label,r['exit_code'])
 return out
save('actor.json',dict(actor='/root/c007_study_coordinator/run02_deliver',start_utc=now(),model_declaration='inherited gpt-6.1-sol/high; actual metadata Root verification; unknown here',usage='unknown',fees='unknown',environment={k:os.environ.get(k) for k in ['PATH','LANG','LC_ALL','LC_CTYPE','GIT_CONFIG_GLOBAL','GIT_CONFIG_SYSTEM','GIT_CONFIG_NOSYSTEM','GIT_CONFIG_COUNT','GIT_INDEX_FILE','GIT_DIR','GIT_WORK_TREE','GIT_AUTHOR_NAME','GIT_AUTHOR_EMAIL','GIT_COMMITTER_NAME','GIT_COMMITTER_EMAIL','RUSTC_WRAPPER','CARGO_TARGET_DIR','SHELTIE_HOME']},initial_tool_reads='binding/brief/inputs/project/context/specs/engineering/skill/capture read via exec tool; raw split stdout/stderr absent; actual tool transcript retained',read_boundary_deviation='rg initial archive search glob exclusion did not exclude runs; returned a few native run preflight/deliver script matches. No opposite report/patch/full script opened; no returned content adopted. Coordinator notified. Protocol eligibility unresolved; no concealment.',diagnostic_failure='zsh glob lookup of preflight*.json failed exit 1; actual command/result retained in tool transcript; no split capture original',no_helpers=True,no_sheltie_progression=True))
inputs={k:identity(v) for k,v in R['inputs'].items()}
expected={'task':'1345a1ce70f26a1311de7d6b8a57b5786bd0ea154f4073a46eccd1fe461e4f64','change':'c32492c68390230be9694edbe441ff1cb63d7053f7ac25a8d897f73a22d6dc8f','checks':'70ef96b75abb845b3b45ef135d937f5c47cdeb5591cc62321064b06487db6785','review':'db039bab3a4f6a74ec1eb39e64dcfca76868a13efdfe7f97c649ff9ce4798ccb'}
assert {k:v['sha256'] for k,v in inputs.items()}==expected
assert Path(R['inputs']['review']).read_text().startswith('建议交付：')
assert run('candidate-identity',['git','rev-parse','HEAD','HEAD^{tree}']).decode().splitlines()==[CAND,TREE]
assert run('candidate-status',['git','status','--porcelain=v1','--untracked-files=all'])==b''
paths=run('candidate-full-paths',['git','diff','--name-only','-z',BASE,'HEAD']).decode().strip('\0').split('\0')
assert sorted(paths)==sorted(ALLOW)
assert run('baseline-identity',['git','rev-parse',BASE+'^{tree}']).decode().strip()==BT
files={f:identity(REPO/f) for f in ALLOW}
checked=json.loads((RUN/'raw/implement-1/checked-input.json').read_text())
assert checked['staged_tree']==TREE and checked['required_argv']==P['checks']
assert checked['allow_files_sha256']=={f:v['sha256'] for f,v in files.items()}
checks=[]
for label,argv in zip(['required-docs','required-staged-diff'],P['checks']):
 p=RUN/'raw/implement-1'/(label+'.json');j=json.loads(p.read_text())
 assert j['exit_code']==0 and not j['timed_out'] and j['within_deadline'] and j['argv']==argv and j['cwd']==str(REPO)
 for stream in ['stdout','stderr']:
  assert sha(Path(j[stream+'_path']).read_bytes())==j[stream+'_sha256']
 checks.append(dict(record=identity(p),start_utc=j['start_utc'],end_utc=j['end_utc'],exit_code=j['exit_code'],argv=j['argv'],stdout=identity(j['stdout_path']),stderr=identity(j['stderr_path'])))
for label in ['staged-tree-before-checks','staged-tree-after-checks']:
 j=json.loads((RUN/'raw/implement-1'/(label+'.json')).read_text())
 assert j['exit_code']==0 and Path(j['stdout_path']).read_text().strip()==TREE
save('verified-inputs.json',dict(observed_utc=now(),inputs=inputs,files=files,candidate_head=CAND,candidate_tree=TREE,initial_head=BASE,initial_tree=BT,clean=True,full_paths=paths,checks=checks,protected_paths_unchanged=True))
patch=Path(R['outputs']['patch']);data=run('generate-full-patch',P['patch_rule']['generate_argv'])
assert 0<len(data)<=8388608
with patch.open('xb') as f:f.write(data)
assert patch.read_bytes()==(RAW/'generate-full-patch.stdout').read_bytes()
assert CHECK.is_dir(),'declared check repo absent; stop rather than choose undeclared snapshot'
assert run('check-initial-identity',['git','rev-parse','HEAD','HEAD^{tree}'],CHECK).decode().splitlines()==[BASE,BT]
assert run('check-initial-status',['git','status','--porcelain=v1','--untracked-files=all'],CHECK)==b''
run('apply-check',['git','apply','--check',str(patch)],CHECK)
run('apply-index',['git','apply','--index',str(patch)],CHECK)
applied=run('applied-tree',['git','write-tree'],CHECK).decode().strip();assert applied==TREE
assert run('candidate-tree-final',['git','rev-parse','HEAD^{tree}']).decode().strip()==TREE
appliedpaths=run('applied-full-paths',['git','diff','--cached','--name-only','-z',BASE],CHECK).decode().strip('\0').split('\0')
assert sorted(appliedpaths)==sorted(ALLOW)
assert run('check-no-unstaged-diff',['git','diff','--exit-code'],CHECK)==b''
assert run('check-no-untracked',['git','ls-files','--others','--exclude-standard'],CHECK)==b''
appliedfiles={f:identity(CHECK/f) for f in ALLOW}
for f in ALLOW:assert (REPO/f).read_bytes()==(CHECK/f).read_bytes()
assert run('candidate-identity-final',['git','rev-parse','HEAD','HEAD^{tree}']).decode().splitlines()==[CAND,TREE]
assert run('candidate-status-final',['git','status','--porcelain=v1','--untracked-files=all'])==b''
for k,v in inputs.items():assert identity(v['path'])==v
save('application-verification.json',dict(end_utc=now(),check_repo=str(CHECK),preexisted=True,initial_head=BASE,initial_tree=BT,initial_clean=True,candidate_head=CAND,candidate_tree=TREE,applied_tree=applied,applied_paths=appliedpaths,candidate_files=files,applied_files=appliedfiles,byte_equal=True,full_tree_equal=True,protected_paths_unchanged=True,no_extra_untracked=True,patch=identity(patch),patch_equals_capture_stdout=True,source_still_clean_same_candidate=True,sealed_inputs_unchanged=True,command_records=records,final_quality='not_run',proxy_acceptance='not_run',protocol_eligibility='unresolved read-boundary deviation'))
print(json.dumps(dict(complete=True,end_utc=now(),patch=identity(patch),tree=applied,raw=str(RAW)),ensure_ascii=False))
