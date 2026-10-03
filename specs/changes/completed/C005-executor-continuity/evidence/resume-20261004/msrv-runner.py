import datetime,hashlib,json,os,shutil,signal,subprocess,time
from pathlib import Path
r=Path('/Users/shushu/orca/workspaces/sheltie/codex');e=r/'specs/changes/active/C005-executor-continuity/evidence/resume-20261004';f=json.loads((e/'technical-freeze.json').read_text());env=os.environ.copy();env['PATH']=str(Path(f['MSRV_trial']['nextest']).parent)+':'+env['PATH'];env['RUSTC_WRAPPER']='';env['CARGO_TARGET_DIR']=f['MSRV_trial']['target_dir'];env['RUSTUP_TOOLCHAIN']='1.85.0';env.pop('SHELTIE_NEXTEST_VERSION_OVERRIDE',None);env.pop('SHELTIE_TEST_ENGINE_BINARY',None);env.pop('SHELTIE_TEST_EXPORT_BINARY',None)
start=time.monotonic();now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat();record={'started_utc':now(),'freeze_sha256':hashlib.sha256((e/'technical-freeze.json').read_bytes()).hexdigest(),'env':{k:env.get(k) for k in ['RUSTC_WRAPPER','CARGO_TARGET_DIR','RUSTUP_TOOLCHAIN','SHELTIE_NEXTEST_VERSION_OVERRIDE','SHELTIE_TEST_ENGINE_BINARY','SHELTIE_TEST_EXPORT_BINARY']},'steps':[],'usage':None}
def check_inputs():
 drift=[]
 for name,h in f['complete192_inputs_sha256'].items():
  q=r/name;b=os.fsencode(os.readlink(q)) if q.is_symlink() else q.read_bytes()
  if hashlib.sha256(b).hexdigest()!=h:drift.append(name)
 if drift:raise RuntimeError('Frozen source drift:'+str(drift))
def save(): (e/'msrv-execution.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n')
def run(name,argv):
 check_inputs();remaining=f['MSRV_trial']['limit_s']-(time.monotonic()-start)
 if remaining<=8:raise RuntimeError('MSRV budget lacks8s owned-cleanup/record reserve before '+name)
 a=now();out=e/(name+'.stdout');err=e/(name+'.stderr');timed_out=False
 with out.open('xb') as o,err.open('xb') as z:
  try:
   x=subprocess.Popen(argv,cwd=r,env=env,stdout=o,stderr=z,start_new_session=True);status=x.wait(timeout=remaining-8)
  except subprocess.TimeoutExpired:
   try:os.killpg(x.pid,signal.SIGTERM)
   except ProcessLookupError:pass
   time.sleep(3)
   try:os.killpg(x.pid,signal.SIGKILL)
   except ProcessLookupError:pass
   x.wait(timeout=3)
   status=x.returncode;timed_out=True
 step=dict(name=name,argv=argv,start_utc=a,end_utc=now(),exit_code=status,timed_out=timed_out,owned_group_termination_requested=timed_out,stdout_sha256=hashlib.sha256(out.read_bytes()).hexdigest(),stderr_sha256=hashlib.sha256(err.read_bytes()).hexdigest());record['steps'].append(step);save();print(name,status,flush=True)
 if timed_out or status!=0:raise RuntimeError('Stopped on '+name+' actualexit='+str(status)+' timeout='+str(timed_out))
 check_inputs();return out
try:
 for name,argv in [('rustc-version',['rustc','+1.85.0','-vV']),('cargo-version',['cargo','+1.85.0','-V']),('nextest-version',[f['MSRV_trial']['nextest'],'--version'])]:run(name,argv)
 out=run('msrv-build',f['MSRV_trial']['commands'][0]);bins={}
 for line in out.read_text().splitlines():
  d=json.loads(line)
  if d.get('reason')=='compiler-artifact' and d.get('executable') and d.get('target',{}).get('name') in ['sheltie','sheltie-export']:bins[d['target']['name']]=d['executable']
 assert set(bins)=={'sheltie','sheltie-export'}
 frozen=Path(f['MSRV_trial']['frozen_binary_dir']);frozen.mkdir();record['actual_cargo_executables']={}
 for name,path in bins.items():
  dest=frozen/name;shutil.copyfile(path,dest);dest.chmod(0o755);record['actual_cargo_executables'][name]={'cargo_path':path,'frozen':str(dest),'sha256':hashlib.sha256(dest.read_bytes()).hexdigest()}
 env['SHELTIE_TEST_ENGINE_BINARY']=str(frozen/'sheltie');env['SHELTIE_TEST_EXPORT_BINARY']=str(frozen/'sheltie-export');record['test_binary_env']={k:env[k] for k in ['SHELTIE_TEST_ENGINE_BINARY','SHELTIE_TEST_EXPORT_BINARY']};save()
 run('msrv-nextest',f['MSRV_trial']['commands'][1]);run('msrv-doctest',f['MSRV_trial']['commands'][2])
 if time.monotonic()-start>=f['MSRV_trial']['limit_s']:raise RuntimeError('Continuous budget exceeded; not a successful trial')
 record['result']='completed';record['source_unchanged']=True
except Exception as ex:
 record['result']='stopped';record['error']=str(ex);save();raise
finally:
 record['ended_utc']=now();record['total_elapsed_s']=time.monotonic()-start;save()
