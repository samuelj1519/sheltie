import hashlib, json, os, subprocess, sys, time
from pathlib import Path
root=Path(__file__).resolve().parents[7]
name, cwd, *command=sys.argv[1:]
out=Path(__file__).parent
files=['tools/workbook-editor/public/model.mjs','tools/workbook-editor/public/app.mjs','tools/workbook-editor/test/model.test.mjs']
identity={p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in files}
started=time.time()
r=subprocess.run(command,cwd=cwd,env=os.environ,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
(out/(name+'.stdout')).write_bytes(r.stdout)
(out/(name+'.stderr')).write_bytes(r.stderr)
record={'command':command,'cwd':cwd,'env':{'SHELTIE_EDITOR_ENGINE':os.environ.get('SHELTIE_EDITOR_ENGINE')},'source_sha256':identity,'started_epoch':started,'finished_epoch':time.time(),'exit_code':r.returncode}
(out/(name+'.json')).write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(record,ensure_ascii=False))
print(r.stdout.decode(errors='replace')[-1800:])
sys.exit(r.returncode)
