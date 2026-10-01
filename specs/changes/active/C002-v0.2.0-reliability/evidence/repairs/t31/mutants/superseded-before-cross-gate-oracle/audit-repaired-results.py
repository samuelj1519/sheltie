import collections, hashlib, json, pathlib, re
out=pathlib.Path(__file__).resolve().parent
inventory=json.loads((out/'inventory.json').read_text())
closure=json.loads((out/'execution-closure.json').read_text())
expected={r['name'] for r in inventory}
stages={'stage1':{},'stage2':{}}
unviable=[]
for directory in sorted(out.glob('stage*-sheltie-*')):
 meta_file=directory/'metadata.json'
 if not meta_file.exists():continue
 meta=json.loads(meta_file.read_text());raw=directory/'raw'
 assert meta['candidate']==closure['candidate'] and meta['closure_sha256']==closure['sha256']
 for filename,field in [('outcomes.json','outcomes_sha256'),('mutants.json','mutants_sha256')]:
  assert hashlib.sha256((raw/filename).read_bytes()).hexdigest()==meta[field]
 data=json.loads((raw/'outcomes.json').read_text());listed=json.loads((raw/'mutants.json').read_text())
 baseline=[r for r in data['outcomes'] if r['scenario']=='Baseline'];assert len(baseline)==1 and baseline[0]['summary']=='Success'
 rows=[r for r in data['outcomes'] if r['scenario']!='Baseline'];names={r['scenario']['Mutant']['name'] for r in rows}
 assert meta['exit'] in [0,2] and len(rows)==len(names)==len(listed) and names=={r['name'] for r in listed}
 stage=directory.name.split('-')[0]
 for row in rows:
  mutant=row['scenario']['Mutant'];name=mutant['name'];assert name in expected and name not in stages[stage]
  stages[stage][name]={'summary':row['summary'],'package':mutant['package'],'raw_directory':str(directory.relative_to(out))+'/raw','log_path':row['log_path'],'diff_path':row['diff_path']}
  if row['summary']=='Unviable':
   log=(raw/row['log_path']).read_text(errors='replace')
   diagnostics=sorted(set(re.findall(r'error\[(E\d+)\]',log)))
   build=[p for p in row['phase_results'] if p['phase']=='Build']
   assert build and build[-1]['process_status'].get('Failure')==101 and diagnostics, name
   unviable.append({'name':name,'stage':stage,'classification':'unviable - compiler rejects the generated program; not caught/PASS','compiler_diagnostics':diagnostics,'raw':stages[stage][name]})
remaining=[]
for name,row in stages['stage1'].items():
 if row['summary'] in ['MissedMutant','Timeout']:
  final=stages['stage2'].get(name)
  if final is None or final['summary']!='CaughtMutant':remaining.append({'name':name,'stage1':row['summary'],'stage2':final['summary'] if final else 'not_run','raw':final or row})
summary={'candidate':closure['candidate'],'inventory':len(expected),'stage1_processed':len(stages['stage1']),'stage1_missing':sorted(expected-set(stages['stage1'])),'stage1_counts':dict(collections.Counter(r['summary'] for r in stages['stage1'].values())),'stage2_processed':len(stages['stage2']),'stage2_counts':dict(collections.Counter(r['summary'] for r in stages['stage2'].values())),'remaining_for_disposition':remaining,'unviable_compiler_confirmed':len(unviable),'complete':len(stages['stage1'])==len(expected) and all(r['stage2']!='not_run' for r in remaining)}
(out/'audited-progress.json').write_text(json.dumps(summary,indent=2)+'\n')
(out/'unviable-dispositions.json').write_text(json.dumps(unviable,indent=2)+'\n')
print(json.dumps({k:v for k,v in summary.items() if k not in ['stage1_missing','remaining_for_disposition']}))
