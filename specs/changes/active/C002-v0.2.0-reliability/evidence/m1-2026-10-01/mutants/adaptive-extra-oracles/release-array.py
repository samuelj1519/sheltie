import argparse,hashlib,json,os,pathlib,platform,subprocess,tempfile
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--source',required=True);p.add_argument('--output',required=True);a=p.parse_args();out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=False);records=[];meta={'binary_sha256':hashlib.sha256(pathlib.Path(a.binary).read_bytes()).hexdigest(),'script_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'cases':[],'complete':False}
def save(p,v):p.write_text(json.dumps(v,sort_keys=True,indent=2)+'\n')
triple=('aarch64' if platform.machine()=='arm64' else 'x86_64')+'-apple-darwin'
def run(home,args,release):
 env=os.environ.copy();env['SHELTIE_RELEASE_BASE']=str(release)
 for k in list(env):
  if k=='SHELTIE_FAILPOINT' or k.startswith('SHELTIE_TEST_'):env.pop(k)
 q=subprocess.run([a.binary,'--home',str(home),'--json',*args],env=env,capture_output=True,text=True,timeout=30);records.append({'argv':q.args,'env_release_base':str(release),'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr});return q.returncode,json.loads(q.stdout)
try:
 for case in ['valid','malformed_length_then_valid','malformed_hex_then_valid']:
  with tempfile.TemporaryDirectory(prefix='m1-extra-release-',dir='/private/tmp') as td:
   base=pathlib.Path(td);home=base/'home';release=base/'release';tag=release/'v9.9.9';tag.mkdir(parents=True);latest=release/'latest';latest.mkdir();asset=b'independently sha-verified installed asset\n';digest=hashlib.sha256(asset).hexdigest();(tag/'good-asset').write_bytes(asset)
   artifacts=[{'kind':'executable-zip','name':'good-asset','target_triples':[triple],'checksum':'sha256:'+digest}]
   if case!='valid':artifacts.insert(0,{'kind':'executable-zip','name':'invalid-not-downloaded','target_triples':[triple],'checksum':'a'*63 if case=='malformed_length_then_valid' else 'g'*64})
   manifest={'dist_version':'0.32.0','announcement_tag':'v9.9.9','artifacts':artifacts};save(tag/'dist-manifest.json',manifest);save(latest/'dist-manifest.json',manifest);save(out/(case+'-release.json'),{'manifest':manifest,'asset_bytes':asset.decode(),'asset_sha256':digest,'invalid_asset_exists':False})
   c,v=run(home,['self','install'],release);assert c==0 and v['ok'],('SEMANTIC local install failed',v)
   before=(home/'bin/sheltie').read_bytes();c,v=run(home,['self','update','--version','9.9.9'],release)
   record={'case':case,'exit':c,'reply':v};meta['cases'].append(record)
   assert c==0 and v['ok'],('SEMANTIC valid array release or invalid candidate filtering rejected',case,c,v)
   actual=(home/'bin/sheltie').read_bytes();save(out/(case+'-installed.json'),{'before_sha256':hashlib.sha256(before).hexdigest(),'after_sha256':hashlib.sha256(actual).hexdigest(),'expected_sha256':digest});assert actual==asset,('SEMANTIC wrong release asset installed',case)
 meta['complete']=True
except Exception as e:meta['error']=repr(e);raise
finally:save(out/'commands.json',records);save(out/'metadata.json',meta)
