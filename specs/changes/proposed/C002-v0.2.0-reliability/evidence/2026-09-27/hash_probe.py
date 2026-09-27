import hashlib, json, os, shutil, subprocess, tempfile
from pathlib import Path
repo=Path(os.environ.get('SHELTIE_REVIEW_REPO',str(Path(__file__).resolve().parents[6])))
binary=os.environ['SHELTIE_REVIEW_BIN']
root=Path(tempfile.mkdtemp(prefix='sheltie-hash-probe-'))
rows=[]
for label,files in [('one',{'za':b'zb\0X'}),('two',{'za':b'','zb':b'X'})]:
 source=root/label;shutil.copytree(repo/'examples/two-step',source)
 for name,data in files.items():(source/name).write_bytes(data)
 home=root/(label+'-home');home.mkdir()
 p=subprocess.run([binary,'--home',str(home),'--json','workbook','add',str(source)],capture_output=True,text=True)
 blob=b''.join(str(f.relative_to(source)).encode()+b'\0'+f.read_bytes() for f in sorted(source.rglob('*')) if f.is_file())
 rows.append({'label':label,'exit':p.returncode,'stdout':json.loads(p.stdout),'independent_single_sha256':hashlib.sha256(blob).hexdigest()})
(root/'raw.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2));print(json.dumps(rows,ensure_ascii=False,indent=2));print(root)
