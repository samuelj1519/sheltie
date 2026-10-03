from pathlib import Path
import subprocess,json,tempfile,hashlib,shutil,os
records=[]
# Executables passed below are validated against actual Cargo JSON, never guessed by the caller.
import sys
engine,exporter=map(Path,sys.argv[1:])
base=Path(tempfile.mkdtemp(prefix='sheltie-c006-guide-',dir='/private/tmp')).resolve()
home=base/'home'; parent=base/'copies'; parent.mkdir();manual=base/'manual';manual.mkdir()
def run(args):
 p=subprocess.run([str(x) for x in args],capture_output=True)
 records.append({'argv':[str(x) for x in args],'exit':p.returncode,'stdout':p.stdout.decode(errors='replace'),'stderr':p.stderr.decode(errors='replace')})
 assert p.returncode==0,records[-1]
 return json.loads(p.stdout)
def cli(*args):return run([engine,'--home',home,'--json',*args])
cli('workbook','add',Path.cwd()/'examples/code-change')
work=cli('work','start','--workbook','code-change','--flow','default','--name','导出手册机制','--input','task=仅核复制机制，不是用户任务','--input','project=临时目录，无真实仓库修改')['data']['work_id']
for node in ['implement','review','deliver']:
 d=cli('attempt','begin',work,'--node',node)['data']
 for key,path in d['outputs'].items():
  # outputs are the current structured declaration; explicit fixture content is not a quality verdict.
  path=path['path'] if isinstance(path,dict) else path
  Path(path).write_bytes(('mechanism-only '+node+' '+key+'\n').encode())
 cli('attempt','submit',work,'--attempt',d['attempt'],'--summary','仅提交机制fixture')
before=cli('work','status',work)
result=cli('work','result',work)['data']
for i,a in enumerate(result['artifacts']):shutil.copyfile(a['path'],manual/str(i))
report=run([exporter,'--sheltie',engine,'--home',home,'--work',work,'--to',parent,'--json'])
assert report['status']=='complete'
target=Path(report['target_path']);manifest=json.loads((target/'manifest.json').read_bytes());assert manifest['result']==result
for i,(a,f) in enumerate(zip(result['artifacts'],manifest['files'],strict=True)):
 data=(target/f['path']).read_bytes(); assert data==(manual/str(i)).read_bytes();assert len(data)==a['bytes'];assert hashlib.sha256(data).hexdigest()==a['sha256']
assert cli('work','status',work)==before
first=target/manifest['files'][0]['path'];first.write_bytes(b'user editable copy')
report2=run([exporter,'--sheltie',engine,'--home',home,'--work',work,'--to',parent,'--json'])
assert report2['target_path']!=str(target) and first.read_bytes()==b'user editable copy'
assert cli('work','status',work)==before
print(json.dumps({'scope':'temporary mechanism only; no real user, quality or cost result','base':str(base),'binary_sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [engine,exporter]},'work':work,'manual_tool_bytes_equal':True,'business_status_equal':True,'rerun_preserves_edits':True,'runs':records,'real_value':{'result':'not_run','minutes':None,'reason':'No real copy purpose, task user or equal-quality paired baseline supplied'}},ensure_ascii=False,indent=2))
