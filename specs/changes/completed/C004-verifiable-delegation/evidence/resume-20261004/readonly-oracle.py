import datetime
import hashlib
import json
import sqlite3
import stat
import subprocess
import sys
from pathlib import Path

e = Path('/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C004-verifiable-delegation/evidence/resume-20261004')
s = json.loads((e / 'run-state.json').read_text())
home = Path(s['home'])
def snapshot():
    with sqlite3.connect('file:' + str(home / 'store.db') + '?mode=ro', uri=True) as db:
        db.execute('BEGIN')
        rows = {t: db.execute('SELECT * FROM ' + t + ' ORDER BY rowid').fetchall() for t in ['workbooks', 'works', 'work_sequence', 'requests', 'audit']}
    objects = {}
    for root in ['works', 'workbooks', 'pending', 'tmp']:
        for p in [home / root, *sorted((home / root).rglob('*'))]:
            st = p.lstat()
            objects[str(p.relative_to(home))] = dict(dev=st.st_dev, ino=st.st_ino, mode=st.st_mode, nlink=st.st_nlink, sha256=hashlib.sha256(p.read_bytes()).hexdigest() if stat.S_ISREG(st.st_mode) else None)
    return dict(rows=rows, objects=objects)
before = snapshot()
calls = []
for op in ['status', 'result']:
    argv = [s['binary'], '--home', s['home'], '--json', 'work', op, s['work_id']]
    start = datetime.datetime.now(datetime.timezone.utc).isoformat()
    r = subprocess.run(argv, capture_output=True, text=True, timeout=30)
    calls.append(dict(argv=argv, start_utc=start, end_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), exit_code=r.returncode, stdout=r.stdout, stderr=r.stderr))
after = snapshot()
out = dict(before=before, calls=calls, after=after, allfive_and_business_objects_unchanged=before == after, sqlite_carrier_policy='database/WAL/SHM/lock inode and bytes are not business-original equality; all five rows are compared', observed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
(e / (sys.argv[1] + '-readonly-oracle.json')).write_text(json.dumps(out, ensure_ascii=False, indent=2) + '\n')
assert before == after
for c in calls:
    assert c['exit_code'] == 0 and json.loads(c['stdout'])['ok']
print(json.dumps(dict(unchanged=True, status=json.loads(calls[0]['stdout']), result=json.loads(calls[1]['stdout'])), ensure_ascii=False))
