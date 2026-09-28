import pathlib, tempfile, os, subprocess, json, time, fcntl, sqlite3
home=pathlib.Path(tempfile.mkdtemp(prefix='m1-init-lock-',dir='/private/tmp'))
lock=(home/'.lock').open('w+');fcntl.flock(lock,fcntl.LOCK_EX)
env=os.environ.copy();env['SHELTIE_HOME']=str(home)
argv=['/private/tmp/sheltie-m1-review-target/debug/sheltie','--json','workbook','add',str(pathlib.Path('examples/two-step').resolve())]
p=subprocess.Popen(argv,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
end=time.monotonic()+3
while not (home/'store.db').exists() and time.monotonic()<end and p.poll() is None:time.sleep(.01)
before=dict(home=str(home),pid=p.pid,root_lock_held=True,process_running=p.poll() is None,db_exists=(home/'store.db').exists())
if before['db_exists']:
 conn=sqlite3.connect(home/'store.db');before['schema_version']=conn.execute('pragma user_version').fetchone()[0];conn.close()
fcntl.flock(lock,fcntl.LOCK_UN);lock.close();stdout,stderr=p.communicate(timeout=10)
result=dict(argv=argv,before_unlock=before,exit=p.returncode,stdout=stdout,stderr=stderr)
print(json.dumps(result,ensure_ascii=False,indent=2))
