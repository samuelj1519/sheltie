import argparse,hashlib,json,os,pathlib,shutil,subprocess,tempfile,time
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--source',required=True);p.add_argument('--output',required=True);a=p.parse_args();out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=False);records=[];meta={'complete':False,'binary_sha256':hashlib.sha256(pathlib.Path(a.binary).read_bytes()).hexdigest(),'script_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()}
def save(p,v):p.write_text(json.dumps(v,sort_keys=True,indent=2)+'\n')
try:
 with tempfile.TemporaryDirectory(prefix='m1-extra-alias-',dir='/private/tmp') as td:
  base=pathlib.Path(td);fixture=base/'fixture';shutil.copytree(pathlib.Path(a.source)/'examples/two-step',fixture);home=base/'home';sync=base/'sync';sync.mkdir();trace=base/'open-trace';leaf=fixture/'workbook.toml';initial=leaf.stat();original=leaf.read_bytes();held=base/'held-original.toml'
  env=os.environ.copy()
  for k in list(env):
   if k=='SHELTIE_FAILPOINT' or k.startswith('SHELTIE_TEST_'):env.pop(k)
  env.update(SHELTIE_TEST_RENDEZVOUS_NAME='external_tree_after_stat',SHELTIE_TEST_RENDEZVOUS_ID=str(leaf),SHELTIE_TEST_RENDEZVOUS_DIR=str(sync),SHELTIE_EXTRA_SOURCE_OPEN_TRACE=str(trace));argv=[a.binary,'--home',str(home),'--json','workbook','add',str(fixture)];child=subprocess.Popen(argv,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  try:
   deadline=time.monotonic()+30
   while not(sync/'reached').exists():
    assert child.poll() is None,'fixture process ended before target event'
    if time.monotonic()>deadline:raise TimeoutError('event timeout, not detection')
    time.sleep(.005)
   leaf.rename(held);leaf.symlink_to(held);assert leaf.stat().st_ino==initial.st_ino
   save(out/'event.json',{'symlink_path':str(leaf),'held_path':str(held),'initial_inode':initial.st_ino,'followed_inode':leaf.stat().st_ino,'initial_sha256':hashlib.sha256(original).hexdigest(),'held_mode':held.stat().st_mode&0o777})
   (sync/'release').touch();stdout,stderr=child.communicate(timeout=30);value=json.loads(stdout);records.append({'argv':argv,'exit':child.returncode,'stdout':stdout,'stderr':stderr});save(out/'sentinel-after.json',{'sha256':hashlib.sha256(held.read_bytes()).hexdigest(),'mode':held.stat().st_mode&0o777,'symlink_retained':leaf.is_symlink(),'home_files':[str(p.relative_to(home)) for p in home.rglob('*') if p.is_file()]})
   opened=trace.read_text().splitlines() if trace.exists() else [];save(out/'opened-alias-trace.json',opened)
   assert not opened,('SEMANTIC source symlink was followed at exact open boundary before later rejection',opened)
   assert child.returncode!=0 and not value['ok'],('SEMANTIC source symlink accepted after observed same-inode alias',value)
   assert held.read_bytes()==original and held.stat().st_ino==initial.st_ino,('SEMANTIC sentinel altered',value)
   meta['complete']=True
  finally:
   (sync/'release').touch()
   if child.poll() is None:child.kill();child.communicate()
except Exception as e:meta['error']=repr(e);raise
finally:save(out/'metadata.json',meta);save(out/'commands.json',records)
