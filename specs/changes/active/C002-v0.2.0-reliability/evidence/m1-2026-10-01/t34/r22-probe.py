import hashlib,json,sqlite3,subprocess,tempfile,pathlib
binary=pathlib.Path('/private/tmp/sheltie-c002-review-9NZouweW/target/debug/sheltie')
expected='7b31c2a4ea086ceed8608c1ae13a3bb9e1ab390b1d018b33d8ce2345b994ec34'
assert hashlib.sha256(binary.read_bytes()).hexdigest()==expected
source='/Users/shushu/orca/workspaces/sheltie/codex/examples/two-step'
records=[]
for changed in [None,'id','version']:
 with tempfile.TemporaryDirectory(prefix='m1-original-binding-',dir='/private/tmp') as area:
  home=pathlib.Path(area)/'home'
  runs=[]
  def cli(args):
   argv=[str(binary),'--home',str(home),'--json']+args
   p=subprocess.run(argv,text=True,capture_output=True)
   run={'argv':argv,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr};runs.append(run)
   return p,json.loads(p.stdout)
  p,added=cli(['--request-id','setup-add','workbook','add',source]);assert p.returncode==0
  argv=['--request-id','remove-original','workbook','remove','two-step@1.0.0']
  p,original=cli(argv);assert p.returncode==0 and original['ok'] is True
  with sqlite3.connect(home/'store.db') as conn:
   raw=conn.execute('SELECT reply_json FROM requests WHERE request_id=?',('remove-original',)).fetchone()[0]
   snapshot=json.loads(raw)
   sql=None
   if changed:
    snapshot['data'][changed]='other-step' if changed=='id' else '2.0.0'
    sql={'statement':'UPDATE requests SET reply_json=? WHERE request_id=?','parameters':[json.dumps(snapshot,separators=(',',':')),'remove-original']}
    conn.execute(sql['statement'],sql['parameters']);conn.commit()
   def rows():
    return {t:conn.execute(f'SELECT * FROM {t} ORDER BY rowid').fetchall() for t in ['workbooks','works','work_sequence','requests','audit']}
   def files():
    answer={}
    for label in ['workbooks','works','pending']:
     root=home/label
     if not root.exists():continue
     for node in [root]+sorted(root.rglob('*')):
      rel=str(node.relative_to(home));meta=node.lstat()
      answer[rel]={'mode':meta.st_mode,'bytes':node.read_bytes().hex() if node.is_file() else None}
    return answer
   before=rows();before_files=files()
   p,replayed=cli(argv)
   after=rows();after_files=files()
   assert before==after and before_files==after_files
   if changed:
    assert p.returncode!=0 and replayed['error']['code']=='EFFECT_PENDING'
    assert replayed['committed'] is True
    assert replayed['original']['ok'] is True
    assert replayed['original']['data'][changed]!=original['data'][changed]
    verdict='CONFIRMED: semantically drifted snapshot projected as ok=true original'
   else:
    assert p.returncode==0 and replayed['data']['id']==original['data']['id'] and replayed['data']['version']==original['data']['version']
    verdict='CONTROL_PASS'
   records.append({'condition':changed or 'no_change','binary_sha256':expected,'product_candidate':'06c3af3106e6669d788a2a718df01d12e0c51fb9','runs':runs,'sql':sql,'submitted_response':original,'projected_response':replayed,'business_fields_before':original['data'],'business_fields_after':replayed.get('original',{}).get('data',replayed.get('data')),'store_rows_unchanged':before==after,'business_files_unchanged':before_files==after_files,'files_before':before_files,'files_after':after_files,'rows_before':before,'rows_after':after,'verdict':verdict})
p=pathlib.Path('/private/tmp/m1-original-binding-probe.json');p.write_text(json.dumps({'records':records},ensure_ascii=False,indent=2));print(p)
for r in records:print(r['condition'],r['verdict'])
