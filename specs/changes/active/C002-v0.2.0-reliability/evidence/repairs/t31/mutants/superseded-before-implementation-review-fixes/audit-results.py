import collections
import hashlib
import json
import re
import tarfile
from pathlib import Path

base = Path(__file__).resolve().parent
candidate = "da2bd13979df88dd987596d7ed726ef99bb89651"
progress = json.loads((base / "progress.json").read_text())
allowed = {"CaughtMutant", "MissedMutant", "Timeout", "Unviable"}
class _NoArchive:
    def __enter__(self):
        return None

    def __exit__(self, *_):
        pass

report = []
for recorded in progress:
    directory = base / recorded["name"]
    metadata = json.loads((directory / "metadata.json").read_text())
    outcomes = json.loads((directory / "raw/outcomes.json").read_text())
    inventory = json.loads((directory / "raw/mutants.json").read_text())
    rows = [row for row in outcomes["outcomes"] if row["scenario"] != "Baseline"]
    caught_causes = collections.Counter()
    baseline = [row for row in outcomes["outcomes"] if row["scenario"] == "Baseline"]
    names = [row["scenario"]["Mutant"]["name"] for row in rows]
    expected = {item["name"] for item in inventory}
    counts = dict(collections.Counter(row["summary"] for row in rows))
    assert metadata == recorded
    assert metadata["candidate"] == candidate
    assert metadata["exit"] in (0, 2)
    assert len(baseline) == 1 and baseline[0]["summary"] == "Success"
    assert all(phase["process_status"] == "Success" for phase in baseline[0]["phase_results"])
    assert len(rows) == len(inventory) == metadata["processed"] == metadata["listed"]
    assert len(set(names)) == len(names) and set(names) == expected
    assert set(counts) <= allowed and metadata["counts"] == counts
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    manifest = directory / "raw-manifest.json"
    if manifest.exists():
        original = json.loads(manifest.read_text())
        assert original["outcomes.json"] == sha(directory / "raw/outcomes.json")
        assert original["mutants.json"] == sha(directory / "raw/mutants.json")
    archive = directory / "raw.tar.gz"
    with tarfile.open(archive) if archive.exists() else _NoArchive() as tar:
        for row in rows:
            phases = {phase["phase"]: phase["process_status"] for phase in row["phase_results"]}
            if row["summary"] == "CaughtMutant":
                assert phases.get("Build") == "Success"
                assert phases.get("Test") == {"Failure": 100}
                location = row["log_path"]
                log = (
                    tar.extractfile("raw/" + location).read()
                    if tar is not None
                    else (directory / "raw" / location).read_bytes()
                )
                assert b"LEAK [" not in log, row["scenario"]
                if re.search(rb"(?m)^\s*FAIL \[", log):
                    caught_causes["test_assertion"] += 1
                elif (
                    (
                        row["scenario"]["Mutant"]["function"]["function_name"],
                        row["scenario"]["Mutant"]["span"]["start"]["line"],
                    ) in {("sync_dir_tree", 1469), ("ExternalReadTree::collect", 1903),
                        ("remove_at", 2423), ("preflight_remove_at", 2488),
                        ("make_directories_writable", 2969), ("set_dir_tree_mode", 3007)}
                    and b"SIGABRT [" in log
                    and b"stack overflow" in log
                ):
                    caught_causes["dot_entry_guard_stack_overflow"] += 1
                else:
                    raise AssertionError(f"caught without a relevant test failure: {row['scenario']}")
            elif row["summary"] == "MissedMutant":
                assert phases.get("Build") == phases.get("Test") == "Success"
            elif row["summary"] == "Unviable":
                assert phases.get("Build") == {"Failure": 101}
            else:
                assert row["summary"] == "Timeout"
    report.append({"stage": recorded["name"], "listed": len(inventory), "counts": counts, "caught_causes": dict(caught_causes)})

(base / "completed-stage-audit.json").write_text(
    json.dumps({"candidate": candidate, "stages": report}, indent=2) + "\n"
)
print(f"Audited {len(report)} complete stages without tool-failure catches")


