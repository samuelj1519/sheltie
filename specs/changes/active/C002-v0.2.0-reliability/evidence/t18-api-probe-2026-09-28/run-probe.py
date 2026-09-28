from pathlib import Path
import hashlib
import os
import platform
import shutil
import subprocess
import time
from datetime import datetime, timezone
import tempfile

evidence = Path(__file__).resolve().parent
repo = Path(__file__).resolve().parents[6]
host_key = f"{platform.system().lower()}-{platform.machine()}"
probe = evidence
target = Path(tempfile.gettempdir()) / "sheltie-c002-t18-target"
binary = target / "debug/sheltie-c002-t18-api-probe"
command = [
    "cargo", "+1.85.0", "run", "--offline", "--locked",
    "--manifest-path", str(evidence / "Cargo.toml"),
]
rustc_info = subprocess.run(
    ["rustc", "+1.85.0", "-vV"], capture_output=True, text=True, check=True
).stdout
(evidence / f"rustc-{host_key}.txt").write_text(rustc_info)
env = os.environ.copy()
env["CARGO_TARGET_DIR"] = str(target)
env["RUSTC_WRAPPER"] = ""
started = time.monotonic()
try:
    result = subprocess.run(command, cwd=repo, env=env, capture_output=True, text=True, timeout=15)
    stdout, stderr, exit_code = result.stdout, result.stderr, result.returncode
except subprocess.TimeoutExpired as error:
    stdout = error.stdout.decode() if isinstance(error.stdout, bytes) else (error.stdout or "")
    stderr = error.stderr.decode() if isinstance(error.stderr, bytes) else (error.stderr or "")
    exit_code = 124
elapsed = time.monotonic() - started
run_id = f"t18-{host_key}-rust-1.85.0-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
stdout_path = evidence / f"{host_key}.stdout"
stderr_path = evidence / f"{host_key}.stderr"
manifest_path = evidence / f"run-manifest-{host_key}.txt"
stdout_path.write_text(stdout)
stderr_path.write_text(stderr)
binary_copy = evidence / f"probe-{host_key}"
if binary.exists():
    shutil.copy2(binary, binary_copy)
main_sha = hashlib.sha256((probe / "src/main.rs").read_bytes()).hexdigest()
lock_sha = hashlib.sha256((probe / "Cargo.lock").read_bytes()).hexdigest()
binary_sha = hashlib.sha256(binary.read_bytes()).hexdigest() if binary.exists() else "missing"
rustc_sha = hashlib.sha256(rustc_info.encode()).hexdigest()
manifest_path.write_text(
    f"run_id={run_id}\n"
    f"command={' '.join(command)}\n"
    f"cwd={repo}\n"
    f"CARGO_TARGET_DIR={target}\n"
    f"RUSTC_WRAPPER=<empty>\n"
    f"timeout_seconds=15\n"
    f"host={platform.system()} {platform.machine()}\n"
    f"rustc_sha256={rustc_sha}\n"
    f"source_sha256={main_sha}\n"
    f"lock_sha256={lock_sha}\n"
    f"binary_sha256={binary_sha}\n"
    f"binary_path={binary_copy}\n"
    f"elapsed_seconds={elapsed:.3f}\n"
    f"exit_code={exit_code}\n"
)
print(manifest_path.read_text(), end="")
raise SystemExit(exit_code)
