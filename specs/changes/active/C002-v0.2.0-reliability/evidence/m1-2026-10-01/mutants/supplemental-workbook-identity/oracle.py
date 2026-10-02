"""CLI oracle: an internally matching historical publication target must still obey
Workbook ID/version grammar. All state edits are in a disposable owned fixture.
"""
import argparse,copy,hashlib,json,os,pathlib,shutil,sqlite3,subprocess,tempfile
ap=argparse.ArgumentParser();ap.add_argument('--binary',required=True);ap.add_argument('--source',required=True);ap.add_argument('--output',required=True);args=ap.parse_args()
binary=pathlib.Path(args.binary).resolve();source=pathlib.Path(args.source).resolve();out=pathlib.Path(args.output).resolve();out.mkdir(parents=True,exist_ok=False)
records=[];meta={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'oracle_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'fixture_sha256':{str(p.relative_to(source)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((source/'examples/two-step').rglob('*')) if p.is_file()},'cases':[],'complete':False}
def run(home,command):
 env=os.environ.copy()
 for key in list(env):
  if key=='SHELTIE_FAILPOINT' or key.startswith('SHELTIE_TEST_'):env.pop(key)
 argv=[str(binary),'--home',str(home),'--json',*command];r=subprocess.run(argv,capture_output=True,text=True,timeout=30,env=env)
 records.append({'argv':argv,'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr});return r.returncode,json.loads(r.stdout)
def facts(home):
 with sqlite3.connect(home/'store.db') as db:
  tables=[r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")]
  rows={t:sorted(repr(r) for r in db.execute('SELECT * FROM "'+t.replace('"','""')+'"')) for t in tables}
 files={}
 for folder in ['works','workbooks','pending']:
  for p in sorted((home/folder).rglob('*')):
   if p.is_file():files[str(p.relative_to(home))]=hashlib.sha256(p.read_bytes()).hexdigest()
 return {'rows':rows,'files':files}
try:
 for name,field,value in [('invalid_id','id','bad_'),('invalid_version','version','invalid_'),('reserved_version','version','.staging'),('overlong_version','version','v'*33)]:
  with tempfile.TemporaryDirectory(prefix='m1-workbook-oracle-',dir='/private/tmp') as directory:
   home=pathlib.Path(directory)/'home';home.mkdir(mode=0o700);fixture=pathlib.Path(directory)/'two-step';shutil.copytree(source/'examples/two-step',fixture)
   rid='identity-add';command=['--request-id',rid,'workbook','add',str(fixture)]
   code,committed=run(home,command);assert code==0 and committed['ok'] is True,(name,committed)
   replay=copy.deepcopy(committed);replay['data']['replayed']=True;assert run(home,command)==(0,replay)
   with sqlite3.connect(home/'store.db') as db:reply,effects=db.execute('SELECT reply_json,effects_json FROM requests WHERE request_id=?',(rid,)).fetchone()
   snapshot=json.loads(reply);snapshot['data'][field]=value;payload=json.loads(effects)
   publications=[x for x in payload if x['kind']=='publish_dir'];assert len(publications)==1
   target=publications[0];target['final']='workbooks/'+snapshot['data']['id']+'/'+snapshot['data']['version'];target['owner']='workbook:'+snapshot['data']['id']+'@'+snapshot['data']['version']
   with sqlite3.connect(home/'store.db') as db:db.execute('UPDATE requests SET reply_json=?,effects_json=? WHERE request_id=?',(json.dumps(snapshot,separators=(',',':')),json.dumps(payload,separators=(',',':')),rid))
   before=facts(home);code,error=run(home,command);after=facts(home)
   folder=out/name;folder.mkdir();record={'name':name,'field':field,'value':value,'fact_sha256':{},'rejected':code==1,'error':error};meta['cases'].append(record)
   for label,v in [('before',before),('after',after)]:
    p=folder/(label+'.json');p.write_text(json.dumps(v,sort_keys=True,indent=2)+'\n');record['fact_sha256'][label]=hashlib.sha256(p.read_bytes()).hexdigest()
   assert code==1 and error['ok'] is False,(name,code,error)
   assert error['error']['code']=='EFFECT_PENDING' and error['error']['detail']['cause']=='STORE_CORRUPT',(name,error)
   assert error['committed'] is True and error['request_id']==rid and 'original' not in error,(name,error)
   assert before==after,(name,'persisted facts changed')
   with sqlite3.connect(home/'store.db') as db:db.execute('UPDATE requests SET reply_json=?,effects_json=? WHERE request_id=?',(reply,effects,rid))
   assert run(home,command)==(0,replay);record['result']='PASS'
   print(json.dumps({'case':name,'result':'PASS'}),flush=True)
 meta['complete']=True
except Exception as e:meta['error']=repr(e);raise
finally:
 (out/'commands.json').write_text(json.dumps(records,indent=2)+'\n');(out/'metadata.json').write_text(json.dumps(meta,indent=2)+'\n')
