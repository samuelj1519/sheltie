"""Controlled observation oracle for the supplemental-only pause after seal hashing.
The identical pause is present in the control and mutant; production source is unchanged.
"""
import argparse,hashlib,json,os,pathlib,sqlite3,subprocess,tempfile,time
ap=argparse.ArgumentParser();ap.add_argument('--binary',required=True);ap.add_argument('--source',required=True);ap.add_argument('--output',required=True);a=ap.parse_args();binary=pathlib.Path(a.binary).resolve();source=pathlib.Path(a.source).resolve();out=pathlib.Path(a.output).resolve();out.mkdir(parents=True,exist_ok=False)
records=[];meta={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'oracle_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'fixture_sha256':{str(p.relative_to(source)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((source/'examples/two-step').rglob('*')) if p.is_file()},'cases':[],'complete':False}
def environment():
 e=os.environ.copy()
 for k in list(e):
  if k=='SHELTIE_FAILPOINT' or k.startswith('SHELTIE_TEST_'):e.pop(k)
 return e
def call(home,args):
 argv=[str(binary),'--home',str(home),'--json',*args];p=subprocess.run(argv,capture_output=True,text=True,env=environment(),timeout=30);records.append({'argv':argv,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr});v=json.loads(p.stdout);assert p.returncode==0 and v['ok'] is True,(args,p.returncode,v);return v
def facts(home):
 with sqlite3.connect(home/'store.db') as d:
  tables=[r[0] for r in d.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")];return {t:sorted(repr(r) for r in d.execute('SELECT * FROM "'+t.replace('"','""')+'"')) for t in tables}
try:
 for changed in [False,True]:
  name='late-growth' if changed else 'unchanged-control';entry={'name':name};meta['cases'].append(entry)
  with tempfile.TemporaryDirectory(prefix='m1-late-size-',dir='/private/tmp') as directory:
   base=pathlib.Path(directory);home=base/'home';home.mkdir(mode=0o700);sync=base/'sync';sync.mkdir()
   call(home,['workbook','add',str(source/'examples/two-step')]);work=call(home,['work','start','--workbook','two-step','--flow','default','--input','topic=x'])['data']['work_id'];begin=call(home,['attempt','begin',work,'--node','outline']);paths=list(begin['data']['outputs'].values());assert len(paths)==1;target=pathlib.Path(paths[0]);original_bytes=b'independent original bytes\n';target.write_bytes(original_bytes);mode=target.stat().st_mode&0o777;rid='late-size-submit'
   argv=[str(binary),'--home',str(home),'--json','--request-id',rid,'attempt','submit',work,'--attempt',begin['data']['attempt'],'--summary','controlled size observation'];env=environment();env.update(SHELTIE_TEST_RENDEZVOUS_NAME='m1_supplement_after_seal_hash',SHELTIE_TEST_RENDEZVOUS_ID=str(target),SHELTIE_TEST_RENDEZVOUS_DIR=str(sync));p=subprocess.Popen(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,env=env)
   try:
    deadline=time.monotonic()+30
    while not (sync/'reached').exists():
     assert p.poll() is None,'child exited before precise observation';assert time.monotonic()<deadline,'precise observation not reached';time.sleep(.002)
    assert (sync/'reached').read_text()=='m1_supplement_after_seal_hash'
    with sqlite3.connect(home/'store.db') as d:row=d.execute('SELECT published,reply_json FROM requests WHERE request_id=?',(rid,)).fetchone()
    assert row and row[0]==0,'observation must occur after COMMIT and before publishing'
    assert target.stat().st_mode&0o777==mode,'mode changed before final stat observation'
    expected_data={'attempt':begin['data']['attempt'],'outputs':{'outline':{'path':str(target),'bytes':len(original_bytes),'sha256':hashlib.sha256(original_bytes).hexdigest()}},'replayed':False,'work_status':{'kind':'active'}}
    expected_snapshot_data={key:value for key,value in expected_data.items() if key!='replayed'}
    stored=json.loads(row[1]);assert stored['request_id']==rid and stored['revision']==3 and stored['replayed'] is False and stored['data']==expected_snapshot_data,stored
    expected_original={'ok':True,'request_id':rid,'revision':3,'data':expected_data,'next':[{'op':'attempt begin','args':{'work':work,'node':'summary'},'edge':'main','executor':'agent','tier':'standard'},{'op':'work cancel','args':{'work':work}}]}
    entry['expected_original']=expected_original
    entry.update(committed_published=row[0],original_reply=json.loads(row[1]),before_mode=mode,before_size=target.stat().st_size)
    if changed:
     with target.open('ab') as f:f.write(b'!')
    before=facts(home);(sync/'release').write_text('release');stdout,stderr=p.communicate(timeout=30);records.append({'argv':argv,'exit':p.returncode,'stdout':stdout,'stderr':stderr,'point':'after hash, before final stat'});value=json.loads(stdout);after=facts(home)
    folder=out/name;folder.mkdir();entry.update(exit=p.returncode,response=value,after_size=target.stat().st_size,after_mode=target.stat().st_mode&0o777,fact_sha256={})
    for label,content in [('before',before),('after',after)]:
     path=folder/(label+'.json');path.write_text(json.dumps(content,sort_keys=True,indent=2)+'\n');entry['fact_sha256'][label]=hashlib.sha256(path.read_bytes()).hexdigest()
    with sqlite3.connect(home/'store.db') as d:published=d.execute('SELECT published FROM requests WHERE request_id=?',(rid,)).fetchone()[0]
    entry['after_published']=published
    if changed:
     assert p.returncode==1 and value['ok'] is False,(p.returncode,value)
     assert value['error']['code']=='EFFECT_PENDING' and value['error']['detail']['cause']=='STORE_CORRUPT',value
     assert value['committed'] is True and value['request_id']==rid and value['original']==expected_original,value
     assert entry['after_size']==entry['before_size']+1 and published==0 and entry['after_mode']==mode and before==after,entry
    else:assert entry['after_size']==entry['before_size'] and p.returncode==0 and value['ok'] is True and published==1 and entry['after_mode']&0o222==0,entry
    entry['result']='PASS';print(json.dumps({'case':name,'result':'PASS'}),flush=True)
   finally:
    if p.poll() is None:(sync/'release').write_text('release');p.kill();p.wait()
 meta['complete']=True
except Exception as e:meta['error']=repr(e);raise
finally:
 (out/'commands.json').write_text(json.dumps(records,indent=2)+'\n');(out/'metadata.json').write_text(json.dumps(meta,indent=2)+'\n')
