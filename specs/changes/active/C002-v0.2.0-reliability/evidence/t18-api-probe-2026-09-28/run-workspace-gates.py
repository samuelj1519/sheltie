from datetime import datetime, timezone
from pathlib import Path
import os
import platform
import subprocess
import tempfile
import time

evidence = Path(__file__).resolve().parent
repo = Path(__file__).resolve().parents[6]
host_key = f"{platform.system().lower()}-{platform.machine()}"
target = Path(tempfile.gettempdir()) / "sheltie-c002-target"
env = os.environ.copy()
env["CARGO_TARGET_DIR"] = str(target)
env["RUSTC_WRAPPER"] = ""
commands = [
    ("diff-check", ["git", "diff", "--check"]),
    ("docs", ["scripts/check-docs.sh"]),
    ("specs", ["scripts/check-specs.sh"]),
    ("tests-map", ["scripts/check-tests.sh"]),
    ("fmt", ["cargo", "fmt", "--all", "--", "--check"]),
    ("check", ["cargo", "check", "--all-targets", "--all-features"]),
    ("clippy", ["cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"]),
    ("nextest", ["cargo", "nextest", "run", "--all-features", "--no-tests=pass"]),
]
run_id = f"t18-workspace-gates-{host_key}-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
records = [f"run_id={run_id}", f"host={platform.system()} {platform.machine()}", f"CARGO_TARGET_DIR={target}", "RUSTC_WRAPPER=<empty>"]
failed = False
for name, command in commands:
    started = time.monotonic()
    result = subprocess.run(command, cwd=repo, env=env, capture_output=True, text=True)
    elapsed = time.monotonic() - started
    (evidence / f"{name}.stdout").write_text(result.stdout)
    (evidence / f"{name}.stderr").write_text(result.stderr)
    records.extend([
        f"[{name}] command={' '.join(command)}",
        f"[{name}] elapsed_seconds={elapsed:.3f}",
        f"[{name}] exit_code={result.returncode}",
    ])
    print(f"{name}: exit={result.returncode}, elapsed={elapsed:.3f}s", flush=True)
    failed = failed or result.returncode != 0
(evidence / f"workspace-gates-{host_key}.txt").write_text("\n".join(records) + "\n")
raise SystemExit(1 if failed else 0)
