import collections
import fcntl
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import time

root = pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex')
clone = root / 'target/m1-validation/source'
out = root / 'specs/changes/active/C002-v0.2.0-reliability/evidence/m1-2026-10-01/mutants'
env = os.environ.copy()
env.update(PATH=str(root / 'target/t31-validation/tools') + ':' + env['PATH'],
           RUSTC_WRAPPER='', CARGO_TARGET_DIR='target', CARGO_NET_OFFLINE='true',
           CARGO_BUILD_JOBS='2', MUTANTS_TIMEOUT='600', NEXTEST_TEST_THREADS='2',
           CARGO_PROFILE_TEST_OPT_LEVEL='1', CARGO_PROFILE_TEST_DEBUG='0',
           CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
for key in ['GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE', 'GIT_COMMON_DIR',
            'GIT_OBJECT_DIRECTORY', 'GIT_ALTERNATE_OBJECT_DIRECTORIES']:
    env.pop(key, None)
out.mkdir(exist_ok=True)
assert not any(out.glob('stage1-*')) and not any(out.glob('stage2-*')), 'existing execution phases must be archived first'
assert not any((out / name).exists() for name in ['execution-closure.json', 'baseline.metadata.json', 'baseline.stdout.txt', 'inventory.json', 'progress.json', 'results.json']), 'refusing to overwrite existing execution evidence'
lock = (root / 'target/m1-validation/pipeline.lock').open('w')
fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
manifest = json.loads((out / 'source-input.json').read_text())
candidate = manifest['candidate']

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def save(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')

source_manifest_sha256 = sha(out / 'source-input.json')
pipeline_sha256 = sha(pathlib.Path(__file__))

def verify():
    assert sha(out / 'source-input.json') == source_manifest_sha256, 'source manifest changed'
    assert sha(pathlib.Path(__file__)) == pipeline_sha256, 'pipeline changed'
    assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=clone, text=True).strip() == candidate
    assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=clone, text=True).strip()
    for entry in manifest['source_files']:
        assert sha(root / entry['path']) == entry['sha256'], entry['path']
        assert sha(clone / entry['path']) == entry['sha256'], entry['path']
    for entry in manifest['frozen_governance_files']:
        assert sha(clone / entry['path']) == entry['sha256'], entry['path']

verify()
closure = {'candidate': candidate, 'source_manifest_sha256': sha(out / 'source-input.json'),
           'pipeline_sha256': sha(pathlib.Path(__file__)), 'features': 'all-features',
           'environment': {k: env[k] for k in ['RUSTC_WRAPPER', 'CARGO_TARGET_DIR', 'CARGO_NET_OFFLINE',
            'CARGO_BUILD_JOBS', 'MUTANTS_TIMEOUT', 'NEXTEST_TEST_THREADS', 'CARGO_PROFILE_TEST_OPT_LEVEL',
            'CARGO_PROFILE_TEST_DEBUG', 'CARGO_PROFILE_TEST_DEBUG_ASSERTIONS', 'CARGO_PROFILE_TEST_OVERFLOW_CHECKS']},
           'jobs': 4, 'nextest_threads': 2,
           'tools': {name: subprocess.check_output(argv, cwd=clone, env=env, text=True).splitlines()[0]
            for name, argv in [('rustc', ['rustc', '--version']), ('cargo', ['cargo', '--version']),
                               ('nextest', ['cargo', 'nextest', '--version']), ('mutants', ['cargo', 'mutants', '--version'])]}}
save(out / 'execution-closure.json', closure)
results = []

def record(item):
    results.append(item)
    save(out / 'progress.json', results)
    print(json.dumps(item), flush=True)

def plain(name, argv):
    started = time.time()
    with (out / (name + '.stdout.txt')).open('wb') as stream:
        process = subprocess.run(argv, cwd=clone, env=env, stdout=stream, stderr=subprocess.STDOUT)
    item = {'name': name, 'argv': argv, 'exit': process.returncode, 'seconds': round(time.time() - started, 3),
            'candidate': candidate, 'execution_closure_sha256': sha(out / 'execution-closure.json')}
    save(out / (name + '.metadata.json'), item)
    record(item)
    assert process.returncode == 0, name
    if name == 'baseline':
        log = (out / (name + '.stdout.txt')).read_text()
        assert re.search(r'699 tests run: 699 passed.*0 skipped', log), 'workspace baseline test closure mismatch'
        run = re.search(r'Nextest run ID ([0-9a-f-]+)', log)
        assert run
        item['run_id'] = run.group(1)
        item['tests'] = 699
        save(out / (name + '.metadata.json'), item)

plain('baseline', ['cargo', 'nextest', 'run', '--all-features', '--no-tests=pass'])
argv = ['cargo', 'mutants', '-p', 'sheltie-core', '-p', 'sheltie-runtime', '--list', '--json',
        '--exclude', 'crates/*/src/testkit.rs']
with (out / 'inventory.json').open('wb') as stream, (out / 'inventory.stderr.txt').open('wb') as errors:
    subprocess.run(argv, cwd=clone, env=env, stdout=stream, stderr=errors, check=True)
inventory = json.loads((out / 'inventory.json').read_text())
assert len(inventory) == len({row['name'] for row in inventory})
by_name = {row['name']: row for row in inventory}
assert {row['package'] for row in inventory} == {'sheltie-core', 'sheltie-runtime'}
save(out / 'inventory-metadata.json', {'argv': argv, 'candidate': candidate, 'count': len(inventory),
     'packages': dict(collections.Counter(row['package'] for row in inventory)), 'sha256': sha(out / 'inventory.json')})

def run(name, package, args, requested):
    verify()
    directory = out / name
    directory.mkdir(exist_ok=False)
    source = clone / 'target/mutants.out'
    assert not source.exists(), 'unclaimed mutation output retained'
    argv = ['scripts/mutants.sh', package, '--all-features', '--copy-vcs', 'true', '--jobs', '4'] + args
    started = time.time()
    save(out / 'current-phase.json', {'name': name, 'argv': argv, 'candidate': candidate, 'started_epoch': started})
    with (directory / 'stdout.txt').open('wb') as stream:
        process = subprocess.run(argv, cwd=clone, env=env, stdout=stream, stderr=subprocess.STDOUT)
    completion = {'name': name, 'argv': argv, 'candidate': candidate, 'exit': process.returncode, 'seconds': round(time.time() - started, 3), 'expected_names': sorted(requested), 'execution_closure_sha256': sha(out / 'execution-closure.json')}
    save(directory / 'process-completion.json', completion)
    assert source.exists(), name + ': missing raw output'
    shutil.move(source, directory / 'raw')
    raw = directory / 'raw'
    save(directory / 'raw-manifest.json', {str(path.relative_to(raw)): {'bytes': path.stat().st_size, 'sha256': sha(path)} for path in raw.rglob('*') if path.is_file()})
    data = json.loads((raw / 'outcomes.json').read_text())
    listed = json.loads((raw / 'mutants.json').read_text())
    rows = [r for r in data['outcomes'] if r['scenario'] != 'Baseline']
    baselines = [r for r in data['outcomes'] if r['scenario'] == 'Baseline']
    names = [r['scenario']['Mutant']['name'] for r in rows]
    expected = {r['name'] for r in listed}
    item = {'name': name, 'argv': argv, 'exit': process.returncode, 'seconds': round(time.time() - started, 3),
      'candidate': candidate, 'execution_closure_sha256': sha(out / 'execution-closure.json'),
      'listed': len(listed), 'processed': len(rows), 'counts': dict(collections.Counter(r['summary'] for r in rows)),
      'outcomes_sha256': sha(raw / 'outcomes.json'), 'mutants_sha256': sha(raw / 'mutants.json')}
    save(directory / 'metadata.json', item)
    record(item)
    assert process.returncode in [0, 2, 3], name
    assert len(listed) == len(expected) and expected == set(requested), name + ': selection mismatch'
    for mutant in listed:
        assert mutant == by_name[mutant['name']], name + ': mutation semantics mismatch'
    assert len(baselines) == 1 and baselines[0]['summary'] == 'Success', name
    assert {phase['phase']: phase['process_status'] for phase in baselines[0]['phase_results']} == {'Build': 'Success', 'Test': 'Success'}, name
    assert len(names) == len(set(names)) == len(expected) and set(names) == expected, name
    assert data['end_time'] is not None, name + ': incomplete outcomes metadata'
    for row in rows:
        obj = dict(row['scenario']['Mutant'])
        expected_obj = dict(by_name[obj['name']]); expected_diff = expected_obj.pop('diff')
        assert obj == expected_obj, name + ': outcome mutation mismatch'
        assert row['diff_path'] and (raw / row['diff_path']).is_file(), name + ': mutation diff raw missing'
        assert (raw / row['diff_path']).read_text() == expected_diff, name + ': diff mismatch'
        phases = {phase['phase']: phase['process_status'] for phase in row['phase_results']}
        for phase in row['phase_results']:
            if '--test-workspace' in args and args[args.index('--test-workspace') + 1] == 'true':
                assert '--workspace' in phase['argv'], name + ': workspace consumers missing'
            else:
                assert f'--package={package}@0.1.0' in phase['argv'], name + ': package tests missing'
        log = (raw / row['log_path']).read_text(errors='replace')
        if row['summary'] == 'CaughtMutant':
            assert phases == {'Build': 'Success', 'Test': {'Failure': 100}}, name + ': non-test catch'
            assert any(token in log for token in ['FAIL', 'ABORT', 'SIGABRT', 'stack overflow', 'test failed']), name + ': missing failure oracle'
        elif row['summary'] == 'Unviable':
            diagnostic = re.search(r'(?ms)^error(?:\[E[0-9]+\])?:[^\n]*\n(?:(?!^error).){0,4000}?^\s*-->[^\n]*\.rs:[0-9]+:[0-9]+', log)
            assert phases.get('Build') == {'Failure': 101} and diagnostic, name + ': unviable lacks a Rust source diagnostic; preserve raw as tool/execution failure'
        elif row['summary'] == 'MissedMutant':
            assert phases == {'Build': 'Success', 'Test': 'Success'}, name
        elif row['summary'] == 'Timeout':
            assert 'Timeout' in phases.values(), name
    if process.returncode == 3:
        assert any(row['summary'] == 'Timeout' for row in rows), name + ': unexplained exit3'
    assert all(r['summary'] in ['CaughtMutant', 'MissedMutant', 'Timeout', 'Unviable'] for r in rows), name
    return rows

def regex(names):
    escape = lambda name: ''.join('\\' + char if char in r'\.^$|?*+()[]{}' else char for char in name)
    return '^(?:' + '|'.join(escape(name) for name in names) + ')$'

def selections(package, count):
    selected = []
    for index in range(count):
        argv = ['cargo', 'mutants', '-p', package, '--list', '--json', '--exclude', 'crates/*/src/testkit.rs', '--shard', f'{index}/{count}']
        with (out / f'{package}-shard-{index}-list.stderr.txt').open('wb') as errors:
            listed = json.loads(subprocess.check_output(argv, cwd=clone, env=env, text=True, stderr=errors))
        assert all(mutant == by_name[mutant['name']] for mutant in listed)
        names = [mutant['name'] for mutant in listed]
        assert len(names) == len(set(names))
        selected.append(names)
        save(out / f'{package}-shard-{index}-selection.json', {'argv': argv, 'names': names})
    union = [name for part in selected for name in part]
    expected = {row['name'] for row in inventory if row['package'] == package}
    assert len(union) == len(set(union)) == len(expected) and set(union) == expected
    return selected

shards = {package: selections(package, count) for package, count in [('sheltie-core', 4), ('sheltie-runtime', 8)]}
stage1 = {}
for package, parts in shards.items():
    rows = []
    for index, names in enumerate(parts):
        rows.extend(run(f'stage1-{package}-{index}-of-{len(parts)}', package,
            ['--test-workspace', 'false', '--shard', f'{index}/{len(parts)}'], names))
    observed = [row['scenario']['Mutant']['name'] for row in rows]
    expected = {row['name'] for row in inventory if row['package'] == package}
    assert len(observed) == len(set(observed)) == len(expected) and set(observed) == expected
    stage1[package] = rows

stage2 = {}
for package, rows in stage1.items():
    pending = [row['scenario']['Mutant']['name'] for row in rows if row['summary'] in ['MissedMutant', 'Timeout']]
    save(out / f'{package}-stage2-selection.json', pending)
    tested = []
    for index in range(8):
        selected = pending[index::8]
        if selected:
            tested.extend(run(f'stage2-{package}-{index}-of-8', package,
                ['--test-workspace', 'true', '--re', regex(selected)], selected))
    observed = [row['scenario']['Mutant']['name'] for row in tested]
    assert len(observed) == len(set(observed)) == len(pending) and set(observed) == set(pending)
    stage2[package] = tested

remaining = [row for rows in stage2.values() for row in rows if row['summary'] != 'CaughtMutant']
verify()
save(out / 'results.json', {'candidate': candidate, 'all_stage1_fresh': True,
    'stage1': stage1, 'stage2': stage2, 'complete_execution': True,
    'disposition_complete': False, 'remaining': remaining})
print('Full fresh core/runtime inventory and all current workspace consumers executed; independent survivor disposition still required.', flush=True)
