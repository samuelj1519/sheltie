import datetime, hashlib, importlib.util, json, os
from pathlib import Path
study=Path('/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study')
run=study/'runs/agent-doc-20261004-06-sheltie-425e9ca4-c426-4002-b29f-12a34464ab17'
raw=run/'raw/deliver-1'
binding=json.loads((run/'run-binding.json').read_text())
begin=json.loads((run/'begin-deliver-1-reply.json').read_text())['data']
project=json.loads((study/'samples/acceptance-handoff/project.md').read_text())
repo=Path(binding['repo']); verify=Path(binding['patch_check_repo'])
base=project['initial_head']; base_tree=project['initial_tree']
candidate='38eca4cf78bbbae002c9d3f255c680f8968dfe12'; tree='1b6deb6e862754a44d001253bf263745e5ad8b4d'
allow=project['allow_files']; deadline=binding['deadline_utc']
sha=lambda b:hashlib.sha256(b).hexdigest()
started=datetime.datetime.now(datetime.timezone.utc).isoformat()
spec=importlib.util.spec_from_file_location('frozen_capture',study/'capture.py'); cap=importlib.util.module_from_spec(spec); spec.loader.exec_module(cap)
def dump(name,value):
    with (raw/name).open('x') as f: json.dump(value,f,ensure_ascii=False,indent=2); f.write('\n')
def command(label,argv,cwd=repo):
    record,out,err=cap.execute(raw,label,argv,cwd,120,deadline)
    if record['exit_code']!=0: raise RuntimeError('Actual failure; retained '+label)
    return out
input_expected={'task':'c92e416ac263ac2f00efd3ced03febc81d78055c1ad77e97f742bac3cdd79f92','change':'c4a913be93f84d04af1b534f63cec8cbeeafe63e9b40d8c4cd790e267b81cb72','checks':'2963800988b22523fa2e8675c4cf5558134fc3a1075c9770d27525afcc836bc2','review':'52777bb41290221352023bac9eeb62578b49f72cf3b4e884209c7269d53bd783'}
inputs={}
for key,path in begin['inputs'].items():
    data=Path(path).read_bytes(); inputs[key]={'path':path,'bytes':len(data),'sha256':sha(data)}
    assert sha(data)==input_expected[key],('input drift',key)
for key in ('delivery','patch'): assert not Path(begin['outputs'][key]).exists(),('output exists',key)
assert verify.is_dir(),'Missing declared independent verification copy; no replacement location'
checks={}
for label in ('check-docs','check-diff'):
    record_path=run/'raw/implement-1'/f'{label}.json'; record=json.loads(record_path.read_text())
    out=Path(record['stdout_path']).read_bytes(); err=Path(record['stderr_path']).read_bytes()
    assert record['exit_code']==0 and not record['timed_out'] and record['within_deadline']
    assert record['cwd']==str(repo) and sha(out)==record['stdout_sha256'] and sha(err)==record['stderr_sha256']
    assert record['argv']==project['checks'][0 if label=='check-docs' else 1]
    checks[label]={'record_path':str(record_path),'record_sha256':sha(record_path.read_bytes()),'record':record,'stdout_bytes':len(out),'stderr_bytes':len(err)}
staged=json.loads((run/'raw/implement-1/staged-candidate.json').read_text())
dump('input-and-check-bindings.json',{'inputs':inputs,'checks':checks,'staged_candidate_record':staged,'capture_sha256':sha((study/'capture.py').read_bytes()),'environment':{k:os.environ.get(k) for k in ['PATH','LANG','LC_ALL','GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_OBJECT_DIRECTORY','GIT_ALTERNATE_OBJECT_DIRECTORIES','SHELTIE_HOME']}})
head=command('source-head',['git','rev-parse','HEAD']).decode().strip(); source_tree=command('source-tree',['git','rev-parse','HEAD^{tree}']).decode().strip()
assert head==candidate and source_tree==tree
assert command('source-clean',['git','status','--porcelain=v1','--untracked-files=all'])==b''
paths=command('source-full-path-set',['git','diff','--name-only','-z',base,'HEAD']).split(b'\0'); paths=[p.decode() for p in paths if p]
assert paths==allow,(paths,allow)
assert command('source-base-tree',['git','rev-parse',base+'^{tree}']).decode().strip()==base_tree
verify_head=command('verify-initial-head',['git','rev-parse','HEAD'],verify).decode().strip(); verify_tree=command('verify-initial-tree',['git','rev-parse','HEAD^{tree}'],verify).decode().strip()
assert verify_head==base and verify_tree==base_tree,'Non-initial verify copy; retained without reset'
assert command('verify-initial-clean',['git','status','--porcelain=v1','--untracked-files=all'],verify)==b''
before_index=command('verify-before-index',['git','ls-files','-s','-z'],verify)
patch=command('generate-complete-patch',project['patch_rule']['generate_argv'])
assert len(patch)<=8388608
assert sha(patch)=='4bdb327e59615761ef331cd2fed5679901d83b231ad57aa270cecba07b95bf26'
assert patch==(run/'raw/implement-1/complete-patch.stdout').read_bytes()
patch_path=Path(begin['outputs']['patch'])
with patch_path.open('xb') as f:f.write(patch)
assert patch_path.read_bytes()==patch
command('apply-check',['git','apply','--check',str(patch_path)],verify)
command('apply-index',['git','apply','--index',str(patch_path)],verify)
applied_tree=command('applied-tree',['git','write-tree'],verify).decode().strip(); assert applied_tree==tree
assert command('verify-final-head',['git','rev-parse','HEAD'],verify).decode().strip()==base
applied_paths=command('applied-full-path-set',['git','diff','--cached','--name-only','-z',base],verify).split(b'\0'); applied_paths=[p.decode() for p in applied_paths if p];assert applied_paths==allow
assert command('verify-unstaged-clean',['git','diff','--name-only','-z'],verify)==b''
assert command('verify-no-untracked',['git','ls-files','--others','--exclude-standard','-z'],verify)==b''
after_index=command('verify-after-index',['git','ls-files','-s','-z'],verify)
def protected_index(data):
    return [entry for entry in data.split(b'\0') if entry and entry.split(b'\t',1)[1].decode() not in allow]
assert protected_index(before_index)==protected_index(after_index),'Protected tracked index changed'
file_records={}
for number,path in enumerate(allow):
    candidate_bytes=command('candidate-blob-'+str(number),['git','show',candidate+':'+path])
    source_bytes=(repo/path).read_bytes(); applied_bytes=(verify/path).read_bytes()
    assert source_bytes==candidate_bytes==applied_bytes
    file_records[path]={'bytes':len(applied_bytes),'sha256':sha(applied_bytes),'source_candidate_applied_bytes_equal':True}
assert file_records[allow[0]]['sha256']=='2cd606363a2efdffb8b28e83c21904017315be3794c9df03e0e33f8f8a70ff45'
assert command('source-final-head',['git','rev-parse','HEAD']).decode().strip()==candidate
assert command('source-final-tree',['git','rev-parse','HEAD^{tree}']).decode().strip()==tree
assert command('source-final-clean',['git','status','--porcelain=v1','--untracked-files=all'])==b''
# The actual independent copy remains with the patch staged; no commit, reset, or source edits.
ended=datetime.datetime.now(datetime.timezone.utc).isoformat()
summary={'conclusion':'independent patch application verified','actor':'/root/c007_study_coordinator/run06_deliver','started_utc':started,'ended_utc':ended,'deadline_utc':deadline,'candidate':candidate,'candidate_tree':tree,'initial_head':base,'initial_tree':base_tree,'verify_repo':str(verify),'applied_tree':applied_tree,'verify_head_still_initial':True,'source_clean_and_candidate_unchanged':True,'full_source_and_applied_path_set':paths,'protected_tracked_index_unchanged':True,'patch':{'path':str(patch_path),'bytes':len(patch),'sha256':sha(patch)},'files':file_records,'inputs':inputs,'raw_directory':str(raw),'commands_per_timeout_seconds':120,'remaining_seconds':(datetime.datetime.fromisoformat(deadline)-datetime.datetime.now(datetime.timezone.utc)).total_seconds()}
dump('application-verification.json',summary)
dump('actor.json',{'actor':'/root/c007_study_coordinator/run06_deliver','role':'actual deliver worker','declared_inherited_model':'gpt-6.1-sol','declared_inherited_reasoning_effort':'high','override':False,'actual_provider_model_metadata':'unknown; coordinator verifies host metadata separately','usage':'unknown','fee':'unknown','human_cost':'unknown','session_refs':'unknown; coordinator verifies actual host records separately','self_started_background_sessions':[],'helpers':[],'tools_used':['functions.exec','exec_command','clock__curr_time'],'started_utc':started,'ended_utc':ended,'early_read_operations':'Actual tool responses retained; not separately captured, no synthetic command evidence','owned_outputs':begin['outputs'],'raw_directory':str(raw)})
print(json.dumps(summary,ensure_ascii=False,indent=2))
