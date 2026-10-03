import datetime
import hashlib
import json
import shutil
import sqlite3
from pathlib import Path

e = Path('/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C004-verifiable-delegation/evidence/resume-20261004')
s = json.loads((e / 'run-state.json').read_text())
o = json.loads((e / 'final-readonly-oracle.json').read_text())
result = json.loads(o['calls'][1]['stdout'])['data']
assert result['final'] and result['status'] == {'kind': 'succeeded'} and not result['effects_pending']
assert [a['key'] for a in result['artifacts']] == ['change', 'delivery', 'review']
with sqlite3.connect('file:' + s['home'] + '/store.db?mode=ro', uri=True) as db:
    state = json.loads(db.execute('SELECT state_json FROM works WHERE work_id=?', (s['work_id'],)).fetchone()[0])
terminal = state['attempts'][-1]
assert terminal['id'] == {'node': 'deliver', 'occurrence': 1, 'number': 0} and terminal['status'] == 'succeeded'
verified = []
for artifact in result['artifacts']:
    key = artifact['key']
    kind = 'output' if key == 'delivery' else 'input'
    reference = terminal['outputs' if kind == 'output' else 'inputs'][key]
    assert artifact['source'] == dict(attempt='deliver#1.0', kind=kind, name=key)
    for field in ['path', 'sha256', 'bytes']:
        assert artifact[field] == reference[field]
    p = Path(artifact['path'])
    body = p.read_bytes()
    assert len(body) == artifact['bytes'] and hashlib.sha256(body).hexdigest() == artifact['sha256']
    if kind == 'input':
        producer = next(a for a in state['attempts'] if a['id']['node'] == ('implement' if key == 'change' else 'review'))
        assert reference == producer['outputs'][key]
    shutil.copyfile(p, e / ('final-' + key + '.md'))
    verified.append(dict(key=key, reference=reference, source=artifact['source'], actual_bytes=len(body), mode=oct(p.stat().st_mode)))
out = dict(verified_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), result=result, exact_three_bound_references=verified, read_queries_preserved_five_tables_and_business_objects=o['allfive_and_business_objects_unchanged'], independent_oracle='Expected keys/terminal/kinds handwritten from frozen code-change Flow; actual bytes/SHA independently calculated; no production projection helper used')
(e / 'final-three-reference-check.json').write_text(json.dumps(out, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(dict(final=True, keys=[a['key'] for a in verified], unchanged=o['allfive_and_business_objects_unchanged']), ensure_ascii=False))
