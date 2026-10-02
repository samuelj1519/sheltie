import hashlib,json,os,pathlib,subprocess,time
root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex');source=root/'target/m1-validation/extra-source';target=root/'target/m1-validation/extra-target'
out=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/m1-2026-10-01/mutants/adaptive-extra-oracles';base=out.parent
inventory={m['name']:m for m in json.loads((base/'inventory.json').read_text())};source_manifest=json.loads((base/'source-input.json').read_text())
env=os.environ.copy();env.update(RUSTC_WRAPPER='',CARGO_TARGET_DIR=str(target),CARGO_BUILD_JOBS='1',CARGO_NET_OFFLINE='true')
jobs=[('crates/sheltie-runtime/src/workbook_repo.rs:342:38: replace || with && in WorkbookRepo::load_checked_request',['audit']),('crates/sheltie-runtime/src/load.rs:279:9: replace || with && in compile_frozen_workbook',['manifest-id','manifest-version']),('crates/sheltie-runtime/src/fsx.rs:217:50: replace match guard create_missing with false in ManagedFs::traverse_root',['nested-root'])]
def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
results=[]
for index,(name,families) in enumerate(jobs):
 for e in source_manifest['source_files']:assert sha(source/e['path'])==e['sha256'],e['path']
 mutant=inventory[name];directory=out/('mutant-'+str(index));directory.mkdir(exist_ok=False);file=source/mutant['file'];original=file.read_bytes();(directory/'original.diff').write_text(mutant['diff'])
 item={'name':name,'family':families,'candidate':source_manifest['candidate'],'original_diff_sha256':sha(directory/'original.diff'),'scope':'supplemental same-oracle control/mutant execution; formal mutation input unchanged','environment':{k:env[k] for k in ['RUSTC_WRAPPER','CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_NET_OFFLINE']},'runs':[]}
 try:
  patch=subprocess.run(['patch','--batch',str(file)],input=mutant['diff'],text=True,capture_output=True);(directory/'patch.stdout.txt').write_text(patch.stdout+patch.stderr);assert patch.returncode==0,'patch failed'
  item['source_input']=[{'path':e['path'],'sha256':sha(source/e['path'])} for e in source_manifest['source_files']]
  argv=['cargo','build','-p','sheltie-cli','--all-features','--message-format=json'];started=time.time()
  with (directory/'build.jsonl').open('w') as stdout,(directory/'build.stderr.txt').open('w') as stderr:r=subprocess.run(argv,cwd=source,env=env,stdout=stdout,stderr=stderr)
  item['build']={'argv':argv,'exit':r.returncode,'seconds':time.time()-started};assert r.returncode==0,'tool/build failure, not detection'
  artifacts=[json.loads(x) for x in (directory/'build.jsonl').read_text().splitlines()];binary=next(x['executable'] for x in artifacts if x.get('reason')=='compiler-artifact' and x.get('executable') and x['target']['name']=='sheltie');item['binary_sha256']=sha(pathlib.Path(binary))
  for family in families:
   assert json.loads((out/'control'/family/'metadata.json').read_text())['complete'],'original control required'
   argv=['python3',str(out/'persisted.py'),'--binary',binary,'--source',str(source),'--output',str(directory/family),'--family',family]
   r=subprocess.run(argv,env=env,capture_output=True,text=True,timeout=180);(directory/(family+'.stdout.txt')).write_text(r.stdout+r.stderr)
   semantic=r.returncode!=0 and 'AssertionError:' in r.stderr and 'SEMANTIC ' in r.stderr;item['runs'].append({'family':family,'argv':argv,'exit':r.returncode,'semantic_failure':semantic})
   if r.returncode!=0 and not semantic:raise RuntimeError('tool/fixture error, not detection')
  item['result']='detected_by_supplemental_oracle' if any(x['semantic_failure'] for x in item['runs']) else 'not_detected'
 except Exception as e:item['result']='execution_error';item['error']=repr(e);raise
 finally:
  file.write_bytes(original);assert sha(file)==next(e['sha256'] for e in source_manifest['source_files'] if e['path']==mutant['file']);save(directory/'metadata.json',item)
  save(directory/'manifest.json',{str(p.relative_to(directory)):sha(p) for p in directory.rglob('*') if p.is_file() and p.name!='manifest.json'});results.append(item);save(pathlib.Path('/private/tmp/m1-extra-progress.json'),{'results':results,'source_restored':True,'self_reviewed':False})
save(out/'persisted-results.json',results)
