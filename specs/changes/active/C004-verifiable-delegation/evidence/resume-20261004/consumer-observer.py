import datetime
import base64
import hashlib
import json
import sqlite3
import stat
import subprocess
import sys
from pathlib import Path

evidence = Path(__file__).resolve().parent
freeze = json.loads((evidence / 'consumer-freeze.json').read_text())
home = Path(freeze['home'])
operation = sys.argv[1] if len(sys.argv) == 2 else ''
if operation not in ['status', 'result']:
    raise SystemExit('Only status/result are permitted')
now = datetime.datetime.now(datetime.timezone.utc)
if now >= datetime.datetime.fromisoformat(freeze['deadline_utc']):
    raise SystemExit('Trial deadline reached; no query executed')
for name, digest in freeze['frozen_files_sha256'].items():
    if hashlib.sha256(Path(name).read_bytes()).hexdigest() != digest:
        raise SystemExit('Frozen input drift; no query executed: ' + name)
outpath = evidence / ('consumer-' + operation + '-observation.json')
if outpath.exists():
    raise SystemExit('Existing observation retained; no rerun')

def snapshot():
    with sqlite3.connect('file:' + str(home / 'store.db') + '?mode=ro', uri=True) as db:
        db.execute('BEGIN')
        rows = {t: db.execute('SELECT * FROM ' + t + ' ORDER BY rowid').fetchall() for t in ['workbooks', 'works', 'work_sequence', 'requests', 'audit']}
    objects = {}
    for name in ['works', 'workbooks', 'pending', 'tmp']:
        for path in [home / name, *sorted((home / name).rglob('*'))]:
            info = path.lstat()
            objects[str(path.relative_to(home))] = dict(dev=info.st_dev, ino=info.st_ino, mode=info.st_mode, nlink=info.st_nlink, sha256=hashlib.sha256(path.read_bytes()).hexdigest() if stat.S_ISREG(info.st_mode) else None)
    return dict(rows=rows, objects=objects)

before = snapshot()
argv = [freeze['binary'], '--home', str(home), '--json', 'work', operation, freeze['work_id']]
start = datetime.datetime.now(datetime.timezone.utc).isoformat()
remaining = (datetime.datetime.fromisoformat(freeze['deadline_utc']) - datetime.datetime.now(datetime.timezone.utc)).total_seconds()
if remaining <= 0:
    raise SystemExit('Deadline reached during preflight; no query executed')
try:
    run = subprocess.run(argv, capture_output=True, timeout=min(30, remaining))
except subprocess.TimeoutExpired as error:
    record = dict(argv=argv, start_utc=start, end_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), result='timeout', cli_exit_code=None, observer_exit_code=1, stdout_base64=base64.b64encode(error.stdout or b'').decode('ascii'), stderr_base64=base64.b64encode(error.stderr or b'').decode('ascii'), before=before, after=snapshot())
    with outpath.open('x') as output:
        json.dump(record, output, ensure_ascii=False, indent=2)
        output.write('\n')
    raise SystemExit('Query timeout; partial output and snapshots retained; no rerun')
after = snapshot()
finished = datetime.datetime.now(datetime.timezone.utc)
within_deadline = finished < datetime.datetime.fromisoformat(freeze['deadline_utc'])
observer_exit = run.returncode if before == after and within_deadline else 1
record = dict(argv=argv, start_utc=start, end_utc=finished.isoformat(), cli_exit_code=run.returncode, observer_exit_code=observer_exit, stdout=run.stdout.decode('utf-8'), stderr=run.stderr.decode('utf-8'), stdout_base64=base64.b64encode(run.stdout).decode('ascii'), stderr_base64=base64.b64encode(run.stderr).decode('ascii'), before=before, after=after, unchanged=before == after, within_deadline=within_deadline, carrier_exception='SQLite database/WAL/SHM/lock bytes excluded; all five rows and business originals included')
with outpath.open('x') as output:
    json.dump(record, output, ensure_ascii=False, indent=2)
    output.write('\n')
sys.stdout.buffer.write(run.stdout)
sys.stderr.buffer.write(run.stderr)
if before != after:
    raise SystemExit('Read-only observation failed; original CLI output retained')
if not within_deadline:
    raise SystemExit('Deadline reached; original CLI result retained but trial not successful')
raise SystemExit(run.returncode)
