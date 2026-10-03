from pathlib import Path
import subprocess,json,tempfile,hashlib,shutil,os
root=Path.cwd();out=root/'specs/changes/active/C007-pre-run-workbook-generation/experiments/preparation'
book=out.parent/'confirmed-input';assert book.exists()
build=json.loads((out/'binary.json').read_text());engine=Path(build['path']);assert hashlib.sha256(engine.read_bytes()).hexdigest()==build['sha256']
base=Path(tempfile.mkdtemp(prefix='sheltie-c007-preparation-',dir='/private/tmp')).resolve();home=base/'home';bad_home=base/'bad-home';sentinel=base/'sentinel';sentinel.write_bytes(b'preserve unrelated input')
raw=[]
def cli(h,args,code=0):
 p=subprocess.run([str(engine),'--home',str(h),'--json',*map(str,args)],capture_output=True,timeout=30)
 x={'argv':[str(engine),'--home',str(h),'--json',*map(str,args)],'exit':p.returncode,'stdout':p.stdout.decode(),'stderr':p.stderr.decode()};raw.append(x)
 assert p.returncode==code,x
 v=json.loads(p.stdout);assert v['ok']==(code==0),x
 return v
before_source={str(p.relative_to(book)):hashlib.sha256(p.read_bytes()).hexdigest() for p in book.rglob('*') if p.is_file()}
a=cli(home,['workbook','add',book]); ident=a['data']['id']+'@'+a['data']['version']
show=cli(home,['workbook','show',ident]);cli(home,['workbook','verify',ident])
assert show['data']['flows'][0]['start_inputs']==['task','project']
bad=base/'invalid-input';shutil.copytree(book,bad)
f=bad/'workbook.toml';f.write_text(f.read_text()+'\nunknown_preparation_field = true\n')
rejected=cli(bad_home,['workbook','add',bad],1);assert rejected['error']['code']=='WORKBOOK_INVALID',rejected
assert not (bad_home/'workbooks'/a['data']['id']).exists()
assert sentinel.read_bytes()==b'preserve unrelated input'
assert before_source=={str(p.relative_to(book)):hashlib.sha256(p.read_bytes()).hexdigest() for p in book.rglob('*') if p.is_file()}
# Preparation fixture only; this deliberately does not solve a genuine task or create a formal run.
work=cli(home,['work','start','--workbook',ident,'--flow','default','--name','方法机制','--input','task=技术机制fixture，不是真实样本','--input','project=临时机制，不修改真实仓库'])['data']['work_id']
for node in ['implement','review','deliver']:
 d=cli(home,['attempt','begin',work,'--node',node])['data']
 for key,path in d['outputs'].items():Path(path).write_bytes(('mechanism-only '+node+' '+key+'\n').encode())
 if node=='implement':
  s=cli(home,['work','status',work]);assert s['data']['resume']['brief_path']==d['brief_path']
 cli(home,['attempt','submit',work,'--attempt',d['attempt'],'--summary','仅机制fixture，不代表代码或质量'])
result=cli(home,['work','result',work])['data'];assert result['final'] is True
assert [x['key'] for x in result['artifacts']]==['change','checks','delivery','patch','review']
(out/'cli-mechanism.json').write_text(json.dumps({'scope':'technical parser/graph/CLI fixture only, not formal sample/run/quality/reuse/real reopen/cost evidence','source_head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'binary':build,'base':str(base),'installed':a['data'],'valid_add_show_verify':True,'invalid_unknown_field_rejected':True,'sentinel_and_source_unchanged':True,'fixture_graph_completed':True,'selected_five_results':True,'not_run':['three genuine tasks','six paired runs','actual actor/history/first reader','real host close/reopen','natural rework','user acceptance and blind quality','human cost and value'],'human_minutes':None,'usage':None,'raw':raw},ensure_ascii=False,indent=2)+'\n')
print('actual preparation CLI PASS',len(raw),'outer calls',base)
