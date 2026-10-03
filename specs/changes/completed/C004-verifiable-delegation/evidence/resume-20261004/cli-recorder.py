import datetime
import json
import subprocess
import sys
from pathlib import Path

e = Path('/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C004-verifiable-delegation/evidence/resume-20261004')
s = json.loads((e / 'run-state.json').read_text())
name, *args = sys.argv[1:]
ids = json.loads((e / 'request-ids.json').read_text())
argv = [s['binary'], '--home', s['home'], '--json']
if name in ids:
    argv += ['--request-id', ids[name]]
argv += args
start = datetime.datetime.now(datetime.timezone.utc).isoformat()
r = subprocess.run(argv, capture_output=True, text=True, timeout=30)
call = dict(name=name, argv=argv, start_utc=start, end_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), exit_code=r.returncode, stdout=r.stdout, stderr=r.stderr)
(e / (name + '.json')).write_text(json.dumps(call, ensure_ascii=False, indent=2) + '\n')
s['calls'].append(call)
(e / 'run-state.json').write_text(json.dumps(s, ensure_ascii=False, indent=2) + '\n')
print(r.stdout, end='')
print(r.stderr, end='', file=sys.stderr)
sys.exit(r.returncode)
