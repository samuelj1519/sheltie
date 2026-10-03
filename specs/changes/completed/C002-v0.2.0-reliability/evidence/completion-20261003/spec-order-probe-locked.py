import subprocess,json,sqlite3,pathlib,hashlib
root=pathlib.Path('/private/tmp/sheltie-completion-20261003/spec-order-probe-locked')
root.mkdir(exist_ok=True)
binary='/private/tmp/sheltie-completion-20261003/sheltie-current'
source='/Users/shushu/orca/workspaces/sheltie/codex/examples/two-step'
results=[]
for field in ['command','data','effects']:
 home=root/field
 def call(args):
  argv=[binary,'--home',str(home),'--json']+args
  p=subprocess.run(argv,capture_output=True,text=True)
  return {'argv':argv,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr,'json':json.loads(p.stdout)}
 assert call(['workbook','add',source])['exit']==0
 start=call(['work','start','--workbook','two-step','--flow','default','--input','topic=x'])
 work=start['json']['data']['work_id']
 args=['--request-id','spec-order-begin','attempt','begin',work,'--node','outline']
 assert call(args)['exit']==0
 conn=sqlite3.connect(home/'store.db')
 table,column=('audit','command_json') if field=='command' else ('requests','effects_json' if field=='effects' else 'reply_json')
 raw=conn.execute(f"SELECT {column} FROM {table} WHERE request_id='spec-order-begin'").fetchone()[0]
 value=json.loads(raw)
 target=value if field=='command' else (value[0] if field=='effects' else value['data'])
 target['unexpected_contract_field']=True
 conn.execute(f"UPDATE {table} SET {column}=? WHERE request_id='spec-order-begin'",[json.dumps(value)])
 conn.commit()
 def rows():
  return {t:conn.execute('SELECT * FROM '+t).fetchall() for t in ['works','requests','audit','work_sequence']}
 before=rows()
 frozen=home/'works'/work/'workbook'
 retained=frozen.with_name('retained-workbook')
 frozen.rename(retained)
 readonly=call(['work','status',work])
 replay=call(args)
 assert rows()==before
 assert not frozen.exists() and retained.exists()
 results.append({'field':field,'read_control':readonly,'same_request_write_replay':replay,'store_rows_unchanged':True,'frozen_absent':True})
 conn.close()
(root/'results.json').write_text(json.dumps({'binary':binary,'binary_sha256':hashlib.sha256(pathlib.Path(binary).read_bytes()).hexdigest(),'results':results},indent=2,ensure_ascii=False))
for x in results:
 print(x['field'],'read:',x['read_control']['stdout'].strip(),'replay:',x['same_request_write_replay']['stdout'].strip())
