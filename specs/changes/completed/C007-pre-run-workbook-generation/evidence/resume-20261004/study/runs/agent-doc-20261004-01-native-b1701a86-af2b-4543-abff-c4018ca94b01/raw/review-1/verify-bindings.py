import hashlib,json,pathlib,subprocess,sys
repo=pathlib.Path.cwd();run=pathlib.Path(sys.argv[1]);base='351feb7ac22c21317a686693b732d5ae0c4b4bcc';candidate='8366454ddbc2a07d25b2e5e949171cde5b5169ae';tree='3b9e14dfb50d3b577cc4caaacdf33e96082026fc'
def git(*args): return subprocess.check_output(['git',*args],text=True).strip()
assert git('rev-parse','HEAD')==candidate
assert git('rev-parse','HEAD^{tree}')==tree
assert git('rev-parse','HEAD^')==base
assert not git('status','--porcelain=v1')
paths=git('diff','--name-only',base,'HEAD').splitlines();assert paths==['README.md','specs/guides/source-quick-start.md'],paths
files={p:{'sha256':hashlib.sha256((repo/p).read_bytes()).hexdigest(),'bytes':(repo/p).stat().st_size} for p in paths}
after=json.loads((run/'raw/implement-1/candidate-after-commit.json').read_text());assert files==after['allow_files']
checks=[]
for binding in json.loads((run/'raw/implement-1/checks-bindings.json').read_text()):
 record=json.loads(pathlib.Path(binding['record']).read_text());assert binding['candidate_index_tree']==tree;assert record['exit_code']==0 and not record['timed_out'] and record['within_deadline']
 for stream in ('stdout','stderr'):
  content=pathlib.Path(record[stream+'_path']).read_bytes();assert hashlib.sha256(content).hexdigest()==record[stream+'_sha256']
 checks.append({'record':binding['record'],'exit_code':record['exit_code'],'stdout':pathlib.Path(record['stdout_path']).read_text(),'stderr':pathlib.Path(record['stderr_path']).read_text(),'bound_tree':tree})
print(json.dumps({'candidate':candidate,'tree':tree,'base':base,'clean':True,'paths':paths,'files':files,'checks':checks},ensure_ascii=False,indent=2))
