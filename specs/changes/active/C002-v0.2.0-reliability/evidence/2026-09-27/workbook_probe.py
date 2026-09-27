import os, tempfile
from pathlib import Path
REVIEW_REPO = Path(os.environ.get("SHELTIE_REVIEW_REPO", str(Path(__file__).resolve().parents[6])))
REVIEW_BIN = os.environ.get("SHELTIE_REVIEW_BIN", "")
import pathlib,subprocess,json,tempfile
root=pathlib.Path(tempfile.mkdtemp(prefix='sheltie-workbook-probe-',dir='/tmp')).resolve(); home=root/'home';home.mkdir(); binary=REVIEW_BIN;log=[]
def c(*args):
 p=subprocess.run([binary,'--home',str(home),'--json',*args],capture_output=True,text=True);o=json.loads(p.stdout);log.append({'args':args,'exit':p.returncode,'out':o});assert p.returncode==0,(args,o);return o
c('workbook','add',str(REVIEW_REPO/'workbooks/spec-dev')); work=c('work','start','--workbook','spec-dev','--flow','default','--name','probe','--input','request=mechanical fixture','--input','project=/tmp/no-project')['data']['work_id']
for node in ['spec','plan','plan-review','scaffold','implement','verify','escalate']:
 begin=c('attempt','begin',work,'--node',node)
 print(node,'bound inputs:',list(begin['data']['inputs']))
 for path in begin['data']['outputs'].values():pathlib.Path(path).write_text('Synthetic fixture: not real execution or approval.\n')
 result=c('attempt','submit',work,'--attempt',begin['data']['attempt'],'--summary','synthetic graph traversal')
 if node=='escalate':print('escalate legal targets:',[op.get('args',{}).get('node') for op in result['next']])
(root/'log.json').write_text(json.dumps(log,ensure_ascii=False,indent=2));print(root)
