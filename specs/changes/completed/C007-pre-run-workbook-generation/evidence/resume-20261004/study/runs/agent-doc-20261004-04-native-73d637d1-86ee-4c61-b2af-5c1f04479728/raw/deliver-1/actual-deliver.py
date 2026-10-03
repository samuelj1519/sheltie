import datetime, hashlib, importlib.util, json, os, sys
from pathlib import Path
run=Path(sys.argv[1]); binding=json.loads((run/'run-binding.json').read_text()); study=Path(binding['study_dir'])
raw=run/'raw/deliver-1'; out=run/'outputs/deliver-1'; out.mkdir(exist_ok=False)
spec=importlib.util.spec_from_file_location('frozen_capture',study/'capture.py');cap=importlib.util.module_from_spec(spec);spec.loader.exec_module(cap)
repo=Path(binding['repo']); verify=Path(binding['patch_check_repo']); deadline=binding['deadline_utc']
base='351feb7ac22c21317a686693b732d5ae0c4b4bcc';basetree='80ea3046b1b3575f07e68313444c051ee7c5b7db';head='c9823575920795db14dfd378d77c8b311ce71d87';tree='404c3004895a9451d7cfc88b0c5a04400a4a0463';allow='specs/guides/continuity-choices.md'
def sha(b): return hashlib.sha256(b).hexdigest()
def save(name,obj):
 with (raw/name).open('x') as f:json.dump(obj,f,ensure_ascii=False,indent=2);f.write('\n')
def command(label,argv,cwd):
 record,stdout,stderr=cap.execute(raw,label,argv,cwd,120,deadline)
 if record['exit_code']!=0: raise RuntimeError('Actual execution failed; retained '+label)
 return stdout
save('actor.json',dict(actor_id='/root/c007_study_coordinator/run04_deliver',model_declared_inherited='gpt-6.1-sol',reasoning_effort_declared='high',override=False,provider_actual_metadata='unknown; Root separately verifies',usage='unknown',fees='unknown',start_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),deadline_utc=deadline,tools='exec_command, Python3, frozen capture.py.execute, Git',helpers=[],environment={n:os.environ.get(n) for n in ['PATH','LANG','LC_ALL','LC_CTYPE','GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_CONFIG_COUNT','GIT_CONFIG_GLOBAL','GIT_CONFIG_SYSTEM','GIT_CONFIG_NOSYSTEM','PYTHONDONTWRITEBYTECODE']},capture_sha256=sha((study/'capture.py').read_bytes())))
inputs={}
for kind,rel in [('change','outputs/implement-1/change.md'),('checks','outputs/implement-1/checks.md'),('review','outputs/review-1/review.md')]:
 p=run/rel;b=p.read_bytes();assert len(b)<=262144;assert head in b.decode() and tree in b.decode();inputs[kind]=dict(path=str(p),bytes=len(b),sha256=sha(b))
assert (run/'outputs/review-1/review.md').read_text().startswith('建议交付')
originals={}
for label in ['check-docs','check-staged-whitespace','staged-tree','post-check-tree']:
 p=run/'raw/implement-1'/f'{label}.json';rec=json.loads(p.read_text());assert rec['exit_code']==0 and not rec['timed_out'] and rec['within_deadline'];assert rec['cwd']==str(repo)
 for side in ['stdout','stderr']:assert sha(Path(rec[side+'_path']).read_bytes())==rec[side+'_sha256']
 if label in ['staged-tree','post-check-tree']:assert Path(rec['stdout_path']).read_text().strip()==tree
 originals[label]=dict(record_path=str(p),record_sha256=sha(p.read_bytes()),record=rec)
save('input-originals.json',dict(reports=inputs,checks=originals))
assert command('candidate-head',['git','rev-parse','HEAD','HEAD^{tree}'],repo).decode().splitlines()==[head,tree]
assert command('candidate-clean',['git','status','--porcelain=v1','--untracked-files=all'],repo)==b''
assert command('candidate-full-paths',['git','diff','--name-status','-z',base,'HEAD'],repo)==('A\0'+allow+'\0').encode()
assert verify.is_dir(), 'Declared verify repository missing; stop'
assert command('verify-initial-head',['git','rev-parse','HEAD','HEAD^{tree}'],verify).decode().splitlines()==[base,basetree]
assert command('verify-initial-clean',['git','status','--porcelain=v1','--untracked-files=all'],verify)==b''
paths=command('verify-initial-files',['git','ls-files','-z'],verify).decode().split('\0')[:-1]
def diskmeta(root,path):
 p=root/path
 if p.is_symlink():data=os.readlink(p).encode();kind='symlink'
 else:data=p.read_bytes();kind='file'
 return dict(kind=kind,bytes=len(data),sha256=sha(data),executable=bool(p.lstat().st_mode & 0o111))
protected_initial={p:diskmeta(verify,p) for p in paths};assert allow not in protected_initial
save('protected-initial.json',protected_initial)
project=json.loads((study/'samples/continuity/project.md').read_text());assert project['allow_files']==[allow]
patchbytes=command('generate-patch',project['patch_rule']['generate_argv'],repo);assert 0<len(patchbytes)<=8388608
patch=out/'change.patch'
with patch.open('xb') as f:f.write(patchbytes)
assert patch.read_bytes()==patchbytes
command('apply-check',['git','apply','--check',str(patch)],verify)
command('apply-index',['git','apply','--index',str(patch)],verify)
applied=command('applied-tree',['git','write-tree'],verify).decode().strip();assert applied==tree
assert command('applied-full-paths',['git','diff','--cached','--name-status','-z'],verify)==('A\0'+allow+'\0').encode()
assert command('applied-unstaged',['git','diff','--name-status','-z'],verify)==b''
assert command('applied-status',['git','status','--porcelain=v1','--untracked-files=all'],verify)==('A  '+allow+'\n').encode()
protected_after={p:diskmeta(verify,p) for p in paths};assert protected_after==protected_initial
source_paths=command('candidate-files',['git','ls-files','-z'],repo).decode().split('\0')[:-1]
applied_paths=command('applied-files',['git','ls-files','-z'],verify).decode().split('\0')[:-1];assert source_paths==applied_paths
source_disk={p:diskmeta(repo,p) for p in source_paths};applied_disk={p:diskmeta(verify,p) for p in applied_paths};assert source_disk==applied_disk
assert (repo/allow).read_bytes()==(verify/allow).read_bytes();assert source_disk[allow]['sha256']=='d7f53c99fae26656972b941b604cb2547fe5461790a973a7cb3dffed8cbc91aa'
assert command('candidate-final-head',['git','rev-parse','HEAD','HEAD^{tree}'],repo).decode().splitlines()==[head,tree]
assert command('candidate-final-clean',['git','status','--porcelain=v1','--untracked-files=all'],repo)==b''
summary=dict(conclusion='complete patch applied to declared independent initial repository',reports=inputs,patch=dict(path=str(patch),bytes=len(patchbytes),sha256=sha(patchbytes),source_stdout_path=str(raw/'generate-patch.stdout')),candidate=head,candidate_tree=tree,initial_head=base,initial_tree=basetree,applied_tree=applied,allowed_file=allow,authorized_bytes=source_disk[allow],full_path_set=[allow],protected_tracked_files=len(paths),all_candidate_tracked_files=len(source_paths),protected_initial_equals_after=True,all_candidate_applied_disk_bytes_equal=True,verify_head_remains_base=True,verify_repo=str(verify),source_repo=str(repo),checks_reused_originals_not_rerun=True,end_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),deadline_utc=deadline,remaining_s=(datetime.datetime.fromisoformat(deadline)-datetime.datetime.now(datetime.timezone.utc)).total_seconds())
save('closure-summary.json',summary);save('candidate-applied-file-manifest.json',dict(source=source_disk,applied=applied_disk));print(json.dumps(summary,ensure_ascii=False,indent=2))
