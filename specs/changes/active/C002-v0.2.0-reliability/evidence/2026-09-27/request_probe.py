import os, tempfile
from pathlib import Path
REVIEW_REPO = Path(os.environ.get("SHELTIE_REVIEW_REPO", str(Path(__file__).resolve().parents[6])))
REVIEW_BIN = os.environ.get("SHELTIE_REVIEW_BIN", "")
import json, os, pathlib, subprocess, uuid, hashlib, sqlite3
ROOT=REVIEW_REPO
BIN=Path(REVIEW_BIN)
BASE=Path(tempfile.mkdtemp(prefix="sheltie-request-probe-"))
log=[]
def call(home,*args,env=None):
 p=subprocess.run([str(BIN),'--home',str(home),'--json',*args],capture_output=True,text=True,env=env)
 try: d=json.loads(p.stdout)
 except ValueError:d={'stdout':p.stdout}
 log.append({'argv':list(args),'home':str(home),'exit':p.returncode,'stdout':d,'stderr':p.stderr})
 return d
def setup(label):
 h=BASE/label; h.mkdir(); assert call(h,'workbook','add',str(ROOT/'examples/two-step'))['ok']; return h
def start(h,name,rid=None):
 args=['work','start','--workbook','two-step','--flow','default','--name',name,'--input','topic=t']
 if rid:args+=['--request-id',rid]
 d=call(h,*args); assert d['ok']; return d['data']['work_id'],args
h=setup('cross-work'); a,_=start(h,'a'); b,_=start(h,'b'); rid=str(uuid.uuid4())
first=call(h,'work','cancel',a,'--request-id',rid); second=call(h,'work','cancel',b,'--request-id',rid)
print('cross_work_cancel',json.dumps({'first':first,'second':second,'b_status':call(h,'work','status',b)},ensure_ascii=False))
h=setup('submit-replay'); w,_=start(h,'s'); begin=call(h,'attempt','begin',w,'--node','outline'); out=pathlib.Path(begin['data']['outputs']['outline']);out.write_text('outline');rid=str(uuid.uuid4());args=['attempt','submit',w,'--attempt','outline#1.0','--summary','done','--request-id',rid]; first=call(h,*args);out.unlink();again=call(h,*args)
print('submit_replay_deleted_output',json.dumps({'first_ok':first['ok'],'again':again},ensure_ascii=False))
h=setup('response-replay');w,_=start(h,'r');begin=call(h,'attempt','begin',w,'--node','outline');pathlib.Path(begin['data']['outputs']['outline']).write_text('outline');rid=str(uuid.uuid4());args=['attempt','submit',w,'--attempt','outline#1.0','--summary','done','--request-id',rid];first=call(h,*args);call(h,'work','cancel',w);again=call(h,*args)
print('submit_replay_changed_response',json.dumps({'first':first,'again':again},ensure_ascii=False))
h=setup('start-replay');rid=str(uuid.uuid4());w,args=start(h,'r',rid);call(h,'work','cancel',w);call(h,'workbook','remove','two-step@1.0.0');again=call(h,*args)
print('start_replay_removed_workbook',json.dumps(again,ensure_ascii=False))
h=setup('invalid-start');before=list((h/'works').glob('*'));bad=call(h,'work','start','--workbook','two-step','--flow','default');after=list((h/'works').glob('*'));print('invalid_start_leftovers',json.dumps({'response':bad,'before':[str(x) for x in before],'after':[str(x) for x in after]},ensure_ascii=False))
wb=BASE/'stats-wb';(wb/'flows').mkdir(parents=True);(wb/'workbook.toml').write_text('schema="workbook/v1"\nid="stats-probe"\nversion="1"\nname="Stats"\nflows=["flows/default.toml"]\n');(wb/'flows/default.toml').write_text('schema="flow/v1"\nid="default"\nentry="one"\n[[nodes]]\nid="one"\ntitle="one"\nexecutor="agent"\ninstruction={text="observe"}\ninputs=[{name="stats",from="engine.stats"}]\n')
h=BASE/'stats';h.mkdir();assert call(h,'workbook','add',str(wb))['ok'];s=call(h,'work','start','--workbook','stats-probe','--flow','default');w=s['data']['work_id'];rid=str(uuid.uuid4());args=['attempt','begin',w,'--node','one','--request-id',rid];b=call(h,*args);p=pathlib.Path(b['data']['inputs']['stats']);original=p.read_bytes();call(h,'attempt','submit',w,'--attempt','one#1.0','--summary','done');p.unlink();again=call(h,*args);restored=p.read_bytes();print('historical_stats_replay',json.dumps({'replay':again,'same':original==restored,'original':json.loads(original),'restored':json.loads(restored)},ensure_ascii=False))
(BASE/'raw.json').write_text(json.dumps(log,indent=2,ensure_ascii=False)); print('RAW',BASE/'raw.json')
