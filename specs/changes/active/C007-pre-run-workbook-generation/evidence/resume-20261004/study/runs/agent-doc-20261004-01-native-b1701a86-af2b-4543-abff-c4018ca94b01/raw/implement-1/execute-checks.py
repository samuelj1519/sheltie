import datetime, hashlib, importlib.util, json, os, subprocess
from pathlib import Path
run = Path("/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01")
binding = json.loads((run / "run-binding.json").read_text())
repo = Path(binding["repo"])
raw = run / "raw" / "implement-1"
module_spec = importlib.util.spec_from_file_location("capture", Path(binding["study_dir"]) / "capture.py")
capture = importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(capture)
def save(name, data):
    with (raw / name).open("x") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")
def git(*args):
    return subprocess.check_output(["git", *args], cwd=repo, text=True).strip()
allowed = binding["allowed_repo_files"]
assert git("rev-parse", "HEAD") == "351feb7ac22c21317a686693b732d5ae0c4b4bcc"
assert git("rev-parse", "HEAD^{tree}") == "80ea3046b1b3575f07e68313444c051ee7c5b7db"
save("pre-stage.json", {"observed_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(), "head":git("rev-parse","HEAD"), "initial_tree":git("rev-parse","HEAD^{tree}"), "status":git("status","--short"), "allowed_repo_files":allowed})
record, out, err = capture.execute(raw, "stage", ["git","add","--",*allowed], repo, 120, binding["deadline_utc"])
assert record["exit_code"] == 0
assert set(git("diff","--cached","--name-only").splitlines()) == set(allowed)
assert not git("diff","--name-only")
assert not git("ls-files","--others","--exclude-standard")
identity = {"head_before_commit":git("rev-parse","HEAD"), "index_tree":git("write-tree"), "allow_files":{p:{"sha256":hashlib.sha256((repo/p).read_bytes()).hexdigest(), "bytes":(repo/p).stat().st_size} for p in allowed}, "relevant_environment":{k:os.environ.get(k) for k in ["PATH","LANG","LC_ALL","LC_CTYPE","SHELL","GIT_INDEX_FILE","GIT_DIR","GIT_WORK_TREE","GIT_CONFIG_COUNT"]}}
save("candidate-before-checks.json", identity)
checks = [["scripts/check-docs.sh","README.md","specs/guides/source-quick-start.md"],["git","diff","--cached","--check"]]
bindings=[]
for label, argv in zip(["check-docs", "check-cached-diff"], checks):
    record, out, err = capture.execute(raw, label, argv, repo, 120, binding["deadline_utc"])
    bindings.append({"record":str(raw/(label+".json")), "candidate_index_tree":identity["index_tree"], "head_before_commit":identity["head_before_commit"], "exit_code":record["exit_code"]})
    print(json.dumps({"label":label,"exit_code":record["exit_code"],"start_utc":record["start_utc"],"end_utc":record["end_utc"],"stdout":out.decode(),"stderr":err.decode()},ensure_ascii=False))
    if record["exit_code"] != 0:
        save("checks-bindings.json", bindings)
        raise SystemExit("Required check failed; retained original and stopped")
assert git("write-tree") == identity["index_tree"]
assert all(hashlib.sha256((repo/p).read_bytes()).hexdigest()==identity["allow_files"][p]["sha256"] for p in allowed)
save("checks-bindings.json", bindings)
print(json.dumps(identity,ensure_ascii=False))
