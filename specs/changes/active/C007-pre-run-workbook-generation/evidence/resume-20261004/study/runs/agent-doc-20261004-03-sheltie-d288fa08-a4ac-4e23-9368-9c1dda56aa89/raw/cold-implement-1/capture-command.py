import sys,json,importlib.util
from pathlib import Path
b=json.loads(Path('/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-03-sheltie-d288fa08-a4ac-4e23-9368-9c1dda56aa89/run-binding.json').read_text())
s=importlib.util.spec_from_file_location("capture",Path(b["study_dir"])/"capture.py"); c=importlib.util.module_from_spec(s);s.loader.exec_module(c)
r,o,e=c.execute(Path(b["raw_dir"])/sys.argv[1],sys.argv[2],sys.argv[3:],b["repo"],120,b["deadline_utc"])
print(json.dumps(r,ensure_ascii=False));print(o.decode());print(e.decode())
