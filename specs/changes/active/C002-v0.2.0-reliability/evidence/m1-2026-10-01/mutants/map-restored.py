import collections,difflib,hashlib,json,pathlib,subprocess
root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex')
old=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/repairs/t31/mutants'
new=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/m1-2026-10-01/mutants'
olditems={m['name']:m for m in json.loads((old/'inventory.json').read_text())}
newitems=json.loads((new/'inventory.json').read_text())
manifest={m['path']:m['sha256'] for m in json.loads((old/'source-input.json').read_text())['source_files']}
deferred=json.loads((old/'user-deferred-ids.json').read_text())['ids']
bykey=collections.defaultdict(list)
for m in newitems:bykey[(m['file'],m['genre'],m['replacement'])].append(m)
cache={};texts={};rows=[]
def fragment(lines,span):
 a=span['start'];b=span['end'];selected=lines[a['line']-1:b['line']]
 if len(selected)==1:return selected[0][a['column']-1:b['column']-1]
 selected[0]=selected[0][a['column']-1:];selected[-1]=selected[-1][:b['column']-1]
 return '\n'.join(x.strip() for x in selected)

for name in deferred:
 m=olditems[name];file=m['file']
 if file not in cache:
  b=subprocess.check_output(['git','show','ca6d92fa:'+file],cwd=root);assert hashlib.sha256(b).hexdigest()==manifest[file],file
  before=b.decode().splitlines();after=(root/file).read_text().splitlines();lines={}
  for block in difflib.SequenceMatcher(None,[x.lstrip() for x in before],[x.lstrip() for x in after],autojunk=False).get_matching_blocks():
   for k in range(block.size):lines[block.a+k+1]=(block.b+k+1, len(after[block.b+k])-len(after[block.b+k].lstrip())-len(before[block.a+k])+len(before[block.a+k].lstrip()))
  cache[file]=lines;texts[file]=(before,after)
 mapping=cache[file].get(m['span']['start']['line']); mapped=mapping[0] if mapping else None; mapped_col=m['span']['start']['column']+(mapping[1] if mapping else 0)
 key=(file,m['genre'],m['replacement'])
 candidates=[n for n in bykey[key] if mapped==n['span']['start']['line'] and n['span']['start']['column']==mapped_col]
 for n in candidates:assert fragment(texts[file][0],m['span'])==fragment(texts[file][1],n['span']),name
 rows.append({'old_id':name,'old_diff_sha256':hashlib.sha256(m['diff'].encode()).hexdigest(),'mapped_line':mapped,'current_ids':[n['name'] for n in candidates],'current_diff_sha256':[hashlib.sha256(n['diff'].encode()).hexdigest() for n in candidates],'mapping':'exact_unchanged_source_span' if len(candidates)==1 else 'requires_review','unresolved_candidates':[n['name'] for n in bykey[key]] if len(candidates)!=1 else []})
record={'old_candidate':'49d3a191aa4c918aab279617fa2bf7d9b3b36a0e','old_product':'ca6d92fa83494feeb9ea55840bfd4bca5f14629c','current_candidate':'95d78e0677f1ffba6abe5ad9c13430ea53957101','current_inventory_sha256':hashlib.sha256((new/'inventory.json').read_bytes()).hexdigest(),'count':len(rows),'scope':'Trace restored old obligations to current inventory; mapping is not an execution result or equivalence proof. Unmatched changed/retired source requires individual review.','rows':rows}
path=pathlib.Path('/private/tmp/m1-restored-269-mapping.json');path.write_text(json.dumps(record,indent=2)+'\n')
print(collections.Counter(r['mapping'] for r in rows))
for r in rows:
 if r['mapping']=='requires_review':print(r['old_id'])
