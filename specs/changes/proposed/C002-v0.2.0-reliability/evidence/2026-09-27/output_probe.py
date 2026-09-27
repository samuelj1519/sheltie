import os, tempfile
from pathlib import Path
REVIEW_REPO = Path(os.environ.get("SHELTIE_REVIEW_REPO", str(Path(__file__).resolve().parents[6])))
REVIEW_BIN = os.environ.get("SHELTIE_REVIEW_BIN", "")
import pathlib, subprocess, json, tempfile
ROOT=pathlib.Path(tempfile.mkdtemp(prefix='sheltie-spec-probe-',dir='/tmp')).resolve()
BIN=REVIEW_BIN
log=[]
def call(home,*args):
 p=subprocess.run([BIN,'--home',str(home),'--json',*args],text=True,capture_output=True)
 o=json.loads(p.stdout) if p.stdout else {'stderr':p.stderr}
 print(args, p.returncode, p.stdout, p.stderr); log.append({'args':args,'exit':p.returncode,'output':o,'stderr':p.stderr})
 return o
for label, outputs in [('stats', '[{name="result",path="stats.json"}]'),('brief-child','[{name="result",path="brief.md/out"}]'),('prefix','[{name="a",path="out"},{name="b",path="out/sub"}]')]:
 wb=ROOT/label; wb.mkdir(); (wb/'flow.toml').write_text('schema="flow/v1"\nid="default"\nentry="a"\n[[nodes]]\nid="a"\ntitle="A"\nexecutor="agent"\ninstruction={text="Produce output"}\ninputs=[{name="stats",from="engine.stats"}]\noutputs='+outputs+'\n')
 (wb/'workbook.toml').write_text('schema="workbook/v1"\nid="'+label+'"\nversion="1.0.0"\nname="Probe"\nflows=["flow.toml"]\n')
 home=ROOT/(label+'-home'); home.mkdir(); call(home,'workbook','add',str(wb)); started=call(home,'work','start','--workbook',label,'--flow','default','--name','probe'); work=started['data']['work_id']; call(home,'attempt','begin',work,'--node','a'); call(home,'attempt','submit',work,'--attempt','a#1.0','--summary','no worker wrote anything'); call(home,'work','status',work)
(ROOT/'log.json').write_text(json.dumps(log,ensure_ascii=False,indent=2)); print(ROOT)
for x in log: print(x['args'][:2],x['exit'], json.dumps(x['output'],ensure_ascii=False)[:450])
