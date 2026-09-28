exec(open('/private/tmp/sheltie-m1-review-e1a8126/probes.py').read().split('# 1 Original')[0])
import shutil, hashlib
# Original O05: advance state, remove historical stats, replay begin; exact original bytes and card oracle.
source=pathlib.Path(tempfile.mkdtemp(prefix='m1-stats-wb-',dir='/private/tmp'));shutil.copytree(pathlib.Path('examples/two-step'),source,dirs_exist_ok=True)
flow=source/'flows/default.toml';text=flow.read_text();old='inputs  = [{ name = "topic", from = "start.topic" }]';assert text.count(old)==1;flow.write_text(text.replace(old,'inputs  = [{ name = "topic", from = "start.topic" }, { name = "stats", from = "engine.stats" }]'))
h=pathlib.Path(tempfile.mkdtemp(prefix='m1-stats-home-',dir='/private/tmp'));v,c=run(h,['workbook','add',str(source)]);assert c==0;v,c=start(h);assert c==0;w=v['data']['work_id']
v,c=run(h,['attempt','begin',w,'--node','outline'],'stats-begin');assert c==0;stats=pathlib.Path(v['data']['inputs']['stats']);original=stats.read_bytes();pathlib.Path(v['data']['outputs']['outline']).write_text('real output');v,c=run(h,['attempt','submit',w,'--attempt','outline#1.0','--summary','advance']);assert c==0
card=h/'works'/w/'status-card.md';card_before=card.read_bytes();stats.unlink();v,c=run(h,['attempt','begin',w,'--node','outline'],'stats-begin');assert c==0;restored=stats.read_bytes();assert restored==original;assert card.read_bytes()==card_before
print(json.dumps({'control':'stats_after_state_advance','exit':c,'original_utf8':original.decode(),'restored_sha256':hashlib.sha256(restored).hexdigest(),'original_sha256':hashlib.sha256(original).hexdigest(),'card_unchanged':True},ensure_ascii=False))
# Effects path pair: actual registered begin effects, only WriteFile.path differs.
for change in [False,True]:
 h=init();v,c=start(h);w=v['data']['work_id'];v,c=run(h,['attempt','begin',w,'--node','outline'],'pair-begin');assert c==0
 conn=sqlite3.connect(h/'store.db');raw=conn.execute('select effects_json from requests where request_id=?',('pair-begin',)).fetchone()[0];ops=json.loads(raw);write=next(op for op in ops if op['kind']=='write_file');oldpath=write['path']
 outside=pathlib.Path(tempfile.mkdtemp(prefix='m1-effects-outside-',dir='/private/tmp'));sentinel=outside/'payload.md'
 if change:write['path']='../'+outside.name+'/payload.md'
 conn.execute('update requests set effects_json=?, published=0 where request_id=?',(json.dumps(ops,separators=(',',':')),'pair-begin'));conn.commit();conn.close()
 v,c=run(h,['work','cancel',w],'pair-cancel');assert c==0
 print(json.dumps({'control':'registered_effect_path_pair','changed_only_path':change,'from':oldpath,'to':write['path'],'exit':c,'outside_created':sentinel.exists(),'outside_sha256':hashlib.sha256(sentinel.read_bytes()).hexdigest() if sentinel.exists() else None,'registered_sha256':write['sha256']},ensure_ascii=False))
BASE.joinpath('extra-controls.json').write_text(json.dumps(logs,ensure_ascii=False,indent=2))
