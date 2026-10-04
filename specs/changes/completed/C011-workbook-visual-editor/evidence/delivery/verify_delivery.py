import datetime, hashlib, json, os, pathlib, subprocess, sys, traceback, zipfile

OWN = pathlib.Path('/private/tmp/sheltie-c011-delivery-20261004-fzbr8xn8/reporter-format-corrected')
REPO = pathlib.Path('/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor')
COPY = REPO.parent / 'patch-check'
WORK = pathlib.Path('/private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/home/works/2026-10-04-001-workbook-visual-editor')
PATCH = WORK / 'attempts/deliver/occurrence-001/attempt-000/outputs/change.patch'
BASE = '5837de68c257f49c970d54254dfcc12bd7fe210d'
TREE = '4bc46c4d32e4ab8a4af39a661876b39cb9b8957d'
ALLOW = ['tools/workbook-editor', 'specs/changes/active/C011-workbook-visual-editor/evidence/implementation']
proof = {'format': 'delivery-independent-application/v1', 'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'initial_head': BASE, 'reviewed_index_tree': TREE, 'commands': [], 'inputs': [], 'inspection': {}, 'complete': False}

def sha(b):
    return hashlib.sha256(b).hexdigest()

def save():
    (OWN / 'proof.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2) + '\n')

def run(label, argv, cwd, data=None, output=None):
    start = datetime.datetime.now(datetime.timezone.utc).isoformat()
    p = subprocess.run(argv, cwd=cwd, input=data, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60)
    out = pathlib.Path(output) if output else OWN / (label + '.stdout')
    err = OWN / (label + '.stderr')
    out.write_bytes(p.stdout)
    err.write_bytes(p.stderr)
    record = {'label': label, 'argv': argv, 'cwd': str(cwd), 'started_utc': start, 'ended_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'exit_code': p.returncode, 'stdout': {'path': str(out), 'bytes': len(p.stdout), 'sha256': sha(p.stdout)}, 'stderr': {'path': str(err), 'bytes': len(p.stderr), 'sha256': sha(p.stderr)}, 'environment': {k: v for k, v in os.environ.items() if k in ('PATH', 'HOME', 'LANG', 'LC_ALL', 'LC_CTYPE') or k.startswith('GIT_')}}
    if data is not None:
        inp = OWN / (label + '.stdin')
        inp.write_bytes(data)
        record['stdin'] = {'path': str(inp), 'bytes': len(data), 'sha256': sha(data)}
    proof['commands'].append(record)
    save()
    assert p.returncode == 0, (label, p.returncode, str(err))
    return p.stdout

def bind(label, path, expected):
    data = path.read_bytes()
    actual = sha(data)
    proof['inputs'].append({'name': label, 'path': str(path), 'bytes': len(data), 'sha256': actual, 'expected_sha256': expected})
    assert actual == expected, (label, actual, expected)
    return data

def readwork(root, path, mode):
    p = root / path
    if mode == '120000':
        assert p.is_symlink(), str(p)
        return os.readlink(p).encode()
    assert p.is_file() and not p.is_symlink(), str(p)
    assert bool(p.stat().st_mode & 0o111) == (mode == '100755'), str(p)
    return p.read_bytes()

try:
    for name, relative, expected in [
        ('task', 'start-inputs/task', '738958b85f082c17594d9271a202de567d268d90f9f98e82153d0b81924f170b'),
        ('project', 'start-inputs/project', 'ddad68d362d727081a868edbdd555aa89ad204caf0a3453db6c48eb4b983de1c'),
        ('change', 'attempts/implement/occurrence-002/attempt-000/outputs/change.md', 'c90077ede12db1b8782e625066212effd7424a36de18dc0886449a6b922afc94'),
        ('checks', 'attempts/implement/occurrence-002/attempt-000/outputs/checks.md', '46793e0a901cc3cf013c4955c6a2bba99cdddd2e66ced46ab2e9e1d819190258'),
        ('review', 'attempts/review/occurrence-002/attempt-000/outputs/review.md', '50dc046357c541c8cd38317eae509ef9137b3f5329f7750001db3527c7f13d36'),
    ]:
        bind(name, WORK / relative, expected)
    assert (WORK / 'attempts/review/occurrence-002/attempt-000/outputs/review.md').read_text().splitlines()[0] == '建议交付'
    assert run('source-head-before', ['git', 'rev-parse', 'HEAD'], REPO).decode().strip() == BASE
    assert run('source-tree-before', ['git', 'write-tree'], REPO).decode().strip() == TREE
    source_status = run('source-status-before', ['git', 'status', '--porcelain=v1', '-z', '--untracked-files=all'], REPO)
    assert run('source-unstaged-before', ['git', 'diff', '--name-only', '-z'], REPO) == b''
    assert run('source-untracked-allowed', ['git', 'ls-files', '--others', '--exclude-standard', '-z', '--', *ALLOW], REPO) == b''
    changed = [p.decode() for p in run('source-changed-paths', ['git', 'diff', '--cached', '--name-only', '-z', BASE], REPO).split(b'\0') if p]
    assert len(changed) == 188, len(changed)
    assert all(any(p.startswith(a + '/') for a in ALLOW) for p in changed), changed
    proof['inspection']['changed_paths'] = changed
    assert run('check-head-initial', ['git', 'rev-parse', 'HEAD'], COPY).decode().strip() == BASE
    assert run('check-status-initial', ['git', 'status', '--porcelain=v1', '-z', '--untracked-files=all'], COPY) == b''
    assert run('check-tree-initial', ['git', 'write-tree'], COPY).decode().strip() == 'a64335ce7c5db92bbf356dc7980052e9e9439208'
    evi = REPO / ALLOW[1]
    runtime = evi / 'inputs-efe644655640d4c19ddfb61636f44d8f4dba3328410b3d317db6388d117e5f07.json'
    manifest = json.loads(bind('final-test-runtime-manifest', runtime, 'efe644655640d4c19ddfb61636f44d8f4dba3328410b3d317db6388d117e5f07'))
    drift = [p for group in ('files_sha256', 'dependencies_sha256') for p, h in manifest[group].items() if sha((REPO / p).read_bytes()) != h]
    assert not drift, drift
    bind('trusted-engine', pathlib.Path(manifest['engine']['path']), '04dba0274004bd46b35e68a2e5fc8f26387cf009b07c33f2e38e3caf75fb45d8')
    originals = []
    for stem, expected_exit in [('20261004T081116-db847bc2', 1), ('20261004T081550-29a4bbe0', 0), ('20261004T081708-f56422d2', 0), ('20261004T081752-6f7b34b0', 0), ('20261004T081709-71eb3e6c', 0), ('20261004T081709-724a0e00', 0), ('20261004T081919-32338448', 0), ('20261004T081919-bdd4806a', 0)]:
        mpath = evi / (stem + '.json')
        m = json.loads(mpath.read_bytes())
        assert m['head'] == BASE and m['exit_code'] == expected_exit, stem
        streams = {}
        for key in ('stdout', 'stderr'):
            p = evi / m[key]
            b = p.read_bytes()
            streams[key] = {'path': str(p), 'bytes': len(b), 'sha256': sha(b)}
            if stem == '20261004T081708-f56422d2' and key == 'stdout':
                assert all(('ℹ ' + value).encode() in b for value in ('tests 28', 'pass 28', 'fail 0', 'skipped 0'))
        originals.append({'metadata': str(mpath), 'metadata_sha256': sha(mpath.read_bytes()), 'argv': m['argv'], 'cwd': m['cwd'], 'exit_code': m['exit_code'], 'streams': streams})
    proof['inspection']['original_checks'] = originals
    proof['inspection']['runtime_drift'] = drift
    browser = pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C011-workbook-visual-editor/evidence/browser')
    browser_files = ['repaired-observations.json', '10-layout-bytes-proof.json', '11-code-change-sample.ax.txt']
    proof['inspection']['root_browser_records'] = [{'path': str(browser / p), 'sha256': sha((browser / p).read_bytes()), 'bytes': (browser / p).stat().st_size} for p in browser_files]
    for p in browser_files[:2]:
        assert json.loads((browser / p).read_bytes())['candidate_tree'] == TREE
    with zipfile.ZipFile(browser / '08-layout-before.zip') as a, zipfile.ZipFile(browser / '10-layout-after.zip') as b:
        assert sorted(a.namelist()) == sorted(b.namelist())
        zs = []
        for p in sorted(a.namelist()):
            x, y = a.read(p), b.read(p)
            assert x == y, p
            zs.append({'path': p, 'bytes': len(x), 'sha256': sha(x)})
        proof['inspection']['root_browser_zip_contents'] = zs
    assert not PATCH.exists(), 'Do not overwrite an existing delivery patch'
    patch = run('generate-complete-binary-patch', ['git', 'diff', '--cached', '--binary', BASE, '--', *ALLOW], REPO, output=PATCH)
    assert len(patch) <= 8388608, len(patch)
    assert len(patch) == 7807709 and sha(patch) == '3717cccd6d8862a6217e67655d61c9e30bdc2c4141b75705345957357f401adc'
    proof['patch'] = {'path': str(PATCH), 'bytes': len(patch), 'sha256': sha(patch)}
    run('independent-apply-check', ['git', '-C', str(COPY), 'apply', '--check', str(PATCH)], REPO)
    run('independent-apply-index', ['git', '-C', str(COPY), 'apply', '--index', str(PATCH)], REPO)
    assert run('source-tree-after', ['git', 'write-tree'], REPO).decode().strip() == TREE
    assert run('check-tree-after', ['git', '-C', str(COPY), 'write-tree'], REPO).decode().strip() == TREE
    si = run('source-index-paths', ['git', 'ls-files', '--stage', '-z'], REPO)
    ci = run('check-index-paths', ['git', 'ls-files', '--stage', '-z'], COPY)
    assert si == ci, 'full index entries differ'
    entries = []
    for item in si.split(b'\0'):
        if not item: continue
        fields, name = item.split(b'\t', 1)
        mode, oid, stage = fields.decode().split()
        assert stage == '0'
        entries.append((mode, oid, name.decode()))
    stream = run('source-index-blob-bytes', ['git', 'cat-file', '--batch'], REPO, data=('\n'.join(e[1] for e in entries) + '\n').encode())
    offset = 0
    compared = []
    for mode, oid, p in entries:
        end = stream.index(b'\n', offset)
        actual_oid, kind, size = stream[offset:end].decode().split()
        assert actual_oid == oid and kind == 'blob'
        offset = end + 1
        blob = stream[offset:offset + int(size)]
        offset += int(size)
        assert stream[offset:offset + 1] == b'\n'
        offset += 1
        assert readwork(REPO, p, mode) == blob, ('source index/work bytes', p)
        assert readwork(COPY, p, mode) == blob, ('copy index/work bytes', p)
        compared.append({'path': p, 'mode': mode, 'git_blob': oid, 'bytes': len(blob), 'sha256': sha(blob), 'changed': p in changed})
    assert offset == len(stream)
    proof['inspection']['full_index_work_copy_byte_comparison'] = compared
    proof['inspection']['changed_file_count'] = sum(f['changed'] for f in compared)
    proof['inspection']['tracked_file_count'] = len(compared)
    assert run('source-head-after', ['git', 'rev-parse', 'HEAD'], REPO).decode().strip() == BASE
    assert run('source-status-after', ['git', 'status', '--porcelain=v1', '-z', '--untracked-files=all'], REPO) == source_status
    assert run('check-unstaged-after', ['git', 'diff', '--name-only', '-z'], COPY) == b''
    proof['check_copy_after_application'] = 'staged candidate; not clean'
    proof['process_fact'] = 'All recorded subprocess.run calls completed and were reaped; no server/background process was started by this delivery agent.'
    proof['complete'] = True
    proof['ended_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    save()
    print(json.dumps({'complete': True, 'proof': str(OWN / 'proof.json'), 'proof_sha256': sha((OWN / 'proof.json').read_bytes()), 'patch': proof['patch'], 'changed_files': proof['inspection']['changed_file_count'], 'tracked_files': len(compared), 'command_exits': [c['exit_code'] for c in proof['commands']]}, ensure_ascii=False, indent=2))
except Exception:
    proof['failure'] = traceback.format_exc()
    proof['ended_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    save()
    print(proof['failure'], file=sys.stderr)
    sys.exit(1)
