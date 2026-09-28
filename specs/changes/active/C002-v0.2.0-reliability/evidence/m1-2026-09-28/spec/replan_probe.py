import pathlib,tempfile,json,subprocess
BIN='/private/tmp/sheltie-m1-review-target/debug/sheltie';BASE=pathlib.Path(tempfile.mkdtemp(prefix='replan-',dir='/private/tmp/sheltie-m1-review-e1a8126/spec'));HOME=BASE/'home'
def c(*args):
 p=subprocess.run([BIN,'--home',str(HOME),'--json',*args],capture_output=True,text=True);d=json.loads(p.stdout);print(json.dumps({'argv':args,'rc':p.returncode,'reply':d},ensure_ascii=False));assert p.returncode==0;return d
c('workbook','add','/Users/shushu/orca/workspaces/sheltie/codex/workbooks/spec-dev');d=c('work','start','--workbook','spec-dev','--flow','default','--name','replan','--input','request=demo','--input','project='+str(BASE));wid=d['data']['work_id']
def begin(node):return c('attempt','begin',wid,'--node',node)
def submit(b,text):
 for path in b['data']['outputs'].values():pathlib.Path(path).write_text(text)
 return c('attempt','submit',wid,'--attempt',b['data']['attempt'],'--summary','done')
submit(begin('spec'),'spec text');b1=begin('plan');submit(b1,'original baseline: '+('a'*40));submit(begin('plan-review'),'修改方案\n批准的规格: demo\n批准的方案: demo\n');b2=begin('plan');print(json.dumps({'previous_plan_path':b1['data']['outputs']['plan'],'second_plan_inputs':b2['data']['inputs'],'second_brief':pathlib.Path(b2['data']['brief_path']).read_text()},ensure_ascii=False))
