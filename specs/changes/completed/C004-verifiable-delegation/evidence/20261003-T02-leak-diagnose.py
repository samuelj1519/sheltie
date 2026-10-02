import hashlib
import json
import os
from pathlib import Path
import selectors
import subprocess
import time

metadata = json.loads(Path('/private/tmp/sheltie-c004-leak-binaries.json').read_text())
suite = metadata['rust-suites']['sheltie-cli::implementation_repairs']
build = metadata['rust-build-meta']
application = next(binary for binaries in build['non-test-binaries'].values()
                   for binary in binaries if binary['name'] == 'sheltie')
application_path = Path(build['target-directory']) / application['path']
test_name = 'killed_first_store_initializer_allows_the_same_add_request_to_retry'
command = [suite['binary-path'], '--exact', test_name, '--nocapture']
environment = os.environ.copy()
environment['CARGO_BIN_EXE_sheltie'] = str(application_path)
started = time.monotonic()
process = subprocess.Popen(command, cwd=suite['cwd'], env=environment,
                           stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                           stderr=subprocess.PIPE, start_new_session=True)
selector = selectors.DefaultSelector()
buffers = {'stdout': bytearray(), 'stderr': bytearray()}
for stream, handle in [('stdout', process.stdout), ('stderr', process.stderr)]:
    os.set_blocking(handle.fileno(), False)
    selector.register(handle, selectors.EVENT_READ, stream)
exit_time = None
eof_times = {}
snapshots = []
next_snapshot = started
deadline = started + 10
while time.monotonic() < deadline:
    now = time.monotonic()
    if now >= next_snapshot:
        inventory = subprocess.run(['/bin/ps', '-axo', 'pid,ppid,pgid,state,comm'],
                                   capture_output=True, text=True, check=True)
        rows = []
        for line in inventory.stdout.splitlines()[1:]:
            fields = line.split(None, 4)
            if len(fields) == 5 and fields[2] == str(process.pid):
                rows.append({'pid': int(fields[0]), 'ppid': int(fields[1]),
                             'pgid': int(fields[2]), 'state': fields[3],
                             'program': fields[4]})
        snapshots.append({'elapsed_ms': round((now-started)*1000, 3), 'processes': rows})
        next_snapshot = time.monotonic() + 0.01
    if process.poll() is not None and exit_time is None:
        exit_time = time.monotonic()
    for key, _ in selector.select(0.002):
        chunk = os.read(key.fileobj.fileno(), 65536)
        if chunk:
            buffers[key.data].extend(chunk)
        else:
            eof_times[key.data] = time.monotonic()
            selector.unregister(key.fileobj)
            key.fileobj.close()
    if exit_time is not None and not selector.get_map() and time.monotonic() >= exit_time + 0.2:
        break
if process.poll() is None:
    process.kill()
process.wait()
report = {
    'scope': 'One isolated diagnostic; does not replace historical nextest LEAK or its 739-test run.',
    'command': command,
    'test_binary_sha256': hashlib.sha256(Path(suite['binary-path']).read_bytes()).hexdigest(),
    'application': str(application_path),
    'application_sha256': hashlib.sha256(application_path.read_bytes()).hexdigest(),
    'exit_code': process.returncode,
    'observed_exit_ms': None if exit_time is None else round((exit_time-started)*1000, 3),
    'pipe_eof_ms': {key: round((value-started)*1000, 3) for key, value in eof_times.items()},
    'pipe_eof_after_observed_exit_ms': {
        key: None if exit_time is None else round((value-exit_time)*1000, 3)
        for key, value in eof_times.items()
    },
    'streams_still_open_at_end': [key.data for key in selector.get_map().values()],
    'process_group_snapshots': snapshots,
    'stdout': bytes(buffers['stdout']).decode('utf-8', errors='replace'),
    'stderr': bytes(buffers['stderr']).decode('utf-8', errors='replace'),
    'limits': 'New process group and direct libtest launch; finite 10ms sampling can miss short-lived descendants. EOF timestamps are observations, not historical closure times.',
}
destination = Path('/private/tmp/sheltie-c004-leak-isolated-diagnostic.json')
destination.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({key: value for key, value in report.items() if key != 'process_group_snapshots'}, indent=2))
print('observed_programs:', sorted({process['program'] for point in snapshots for process in point['processes']}))
print('last_process_group_snapshot:', snapshots[-1])
