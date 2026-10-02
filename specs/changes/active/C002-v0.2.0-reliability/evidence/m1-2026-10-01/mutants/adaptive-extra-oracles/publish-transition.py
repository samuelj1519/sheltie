import argparse,hashlib,json,os,pathlib,shutil,sqlite3,subprocess,tempfile,time
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--source',required=True);p.add_argument('--output',required=True);p.add_argument('--kind',required=True);a=p.parse_args();out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=False);records=[];meta={'complete':False,'binary_sha256':hashlib.sha256(pathlib.Path(a.binary).read_bytes()).hexdigest(),'script_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()}
def save(p,v):p.write_text(json.dumps(v,sort_keys=True,indent=2)+'\n')
def envclean():
 env=os.environ.copy()
 for k in list(env):
  if k=='SHELTIE_FAILPOINT' or k.startswith('SHELTIE_TEST_'):env.pop(k)
 return env
def run(home,args,point=None):
 env=envclean()
 if point:env['SHELTIE_FAILPOINT']=point
 q=subprocess.run([a.binary,'--home',str(home),'--json',*args],env=env,capture_output=True,text=True,timeout=30);records.append({'argv':q.args,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr,'failpoint':point});return q.returncode,json.loads(q.stdout) if q.stdout else None
def facts(home):
 with sqlite3.connect(home/'store.db') as db:rows={t:sorted(repr(r) for r in db.execute('SELECT * FROM '+t)) for (t,) in db.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()}
 return {'rows':rows,'files':{str(p.relative_to(home)):hashlib.sha256(p.read_bytes()).hexdigest() for folder in ['works','workbooks','pending'] for p in (home/folder).rglob('*') if p.is_file()}}
try:
 with tempfile.TemporaryDirectory(prefix='m1-extra-transition-',dir='/private/tmp') as td:
  base=pathlib.Path(td);home=base/'home';fixture=base/'fixture';shutil.copytree(pathlib.Path(a.source)/'examples/two-step',fixture);sync=base/'sync';sync.mkdir();rid='extra-transition'
  if a.kind=='workbook':writer=['--request-id',rid,'workbook','add',str(fixture)];reader=['workbook','show','two-step@1.0.0']
  else:
   c,v=run(home,['workbook','add',str(fixture)]);assert c==0
   writer=['--request-id',rid,'work','start','--workbook','two-step','--flow','default','--input','topic=transition']
  c,v=run(home,writer,'after_commit_before_effects');assert c==70,'actual commit event failed'
  with sqlite3.connect(home/'store.db') as db:
   row=db.execute('SELECT * FROM requests WHERE request_id=?',(rid,)).fetchone();snapshot=json.loads(row[3]);effects=json.loads(row[4]);pending=home/next(o['pending'] for o in effects if o['kind']=='publish_dir');owner=home/'pending'/(pending.parent.name+'.owner');original_row=row
  assert owner.exists(),'fixture owner missing before transition'
  if a.kind=='work':reader=['work','status',snapshot['data']['work_id']]
  env=envclean();env.update(SHELTIE_TEST_RENDEZVOUS_NAME='extra_before_owner_validation',SHELTIE_TEST_RENDEZVOUS_ID=rid,SHELTIE_TEST_RENDEZVOUS_DIR=str(sync));argv=[a.binary,'--home',str(home),'--json',*reader];child=subprocess.Popen(argv,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  try:
   deadline=time.monotonic()+30
   while not(sync/'reached').exists():
    assert child.poll() is None,'fixture reader ended before owner event'
    if time.monotonic()>deadline:raise TimeoutError('owner event timeout, not detection')
    time.sleep(.005)
   c,v=run(home,writer);assert c==0 and v['ok'],'actual legal writer publication failed'
   with sqlite3.connect(home/'store.db') as db:current=db.execute('SELECT * FROM requests WHERE request_id=?',(rid,)).fetchone()
   assert original_row[:5]==current[:5] and original_row[6:]==current[6:] and original_row[5]==0 and current[5]==1 and not owner.exists(),'legal row transition was not exact published-only+cleanup'
   save(out/'transition.json',{'before':list(original_row),'after':list(current),'owner_before':True,'owner_after':owner.exists(),'writer_reply':v});before=facts(home);(sync/'release').touch();stdout,stderr=child.communicate(timeout=30);value=json.loads(stdout);records.append({'argv':argv,'exit':child.returncode,'stdout':stdout,'stderr':stderr});after=facts(home);save(out/'before-reader.json',before);save(out/'after-reader.json',after)
   assert child.returncode==0 and value['ok'],('SEMANTIC legitimate published-only transition rejected by stale readonly reader',value)
   assert before==after,('SEMANTIC readonly transition consumer mutated business facts',value)
   meta['complete']=True
  finally:
   (sync/'release').touch()
   if child.poll() is None:child.kill();child.communicate()
except Exception as e:meta['error']=repr(e);raise
finally:save(out/'metadata.json',meta);save(out/'commands.json',records)
