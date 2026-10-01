import hashlib
import json
import shutil
import tarfile
from pathlib import Path

base = Path(__file__).resolve().parent
for raw in sorted(base.rglob("raw")):
    if not raw.is_dir():
        continue
    metadata = raw.parent / "metadata.json"
    archive = raw.parent / "raw.tar.gz"
    if archive.exists() or not metadata.exists():
        continue
    record = json.loads(metadata.read_text())
    if record.get("listed") != record.get("processed") and not any(
        part.startswith("superseded-") for part in raw.parts
    ):
        continue
    files = sorted(p for p in raw.rglob("*") if p.is_file())
    manifest = {str(p.relative_to(raw)): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
    with tarfile.open(archive, "w:gz") as tar:
        for path in files:
            tar.add(path, arcname=str(Path("raw") / path.relative_to(raw)), recursive=False)
    (raw.parent / "raw-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    for path in list(raw.iterdir()):
        if path.name in ("outcomes.json", "mutants.json"):
            continue
        if path.is_dir():
            shutil.rmtree(path)
        else:
            path.unlink()
    print(raw.parent.name)
