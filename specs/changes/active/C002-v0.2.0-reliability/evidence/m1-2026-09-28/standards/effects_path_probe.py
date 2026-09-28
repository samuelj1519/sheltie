import json, subprocess, tempfile, pathlib, sqlite3, hashlib
BIN='/private/tmp/sheltie-m1-review-target/debug/sheltie';root=pathlib.Path(tempfile.mkdtemp(prefix='sheltie-effects-path-',dir='/private/tmp'));home=root/'home';outside=root/'outside';outside.mkdir()
base=[BIN,'--json','--home',str(home),'workbook','add','/Users/shushu/orca/workspaces/sheltie/codex/examples/two-step']
def call(cmd):
    p=subprocess.run(cmd,capture_output=True,text=True);return {'command':cmd,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr}
first=call(base);rid=json.loads(first['stdout'])['request_id'];payload='unconfined-persisted-effect\n';effect={'kind':'write_file','path':'../outside/payload.md','sha256':hashlib.sha256(payload.encode()).hexdigest(),'content':payload}
with sqlite3.connect(home/'store.db') as c: c.execute('UPDATE requests SET effects_json=?, published=0 WHERE request_id=?',(json.dumps([effect]),rid))
replay=call(base);target=outside/'payload.md'
print(json.dumps({'root':str(root),'first':first,'injected_effect':effect,'replay':replay,'root_external_target_exists':target.exists(),'root_external_bytes':target.read_text() if target.exists() else None},ensure_ascii=False,indent=2))
