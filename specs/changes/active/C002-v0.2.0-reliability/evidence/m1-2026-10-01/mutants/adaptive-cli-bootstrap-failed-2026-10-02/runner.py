import collections,fcntl,hashlib,json,os,pathlib,re,shutil,subprocess,time
root=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex');clone=root/'target/m1-validation/focused-source';base=root/'specs/changes/active/C002-v0.2.0-reliability/evidence/m1-2026-10-01/mutants';out=base/'adaptive-cli-groups';out.mkdir(exist_ok=False)
lock=(root/'target/m1-validation/adaptive-cli.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
groups_path=base/'adaptive-validation-2026-10-02/groups.json';groups=json.loads(groups_path.read_text());manifest=json.loads((base/'source-input.json').read_text());inventory={m['name']:m for m in json.loads((base/'inventory.json').read_text())}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,d):p.write_text(json.dumps(d,indent=2)+'\n')
self_path=pathlib.Path(__file__);self_sha=sha(self_path);manifest_sha=sha(base/'source-input.json');inventory_sha=sha(base/'inventory.json');groups_sha=sha(groups_path)
env=os.environ.copy();env.update(PATH=str(root/'target/t31-validation/tools')+':'+env['PATH'],RUSTC_WRAPPER='',CARGO_TARGET_DIR='target',CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='2',NEXTEST_TEST_THREADS='2',CARGO_PROFILE_TEST_OPT_LEVEL='1',CARGO_PROFILE_TEST_DEBUG='0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
for k in ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_COMMON_DIR','GIT_OBJECT_DIRECTORY','GIT_ALTERNATE_OBJECT_DIRECTORIES']:env.pop(k,None)
def verify():
 assert sha(self_path)==self_sha and sha(base/'source-input.json')==manifest_sha and sha(base/'inventory.json')==inventory_sha and sha(groups_path)==groups_sha
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=clone,text=True).strip()==manifest['candidate'];assert not subprocess.check_output(['git','status','--porcelain'],cwd=clone,text=True).strip()
 for e in manifest['source_files']:assert sha(root/e['path'])==sha(clone/e['path'])==e['sha256'],e['path']
verify()
short=['committed_history_and_attempt_directory_type_changes_are_integrity_errors_with_original_snapshot','zero_work_revision_is_rejected_by_readonly_and_write_cli_entries','impossible_persisted_blocked_counts_are_structured_errors_without_writes_or_panic','exact_system_tmp_parent_alias_reads_the_same_at_file_and_rejects_user_symlinks','file_inputs_reject_same_volume_replacement_links_and_special_files_at_open','workbook_source_rejects_a_replaced_directory_at_the_open_boundary','readonly_sqlite_open_rejects_root_or_store_rebinding_with_unchanged_database_bytes','relative_input_resolution_reports_a_removed_current_directory_without_registering_a_request','corrupt_remove_snapshot_cannot_be_projected_as_a_successful_original','corrupt_pending_remove_snapshot_cannot_be_projected_as_a_blockers_original','add_snapshot_target_is_bound_before_an_original_response_is_released','fail_snapshot_status_matches_the_original_retry_even_after_later_progress','begin_snapshot_requires_match_the_frozen_node_even_when_reply_and_data_agree']
configs={'G01':['replay','attempt','scenario_gated_release'],'G02':['workbook','scenario_workbook_lifecycle','replay'],'G03':['os_process','self_cmd','output_paths','work'],'G04':['os_process','self_cmd','output_paths','workbook','scenario_artifacts'],'G05':['os_process','self_cmd','output_paths','scenario_artifacts'],'G06':['workbook','work','output_paths'],'G07':['attempt','output_paths','scenario_artifacts'],'G08':['replay','scenario_workbook_lifecycle','crash'],'G09':['replay','self_cmd','workbook'],'G10':['work','replay'],'G11':['self_cmd','remote_update'],'G12':['remote_update'],'G13':['crash']}
plans=[]
for gid,binaries in configs.items():
 ids=[x['id'] for x in groups['items'] if x['group_id']==gid and x['adaptive_status']=='adaptive_not_run' and x['remaining_execution_route']=='only_unexecuted_cross_crate_consumer_then_shared_new_condition']
 if not ids:continue
 selectors=[f'binary(={b})' for b in binaries]+[f'test(={t})' for t in short if gid not in ['G11','G12','G13']]
 expression=' | '.join(selectors)
 if gid=='G11':expression='('+expression+') & !test(=remote_update_accepts_exact_stream_limits_and_preserves_state_on_one_more_byte) & !test(=remote_update_accepts_exact_asset_limit_and_rejects_one_more_byte)'
 plans.append({'group':gid,'ids':ids,'filter':expression})
assert sum(len(p['ids']) for p in plans)==181 and len({x for p in plans for x in p['ids']})==181
closure={'candidate':manifest['candidate'],'product':groups['product_candidate'],'runner_sha256':self_sha,'source_manifest_sha256':manifest_sha,'inventory_sha256':inventory_sha,'groups_sha256':groups_sha,'environment':{k:v for k,v in env.items() if k.startswith('CARGO_') or k in ['RUSTC_WRAPPER','NEXTEST_TEST_THREADS']},'plans':plans,'scope':'Only181 exact first-stage missed IDs without completed workspace test. CLI consumers only, grouped filters. No unchanged same-package retest. All residuals retain open disposition until direct oracle/static proof.','tools':{n:subprocess.check_output(a,cwd=clone,env=env,text=True).splitlines()[0] for n,a in [('rustc',['rustc','--version']),('cargo',['cargo','--version']),('nextest',['cargo','nextest','--version']),('mutants',['cargo','mutants','--version'])]}}
original_tools=json.loads((base/'execution-closure.json').read_text())['tools'];assert closure['tools']==original_tools,'toolchain drift'
save(out/'closure.json',closure);shutil.copyfile(self_path,out/'runner.py');results=[]
for plan in plans:
 verify();folder=out/plan['group'];folder.mkdir();assert not (clone/'target/mutants.out').exists()
 pattern='^(?:'+'|'.join(re.escape(n) for n in plan['ids'])+')$'
 argv=['cargo','mutants','-p','sheltie-runtime','--all-features','--copy-vcs','true','--jobs','4','--timeout','300','--test-tool','nextest','--test-package','sheltie-cli','--exclude','crates/*/src/testkit.rs','--output','target','--re',pattern,'--','-E',plan['filter'],'--no-tests=fail']
 save(out/'current-phase.json',{'group':plan['group'],'requested':len(plan['ids']),'argv':argv,'started_epoch':time.time()});start=time.time()
 with (folder/'stdout.txt').open('wb') as stream:p=subprocess.run(argv,cwd=clone,env=env,stdout=stream,stderr=subprocess.STDOUT)
 save(folder/'process.json',{'exit':p.returncode,'argv':argv,'seconds':time.time()-start});raw=clone/'target/mutants.out';assert raw.exists();shutil.move(raw,folder/'raw');raw=folder/'raw';save(folder/'raw-manifest.json',{str(f.relative_to(raw)):{'bytes':f.stat().st_size,'sha256':sha(f)} for f in raw.rglob('*') if f.is_file()})
 d=json.loads((raw/'outcomes.json').read_text());listed=json.loads((raw/'mutants.json').read_text());rows=[r for r in d['outcomes'] if r['scenario']!='Baseline'];baseline=[r for r in d['outcomes'] if r['scenario']=='Baseline']
 assert p.returncode in [0,2,3] and d['end_time'] is not None;assert len(baseline)==1 and baseline[0]['summary']=='Success';assert set(m['name'] for m in listed)==set(plan['ids']) and len(listed)==len(plan['ids']);assert len(rows)==len({r['scenario']['Mutant']['name'] for r in rows})==len(listed) and {r['scenario']['Mutant']['name'] for r in rows}==set(plan['ids'])
 assert {x['phase']:x['process_status'] for x in baseline[0]['phase_results']}=={'Build':'Success','Test':'Success'}
 for x in baseline[0]['phase_results']:assert '--package=sheltie-cli@0.1.0' in x['argv'] and plan['filter'] in x['argv'] and '--no-tests=fail' in x['argv']
 baseline_log=(raw/baseline[0]['log_path']).read_text(errors='replace');assert re.search(r'[1-9][0-9]* tests run:',baseline_log),'no nonzero selected baseline tests'
 assert d['total_mutants']==len(rows) and d['caught']==sum(r['summary']=='CaughtMutant' for r in rows) and d['missed']==sum(r['summary']=='MissedMutant' for r in rows) and d['timeout']==sum(r['summary']=='Timeout' for r in rows) and d['unviable']==sum(r['summary']=='Unviable' for r in rows)
 if p.returncode==3:assert d['timeout']>0
 if p.returncode==0:assert d['missed']==d['timeout']==0
 if p.returncode==2:assert d['missed']>0 and d['timeout']==0
 for m in listed:assert m==inventory[m['name']]
 for r in rows:
  m=dict(r['scenario']['Mutant']);expected=dict(inventory[m['name']]);diff=expected.pop('diff');assert m==expected;assert (raw/r['diff_path']).read_text()==diff;phases={x['phase']:x['process_status'] for x in r['phase_results']};log=(raw/r['log_path']).read_text(errors='replace')
  for x in r['phase_results']:assert '--package=sheltie-cli@0.1.0' in x['argv'] and plan['filter'] in x['argv'],x['argv']
  if r['summary']=='CaughtMutant':assert phases=={'Build':'Success','Test':{'Failure':100}} and any(t in log for t in ['FAIL','ABORT','SIGABRT','stack overflow','test failed'])
  elif r['summary']=='MissedMutant':assert phases=={'Build':'Success','Test':'Success'}
  elif r['summary']=='Timeout':assert 'Timeout' in phases.values()
  elif r['summary']=='Unviable':assert phases=={'Build':{'Failure':101}} and re.search(r'(?ms)^error(?:\[E[0-9]+\])?:[^\n]*\n(?:(?!^error).){0,4000}?^\s*-->[^\n]*\.rs:[0-9]+:[0-9]+',log)
  else:raise AssertionError(r['summary'])
 item={'group':plan['group'],'requested':len(listed),'processed':len(rows),'counts':dict(collections.Counter(r['summary'] for r in rows)),'exit':p.returncode,'seconds':time.time()-start,'filter':plan['filter'],'outcomes_sha256':sha(raw/'outcomes.json')};save(folder/'metadata.json',item);results.append(item);save(out/'progress.json',results);print(json.dumps(item),flush=True)
verify();save(out/'results.json',{'complete_execution':True,'requested':181,'groups':results,'disposition_complete':False})
