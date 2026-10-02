import argparse,hashlib,json,os,pathlib,shutil,sqlite3,subprocess,tempfile,time
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--source',required=True);p.add_argument('--output',required=True);a=p.parse_args();out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=False);records=[];meta={'complete':False,'binary_sha256':hashlib.sha256(pathlib.Path(a.binary).read_bytes()).hexdigest(),'script_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()}
def save(path,obj):path.write_text(json.dumps(obj,indent=2,sort_keys=True)+'\n')
try:
 with tempfile.TemporaryDirectory(prefix='m1-extra-tree-',dir='/private/tmp') as td:
  base=pathlib.Path(td);home=base/'home';sync=base/'sync';sync.mkdir();trace=base/'trace';fixture=base/'fixture';shutil.copytree(pathlib.Path(a.source)/'examples/two-step',fixture)
  env=os.environ.copy()
  for k in list(env):
   if k=='SHELTIE_FAILPOINT' or k.startswith('SHELTIE_TEST_'):env.pop(k)
  argv=[a.binary,'--home',str(home),'--json','--request-id','extra-tree','workbook','add',str(fixture)]
  env.update(SHELTIE_TEST_RENDEZVOUS_NAME='publish_after_tree_sync',SHELTIE_TEST_RENDEZVOUS_ID='extra-tree',SHELTIE_TEST_RENDEZVOUS_DIR=str(sync),SHELTIE_EXTRA_ACCEPT_TREE=str(trace))
  child=subprocess.Popen(argv,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  try:
   deadline=time.monotonic()+30
   while not(sync/'reached').exists():
    assert child.poll() is None,'tool/fixture process ended before exact event'
    if time.monotonic()>deadline:raise TimeoutError('event timeout, not detection')
    time.sleep(.005)
   payloads=list((home/'pending').glob('*/payload'));assert len(payloads)==1
   payload=payloads[0];original=payload.stat();held=payload.with_name('held-original');payload.rename(held);shutil.copytree(held,payload);replacement=payload.stat();assert original.st_ino!=replacement.st_ino
   save(out/'event.json',{'path':str(payload),'original_inode':original.st_ino,'replacement_inode':replacement.st_ino,'original_dev':original.st_dev,'replacement_dev':replacement.st_dev,'replacement_files':{str(f.relative_to(payload)):hashlib.sha256(f.read_bytes()).hexdigest() for f in payload.rglob('*') if f.is_file()}})
   with sqlite3.connect(home/'store.db') as db:before={t:sorted(repr(r) for r in db.execute('SELECT * FROM '+t)) for (t,) in db.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()}
   (sync/'release').touch();stdout,stderr=child.communicate(timeout=30);value=json.loads(stdout);records.append({'argv':argv,'exit':child.returncode,'stdout':stdout,'stderr':stderr})
   with sqlite3.connect(home/'store.db') as db:after={t:sorted(repr(r) for r in db.execute('SELECT * FROM '+t)) for (t,) in db.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()}
   save(out/'before.json',before);save(out/'after.json',after)
   accepted=[list(map(int,line.split())) for line in trace.read_text().splitlines()] if trace.exists() else [];save(out/'accepted-tree-trace.json',accepted)
   mismatches=[r for r in accepted if r[:2]!=r[2:]]
   assert not mismatches,('SEMANTIC replaced tree acknowledged as original bound object',mismatches)
   assert child.returncode==1 and value['error']['code']=='EFFECT_PENDING' and value['error']['detail']['cause']=='STORE_CORRUPT',('SEMANTIC replaced tree published',value)
   assert before==after and held.stat().st_ino==original.st_ino and payload.stat().st_ino==replacement.st_ino,('SEMANTIC wrong endpoint changed',value)
   meta['complete']=True
  finally:
   (sync/'release').touch()
   if child.poll() is None:child.kill();child.communicate()
except Exception as e:meta['error']=repr(e);raise
finally:save(out/'commands.json',records);save(out/'metadata.json',meta)
