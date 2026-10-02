import argparse, copy, hashlib, json, os, pathlib, sqlite3, struct, subprocess, tempfile

p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--source',required=True);p.add_argument('--output',required=True);p.add_argument('--family',required=True);a=p.parse_args()
binary=pathlib.Path(a.binary);source=pathlib.Path(a.source);out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=False)
records=[];observations=[]
def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
def facts(home):
 with sqlite3.connect(home/'store.db') as db:
  rows={t:sorted(repr(r) for r in db.execute('SELECT * FROM "'+t+'"')) for (t,) in db.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").fetchall()}
 files={str(f.relative_to(home)):{'sha':hashlib.sha256(f.read_bytes()).hexdigest(),'mode':f.stat().st_mode&0o777} for folder in ['works','workbooks','pending'] for f in (home/folder).rglob('*') if f.is_file()}
 return {'rows':rows,'files':files}
def run(home,args):
 env=os.environ.copy()
 for key in list(env):
  if key=='SHELTIE_FAILPOINT' or key.startswith('SHELTIE_TEST_'):env.pop(key)
 q=subprocess.run([str(binary),'--home',str(home),'--json',*args],env=env,capture_output=True,text=True,timeout=30)
 records.append({'argv':q.args,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 return q.returncode,json.loads(q.stdout)
def ok(home,args):
 code,value=run(home,args);assert code==0 and value['ok'],('SEMANTIC legal control rejected',args,code,value);return value
def rejection(home,args,label):
 before=facts(home);code,value=run(home,args);after=facts(home)
 save(out/(label+'-before.json'),before);save(out/(label+'-after.json'),after)
 observations.append({'label':label,'equal':before==after,'result':value})
 assert code==1 and not value['ok'] and value['error']['code'] in ['STORE_CORRUPT','EFFECT_PENDING'],('SEMANTIC corrupt fact accepted or wrong error',label,code,value)
 if value['error']['code']=='EFFECT_PENDING':assert value['error']['detail']['cause']=='STORE_CORRUPT' and 'original' not in value,('SEMANTIC corrupt response qualified',value)
 assert before==after,('SEMANTIC corrupt fact mutated store/files',label)
def digest(tree):
 fs=sorted([f for f in tree.rglob('*') if f.is_file()],key=lambda f:str(f.relative_to(tree)).encode())
 h=hashlib.sha256(b'sheltie-workbook-digest/v2\0'+struct.pack('>Q',len(fs)))
 for f in fs:
  path=str(f.relative_to(tree)).encode();data=f.read_bytes();h.update(struct.pack('>Q',len(path))+path+struct.pack('>Q',len(data))+data)
 return h.hexdigest()
def replace(value,old,new):
 if isinstance(value,dict):return {k:replace(v,old,new) for k,v in value.items()}
 if isinstance(value,list):return [replace(v,old,new) for v in value]
 return new if value==old else value
meta={'family':a.family,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'script_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'complete':False}
try:
 with tempfile.TemporaryDirectory(prefix='m1-extra-fixture-',dir='/private/tmp') as td:
  base=pathlib.Path(td);home=base/'home'
  if a.family=='nested-root':
   home=base/'absent'/'nested'/'home';value=ok(home,['self','install']);assert home.is_dir() and (home/'store.db').is_file() and (home/'bin/sheltie').is_file(),('SEMANTIC nested Home missing',value)
  else:
   fixture=base/'fixture';import shutil;shutil.copytree(source/'examples/two-step',fixture)
   meta['fixture_sha256']={str(f.relative_to(fixture)):hashlib.sha256(f.read_bytes()).hexdigest() for f in fixture.rglob('*') if f.is_file()}
   add=['--request-id','extra-add','workbook','add',str(fixture)];original=ok(home,add);replay=copy.deepcopy(original);replay['data']['replayed']=True;assert ok(home,add)==replay
   if a.family=='audit':
    for field,value in [('work_id','2026-10-01-001-default'),('revision',1)]:
     with sqlite3.connect(home/'store.db') as db:old=db.execute('SELECT '+field+' FROM audit WHERE request_id=?',('extra-add',)).fetchone()[0];db.execute('UPDATE audit SET '+field+'=? WHERE request_id=?',(value,'extra-add'))
     rejection(home,add,field)
     with sqlite3.connect(home/'store.db') as db:db.execute('UPDATE audit SET '+field+'=? WHERE request_id=?',(old,'extra-add'))
     assert ok(home,add)==replay
   else:
    work=ok(home,['--request-id','extra-start','work','start','--workbook','two-step','--flow','default','--input','topic=independent'])['data']['work_id'];ok(home,['work','status',work])
    frozen=home/'works'/work/'workbook';manifest=frozen/'workbook.toml';oldbytes=manifest.read_bytes();old=digest(frozen)
    change=('id = "two-step"','id = "other-step"') if a.family=='manifest-id' else ('version = "1.0.0"','version = "2.0.0"')
    manifest.chmod(0o644);manifest.write_text(oldbytes.decode().replace(*change));manifest.chmod(0o444);new=digest(frozen)
    with sqlite3.connect(home/'store.db') as db:
     for table,columns in [('works',['state_json']),('requests',['reply_json','effects_json']),('audit',['command_json'])]:
      for col in columns:
       for rowid,value in db.execute('SELECT rowid,'+col+' FROM '+table).fetchall():
        obj=json.loads(value);changed=replace(obj,old,new);db.execute('UPDATE '+table+' SET '+col+'=? WHERE rowid=?',(json.dumps(changed,separators=(',',':')),rowid))
    save(out/'manifest-change.json',{'old_bytes':oldbytes.decode(),'new_bytes':manifest.read_text(),'old_digest':old,'new_digest':new,'retained_state_identity':'two-step@1.0.0'})
    rejection(home,['work','status',work],a.family)
 meta['complete']=True
except Exception as e:meta['error']=repr(e);raise
finally:
 save(out/'commands.json',records);save(out/'metadata.json',meta);save(out/'observations.json',observations)
