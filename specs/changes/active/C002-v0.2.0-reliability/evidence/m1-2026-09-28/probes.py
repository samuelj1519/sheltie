import pathlib, tempfile, subprocess, os, json, sqlite3
BASE=pathlib.Path('/private/tmp/sheltie-m1-review-e1a8126')
BIN='/private/tmp/sheltie-m1-review-target/debug/sheltie'
WB=str(pathlib.Path('examples/two-step').resolve())
logs=[]
def run(home,args,rid=None,fp=None):
 env=os.environ.copy(); env['SHELTIE_HOME']=str(home); env.pop('SHELTIE_FAILPOINT',None)
 if fp: env['SHELTIE_FAILPOINT']=fp
 argv=[BIN,'--json']+(['--request-id',rid] if rid else [])+args
 p=subprocess.run(argv,env=env,capture_output=True,text=True)
 try: v=json.loads(p.stdout)
 except: v=None
 logs.append(dict(argv=argv,home=str(home),failpoint=fp,exit=p.returncode,stdout=p.stdout,stderr=p.stderr))
 return v,p.returncode

def init():
 home=pathlib.Path(tempfile.mkdtemp(prefix='m1-probe-',dir='/private/tmp'))
 v,c=run(home,['workbook','add',WB]); assert c==0,(v,c)
 return home

def start(home,rid=None,fp=None):
 return run(home,['work','start','--workbook','two-step','--flow','default','--input','topic=probe'],rid,fp)

# 1 Original @summary accepted; removed file must not affect replay.
h=init(); v,c=start(h); w=v['data']['work_id']; v,c=run(h,['attempt','begin',w,'--node','outline']);
p=pathlib.Path(v['data']['outputs']['outline']); p.write_text('independent output')
s=h/'summary-source.txt'; s.write_text('original summary')
argv=['attempt','submit',w,'--attempt','outline#1.0','--summary','@'+str(s)]
v,c=run(h,argv,'summary-replay'); assert c==0,(v,c); s.unlink(); replay,rc=run(h,argv,'summary-replay')
print('summary_deleted_replay',rc,replay)
# 2 card projection failure after commit must be EFFECT_PENDING with original+revision.
h=init(); v,c=start(h); w=v['data']['work_id']; card=h/'works'/w/'status-card.md'; card.unlink(); card.mkdir()
v,c=run(h,['attempt','begin',w,'--node','outline'],'card-failure')
conn=sqlite3.connect(h/'store.db'); row=conn.execute('select published,reply_json from requests where request_id=?',('card-failure',)).fetchone();conn.close()
print('card_failure',c,v,'committed_row',row)
# 3 replay pending A vs B blocked response must identify A/B and truthful committed.
h=init(); v,c=start(h,'pending-start','after_commit_before_effects'); assert c==70,(v,c)
conn=sqlite3.connect(h/'store.db'); row=conn.execute('select work_id,effects_json from requests where request_id=?',('pending-start',)).fetchone(); conn.close()
w=row[0]; op=json.loads(row[1])[0]; src=h/op['pending']/op['digest_root']/'workbook.toml'; src.chmod(0o644); src.write_text(src.read_text()+'\n# changed\n')
same,c= start(h,'pending-start'); print('own_pending_replay',c,same)
other,c=run(h,['work','cancel',w],'new-blocked-request'); print('other_pending_blocked',c,other)
# 4 replay published begin with missing parent: EFFECT_PENDING, not STORE_CORRUPT.
h=init(); v,c=start(h);w=v['data']['work_id']; v,c=run(h,['attempt','begin',w,'--node','outline'],'begin-parent'); brief=pathlib.Path(v['data']['brief_path']); brief.unlink();
import shutil
shutil.rmtree(brief.parent)
v,c=run(h,['attempt','begin',w,'--node','outline'],'begin-parent'); print('begin_missing_parent_replay',c,v)
# 5 deletion complete, crash before marker -> unknown outcome; erase completion marker + unpublish row to recreate window.
h=init(); v,c=run(h,['workbook','remove','two-step@1.0.0'],'remove-window'); assert c==0,(v,c)
conn=sqlite3.connect(h/'store.db'); row=conn.execute('select effects_json from requests where request_id=?',('remove-window',)).fetchone(); op=json.loads(row[0])[0]; pen=h/op['pending']; marker=h/'pending'/'payload.deleted'; print('marker_candidate',marker, marker.exists());
marker.unlink(); conn.execute('update requests set published=0 where request_id=?',('remove-window',)); conn.commit();conn.close()
v,c=run(h,['workbook','add',WB],'after-unknown-delete')
conn=sqlite3.connect(h/'store.db'); published=conn.execute('select published from requests where request_id=?',('remove-window',)).fetchone()[0];conn.close()
print('unknown_delete_recovery',c,v,'remove_published',published,'marker_recreated',marker.exists())
BASE.joinpath('probes.json').write_text(json.dumps(logs,ensure_ascii=False,indent=2))
