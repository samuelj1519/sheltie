import datetime
import hashlib
import json
import os
import signal
import subprocess
import time
from pathlib import Path


def execute(record_directory, label, argv, cwd, timeout_seconds, deadline_utc=None):
    """Save one actual command; this does not interpret a Flow or a Store."""
    directory = Path(record_directory)
    if not label or not all(
        character.isalnum() or character in '-_' for character in label
    ):
        raise ValueError('Record label must be one nonempty alphanumeric/hyphen/underscore leaf')
    directory.mkdir(parents=True, exist_ok=True)
    prefix = directory / label
    if Path(str(prefix) + '.json').exists():
        raise RuntimeError('Existing command record retained: ' + label)
    limit = timeout_seconds
    if deadline_utc:
        remaining = (
            datetime.datetime.fromisoformat(deadline_utc)
            - datetime.datetime.now(datetime.timezone.utc)
        ).total_seconds()
        limit = min(limit, remaining)
    if limit <= 8:
        raise RuntimeError('Insufficient time for owned command and cleanup')
    start = datetime.datetime.now(datetime.timezone.utc)
    timed_out = False
    stdout_path, stderr_path = (
        Path(str(prefix) + '.stdout'),
        Path(str(prefix) + '.stderr'),
    )
    with stdout_path.open('xb') as out, stderr_path.open('xb') as err:
        child = subprocess.Popen(
            argv,
            cwd=cwd,
            stdin=subprocess.DEVNULL,
            stdout=out,
            stderr=err,
            start_new_session=True,
        )
        try:
            code = child.wait(timeout=limit - 8)
        except subprocess.TimeoutExpired:
            timed_out = True
            try:
                os.killpg(child.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            time.sleep(3)
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            code = child.wait(timeout=3)
    end = datetime.datetime.now(datetime.timezone.utc)
    within_deadline = (
        not deadline_utc or end < datetime.datetime.fromisoformat(deadline_utc)
    )
    record = dict(
        argv=argv,
        cwd=str(cwd),
        start_utc=start.isoformat(),
        end_utc=end.isoformat(),
        elapsed_s=(end - start).total_seconds(),
        exit_code=code,
        timed_out=timed_out,
        owned_group_termination_requested=timed_out,
        within_deadline=within_deadline,
        stdout_path=str(stdout_path),
        stderr_path=str(stderr_path),
        stdout_sha256=hashlib.sha256(stdout_path.read_bytes()).hexdigest(),
        stderr_sha256=hashlib.sha256(stderr_path.read_bytes()).hexdigest(),
        control_environment={
            name: os.environ.get(name)
            for name in [
                'SHELTIE_HOME',
                'SHELTIE_FAILPOINT',
                'SHELTIE_EXPORT_TEST_POINT',
                'SHELTIE_EXPORT_TEST_DIRECTORY',
                'SHELTIE_EXPORT_TEST_SYNC_ERROR',
                'SHELTIE_TEST_RENDEZVOUS_NAME',
                'SHELTIE_TEST_RENDEZVOUS_ID',
                'SHELTIE_TEST_RENDEZVOUS_DIR',
            ]
        },
    )
    with Path(str(prefix) + '.json').open('x') as output:
        json.dump(record, output, ensure_ascii=False, indent=2)
        output.write('\n')
    if timed_out or not within_deadline:
        raise RuntimeError('Timeout/deadline: actual record retained')
    return record, stdout_path.read_bytes(), stderr_path.read_bytes()
