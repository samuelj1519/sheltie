import ast,collections,fcntl,hashlib,json,os,pathlib,re,shutil,subprocess,time
root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex');clone=root/'target/m1-validation/source';out=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/m1-2026-10-01/mutants'
assert not (out/'results.json').exists(),'refusing to overwrite full execution results'
audit_file=out/'independent-resume-audit.json';audit=json.loads(audit_file.read_text());assert audit['result']=='PASS' and not audit['errors']
audit_sha=hashlib.sha256(audit_file.read_bytes()).hexdigest()
original=pathlib.Path('/private/tmp/sheltie-m1-final-pipeline.py');resume_script=pathlib.Path(__file__);resume_sha=hashlib.sha256(resume_script.read_bytes()).hexdigest()
lock=(root/'target/m1-validation/pipeline.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
manifest=json.loads((out/'source-input.json').read_text());candidate=manifest['candidate'];closure=json.loads((out/'execution-closure.json').read_text());inventory=json.loads((out/'inventory.json').read_text());by_name={m['name']:m for m in inventory}
env=os.environ.copy();env.update({k:str(v) for k,v in closure['environment'].items()});env['PATH']=str(root/'target/t31-validation/tools')+':'+env['PATH']
for key in ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_COMMON_DIR','GIT_OBJECT_DIRECTORY','GIT_ALTERNATE_OBJECT_DIRECTORIES']:env.pop(key,None)
assert hashlib.sha256((out/'execution-closure.json').read_bytes()).hexdigest()=='76e3ee520154f64b232f3f0bdb8d693e8b60ca17231d89bc656b45950c370a56'
assert hashlib.sha256((out/'inventory.json').read_bytes()).hexdigest()=='efb63f60dd1e3acd60553f49de5cd5f1a15948b8c8a59feda431df8279d7ad3f'
completed_metadata_sha={name:hashlib.sha256((out/name/'metadata.json').read_bytes()).hexdigest() for name in audit['shards']}
source_manifest_sha256=closure['source_manifest_sha256'];pipeline_sha256=closure['pipeline_sha256'];results=json.loads((out/'progress.json').read_text())
ns=globals();ns['__file__']=str(original)
text=original.read_text();tree=ast.parse(text)
for node in tree.body:
 if isinstance(node,ast.FunctionDef) and node.name in ['sha','save','verify','record','run','regex']:exec(compile(ast.Module(body=[node],type_ignores=[]),str(original),'exec'),ns)
original_verify=verify
def verify():
 assert sha(resume_script)==resume_sha,'continuation driver changed'
 assert sha(out/'execution-closure.json')=='76e3ee520154f64b232f3f0bdb8d693e8b60ca17231d89bc656b45950c370a56','original execution closure changed'
 assert sha(out/'inventory.json')=='efb63f60dd1e3acd60553f49de5cd5f1a15948b8c8a59feda431df8279d7ad3f','inventory changed'
 assert sha(audit_file)==audit_sha,'reuse audit changed'
 for name,digest in completed_metadata_sha.items():assert sha(out/name/'metadata.json')==digest,name+' prior metadata changed'
 original_verify()
assert sha(original)==pipeline_sha256
verify()
for name,argv in [('rustc',['rustc','--version']),('cargo',['cargo','--version']),('nextest',['cargo','nextest','--version']),('mutants',['cargo','mutants','--version'])]:assert subprocess.check_output(argv,cwd=clone,env=env,text=True).splitlines()[0]==closure['tools'][name],name
resume_dir=out/'continuation-2026-10-02';resume_dir.mkdir(exist_ok=False)
record_resume={'candidate':candidate,'original_execution_closure_sha256':sha(out/'execution-closure.json'),'original_pipeline_sha256':pipeline_sha256,'resume_driver_sha256':resume_sha,'source_manifest_sha256':source_manifest_sha256,'scope':'Continue only after owned execution process disappeared and lock was free. Complete phases retain original run IDs, metadata and raw bytes. Partial phase3 is archived and all45 requested IDs are rerun; no partial outcome reuse. Same source/config/tool/features/environment/test argv as original. Orchestration driver differs and is separately pinned.','started_epoch':time.time(),'completed_phases_reused':[x['name'] for x in results],'independent_reuse_audit_sha256':audit_sha,'completed_metadata_sha256':completed_metadata_sha}
save(resume_dir/'closure.json',record_resume);shutil.copyfile(resume_script,resume_dir/'driver.py')
active=out/'stage2-sheltie-runtime-3-of-8';raw=clone/'target/mutants.out'
assert active.exists() and raw.exists() and not (active/'metadata.json').exists()
assert json.loads((raw/'outcomes.json').read_text())['end_time'] is None
shutil.copyfile(out/'current-phase.json',active/'interrupted-current-phase.json');shutil.move(raw,active/'raw')
save(active/'raw-manifest.json',{str(p.relative_to(active/'raw')):{'bytes':p.stat().st_size,'sha256':sha(p)} for p in (active/'raw').rglob('*') if p.is_file()})
partial=json.loads((active/'raw/outcomes.json').read_text());save(active/'interruption.json',{'status':'incomplete_execution','process_exit':None,'cause':'Execution session unavailable, no owned pipeline/mutants/nextest process found; fcntl lock free. Exact external stop cause unknown. No platform safety notice observed.','processed_partial':partial['total_mutants'],'end_time':partial['end_time'],'acceptance':'WIP only; no partial result reused','archive_epoch':time.time()})
shutil.move(active,resume_dir/'interrupted-stage2-runtime3')
def load_rows(name):
 p=out/name;meta=json.loads((p/'metadata.json').read_text());d=json.loads((p/'raw/outcomes.json').read_text());assert d['end_time'] is not None and meta['candidate']==candidate and meta['execution_closure_sha256']==sha(out/'execution-closure.json');assert sha(p/'raw/outcomes.json')==meta['outcomes_sha256']
 listed=json.loads((p/'raw/mutants.json').read_text());assert sha(p/'raw/mutants.json')==meta['mutants_sha256'];rows=[r for r in d['outcomes'] if r['scenario']!='Baseline'];names=[r['scenario']['Mutant']['name'] for r in rows];expected={m['name'] for m in listed}
 assert len(names)==len(set(names))==len(listed)==meta['processed']==meta['listed'] and set(names)==expected,name
 if name in audit['shards']:
  a=audit['shards'][name];assert a['result']=='PASS' and not a['errors'];assert meta['outcomes_sha256']==a['outcomes_sha256'] and meta['mutants_sha256']==a['mutants_sha256'];assert meta['counts']==a['counts'] and meta['exit']==a['exit']
 baseline=[r for r in d['outcomes'] if r['scenario']=='Baseline'];assert len(baseline)==1 and baseline[0]['summary']=='Success'
 for m in listed:assert m==by_name[m['name']],name
 for r in rows:
  obj=dict(r['scenario']['Mutant']);expected_obj=dict(by_name[obj['name']]);expected_obj.pop('diff');assert obj==expected_obj,name
  phases={x['phase']:x['process_status'] for x in r['phase_results']}
  if r['summary']=='CaughtMutant':assert phases=={'Build':'Success','Test':{'Failure':100}},name
  elif r['summary']=='MissedMutant':assert phases=={'Build':'Success','Test':'Success'},name
  elif r['summary']=='Unviable':assert phases.get('Build')=={'Failure':101},name
  elif r['summary']=='Timeout':assert 'Timeout' in phases.values(),name
  else:raise AssertionError(r['summary'])
  package=by_name[obj['name']]['package']
  for x in r['phase_results']:assert ('--workspace' in x['argv']) if name.startswith('stage2-') else (f'--package={package}@0.1.0' in x['argv']),name
 return rows
stage1={}
for package,count in [('sheltie-core',4),('sheltie-runtime',8)]:
 rows=[r for i in range(count) for r in load_rows(f'stage1-{package}-{i}-of-{count}')];names=[r['scenario']['Mutant']['name'] for r in rows];expected={m['name'] for m in inventory if m['package']==package};assert len(names)==len(set(names))==len(expected) and set(names)==expected;stage1[package]=rows
stage2={}
for package,rows in stage1.items():
 pending=[r['scenario']['Mutant']['name'] for r in rows if r['summary'] in ['MissedMutant','Timeout']];assert pending==json.loads((out/f'{package}-stage2-selection.json').read_text());tested=[]
 for i in range(8):
  names=pending[i::8];name=f'stage2-{package}-{i}-of-8'
  if not names:continue
  if (out/name/'metadata.json').exists():
   group=load_rows(name);assert set(r['scenario']['Mutant']['name'] for r in group)==set(names)
  else:group=run(name,package,['--test-workspace','true','--re',regex(names)],names)
  tested.extend(group)
 observed=[r['scenario']['Mutant']['name'] for r in tested];assert len(observed)==len(set(observed))==len(pending) and set(observed)==set(pending);stage2[package]=tested
verify();remaining=[r for rows in stage2.values() for r in rows if r['summary']!='CaughtMutant']
save(out/'results.json',{'candidate':candidate,'all_stage1_fresh':True,'stage1':stage1,'stage2':stage2,'complete_execution':True,'disposition_complete':False,'remaining':remaining,'continuation':'continuation-2026-10-02/closure.json'})
print('Full execution complete. Independent per-ID dispositions and final M1 review still required.',flush=True)
