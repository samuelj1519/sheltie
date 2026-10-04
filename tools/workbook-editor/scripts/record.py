#!/usr/bin/env python3
import datetime, hashlib, json, os, pathlib, subprocess, sys, uuid
root = pathlib.Path(__file__).resolve().parents[3]
evidence = root / 'specs/changes/active/C011-workbook-visual-editor/evidence/implementation'
name = datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S') + '-' + uuid.uuid4().hex[:8]
argv = sys.argv[1:]
paths = subprocess.check_output(['git', 'ls-files', '-co', '--exclude-standard'], cwd=root, text=True).splitlines()
consumers = ('tools/workbook-editor/', 'examples/two-step/', 'examples/code-change/', 'workbooks/spec-dev/')
files = {p: hashlib.sha256((root / p).read_bytes()).hexdigest() for p in sorted(set(paths)) if p.startswith(consumers) and (root / p).is_file()}
engine = os.environ.get('SHELTIE_EDITOR_ENGINE')
dependency_files = [p for p in (root / 'tools/workbook-editor/node_modules/smol-toml/dist').glob('*.js')] + [root / 'tools/workbook-editor/node_modules/fflate/esm/index.mjs', root / 'tools/workbook-editor/node_modules/smol-toml/package.json', root / 'tools/workbook-editor/node_modules/fflate/package.json']
dependencies = {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(dependency_files)}
manifest = {'dependencies_sha256': dependencies, 'format': 'editor-check-inputs/v1', 'files_sha256': files, 'node': subprocess.check_output(['node', '--version'], text=True).strip(), 'engine': {'path': engine, 'sha256': hashlib.sha256(pathlib.Path(engine).read_bytes()).hexdigest()} if engine else None}
encoded = (json.dumps(manifest, ensure_ascii=False, sort_keys=True, indent=2) + '\n').encode()
manifest_sha = hashlib.sha256(encoded).hexdigest()
manifest_path = evidence / ('inputs-' + manifest_sha + '.json')
if manifest_path.exists():
    assert manifest_path.read_bytes() == encoded
else:
    with manifest_path.open('xb') as f: f.write(encoded)
start = datetime.datetime.now(datetime.timezone.utc).isoformat()
result = subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
(evidence / (name + '.stdout')).write_bytes(result.stdout)
(evidence / (name + '.stderr')).write_bytes(result.stderr)
record = {'argv': argv, 'cwd': os.getcwd(), 'started': start, 'ended': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'exit_code': result.returncode, 'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(), 'inputs_manifest': str(manifest_path.relative_to(root)), 'inputs_manifest_sha256': manifest_sha, 'environment': {k:v for k,v in os.environ.items() if k in ['SHELTIE_EDITOR_ENGINE', 'npm_config_cache', 'PATH', 'NODE_OPTIONS']}, 'stdout': name + '.stdout', 'stderr': name + '.stderr'}
(evidence / (name + '.json')).write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n')
sys.stdout.buffer.write(result.stdout)
sys.stderr.buffer.write(result.stderr)
print('\nRECORD ' + str(evidence / (name + '.json')))
sys.exit(result.returncode)
