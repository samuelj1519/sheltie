import os, tempfile
from pathlib import Path
REVIEW_REPO = Path(os.environ.get("SHELTIE_REVIEW_REPO", str(Path(__file__).resolve().parents[6])))
REVIEW_BIN = os.environ.get("SHELTIE_REVIEW_BIN", "")
from pathlib import Path
import subprocess,json,tempfile,os,hashlib,sqlite3,shutil
root=Path(tempfile.mkdtemp(prefix='sheltie-standards-'))
bin=REVIEW_BIN
repo=REVIEW_REPO
def run(home,*args,env=None):
 p=subprocess.run([bin,'--home',str(home),'--json',*args],capture_output=True,text=True,env=env)
 try: v=json.loads(p.stdout)
 except: v=p.stdout
 return {'exit':p.returncode,'stdout':v,'stderr':p.stderr}
def add(home):
 home.mkdir(parents=True,exist_ok=True)
 return run(home,'workbook','add',str(repo/'examples/two-step'))
def start(home): return run(home,'work','start','--workbook','two-step','--flow','default','--input','topic=x')
out={'root':str(root)}
out['fresh_install']=run(root/'missing'/'home','self','install')
h=root/'digest';a=add(h);src=repo/'examples/two-step';blob=b''.join(str(p.relative_to(src)).encode()+b'\0'+p.read_bytes() for p in sorted(src.rglob('*')) if p.is_file());out['digest']={'got':a['stdout']['data']['digest'],'expected':hashlib.sha256(blob).hexdigest(),'double':hashlib.sha256(hashlib.sha256(blob).digest()).hexdigest()}
f=h/'workbooks/two-step/1.0.0/instructions/outline.md';f.chmod(0o644);f.write_text('Changed instruction');out['tamper_verify']=run(h,'workbook','verify');out['tamper_start']=start(h)
h=root/'escape';add(h);outside=root/'outside';outside.mkdir();(h/'works').symlink_to(outside,target_is_directory=True);out['symlink_start']=start(h);out['symlink_outside_files']=[str(p.relative_to(outside)) for p in outside.rglob('*')]
h=root/'pending';add(h);s=start(h);wid=s['stdout']['data']['work_id'];target=root/'outside-pending.txt';target.write_text('Original');(h/'works'/wid/'status-card.tmp-pending').symlink_to(target);out['pending_write']=run(h,'work','cancel',wid);out['pending_target']=target.read_text()
h=root/'add-failure';h.mkdir();d=h/'workbooks/two-step/1.0.0';d.mkdir(parents=True);(d/'collision').write_text('occupied');out['add_rename_failure']=add(h);out['after_failed_add_list']=run(h,'workbook','list');out['after_failed_add_retry']=add(h)
h=root/'replay-add';h.mkdir();out['add_request_first']=run(h,'--request-id','f47ac10b-58cc-4372-a567-0e02b2c3d479','workbook','add',str(src));out['add_request_replay']=run(h,'--request-id','f47ac10b-58cc-4372-a567-0e02b2c3d479','workbook','add',str(src))
h=root/'json-install';h.mkdir();fake=root/'fake-user-home';fake.mkdir();env=dict(os.environ,HOME=str(fake),SHELL='/bin/zsh');out['json_modify_path']=run(h,'self','install','--modify-path',env=env)
(root/'raw.json').write_text(json.dumps(out,ensure_ascii=False,indent=2));print(json.dumps(out,ensure_ascii=False,indent=2))
