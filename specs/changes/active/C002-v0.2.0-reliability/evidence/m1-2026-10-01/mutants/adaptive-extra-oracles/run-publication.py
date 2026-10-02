import pathlib,json,hashlib,os,subprocess,time
root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex');source=root/'target/m1-validation/extra-source';target=root/'target/m1-validation/extra-target';out=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/m1-2026-10-01/mutants/adaptive-extra-oracles';base=out.parent;man=json.loads((base/'source-input.json').read_text());inv={x['name']:x for x in json.loads((base/'inventory.json').read_text())};sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();save=lambda p,v:p.write_text(json.dumps(v,indent=2,sort_keys=True)+'\n');env=os.environ.copy();env.update(RUSTC_WRAPPER='',CARGO_TARGET_DIR=str(target),CARGO_BUILD_JOBS='1',CARGO_NET_OFFLINE='true')
runroot=out/'publication';runroot.mkdir();jobs=[('crates/sheltie-runtime/src/effects.rs:1050:9: replace || with && in verify_publish_object','publication.py')]
meta={'candidate':man['candidate'],'environment':{k:env[k] for k in ['RUSTC_WRAPPER','CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_NET_OFFLINE']},'tools':{n:subprocess.check_output(a,cwd=source,text=True).strip() for n,a in [('rustc',['rustc','--version']),('cargo',['cargo','--version'])]},'scripts':{n:sha(out/n) for n in ['publication.py']},'runner_sha256':sha(pathlib.Path(__file__)),'control':[],'mutants':[]}
for x in man['source_files']:assert sha(source/x['path'])==x['sha256']
def build(directory):
 directory.mkdir();item={'source_input':[{'path':x['path'],'sha256':sha(source/x['path'])} for x in man['source_files']]};args=['cargo','build','-p','sheltie-cli','--all-features','--message-format=json'];start=time.time()
 with (directory/'build.jsonl').open('w') as o,(directory/'build.stderr.txt').open('w') as e:r=subprocess.run(args,cwd=source,env=env,stdout=o,stderr=e)
 item['build']={'argv':args,'exit':r.returncode,'seconds':time.time()-start};assert r.returncode==0,'build/tool error'
 arts=[json.loads(x) for x in (directory/'build.jsonl').read_text().splitlines()];binary=next(x['executable'] for x in arts if x.get('reason')=='compiler-artifact' and x.get('executable') and x['target']['name']=='sheltie');item['binary_sha256']=sha(pathlib.Path(binary));save(directory/'build-closure.json',item);return binary
try:
 binary=build(runroot/'control-build')
 for script in ['publication.py']:
  args=['python3',str(out/script),'--binary',binary,'--source',str(source),'--output',str(runroot/('control-'+script[:-3]))];r=subprocess.run(args,env=env,capture_output=True,text=True,timeout=180);(runroot/(script+'-control.stdout.txt')).write_text(r.stdout+r.stderr);meta['control'].append({'script':script,'argv':args,'exit':r.returncode});assert r.returncode==0,'original control failure '+r.stderr[-700:]
 for i,(name,script) in enumerate(jobs):
  mutant=inv[name];file=source/mutant['file'];original=file.read_bytes();directory=runroot/('mutant-'+str(i));item={'name':name}
  try:
   r=subprocess.run(['patch','--batch',str(file)],input=mutant['diff'],text=True,capture_output=True);assert r.returncode==0;rraw=r.stdout+r.stderr;binary=build(directory);(directory/'original.diff').write_text(mutant['diff']);(directory/'patch.stdout.txt').write_text(rraw);item['original_diff_sha256']=sha(directory/'original.diff');args=['python3',str(out/script),'--binary',binary,'--source',str(source),'--output',str(directory/'oracle')];r=subprocess.run(args,env=env,capture_output=True,text=True,timeout=180);(directory/'oracle.stdout.txt').write_text(r.stdout+r.stderr);semantic=r.returncode!=0 and 'AssertionError:' in r.stderr and 'SEMANTIC ' in r.stderr;item.update(argv=args,exit=r.returncode,result='detected_by_supplemental_oracle' if semantic else 'not_detected')
   if r.returncode and not semantic:raise RuntimeError('tool/fixture failure '+r.stderr[-700:])
  finally:
   file.write_bytes(original);assert sha(file)==next(x['sha256'] for x in man['source_files'] if x['path']==mutant['file']);meta['mutants'].append(item);save(runroot/'results.json',meta)
finally:
 save(runroot/'results.json',meta);save(runroot/'manifest.json',{str(p.relative_to(runroot)):sha(p) for p in runroot.rglob('*') if p.is_file() and p.name!='manifest.json'});progress=pathlib.Path('/private/tmp/m1-extra-progress.json');d=json.loads(progress.read_text());d['schema_release']=meta;save(progress,d)
print([(x['name'],x.get('result')) for x in meta['mutants']])
