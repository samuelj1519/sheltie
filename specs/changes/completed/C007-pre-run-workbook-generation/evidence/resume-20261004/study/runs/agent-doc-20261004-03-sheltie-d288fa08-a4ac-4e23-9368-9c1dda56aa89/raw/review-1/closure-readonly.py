import hashlib,json,pathlib,subprocess,os,platform,sys
run=pathlib.Path(sys.argv[1]); repo=pathlib.Path(sys.argv[2]);base="351feb7ac22c21317a686693b732d5ae0c4b4bcc";candidate="89c1b6022ced9cfe698e56a0f1c08bc65168cf46";tree="5640cd4993bcac032bc15fcdee17a9e05cf42eb5";guide="specs/guides/continuity-choices.md"
def git(*args):return subprocess.check_output(["git",*args],cwd=repo)
def sha(data):return hashlib.sha256(data).hexdigest()
assert git("rev-parse","HEAD").decode().strip()==candidate
assert git("rev-parse","HEAD^{tree}").decode().strip()==tree
assert git("rev-parse",base+"^{tree}").decode().strip()=="80ea3046b1b3575f07e68313444c051ee7c5b7db"
assert git("status","--porcelain")==b""
assert git("diff","--name-only",base,"HEAD").decode().splitlines()==[guide]
assert git("ls-files","--others","--exclude-standard")==b""
blob=git("show","HEAD:"+guide);assert blob==(repo/guide).read_bytes()
assert sha(blob)=="a028106d07209ad3026af594e88004acda12e878f2a710f0402bd379fddcf890"
patch=(run/"raw/implement-1/06-full-patch.stdout").read_bytes();assert patch==git("diff","--binary","--full-index",base,"HEAD","--",guide)
assert sha(patch)=="a10bef129228ddc38cb77f9fbbda4e4e21ad1d62b84fc5032e3b3171ef41d1ac"
reply=json.loads((run/"begin-review-1-reply.json").read_text());inputs={}
expected={"task":"a0e91be95c47fb82b5ca3af39c04956fb5d4f944bf9b86686a3f3482db44898d","project":"0e178e656e5559cd087437a4dbfa2e2c9dfdb630876e3ad35d8a5bc425998905","change":"664b3ed17f4effbc1d5d5425aeb8485335bccf9c7103f1858b4755ed98ce065c","checks":"f30f51bb8353d8b886bf5bfcf61be69ef22827f6dbbcbcdf85edf7859b58a9cd"}
for key,path in reply["data"]["inputs"].items():
 data=pathlib.Path(path).read_bytes();assert sha(data)==expected[key];inputs[key]={"path":path,"sha256":sha(data),"bytes":len(data)}
raw_records=[]
for label in ["01-stage","02-staged-identity","03-check-docs","04-check-diff","05-commit","06-full-patch","07-candidate-closure"]:
 prefix=run/"raw/implement-1"/label;r=json.loads(pathlib.Path(str(prefix)+".json").read_text())
 for stream in ["stdout","stderr"]:assert sha(pathlib.Path(r[stream+"_path"]).read_bytes())==r[stream+"_sha256"]
 assert r["exit_code"]==0 and not r["timed_out"] and r["cwd"]==str(repo)
 raw_records.append({"label":label,"argv":r["argv"],"exit_code":r["exit_code"],"start_utc":r["start_utc"],"end_utc":r["end_utc"],"stdout_sha256":r["stdout_sha256"],"stderr_sha256":r["stderr_sha256"]})
staged=json.loads((run/"raw/implement-1/02-staged-identity.stdout").read_text());assert staged["staged_tree"]==tree and staged["guide_sha256"]==sha(blob)
print(json.dumps({"candidate":candidate,"tree":tree,"base":base,"paths":[guide],"guide_sha256":sha(blob),"guide_bytes":len(blob),"patch_sha256":sha(patch),"patch_bytes":len(patch),"inputs":inputs,"verified_raw_records":raw_records,"reused_checks":"exact staged tree equals committed candidate; not independently rerun","independent_apply":"not_run: deliver owner","environment":{k:os.environ.get(k) for k in ["PATH","LANG","LC_ALL","LC_CTYPE","GIT_INDEX_FILE","GIT_DIR","GIT_WORK_TREE"]},"platform":platform.platform()},ensure_ascii=False,indent=2))
