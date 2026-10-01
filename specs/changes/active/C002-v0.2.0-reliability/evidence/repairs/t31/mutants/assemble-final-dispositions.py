import pathlib,json,collections,hashlib

b=pathlib.Path(__file__).resolve().parent
inventory=json.loads((b/'inventory.json').read_text());stage1={};stage2={}
for directory in sorted(b.glob('stage*-sheltie-*')):
    if not (directory/'metadata.json').exists():continue
    stage=directory.name.split('-')[0]
    for r in json.loads((directory/'raw/outcomes.json').read_text())['outcomes']:
        if r['scenario']=='Baseline':continue
        n=r['scenario']['Mutant']['name'];item={'result':r['summary'],'raw_directory':str(directory.relative_to(b))+'/raw','log_path':r['log_path'],'diff_path':r['diff_path']}
        dest=stage1 if stage=='stage1' else stage2;assert n not in dest;dest[n]=item
proofs={r['name']:r for r in json.loads((b/'independent-proof-candidates.json').read_text())['proofs']};supp={}
for p in sorted(b.parent.glob('supplemental-oracles/round*/results.json')):
    for r in json.loads(p.read_text()):
        if r['caught_by_real_oracle']:
            supp[r['name']]={'result':'caught_by_real_oracle','results':str(p.relative_to(b.parent)),'closure_sha256':r['closure_sha256'],'diff':r['diff'],'stdout':r['stdout']}
rows=[]
for m in inventory:
    n=m['name'];a=stage1[n];z=stage2.get(n)
    r={'name':n,'function':(m.get('function') or {}).get('function_name','module constant'),'file':m['file'],'stage1':a,'stage2':z or {'result':'not_run'}}
    if a['result']=='CaughtMutant':r['classification']='caught_stage1'
    elif a['result']=='Unviable':r['classification']='unviable_compiler_confirmed'
    elif z and z['result']=='CaughtMutant':r['classification']='caught_stage2'
    elif n in supp:r['classification']='caught_supplemental';r['supplemental']=supp[n]
    elif m['package']=='sheltie-core':
        assert 'parse_input_source' in n and z and z['result']=='MissedMutant'
        r['classification']='equivalent_reviewed';r['proof']='parse-input-source-equivalence-review.md'
    elif z and z['result']=='MissedMutant' and n in proofs:
        r['classification']=proofs[n]['classification']+'_reviewed';r['proof']=proofs[n]
    else:
        r['classification']='deferred_by_user'
        r['reason']='2026-10-01 user requests skipping potentially safety-triggering tasks. Remaining mixed runtime mutation reproduction/review omitted from this T31 acceptance; neither caught nor equivalent nor safety PASS.'
    rows.append(r)
counts=dict(collections.Counter(r['classification'] for r in rows))
summary={'candidate':'49d3a191aa4c918aab279617fa2bf7d9b3b36a0e','inventory':len(rows),'classification_counts':counts,'source_input_sha256':hashlib.sha256((b/'source-input.json').read_bytes()).hexdigest(),'original_full_mutation_execution_complete':False,'dispositions_accounted_with_explicit_user_deferral':True,'security_validation_pass':False,'acceptance_scope':'T31 complete only under explicit 2026-10-01 user waiver recorded in plan.md. M1 remains not_run. Linux remains not_run.','items':rows}
(b/'final-dispositions.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')
(b/'user-deferred-ids.json').write_text(json.dumps({'authorization':'先跳过可能会触发安全的相关任务，继续执行到 T31 完成','candidate':summary['candidate'],'count':counts.get('deferred_by_user',0),'ids':[r['name'] for r in rows if r['classification']=='deferred_by_user']},ensure_ascii=False,indent=2)+'\n')
print(json.dumps({k:v for k,v in summary.items() if k!='items'},ensure_ascii=False))
