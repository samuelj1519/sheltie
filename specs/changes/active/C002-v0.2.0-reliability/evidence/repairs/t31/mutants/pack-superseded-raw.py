import hashlib, json, pathlib, tarfile

base=pathlib.Path('/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C002-v0.2.0-reliability/evidence/repairs/t31/mutants')
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
for raw in sorted(base.glob('superseded-*/**/raw')):
    archive=raw.parent/'raw.tar.gz'
    if archive.exists():
        continue
    files=sorted(x for x in raw.rglob('*') if x.is_file())
    assert all(not x.is_symlink() for x in raw.rglob('*'))
    expected={str(x.relative_to(raw)): {'sha256':sha(x),'bytes':x.stat().st_size} for x in files}
    temp=raw.parent/'raw.tar.gz.part'
    assert not temp.exists()
    with tarfile.open(temp,'w:gz',compresslevel=6) as tar:
        for path in files:
            tar.add(path,arcname='raw/'+str(path.relative_to(raw)),recursive=False)
    observed={}
    with tarfile.open(temp,'r:gz') as tar:
        for member in tar.getmembers():
            assert member.isfile() and member.name.startswith('raw/')
            key=member.name[4:]; assert key not in observed
            data=tar.extractfile(member).read()
            observed[key]={'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
    assert observed==expected
    temp.rename(archive)
    manifest={'archive':'raw.tar.gz','archive_sha256':sha(archive),'members':expected,'verified_all_members':True,'loose_index_files':['outcomes.json','mutants.json'],'reason':'Lossless packing of superseded owned execution artifacts. Run outcomes and original bytes unchanged; restore members under raw/ to inspect logs/diffs.'}
    (raw.parent/'raw-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    before=sum(x['bytes'] for x in expected.values())
    for path in files:
        assert sha(path)==expected[str(path.relative_to(raw))]['sha256']
        if str(path.relative_to(raw)) not in manifest['loose_index_files']:
            path.unlink()
    for directory in sorted((x for x in raw.rglob('*') if x.is_dir()),key=lambda p:len(p.parts),reverse=True):
        if not any(directory.iterdir()):directory.rmdir()
    print(json.dumps({'run':str(raw.parent.relative_to(base)),'members':len(files),'original_bytes':before,'archive_bytes':archive.stat().st_size,'verified':True}),flush=True)
