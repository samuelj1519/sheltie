exec(open('/private/tmp/sheltie-m1-review-e1a8126/probes.py').read().split('# 1 Original')[0])
# A Before-commit Workbook crash: orphan private pending must be safely cleaned on next write.
h=pathlib.Path(tempfile.mkdtemp(prefix='m1-precommit-',dir='/private/tmp')); v,c=run(h,['workbook','add',WB],'pre-add','before_commit'); assert c==70
before=sorted(str(x.relative_to(h)) for x in (h/'pending').rglob('*'))
v,c=run(h,['workbook','add',WB],'retry-add')
after=sorted(str(x.relative_to(h)) for x in (h/'pending').rglob('*'))
print('precommit_pending',before,'after_next_write',after,'exit',c)
# B Actual Work start kill then Workbook write must recover Work card, not skip it.
h=init(); v,c=start(h,'recover-by-workbook','after_commit_before_effects'); assert c==70
conn=sqlite3.connect(h/'store.db'); w=conn.execute('select work_id from requests where request_id=?',('recover-by-workbook',)).fetchone()[0];conn.close()
v,c=run(h,['workbook','add',str(pathlib.Path('examples/article-review').resolve())],'new-workbook')
conn=sqlite3.connect(h/'store.db'); pub=conn.execute('select published from requests where request_id=?',('recover-by-workbook',)).fetchone()[0];conn.close()
print('work_recovered_by_workbook','exit',c,'published',pub,'card_exists',(h/'works'/w/'status-card.md').exists())
# C Each single condition: corrupt one start input OR corrupt only owner sidecar.
for condition in ['input','owner']:
 h=init(); v,c=start(h,'input-recovery','after_commit_before_effects'); assert c==70
 conn=sqlite3.connect(h/'store.db'); w,effects=conn.execute('select work_id,effects_json from requests where request_id=?',('input-recovery',)).fetchone(); conn.close()
 payload=h/json.loads(effects)[0]['pending']
 if condition=='input': (payload/'start-inputs'/'topic').write_text('CHANGED BY EXTERNAL ACTOR')
 else: (h/'pending'/(payload.parent.name+'.owner')).write_text('INVALID SIDECAR')
 v,c=run(h,['work','cancel',w],'cancel-after-tamper')
 print('tampered_pending_'+condition,'exit',c,'reply',v,'final_input',(h/'works'/w/'start-inputs'/'topic').read_text())
# D Replay of original Work prefix must retain original target once prefix becomes ambiguous.
h=init(); v,c=start(h);w=v['data']['work_id'];prefix=w[:10]
v,c=run(h,['attempt','begin',prefix,'--node','outline'],'prefix-begin'); assert c==0
v,c=start(h);v,c=run(h,['attempt','begin',prefix,'--node','outline'],'prefix-begin');print('prefix_replay',c,v)
# E AtFile start must replay after input source deletion.
h=init();source=h/'topic.txt';source.write_text('original');argv=['work','start','--workbook','two-step','--flow','default','--input','topic=@'+str(source)]
v,c=run(h,argv,'start-file');assert c==0;source.unlink();v,c=run(h,argv,'start-file');print('start_atfile_replay',c,v)
BASE.joinpath('recovery-probes.json').write_text(json.dumps(logs,ensure_ascii=False,indent=2))
