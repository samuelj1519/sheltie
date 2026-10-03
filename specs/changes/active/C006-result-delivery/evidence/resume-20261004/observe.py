import base64
import datetime
import hashlib
import json
import os
import signal
import sqlite3
import stat
import subprocess
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[5]
FREEZE = json.loads((EVIDENCE / 'freeze.json').read_text())


def utc():
    return datetime.datetime.now(datetime.timezone.utc)


def check_frozen():
    for name, value in FREEZE['relevant_initial_environment'].items():
        if os.environ.get(name) != value:
            raise RuntimeError('Frozen control environment drift: ' + name)
    for name, digest in FREEZE['frozen_source_files_sha256'].items():
        if hashlib.sha256(Path(name).read_bytes()).hexdigest() != digest:
            raise RuntimeError('Frozen source drift: ' + name)
    for name, digest in FREEZE['complete192_inputs_sha256'].items():
        path = ROOT / name
        body = os.fsencode(os.readlink(path)) if path.is_symlink() else path.read_bytes()
        if hashlib.sha256(body).hexdigest() != digest:
            raise RuntimeError('Frozen product input drift: ' + name)


def snapshot():
    home = Path(FREEZE['home'])
    with sqlite3.connect('file:' + str(home / 'store.db') + '?mode=ro', uri=True) as db:
        db.execute('BEGIN')
        rows = {
            table: db.execute('SELECT * FROM ' + table + ' ORDER BY rowid').fetchall()
            for table in ['workbooks', 'works', 'work_sequence', 'requests', 'audit']
        }
    objects = {}
    for name in ['works', 'workbooks', 'pending', 'tmp']:
        for path in [home / name, *sorted((home / name).rglob('*'))]:
            info = path.lstat()
            objects[str(path.relative_to(home))] = dict(
                dev=info.st_dev,
                ino=info.st_ino,
                mode=info.st_mode,
                nlink=info.st_nlink,
                sha256=hashlib.sha256(path.read_bytes()).hexdigest()
                if stat.S_ISREG(info.st_mode) else None,
            )
    return dict(rows=rows, objects=objects)


def start_run():
    check_frozen()
    statepath = EVIDENCE / 'run-state.json'
    if statepath.exists():
        return json.loads(statepath.read_text())
    start = utc()
    state = dict(
        start_utc=start.isoformat(),
        deadline_utc=(
            start + datetime.timedelta(seconds=FREEZE['budget']['run_s'])
        ).isoformat(),
        calls=[],
        usage=None,
        paid_cost=None,
        human_activity=None,
    )
    with statepath.open('x') as output:
        json.dump(state, output, ensure_ascii=False, indent=2)
        output.write('\n')
    return state


def call(name, argv, observe_home=True):
    check_frozen()
    statepath = EVIDENCE / 'run-state.json'
    state = start_run()
    deadline = datetime.datetime.fromisoformat(state['deadline_utc'])
    remaining = (deadline - utc()).total_seconds()
    if remaining <= 8:
        raise RuntimeError('Continuous deadline lacks cleanup reserve; no command started')
    outputpath = EVIDENCE / (name + '.json')
    if outputpath.exists():
        raise RuntimeError('Existing command evidence retained; no repeated run: ' + name)
    before = snapshot() if observe_home else None
    remaining = (deadline - utc()).total_seconds()
    if remaining <= 8:
        raise RuntimeError('Deadline reached during snapshot; no command started')
    outpath, errpath = (
        EVIDENCE / (name + '.stdout'),
        EVIDENCE / (name + '.stderr'),
    )
    timed_out = False
    started = utc()
    with outpath.open('xb') as out, errpath.open('xb') as err:
        process = subprocess.Popen(
            argv,
            cwd=ROOT,
            stdin=subprocess.DEVNULL,
            stdout=out,
            stderr=err,
            start_new_session=True,
        )
        try:
            code = process.wait(
                timeout=min(FREEZE['budget']['per_command_s'], remaining) - 8
            )
        except subprocess.TimeoutExpired:
            timed_out = True
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            import time
            time.sleep(3)
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            code = process.wait(timeout=3)
    after = snapshot() if observe_home else None
    ended = utc()
    record = dict(
        name=name,
        argv=argv,
        start_utc=started.isoformat(),
        end_utc=ended.isoformat(),
        elapsed_s=(ended - started).total_seconds(),
        exit_code=code,
        timed_out=timed_out,
        owned_group_termination_requested=timed_out,
        before=before,
        after=after,
        home_unchanged=before == after if observe_home else None,
        within_deadline=ended < deadline,
        stdout_base64=base64.b64encode(outpath.read_bytes()).decode('ascii'),
        stderr_base64=base64.b64encode(errpath.read_bytes()).decode('ascii'),
        stdout_sha256=hashlib.sha256(outpath.read_bytes()).hexdigest(),
        stderr_sha256=hashlib.sha256(errpath.read_bytes()).hexdigest(),
        sqlite_carrier_exception='Database/WAL/SHM/lock bytes excluded; all5table rows/business originals included',
    )
    with outputpath.open('x') as output:
        json.dump(record, output, ensure_ascii=False, indent=2)
        output.write('\n')
    state['calls'].append(record)
    statepath.write_text(json.dumps(state, ensure_ascii=False, indent=2) + '\n')
    if (
        timed_out or code != 0 or not record['within_deadline']
        or (observe_home and before != after)
    ):
        raise RuntimeError('Actual command/observation failed; evidence retained: ' + name)
    check_frozen()
    return outpath.read_bytes()
