"""CLI snapshot identity at the existing pending-publication/outer-reread boundary.
All data and named events are isolated temporary fixtures; production source unchanged.
"""
import argparse,copy,hashlib,json,os,pathlib,shutil,sqlite3,subprocess,tempfile,time
ap=argparse.ArgumentParser();ap.add_argument('--binary',required=True);ap.add_argument('--source',required=True);ap.add_argument('--output',required=True);a=ap.parse_args();binary=pathlib.Path(a.binary).resolve();source=pathlib.Path(a.source).resolve();out=pathlib.Path(a.output).resolve();out.mkdir(parents=True,exist_ok=False)
records=[];meta={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'oracle_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'complete':False,'cases':[]}
def env():
 e=os.environ.copy()
 for k in list(e):
  if k=='SHELTIE_FAILPOINT' or k.startswith('SHELTIE_TEST_'):e.pop(k)
 return e
def call(home,args,extra=None):
 e=env();e.update(extra or {});argv=[str(binary),'--home',str(home),'--json',*args];p=subprocess.run(argv,capture_output=True,text=True,env=e,timeout=30);records.append({'argv':argv,'env':extra,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr});return p.returncode,json.loads(p.stdout) if p.stdout else None
try:
 for kind in ['add','remove']:
  for change in ['control','request_id','replayed']:
   case=kind+'-'+change;entry={'name':case};meta['cases'].append(entry)
   with tempfile.TemporaryDirectory(prefix='m1-reread-',dir='/private/tmp') as directory:
    base=pathlib.Path(directory);home=base/'home';home.mkdir(mode=0o700);b=base/'cached-source';pending=base/'later-source';sync=base/'sync';sync.mkdir()
    for path,name in [(b,'cached-book'),(pending,'later-book')]:
     shutil.copytree(source/'examples/two-step',path);p=path/'workbook.toml';p.write_text(p.read_text().replace('id = "two-step"',f'id = "{name}"'))
    rid='completed-b';command=['--request-id',rid,'workbook','add',str(b)]
    code,added=call(home,command);assert code==0 and added['ok'] is True
    if kind=='remove':
     command=['--request-id','completed-remove-b','workbook','remove','cached-book@1.0.0'];rid='completed-remove-b';code,committed=call(home,command);assert code==0 and committed['ok'] is True
    else:committed=added
    expected=copy.deepcopy(committed);expected['data']['replayed']=True;assert call(home,command)==(0,expected)
    code,_=call(home,['--request-id','pending-a','workbook','add',str(pending)],{'SHELTIE_FAILPOINT':'after_commit_before_effects'});assert code==70
    with sqlite3.connect(home/'store.db') as db:
     original=db.execute('SELECT reply_json FROM requests WHERE request_id=?',(rid,)).fetchone()[0];assert db.execute('SELECT published FROM requests WHERE request_id=?',('pending-a',)).fetchone()[0]==0
    argv=[str(binary),'--home',str(home),'--json',*command];e=env();e.update(SHELTIE_TEST_RENDEZVOUS_NAME='publish_after_tree_sync',SHELTIE_TEST_RENDEZVOUS_ID='pending-a',SHELTIE_TEST_RENDEZVOUS_DIR=str(sync));p=subprocess.Popen(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,env=e)
    try:
     deadline=time.monotonic()+30
     while not (sync/'reached').exists():
      assert p.poll() is None,'publication point not reached';assert time.monotonic()<deadline,'publication point timeout';time.sleep(.002)
     assert (sync/'reached').read_text()=='publish_after_tree_sync'
     changed=json.loads(original)
     if change=='request_id':changed['request_id']='another-b'
     if change=='replayed':changed['replayed']=True
     with sqlite3.connect(home/'store.db') as db:db.execute('UPDATE requests SET reply_json=? WHERE request_id=?',(json.dumps(changed,separators=(',',':')),rid))
     entry['original_snapshot']=json.loads(original);entry['changed_snapshot']=changed;(sync/'release').write_text('release');stdout,stderr=p.communicate(timeout=30);value=json.loads(stdout);records.append({'argv':argv,'exit':p.returncode,'stdout':stdout,'stderr':stderr,'point':'publish_after_tree_sync(pending-a) after cached-b validation, before outer reread'});entry.update(exit=p.returncode,response=value)
     with sqlite3.connect(home/'store.db') as db:
      entry['a_published']=db.execute('SELECT published FROM requests WHERE request_id=?',('pending-a',)).fetchone()[0];after=db.execute('SELECT reply_json FROM requests WHERE request_id=?',(rid,)).fetchone()[0]
     assert entry['a_published']==1,'lawful pending A must complete'
     if change=='control':assert p.returncode==0 and value==expected,(case,value)
     else:
      assert p.returncode==1 and value['ok'] is False and value['error']['code']=='STORE_CORRUPT',(case,p.returncode,value)
      assert json.loads(after)==changed,'bad B snapshot must not be repaired'
     entry['result']='PASS';print(json.dumps({'case':case,'result':'PASS'}),flush=True)
    finally:
     if p.poll() is None:(sync/'release').write_text('release');p.kill();p.wait()
 meta['complete']=True
except Exception as e:meta['error']=repr(e);raise
finally:
 (out/'commands.json').write_text(json.dumps(records,indent=2)+'\n');(out/'metadata.json').write_text(json.dumps(meta,indent=2)+'\n')
