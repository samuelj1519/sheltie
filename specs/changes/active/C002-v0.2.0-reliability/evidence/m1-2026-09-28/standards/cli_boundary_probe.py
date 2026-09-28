import os, json, stat, hashlib, subprocess, tempfile, pathlib, tarfile, threading, time
BIN='/private/tmp/sheltie-m1-review-target/debug/sheltie'
ROOT=pathlib.Path(tempfile.mkdtemp(prefix='sheltie-standards-',dir='/private/tmp'))
def run(home,*args,env=None):
    p=subprocess.run([BIN,'--json','--home',str(home),*args],capture_output=True,text=True,env=env)
    return {'command':[BIN,'--json','--home',str(home),*args], 'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr}
results={'temp_root':str(ROOT)}
# bin ancestor symlink: rollback must reject rather than destroy external bytes.
home=ROOT/'rollback-home';home.mkdir();outside=ROOT/'rollback-outside';outside.mkdir();(home/'bin').symlink_to(outside,target_is_directory=True)
(outside/'sheltie').write_bytes(b'outside-current');(outside/'sheltie.prev').write_bytes(b'outside-prev')
results['rollback']={'before': {p.name:p.read_text() for p in outside.iterdir()},'run':run(home,'self','rollback'),'after':{p.name:p.read_text() for p in outside.iterdir()}}
# tmp ancestor symlink: capture transient UUID staging on the external directory.
home=ROOT/'install-home';home.mkdir();outside=ROOT/'install-outside';outside.mkdir();(home/'tmp').symlink_to(outside,target_is_directory=True)
seen=[];done=threading.Event()
def watch():
    while not done.is_set():
        for p in outside.iterdir():
            if p.name not in seen: seen.append(p.name)
thread=threading.Thread(target=watch);thread.start()
r=run(home,'self','install');done.set();thread.join()
results['install_tmp_symlink']={'run':r,'outside_transient_entries':seen,'installed':(home/'bin/sheltie').exists()}
# Normal home with frozen Workbook: purge should remove all managed content.
home=ROOT/'purge-home'
add=run(home,'workbook','add','/Users/shushu/orca/workspaces/sheltie/codex/examples/two-step')
results['purge_frozen_workbook']={'add':add,'purge':run(home,'self','uninstall','--purge','--yes'),'root_still_exists':home.exists(),'remaining':[str(p.relative_to(home)) for p in home.rglob('*')] if home.exists() else []}
# Valid checksum archive containing a sheltie symlink to a root-external file.
home=ROOT/'archive-home';home.mkdir();(home/'bin').mkdir();(home/'bin/sheltie').write_bytes(b'old-binary')
sentinel=ROOT/'archive-outside-sentinel';sentinel.write_bytes(b'do-not-chmod');sentinel.chmod(0o600)
release=ROOT/'release/v9.9.9';release.mkdir(parents=True);asset=release/'sheltie.tar.gz'
with tarfile.open(asset,'w:gz') as tf:
    item=tarfile.TarInfo('sheltie');item.type=tarfile.SYMTYPE;item.linkname=str(sentinel);tf.addfile(item)
platform=json.loads(run(home,'self','version')['stdout'])['data']['platform']
manifest={'version':'9.9.9','assets':[{'platform':platform,'name':asset.name,'sha256':hashlib.sha256(asset.read_bytes()).hexdigest()}]}
(release/'dist-manifest.json').write_text(json.dumps(manifest));env=dict(os.environ,SHELTIE_RELEASE_BASE=str(ROOT/'release'))
results['archive_symlink']={'manifest':manifest,'before_mode':'0600','run':run(home,'self','update','--version','9.9.9',env=env),'after_mode':oct(stat.S_IMODE(sentinel.stat().st_mode)),'installed_is_symlink':(home/'bin/sheltie').is_symlink(),'installed_target':os.readlink(home/'bin/sheltie') if (home/'bin/sheltie').is_symlink() else None}
print(json.dumps(results,ensure_ascii=False,indent=2))
