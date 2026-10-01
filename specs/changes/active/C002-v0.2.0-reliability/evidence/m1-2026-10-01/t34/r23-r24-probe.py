import hashlib,json,pathlib,sqlite3,subprocess,tempfile
binary=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex/target/m1-validation/source/target/debug/sheltie')
sha='90ca2d18fbcd912cd085e08792b7a70bab497d4e406ac56d86b9c947282ede0d'
assert hashlib.sha256(binary.read_bytes()).hexdigest()==sha
source=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex/examples/two-step')
records=[]
for case in ['fail_succeeded','fail_cancelled','begin_requires']:
 with tempfile.TemporaryDirectory(prefix='m1-snapshot-semantics-',dir='/private/tmp') as area:
  home=pathlib.Path(area)/'home';runs=[]
  def cli(args):
   argv=[str(binary),'--home',str(home),'--json']+args
   p=subprocess.run(argv,capture_output=True,text=True)
   runs.append({'argv':argv,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
   return p,json.loads(p.stdout)
  p,added=cli(['--request-id','add','workbook','add',str(source)]);assert p.returncode==0
  p,start=cli(['--request-id','start','work','start','--workbook','two-step','--flow','default','--input','topic=x']);assert p.returncode==0
  work=start['data']['work_id']
  begin_args=['--request-id','begin','attempt','begin',work,'--node','outline']
  p,begin=cli(begin_args);assert p.returncode==0
  if case.startswith('fail_'):
   attempt=begin['data']['attempt'];rid='fail';args=['--request-id',rid,'attempt','fail',work,'--attempt',attempt,'--reason','controlled failure']
   p,committed=cli(args);assert p.returncode==0
  else:rid='begin';args=begin_args;committed=begin
  p,control=cli(args);assert p.returncode==0
  with sqlite3.connect(home/'store.db') as db:
   raw=db.execute('SELECT reply_json FROM requests WHERE request_id=?',(rid,)).fetchone()[0];snapshot=json.loads(raw)
   if case.startswith('fail_'):
    replacement={'kind':case[len('fail_'):]};snapshot['data']['work_status']=replacement
    logical_change='only data.work_status'
   else:
    import tomllib
    flow=tomllib.loads((source/'flows/default.toml').read_text());node=next(n for n in flow['nodes'] if n['id']=='outline')
    assert node.get('requires',[])==[] and committed['data']['requires']==[]
    replacement=[{'kind':'mcp','name':'extra-resource','version':None,'digest':None,'source':None}]
    assert snapshot['reply']['requires']==[] and snapshot['data']['requires']==[]
    snapshot['reply']['requires']=replacement;snapshot['data']['requires']=replacement
    logical_change='one requires fact at matching Reply/data locations, internally consistent'
   sql={'statement':'UPDATE requests SET reply_json=? WHERE request_id=?','parameters':[json.dumps(snapshot,separators=(',',':')),rid]}
   db.execute(sql['statement'],sql['parameters']);db.commit()
   def rows():return {t:db.execute(f'SELECT * FROM {t} ORDER BY rowid').fetchall() for t in ['workbooks','works','work_sequence','requests','audit']}
   def files():
    d={}
    for k in ['workbooks','works','pending']:
     root=home/k
     if not root.exists():continue
     for path in [root]+sorted(root.rglob('*')):d[str(path.relative_to(home))]={'mode':path.lstat().st_mode,'bytes':path.read_bytes().hex() if path.is_file() else None}
    return d
   before=rows();before_files=files();p,result=cli(args);after=rows();after_files=files()
   assert before==after and before_files==after_files
   assert p.returncode==0 and result['ok'] is True and result['data']['replayed'] is True
   field='work_status' if case.startswith('fail_') else 'requires'
   assert result['data'][field]==replacement and result['data'][field]!=committed['data'][field]
   records.append({'case':case,'binary_sha256':sha,'product_candidate':'06c3af3106e6669d788a2a718df01d12e0c51fb9','runs':runs,'submitted_response':committed,'legal_control_response':control,'logical_changed_condition':logical_change,'sql':sql,'replayed_response':result,'declaration_oracle':{'outline_requires':[]} if case=='begin_requires' else {'command':'FailAttempt','impossible_work_status':['succeeded','cancelled']},'rows_before':before,'rows_after':after,'files_before':before_files,'files_after':after_files,'store_rows_unchanged':before==after,'business_files_unchanged':before_files==after_files,'verdict':'CONFIRMED non-mutated successful replay returns semantically impossible/drifted snapshot field'})
path=pathlib.Path('/private/tmp/m1-snapshot-semantics-probe.json');path.write_text(json.dumps({'records':records},ensure_ascii=False,indent=2));print(path)
for r in records:print(r['case'],r['verdict'])
