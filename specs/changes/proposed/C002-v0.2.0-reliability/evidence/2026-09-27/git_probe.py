import os, tempfile
from pathlib import Path
REVIEW_REPO = Path(os.environ.get("SHELTIE_REVIEW_REPO", str(Path(__file__).resolve().parents[6])))
REVIEW_BIN = os.environ.get("SHELTIE_REVIEW_BIN", "")
import tempfile,pathlib,subprocess,json
root=pathlib.Path(tempfile.mkdtemp(prefix='sheltie-git-probe-',dir='/tmp')).resolve()
def git(*a): return subprocess.check_output(['git','-C',str(root),*a],text=True).strip()
git('init','-q');git('config','user.name','Synthetic Probe');git('config','user.email','probe@example.invalid')
(root/'a.rs').write_text('fn a() { todo!() }\n');(root/'b.rs').write_text('fn b() { todo!() }\n');git('add','.');git('commit','-qm','scaffold');base=git('rev-parse','HEAD')
(root/'a.rs').write_text('fn a() {}\n');git('add','a.rs');git('commit','-qm','task 1');first=git('rev-parse','HEAD')
(root/'b.rs').write_text('fn b() {}\n');git('add','b.rs');git('commit','-qm','task 2');second=git('rev-parse','HEAD')
result={'root':str(root),'scaffold':base,'task1':first,'task2':second,'verify_task2_baseline_to_HEAD':git('diff','--name-only',base+'..HEAD').splitlines(),'actual_task2_changed':git('diff','--name-only','HEAD^..HEAD').splitlines(),'final_review_after_replan_at_task1':git('diff','--name-only',first+'..HEAD').splitlines(),'true_whole_change':git('diff','--name-only',base+'..HEAD').splitlines()}
(root/'probe-result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
