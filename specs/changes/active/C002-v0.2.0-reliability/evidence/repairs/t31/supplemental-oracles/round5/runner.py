import collections, difflib, hashlib, json, os, pathlib, subprocess, time

root = pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex')
source = root / 'target/t31-validation/oracle-source'
out = root / 'specs/changes/active/C002-v0.2.0-reliability/evidence/repairs/t31/supplemental-oracles/round5'
mutants = out.parent.parent / 'mutants'
manifest = json.loads((mutants / 'source-input.json').read_text())
inventory = json.loads((mutants / 'inventory.json').read_text())
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
for row in manifest['source_files']:
    assert sha(source / row['path']) == row['sha256']
tests = ['crates/sheltie-runtime/tests/disposition_oracles.rs', 'crates/sheltie-cli/tests/disposition_remote.rs']
for path in tests:
    (out / (pathlib.Path(path).stem + '.rs.txt')).write_bytes((source / path).read_bytes())
env = os.environ.copy()
env.update(RUSTC_WRAPPER='', CARGO_NET_OFFLINE='true', CARGO_BUILD_JOBS='2', CARGO_TARGET_DIR='/private/tmp/sheltie-c002-review-9NZouweW/supplement-target')
closure = {'candidate': manifest['candidate'], 'source_manifest_sha256':sha(mutants / 'source-input.json'), 'extra_tests': [{'path':path, 'sha256':sha(source / path)} for path in tests], 'features':['all-features'], 'profile':'default cargo test debug assertions and overflow checks', 'source':str(source), 'tools':json.loads((mutants / 'execution-closure.json').read_text())['tools'], 'scope':'Supplemental real consumer oracles only. Separate input identity; neither replacement nor retroactive PASS for immutable full-workspace runs.'}
closure['sha256'] = hashlib.sha256(json.dumps(closure, sort_keys=True).encode()).hexdigest()
(out / 'closure.json').write_text(json.dumps(closure, indent=2)+'\n')
chosen = []
for m in inventory:
    n = m['name']
    if 'fsx.rs:1431:37:' in n and '|| with &&' in n:
        chosen.append((m,'sheltie-runtime','disposition_oracles','supplemental_file_mode'))
assert len(chosen) == 1
results=[]
for i,(m,package,binary,filter_name) in enumerate(chosen):
    path = source / m['file']; before = path.read_bytes(); text = before.decode(); lines = text.splitlines(True)
    def offset(position):
        return sum(len(x) for x in lines[:position['line']-1]) + position['column']-1
    a=offset(m['span']['start']); b=offset(m['span']['end'])
    if m['genre']=='FnValue':
        a=offset(m['function']['span']['start']); b=offset(m['function']['span']['end']); part=text[a:b]
        changed=text[:a]+part[:part.index('{')+1]+' '+m['replacement']+' '+part[part.rindex('}'):]+text[b:]
    else:
        assert m['genre'] in ['BinaryOperator','MatchArmGuard']
        changed=text[:a]+m['replacement']+text[b:]
    stem=f'{i:02d}'; (out / (stem+'.diff')).write_text(''.join(difflib.unified_diff(text.splitlines(True),changed.splitlines(True),fromfile=m['file'],tofile=m['name'])))
    argv=['cargo','test','--manifest-path',str(source/'Cargo.toml'),'-p',package,'--all-features','--test',binary,filter_name,'--','--test-threads=1']
    started=time.time()
    try:
        path.write_text(changed)
        with (out / (stem+'.stdout.txt')).open('wb') as stream:
            result=subprocess.run(argv,env=env,stdout=stream,stderr=subprocess.STDOUT)
    finally:
        path.write_bytes(before)
    assert sha(path)==hashlib.sha256(before).hexdigest()
    raw=(out / (stem+'.stdout.txt')).read_text()
    item={'name':m['name'],'closure_sha256':closure['sha256'],'argv':argv,'exit':result.returncode,'seconds':round(time.time()-started,3),'caught_by_real_oracle':result.returncode==101 and 'test result: FAILED.' in raw,'diff':stem+'.diff','stdout':stem+'.stdout.txt','restored_source_sha256':sha(path)}
    results.append(item); (out/'results.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(item),flush=True)
for row in manifest['source_files']:
    assert sha(source / row['path'])==row['sha256']
