"""Independent CLI oracle for immutable historical submit/approve response status.
Fixtures are isolated copies of frozen examples. No production source is changed.
Only impossible statuses are rejected: nonterminal Active/NoLegalEdge remain allowed
by the adopted historical necessary-condition contract.
"""
import argparse,copy,hashlib,json,os,pathlib,shutil,sqlite3,subprocess,tempfile,time
ap=argparse.ArgumentParser();ap.add_argument('--binary',required=True);ap.add_argument('--source',required=True);ap.add_argument('--output',required=True);args=ap.parse_args()
binary=pathlib.Path(args.binary).resolve();source=pathlib.Path(args.source).resolve();out=pathlib.Path(args.output).resolve();out.mkdir(parents=True,exist_ok=False)
records=[]
statuses=[{'kind':'active'},{'kind':'succeeded'},{'kind':'cancelled'},{'kind':'blocked','reason':'gate'},{'kind':'blocked','reason':'no_legal_edge'},{'kind':'blocked','reason':'retries_exhausted'}]
meta={'binary':str(binary),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'source':str(source),'oracle_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'fixture_sha256':{},'cases':[],'complete':False}
for example in ['two-step','gated-release']:
 for p in sorted((source/'examples'/example).rglob('*')):
  if p.is_file():meta['fixture_sha256'][str(p.relative_to(source))]=hashlib.sha256(p.read_bytes()).hexdigest()
def run(home,argv):
 env=os.environ.copy()
 for key in list(env):
  if key=='SHELTIE_FAILPOINT' or key.startswith('SHELTIE_TEST_'):env.pop(key)
 command=[str(binary),'--home',str(home),'--json',*argv]
 p=subprocess.run(command,capture_output=True,text=True,timeout=30,env=env)
 record={'argv':command,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr};records.append(record)
 value=json.loads(p.stdout);return p.returncode,value
def ok(home,argv):
 code,v=run(home,argv);assert code==0 and v['ok'] is True,(argv,code,v);return v
def facts(home):
 with sqlite3.connect(home/'store.db') as db:
  tables=[r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")]
  rows={t:sorted(repr(r) for r in db.execute('SELECT * FROM "'+t.replace('"','""')+'"')) for t in tables}
 files={}
 for directory in ['works','workbooks','pending']:
  for p in sorted((home/directory).rglob('*')):
   if p.is_file():files[str(p.relative_to(home))]=hashlib.sha256(p.read_bytes()).hexdigest()
 return {'rows':rows,'files':files}
def submit(home,work,node,rid):
 begun=ok(home,['attempt','begin',work,'--node',node])
 for value in begun['data']['outputs'].values():
  p=pathlib.Path(value);p.parent.mkdir(parents=True,exist_ok=True);p.write_text('independent output\n')
 command=['--request-id',rid,'attempt','submit',work,'--attempt',begun['data']['attempt'],'--summary','independent historical response']
 return command,ok(home,command)
try:
 for case in ['submit_active','submit_terminal','submit_gate','submit_terminal_gate','approve_active','approve_terminal']:
  start=len(records)
  with tempfile.TemporaryDirectory(prefix='m1-status-oracle-',dir='/private/tmp') as directory:
   base=pathlib.Path(directory);home=base/'home';home.mkdir(mode=0o700)
   example='two-step' if case in ['submit_active','submit_terminal'] else 'gated-release'
   fixture=base/example;shutil.copytree(source/'examples'/example,fixture)
   if case in ['submit_terminal_gate','approve_terminal']:
    path=fixture/'flows/default.toml';text=path.read_text();text=text[:text.index('[[nodes]]\nid = "archive"')];path.write_text(text)
   ok(home,['workbook','add',str(fixture)])
   field='topic=x' if example=='two-step' else 'version=1.2.0'
   work=ok(home,['work','start','--workbook',example,'--flow','default','--input',field])['data']['work_id']
   if case=='submit_terminal':submit(home,work,'outline','setup-submit')
   node='summary' if case=='submit_terminal' else 'outline' if example=='two-step' else 'notes'
   command,committed=submit(home,work,node,'status-submit')
   rid='status-submit'
   if case.startswith('approve'):
    rid='status-approve';command=['--request-id',rid,'gate','approve',work,'--node','notes'];committed=ok(home,command)
   expected={'kind':'succeeded'} if case in ['submit_terminal','approve_terminal'] else {'kind':'blocked','reason':'gate'} if case in ['submit_gate','submit_terminal_gate'] else {'kind':'active'}
   assert committed['data']['work_status']==expected,(case,committed)
   if expected['kind']!='succeeded':ok(home,['work','cancel',work])
   replay=copy.deepcopy(committed);replay['data']['replayed']=True
   assert ok(home,command)==replay,case
   with sqlite3.connect(home/'store.db') as db:original=db.execute('SELECT reply_json FROM requests WHERE request_id=?',(rid,)).fetchone()[0]
   allowed=[expected]
   if case in ['submit_active','approve_active']:allowed.append({'kind':'blocked','reason':'no_legal_edge'})
   negatives=[]
   for status in statuses:
    if status in allowed:continue
    changed=json.loads(original);changed['data']['work_status']=status
    with sqlite3.connect(home/'store.db') as db:db.execute('UPDATE requests SET reply_json=? WHERE request_id=?',(json.dumps(changed,separators=(',',':')),rid))
    before=facts(home);code,error=run(home,command)
    assert code==1 and error['ok'] is False,(case,status,code,error)
    assert error['error']['code']=='EFFECT_PENDING' and error['error']['detail']['cause']=='STORE_CORRUPT',(case,status,error)
    assert error['committed'] is True and error['request_id']==rid and 'original' not in error,(case,status,error)
    assert facts(home)==before,(case,status,'persisted facts changed')
    negatives.append(status)
    with sqlite3.connect(home/'store.db') as db:db.execute('UPDATE requests SET reply_json=? WHERE request_id=?',(original,rid))
   assert ok(home,command)==replay,case
   meta['cases'].append({'name':case,'legal_status':expected,'allowed_necessary_statuses':allowed,'negative_statuses':negatives,'record_range':[start,len(records)],'result':'PASS'})
  print(json.dumps({'case':case,'result':'PASS','negative_count':len(negatives)}),flush=True)
 meta['complete']=True
except Exception as e:
 meta['error']=repr(e);raise
finally:
 (out/'commands.json').write_text(json.dumps(records,indent=2)+'\n');(out/'metadata.json').write_text(json.dumps(meta,indent=2)+'\n')
