import os, pathlib, subprocess, json, sqlite3, shutil, tempfile
BASE=pathlib.Path(tempfile.mkdtemp(prefix='probe-',dir='/private/tmp/sheltie-m1-review-e1a8126/spec'));print(json.dumps({'probe_root':str(BASE)}))
BIN='/private/tmp/sheltie-m1-review-target/debug/sheltie'
SRC='/Users/shushu/orca/workspaces/sheltie/codex/examples/two-step'
def writable(root):
 for path in [root,*root.rglob('*')]: path.chmod(0o755 if path.is_dir() else 0o644)
def call(home,*args):
 p=subprocess.run([BIN,'--home',str(home),'--json',*args],capture_output=True,text=True)
 out={'argv':args,'rc':p.returncode,'stdout':p.stdout,'stderr':p.stderr};print(json.dumps(out,ensure_ascii=False));return json.loads(p.stdout)
home=BASE/'pending-home';call(home,'--request-id','r-add','workbook','add',SRC)
db=sqlite3.connect(home/'store.db');effects=json.loads(db.execute("select effects_json from requests where request_id='r-add'").fetchone()[0]);pen=home/effects[0]['pending'];fin=home/effects[0]['final'];pen.parent.mkdir(parents=True,exist_ok=True);writable(fin);fin.rename(pen);db.execute("update requests set published=0 where request_id='r-add'");db.commit()
call(home,'workbook','list');call(home,'workbook','show','two-step');call(home,'workbook','verify','two-step@1.0.0');call(home,'--request-id','r-start','work','start','--workbook','two-step','--flow','default','--name','pending','--input','topic=x');call(home,'--request-id','r-add','workbook','add',SRC)
print(json.dumps({'final_exists_after_replay':fin.exists(),'pending_exists_after_replay':pen.exists(),'published':db.execute("select published from requests where request_id='r-add'").fetchone()[0]}))
# Single-condition root symlink: valid registered bytes moved outside home.
home2=BASE/'symlink-home';call(home2,'workbook','add',SRC);fin2=home2/'workbooks/two-step/1.0.0';outside=BASE/'outside-workbook';writable(fin2);fin2.rename(outside);fin2.symlink_to(outside,target_is_directory=True);call(home2,'workbook','verify','two-step@1.0.0');call(home2,'workbook','show','two-step')
