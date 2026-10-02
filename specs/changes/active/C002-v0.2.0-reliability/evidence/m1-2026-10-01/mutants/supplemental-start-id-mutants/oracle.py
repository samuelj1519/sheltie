"""Real CLI oracle: historical Start data.work_id must identify its actual Work.
Only this field changes in an isolated owned fixture; reply/audit/row identity stay legal.
"""
import argparse,copy,hashlib,json,os,pathlib,shutil,sqlite3,subprocess,tempfile
ap=argparse.ArgumentParser();ap.add_argument('--binary',required=True);ap.add_argument('--source',required=True);ap.add_argument('--output',required=True);args=ap.parse_args()
binary=pathlib.Path(args.binary).resolve();source=pathlib.Path(args.source).resolve();out=pathlib.Path(args.output).resolve();out.mkdir(parents=True,exist_ok=False)
records=[];meta={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'oracle_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'fixture_sha256':{str(p.relative_to(source)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((source/'examples/two-step').rglob('*')) if p.is_file()},'complete':False}
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
 with tempfile.TemporaryDirectory(prefix='m1-start-id-',dir='/private/tmp') as directory:
  home=pathlib.Path(directory)/'home';home.mkdir(mode=0o700)
  code,value=run(home,['workbook','add',str(source/'examples/two-step')]);assert code==0 and value['ok'] is True
  command=['--request-id','start-id-oracle','work','start','--workbook','two-step','--flow','default','--input','topic=x','--name','identity-check']
  code,committed=run(home,command);assert code==0 and committed['ok'] is True
  work=committed['data']['work_id'];replay=copy.deepcopy(committed);replay['data']['replayed']=True;assert run(home,command)==(0,replay)
  with sqlite3.connect(home/'store.db') as db:original=db.execute('SELECT reply_json FROM requests WHERE request_id=?',('start-id-oracle',)).fetchone()[0]
  snapshot=json.loads(original);snapshot['data']['work_id']=work+'-other';meta['changed_work_id']=snapshot['data']['work_id'];meta['original_work_id']=work
  with sqlite3.connect(home/'store.db') as db:db.execute('UPDATE requests SET reply_json=? WHERE request_id=?',(json.dumps(snapshot,separators=(',',':')),'start-id-oracle'))
  before=facts(home);code,error=run(home,command);after=facts(home);meta['error']=error;meta['fact_sha256']={}
  for label,value in [('before',before),('after',after)]:
   p=out/(label+'.json');p.write_text(json.dumps(value,sort_keys=True,indent=2)+'\n');meta['fact_sha256'][label]=hashlib.sha256(p.read_bytes()).hexdigest()
  assert code==1 and error['ok'] is False,(code,error)
  assert error['error']['code']=='EFFECT_PENDING' and error['error']['detail']['cause']=='STORE_CORRUPT',error
  assert error['committed'] is True and error['request_id']=='start-id-oracle' and 'original' not in error,error
  assert before==after,'persisted facts changed'
  with sqlite3.connect(home/'store.db') as db:db.execute('UPDATE requests SET reply_json=? WHERE request_id=?',(original,'start-id-oracle'))
  assert run(home,command)==(0,replay);meta['complete']=True
 print('PASS: legal replay, one changed Start Work ID rejected, facts unchanged, restored replay.',flush=True)
except Exception as e:meta['error_detail']=repr(e);raise
finally:
 (out/'commands.json').write_text(json.dumps(records,indent=2)+'\n');(out/'metadata.json').write_text(json.dumps(meta,indent=2)+'\n')
