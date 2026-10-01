"""Reproduce two review findings using a Cargo-reported CLI executable and fresh homes."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time


parser = argparse.ArgumentParser()
parser.add_argument("--cargo-json", type=Path, required=True)
parser.add_argument("--repo", type=Path, required=True)
options = parser.parse_args()
artifacts = [json.loads(line) for line in options.cargo_json.read_text().splitlines()]
executables = {
    item["executable"]
    for item in artifacts
    if item.get("reason") == "compiler-artifact"
    and item.get("target", {}).get("name") == "sheltie"
    and "bin" in item["target"]["kind"]
    and item.get("executable")
}
assert len(executables) == 1, executables
binary = executables.pop()
repo = options.repo.resolve()
scratch = Path(tempfile.mkdtemp(prefix="sheltie-c002-review-probes-", dir="/private/tmp"))
events = []
environment = os.environ.copy()
for key in list(environment):
    if key == "SHELTIE_FAILPOINT" or key.startswith("SHELTIE_TEST_"):
        del environment[key]


def call(home, *arguments):
    argv = [binary, "--home", str(home), "--json", *arguments]
    output = subprocess.run(argv, env=environment, capture_output=True, text=True, timeout=60)
    event = {
        "argv": argv,
        "exit": output.returncode,
        "stdout": output.stdout,
        "stderr": output.stderr,
    }
    events.append(event)
    return output.returncode, json.loads(output.stdout)


def ok(home, *arguments):
    code, reply = call(home, *arguments)
    assert code == 0 and reply["ok"], events[-1]
    return reply


# Only the child created by this probe is killed, at its exact initialization checkpoint.
home = scratch / "initialization-home"
sync = scratch / "initialization-sync"
sync.mkdir()
child_environment = environment | {
    "SHELTIE_TEST_RENDEZVOUS_NAME": "write_session_after_store_create",
    "SHELTIE_TEST_RENDEZVOUS_ID": str(home),
    "SHELTIE_TEST_RENDEZVOUS_DIR": str(sync),
}
arguments = ["--request-id", "init-kill", "workbook", "add", str(repo / "examples/two-step")]
argv = [binary, "--home", str(home), "--json", *arguments]
child = subprocess.Popen(argv, env=child_environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
try:
    deadline = time.monotonic() + 60
    while not (sync / "reached").exists():
        assert child.poll() is None, "initializer exited before checkpoint"
        assert time.monotonic() < deadline, "initializer checkpoint timed out"
        time.sleep(0.01)
    assert (sync / "reached").read_text() == "write_session_after_store_create"
    assert (home / "store.db").stat().st_size == 0
    child.kill()
    stdout, stderr = child.communicate(timeout=60)
finally:
    if child.poll() is None:
        child.kill()
        child.wait(timeout=60)
events.append({"argv": argv, "exit": child.returncode, "stdout": stdout.decode(), "stderr": stderr.decode()})
code, reply = call(home, *arguments)
assert child.returncode == -9 and code == 1
assert reply["error"]["code"] == "STORE_SCHEMA_MISMATCH"
assert (home / "store.db").stat().st_size == 0

# Change exactly one persisted field in the probe's own Work after a valid approval.
import sqlite3

home = scratch / "gate-home"
ok(home, "workbook", "add", str(repo / "examples/gated-release"))
work = ok(home, "work", "start", "--workbook", "gated-release", "--flow", "default", "--input", "version=vtest")["data"]["work_id"]
begun = ok(home, "attempt", "begin", work, "--node", "notes")["data"]
Path(begun["outputs"]["notes"]).write_text("release notes\n")
ok(home, "attempt", "submit", work, "--attempt", begun["attempt"], "--summary", "done")
ok(home, "gate", "approve", work, "--node", "notes")
ok(home, "work", "status", work)
with sqlite3.connect(home / "store.db") as connection:
    state = json.loads(connection.execute("SELECT state_json FROM works WHERE work_id=?", (work,)).fetchone()[0])
    assert len(state["approvals"]) == 1
    events.append({"changed_field": "state_json.approvals", "before": state["approvals"], "after": []})
    state["approvals"] = []
    connection.execute("UPDATE works SET state_json=? WHERE work_id=?", (json.dumps(state), work))
ok(home, "work", "status", work)
ok(home, "attempt", "begin", work, "--node", "archive")
evidence = scratch / "evidence.json"
evidence.write_text(json.dumps(events, ensure_ascii=False, indent=2) + "\n")
print(json.dumps({"result": "both defects reproduced; this is not a product PASS", "evidence": str(evidence)}, ensure_ascii=False))
