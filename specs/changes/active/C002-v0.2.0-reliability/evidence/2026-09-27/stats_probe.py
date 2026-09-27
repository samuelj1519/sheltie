import os, tempfile
from pathlib import Path
REVIEW_REPO = Path(os.environ.get("SHELTIE_REVIEW_REPO", str(Path(__file__).resolve().parents[6])))
REVIEW_BIN = os.environ.get("SHELTIE_REVIEW_BIN", "")
import pathlib,tempfile,subprocess,json
root=pathlib.Path(tempfile.mkdtemp(prefix='sheltie-stats-probe-',dir='/tmp')).resolve();home=root/'home';home.mkdir();wb=root/'wb';wb.mkdir();binary=REVIEW_BIN;log=[]
(wb/'workbook.toml').write_text('schema="workbook/v1"\nid="stats-probe"\nversion="1.0.0"\nname="probe"\nflows=["flow.toml"]\n')
flow='schema="flow/v1"\nid="default"\nentry="a"\n'
for n,visits in [('a',2),('b',1),('c',1)]:flow+=f'[[nodes]]\nid="{n}"\ntitle="{n}"\nexecutor="agent"\ninstruction={{text="Synthetic fixture"}}\nmax_visits={visits}\n'
for a,b,kind in [('a','b','main'),('b','a','back'),('b','c','main')]:flow+=f'[[edges]]\nfrom="{a}"\nto="{b}"\nkind="{kind}"\n'
(wb/'flow.toml').write_text(flow)
def c(*args):
 p=subprocess.run([binary,'--home',str(home),'--json',*args],capture_output=True,text=True);o=json.loads(p.stdout);log.append({'args':args,'out':o});assert p.returncode==0,o;return o
c('workbook','add',str(wb));work=c('work','start','--workbook','stats-probe','--flow','default')['data']['work_id']
for node in ['a','b','a']:
 a=c('attempt','begin',work,'--node',node)['data']['attempt'];c('attempt','submit',work,'--attempt',a,'--summary','synthetic traversal')
before=c('work','stats',work)['data'];c('work','cancel',work);after=c('work','stats',work)['data'];print('blocked_count before cancel',before['blocked_count'],'after',after['blocked_count']); print('a entered_via',before['nodes'][0]['entered_via']);(root/'log.json').write_text(json.dumps(log,ensure_ascii=False,indent=2));print(root)
