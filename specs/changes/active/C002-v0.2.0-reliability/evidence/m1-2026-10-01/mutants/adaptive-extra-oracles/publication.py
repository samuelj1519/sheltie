import argparse,hashlib,json,os,pathlib,shutil,sqlite3,struct,subprocess,tempfile
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--source',required=True);p.add_argument('--output',required=True);a=p.parse_args();out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=False);records=[];meta={'binary_sha256':hashlib.sha256(pathlib.Path(a.binary).read_bytes()).hexdigest(),'script_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'complete':False,'cases':[]}
def save(p,v):p.write_text(json.dumps(v,sort_keys=True,indent=2)+'\n')
def run(home,args,failpoint=None):
 env=os.environ.copy()
 for k in list(env):
  if k=='SHELTIE_FAILPOINT' or k.startswith('SHELTIE_TEST_'):env.pop(k)
 if failpoint:env['SHELTIE_FAILPOINT']=failpoint
 q=subprocess.run([a.binary,'--home',str(home),'--json',*args],env=env,capture_output=True,text=True,timeout=30);records.append({'argv':q.args,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr,'failpoint':failpoint});return q.returncode,json.loads(q.stdout) if q.stdout else None
def facts(home):
 with sqlite3.connect(home/'store.db') as db:rows={t:sorted(repr(r) for r in db.execute('SELECT * FROM '+t)) for (t,) in db.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()}
 files={str(f.relative_to(home)):{'sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'mode':f.stat().st_mode&0o777} for folder in ['workbooks','pending'] for f in (home/folder).rglob('*') if f.is_file()};return {'rows':rows,'files':files}
def digest(tree):
 fs=sorted([f for f in tree.rglob('*') if f.is_file()],key=lambda f:str(f.relative_to(tree)).encode());h=hashlib.sha256(b'sheltie-workbook-digest/v2\0'+struct.pack('>Q',len(fs)))
 for f in fs:
  name=str(f.relative_to(tree)).encode();data=f.read_bytes();h.update(struct.pack('>Q',len(name))+name+struct.pack('>Q',len(data))+data)
 return h.hexdigest()
def replace(o,old,new):
 if isinstance(o,dict):return {k:replace(v,old,new) for k,v in o.items()}
 if isinstance(o,list):return [replace(v,old,new) for v in o]
 return new if o==old else o
try:
 for kind in ['legal','manifest-id','manifest-version']:
  with tempfile.TemporaryDirectory(prefix='m1-extra-publication-',dir='/private/tmp') as td:
   base=pathlib.Path(td);home=base/'home';fixture=base/'fixture';shutil.copytree(pathlib.Path(a.source)/'examples/two-step',fixture);args=['--request-id','extra-publication','workbook','add',str(fixture)];c,v=run(home,args,'after_commit_before_effects');assert c==70,('tool/fixture actual commit failpoint unavailable',c,v)
   with sqlite3.connect(home/'store.db') as db:original=json.loads(db.execute('SELECT reply_json FROM requests WHERE request_id=?',('extra-publication',)).fetchone()[0]);effects=json.loads(db.execute('SELECT effects_json FROM requests WHERE request_id=?',('extra-publication',)).fetchone()[0])
   if kind!='legal':
    pending=home/next(e['pending'] for e in effects if e['kind']=='publish_dir');manifest=pending/'workbook.toml';oldbytes=manifest.read_bytes();old=digest(pending);assert old==original['data']['digest'],'independent original framing disagrees'
    change=('id = "two-step"','id = "other-step"') if kind=='manifest-id' else ('version = "1.0.0"','version = "2.0.0"');manifest.chmod(0o644);manifest.write_text(oldbytes.decode().replace(*change));new=digest(pending)
    with sqlite3.connect(home/'store.db') as db:
     for table,columns in [('requests',['reply_json','effects_json']),('audit',['command_json'])]:
      for col in columns:
       for rowid,value in db.execute('SELECT rowid,'+col+' FROM '+table).fetchall():db.execute('UPDATE '+table+' SET '+col+'=? WHERE rowid=?',(json.dumps(replace(json.loads(value),old,new),separators=(',',':')),rowid))
     db.execute('UPDATE workbooks SET digest=? WHERE digest=?',(new,old))
    save(out/(kind+'-event.json'),{'old_digest':old,'new_digest':new,'old_manifest':oldbytes.decode(),'new_manifest':manifest.read_text(),'expected_id':'two-step','expected_version':'1.0.0'})
   before=facts(home);c,v=run(home,args);after=facts(home);save(out/(kind+'-before.json'),before);save(out/(kind+'-after.json'),after);meta['cases'].append({'kind':kind,'exit':c,'result':v})
   if kind=='legal':assert c==0 and v['ok'],('SEMANTIC legitimate pending publication rejected',v)
   else:
    assert c==1 and v['error']['code']=='EFFECT_PENDING' and v['error']['detail']['cause']=='STORE_CORRUPT',('SEMANTIC mismatched payload identity accepted',kind,c,v)
    assert before==after,('SEMANTIC mismatched payload publication changed facts',kind)
 meta['complete']=True
except Exception as e:meta['error']=repr(e);raise
finally:save(out/'commands.json',records);save(out/'metadata.json',meta)
