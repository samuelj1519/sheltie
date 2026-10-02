import pathlib,json,hashlib,subprocess,os
root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex');source=root/'target/m1-validation/extra-source';out=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/m1-2026-10-01/mutants/adaptive-extra-oracles';man=json.loads((out.parent/'source-input.json').read_text());inv={x['name']:x for x in json.loads((out.parent/'inventory.json').read_text())};name='crates/sheltie-runtime/src/fsx.rs:1881:47: replace | with & in ExternalReadTree::open_file';mut=inv[name];file=source/mut['file'];original=file.read_bytes();env=os.environ.copy();env.update(RUSTC_WRAPPER='',CARGO_TARGET_DIR=str(root/'target/m1-validation/extra-target'),CARGO_BUILD_JOBS='1',CARGO_NET_OFFLINE='true');directory=out/'source-alias-v2';directory.mkdir();results=[];sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();save=lambda p,o:p.write_text(json.dumps(o,indent=2,sort_keys=True)+'\n')
addition='''        if let Some(path) = std::env::var_os("SHELTIE_EXTRA_SOURCE_OPEN_TRACE") {
            if std::env::var("SHELTIE_TEST_RENDEZVOUS_ID").ok().as_deref() == Some(&display) {
                use std::io::Write;
                let mut trace = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
                writeln!(trace, "{} {} {}", display, opened.st_dev, opened.st_ino).unwrap();
            }
        }
'''
def instrument(data):
 text=data.decode();needle='        Ok(ExternalTreeFileHandle(std::fs::File::from(fd)))';assert text.count(needle)==1;return text.replace(needle,addition+needle).encode()
save(directory/'execution-closure.json',{'candidate':man['candidate'],'tools':{n:subprocess.check_output(a,cwd=source,text=True).strip() for n,a in [('cargo',['cargo','--version']),('rustc',['rustc','--version'])]},'environment':{k:env[k] for k in ['RUSTC_WRAPPER','CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_NET_OFFLINE']},'oracle_sha256':sha(out/'source-alias-v2.py'),'runner_sha256':sha(pathlib.Path(__file__)),'scope':'same-inode alias post-open observation on both control/mutant; subsequent final CLI rejection is preserved, detection only early following'});(directory/'instrumentation.txt').write_text(addition);(directory/'original.diff').write_text(mut['diff'])
try:
 for mode in ['control','mutant']:
  file.write_bytes(original)
  if mode=='mutant':
   r=subprocess.run(['patch','--batch',str(file)],input=mut['diff'],text=True,capture_output=True);assert r.returncode==0;(directory/'patch.stdout.txt').write_text(r.stdout+r.stderr)
  file.write_bytes(instrument(file.read_bytes()));item={'mode':mode,'name':name,'original_diff_sha256':sha(directory/'original.diff'),'source_input':[{'path':x['path'],'sha256':sha(source/x['path'])} for x in man['source_files']]};args=['cargo','build','-p','sheltie-cli','--all-features','--message-format=json']
  with (directory/(mode+'-build.jsonl')).open('w') as o,(directory/(mode+'-build.stderr')).open('w') as e:r=subprocess.run(args,cwd=source,env=env,stdout=o,stderr=e)
  item['build']={'argv':args,'exit':r.returncode};assert r.returncode==0
  arts=[json.loads(x) for x in (directory/(mode+'-build.jsonl')).read_text().splitlines()];binary=next(x['executable'] for x in arts if x.get('reason')=='compiler-artifact' and x.get('executable') and x['target']['name']=='sheltie');item['binary_sha256']=sha(pathlib.Path(binary));args=['python3',str(out/'source-alias-v2.py'),'--binary',binary,'--source',str(source),'--output',str(directory/mode)];r=subprocess.run(args,env=env,capture_output=True,text=True,timeout=180);(directory/(mode+'.stdout.txt')).write_text(r.stdout+r.stderr);item['oracle']={'argv':args,'exit':r.returncode,'semantic_failure':r.returncode!=0 and 'AssertionError:' in r.stderr and 'SEMANTIC ' in r.stderr};results.append(item)
  if mode=='control' and r.returncode:raise RuntimeError('original control failed '+r.stderr[-500:])
  if mode=='mutant' and r.returncode and not item['oracle']['semantic_failure']:raise RuntimeError('tool/fixture error')
finally:
 file.write_bytes(original);save(directory/'results.json',results);save(directory/'manifest.json',{str(p.relative_to(directory)):sha(p) for p in directory.rglob('*') if p.is_file() and p.name!='manifest.json'});progress=pathlib.Path('/private/tmp/m1-extra-progress.json');d=json.loads(progress.read_text());d['source_alias_v2']=results;save(progress,d)
print([(x['mode'],x['oracle']['exit'],x['oracle']['semantic_failure']) for x in results])
