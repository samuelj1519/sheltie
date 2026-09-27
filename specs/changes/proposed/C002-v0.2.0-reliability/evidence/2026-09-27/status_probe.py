import json, os, subprocess, tempfile
from pathlib import Path
repo=Path(os.environ.get('SHELTIE_REVIEW_REPO',str(Path(__file__).resolve().parents[6])))
binary=os.environ['SHELTIE_REVIEW_BIN'];root=Path(tempfile.mkdtemp(prefix='sheltie-status-probe-'));rows=[]
def run(*args,structured=True):
 p=subprocess.run([binary,'--home',str(root),*(['--json'] if structured else []),*args],capture_output=True,text=True)
 data=json.loads(p.stdout) if structured else p.stdout
 rows.append({'args':args,'exit':p.returncode,'output':data,'stderr':p.stderr});return data
run('workbook','add',str(repo/'examples/two-step'))
w=run('work','start','--workbook','two-step','--flow','default','--input','topic=fixture')['data']['work_id']
run('attempt','begin',w,'--node','outline');run('attempt','fail',w,'--attempt','outline#1.0','--reason','distinct failure reason 123');run('work','status',w);run('work','status',w,structured=False)
(root/'raw.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2));print(json.dumps(rows,ensure_ascii=False,indent=2));print(root)
