import argparse,hashlib,json,os,pathlib,shutil,sqlite3,subprocess,tempfile
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--source',required=True);p.add_argument('--output',required=True);a=p.parse_args();out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=False);records=[];meta={'binary_sha256':hashlib.sha256(pathlib.Path(a.binary).read_bytes()).hexdigest(),'script_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'complete':False}
def save(p,v):p.write_text(json.dumps(v,sort_keys=True,indent=2)+'\n')
def facts(home):
 with sqlite3.connect(home/'store.db') as db:rows={t:sorted(repr(r) for r in db.execute('SELECT * FROM '+t)) for (t,) in db.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()};schema=db.execute('SELECT name,sql FROM sqlite_master ORDER BY name').fetchall()
 return {'rows':rows,'schema':schema,'files':{str(p.relative_to(home)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (home/'works').rglob('*') if p.is_file()}}
def run(home,args):
 env=os.environ.copy()
 for k in list(env):
  if k=='SHELTIE_FAILPOINT' or k.startswith('SHELTIE_TEST_'):env.pop(k)
 q=subprocess.run([a.binary,'--home',str(home),'--json',*args],env=env,capture_output=True,text=True,timeout=30);records.append({'argv':q.args,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr});return q.returncode,json.loads(q.stdout)
def ok(home,args):
 c,v=run(home,args);assert c==0 and v['ok'],('SEMANTIC legal schema control rejected',v);return v
try:
 with tempfile.TemporaryDirectory(prefix='m1-extra-schema-',dir='/private/tmp') as td:
  base=pathlib.Path(td);home=base/'home';fixture=base/'fixture';shutil.copytree(pathlib.Path(a.source)/'examples/two-step',fixture);ok(home,['workbook','add',str(fixture)]);work=ok(home,['work','start','--workbook','two-step','--flow','default','--input','topic=controlled'])['data']['work_id'];ok(home,['work','status',work])
  with sqlite3.connect(home/'store.db') as db:
   original=db.execute("SELECT sql FROM sqlite_master WHERE name='works'").fetchone()[0];changed=original.replace('revision    INTEGER NOT NULL','revision    INTEGER NOT NULL DEFAULT 0');assert changed!=original,'fixture schema text not found';db.execute('PRAGMA writable_schema=ON');db.execute("UPDATE sqlite_master SET sql=? WHERE name='works'",(changed,));version=db.execute('PRAGMA schema_version').fetchone()[0];db.execute('PRAGMA schema_version='+str(version+1))
  before=facts(home);c,v=run(home,['work','status',work]);after=facts(home);save(out/'before.json',before);save(out/'after.json',after)
  assert c==1 and v['error']['code']=='STORE_CORRUPT',('SEMANTIC altered schema accepted or wrong error',c,v)
  assert before==after,('SEMANTIC altered schema operation wrote facts',v)
  meta['complete']=True
except Exception as e:meta['error']=repr(e);raise
finally:save(out/'commands.json',records);save(out/'metadata.json',meta)
