exec(open('/private/tmp/sheltie-m1-review-e1a8126/probes.py').read().split('# 1 Original')[0])
import hashlib, struct
# independent digest from filesystem bytes (Python sha256, explicit framing)
h=init();dir=h/'workbooks'/'two-step'/'1.0.0';files=sorted((p.relative_to(dir).as_posix().encode(),p.read_bytes()) for p in dir.rglob('*') if p.is_file());
stream=b'sheltie-workbook-digest/v2\x00'+struct.pack('>Q',len(files))
for path,data in files:stream+=struct.pack('>Q',len(path))+path+struct.pack('>Q',len(data))+data
conn=sqlite3.connect(h/'store.db');got=conn.execute('select digest from workbooks').fetchone()[0];conn.close();expect=hashlib.sha256(stream).hexdigest();assert got==expect;print('digest_independent',got)
# cross-work request ID collision leaves B state unchanged
v,c=start(h);a=v['data']['work_id'];v,c=start(h);b=v['data']['work_id'];v,c=run(h,['work','cancel',a],'shared');v,c=run(h,['work','cancel',b],'shared');assert c==1 and v['error']['code']=='REQUEST_CONFLICT';v,c=run(h,['work','status',b]);assert v['data']['status']['kind']=='active';print('cross_target_conflict','PASS')
# historical successful submit after cancel returns original data/revision/next
v,c=run(h,['attempt','begin',b,'--node','outline']);pathlib.Path(v['data']['outputs']['outline']).write_text('saved bytes')
argv=['attempt','submit',b,'--attempt','outline#1.0','--summary','literal summary'];first,c=run(h,argv,'original-submit');assert c==0
v,c=run(h,['work','cancel',b]);again,c=run(h,argv,'original-submit');assert c==0;f=dict(first);g=dict(again);f['data']=dict(f['data']);g['data']=dict(g['data']);f['data'].pop('replayed');g['data'].pop('replayed');assert f==g;print('history_snapshot','PASS')
# positive own pending recover produces card and original response
h=init();v,c=start(h,'good-pending','after_commit_before_effects');assert c==70;again,c=start(h,'good-pending');assert c==0 and again['data']['replayed'];w=again['data']['work_id'];assert (h/'works'/w/'status-card.md').exists();print('own_pending_positive','PASS')
# readonly pending Work reports marker per protocol? Actual state remains readable, pending flag absent.
h=init();v,c=start(h,'readonly-pending','after_commit_before_effects');conn=sqlite3.connect(h/'store.db');w=conn.execute('select work_id from requests where request_id=?',('readonly-pending',)).fetchone()[0];conn.close();v,c=run(h,['work','status',w]);print('pending_work_status',c,'flag',v['data'].get('pending_publish'),'status',v['data']['status'])
BASE.joinpath('controls.json').write_text(json.dumps(logs,ensure_ascii=False,indent=2))
