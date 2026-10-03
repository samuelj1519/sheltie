import hashlib
import json
import os
import stat
import time
from pathlib import Path

from observe import EVIDENCE, FREEZE, call, check_frozen, start_run, utc


def verify_copy(target, result):
    assert target.is_absolute() and not target.is_symlink()
    manifest = json.loads((target / 'manifest.json').read_text())
    manifest_info = (target / 'manifest.json').lstat()
    assert (
        stat.S_ISREG(manifest_info.st_mode)
        and manifest_info.st_nlink == 1
        and stat.S_IMODE(manifest_info.st_mode) == 0o600
    )
    assert manifest['format'] == 'work-export-manifest/v1' and manifest['result'] == result
    assert [item['key'] for item in manifest['files']] == [
        item['key'] for item in result['artifacts']
    ]
    observed = []
    for source, copied in zip(result['artifacts'], manifest['files']):
        expected_path = 'artifacts/' + f"{len(observed) + 1:04}" + '/' + Path(source['path']).name
        assert copied == dict(
            key=source['key'],
            path=expected_path,
            sha256=source['sha256'],
            bytes=source['bytes'],
        )
        path = target / expected_path
        body, info = path.read_bytes(), path.lstat()
        assert (
            stat.S_ISREG(info.st_mode)
            and info.st_nlink == 1
            and stat.S_IMODE(info.st_mode) == 0o600
        )
        assert len(body) == source['bytes'] and hashlib.sha256(body).hexdigest() == source['sha256']
        observed.append(dict(
            key=source['key'],
            path=str(path),
            bytes=len(body),
            sha256=source['sha256'],
            mode=oct(stat.S_IMODE(info.st_mode)),
            nlink=info.st_nlink,
        ))
    for directory in [target, *[p for p in target.rglob('*') if p.is_dir()]]:
        info = directory.lstat()
        assert stat.S_ISDIR(info.st_mode) and stat.S_IMODE(info.st_mode) == 0o700
    expected_files = {'manifest.json', *[item['path'] for item in manifest['files']]}
    actual_files = {
        str(path.relative_to(target))
        for path in target.rglob('*') if not path.is_dir()
    }
    assert actual_files == expected_files
    return manifest, observed


def main():
    check_frozen()
    start_run()
    start = time.monotonic()
    manual = Path(FREEZE['manual_root'])
    manual.mkdir(mode=0o700)
    (manual / 'artifacts').mkdir(mode=0o700)
    result = json.loads(call('manual-find-result', [
        FREEZE['binary'], '--home', FREEZE['home'], '--json',
        'work', 'result', FREEZE['work_id'],
    ]))['data']
    assert result == FREEZE['result']
    found = time.monotonic()
    files = []
    for index, artifact in enumerate(result['artifacts'], 1):
        body = call('manual-read-' + str(index), [
            FREEZE['binary'], '--home', FREEZE['home'],
            'work', 'result', FREEZE['work_id'],
            '--artifact=' + artifact['key'], '--revision', str(result['revision']),
        ])
        assert len(body) == artifact['bytes'] and hashlib.sha256(body).hexdigest() == artifact['sha256']
        relative = 'artifacts/' + f'{index:04}' + '/' + Path(artifact['path']).name
        target = manual / relative
        target.parent.mkdir(mode=0o700)
        with target.open('xb') as output:
            os.fchmod(output.fileno(), 0o600)
            output.write(body)
        files.append(dict(
            key=artifact['key'],
            path=relative,
            sha256=artifact['sha256'],
            bytes=artifact['bytes'],
        ))
    with (manual / 'manifest.json').open('x') as output:
        os.fchmod(output.fileno(), 0o600)
        json.dump(
            dict(format='work-export-manifest/v1', result=result, files=files),
            output, ensure_ascii=False, indent=2,
        )
        output.write('\n')
    received = time.monotonic()
    manual_manifest, manual_files = verify_copy(manual, result)
    checked = time.monotonic()
    tool_start = time.monotonic()
    reply = json.loads(call('tool-export-first', [
        FREEZE['exporter'], '--sheltie', FREEZE['binary'],
        '--home', FREEZE['home'], '--work', FREEZE['work_id'],
        '--to', FREEZE['native_parent'], '--json',
    ]))
    assert reply['format'] == 'work-export/v1' and reply['status'] == 'complete'
    assert reply['work_id'] == result['work_id'] and reply['revision'] == result['revision']
    target = Path(reply['target_path'])
    assert target.parent == Path(FREEZE['native_parent'])
    exported = time.monotonic()
    tool_manifest, tool_files = verify_copy(target, result)
    tool_checked = time.monotonic()
    assert manual_manifest == tool_manifest
    binding = dict(
        work_id=FREEZE['work_id'],
        result=result,
        manual_files=manual_files,
        tool_files=tool_files,
        tool_first_target=str(target),
        annotation=FREEZE['expected_annotation'],
        report_path=str(EVIDENCE / 'consumer-report.md'),
        frozen_C007_entry=str(
            Path.cwd() / 'specs/changes/completed/C007-pre-run-workbook-generation/plan.md'
        ),
    )
    with (EVIDENCE / 'consumer-binding.json').open('x') as output:
        json.dump(binding, output, ensure_ascii=False, indent=2)
        output.write('\n')
    cost = dict(
        observed_utc=utc().isoformat(),
        manual=dict(
            find_result_s=found - start,
            receive_s=received - found,
            independent_verify_s=checked - received,
            total_s=checked - start,
            outer_CLI_calls=4,
        ),
        tool=dict(
            combined_find_receive_verify_publish_s=exported - tool_start,
            independent_verify_s=tool_checked - exported,
            total_s=tool_checked - tool_start,
            outer_exporter_calls=1,
            inner_engine_calls='metadata1 + raw3 from current source; not instrumented separate process counts',
        ),
        interpretation='One real agent/script pair, same source/content standard. Timings include recorder snapshots and file setup; neither human activity nor general net benefit.',
        usage=None,
        paid_cost=None,
        human_activity=None,
    )
    (EVIDENCE / 'native-pair-result.json').write_text(json.dumps(dict(
        manual_root=str(manual),
        target=str(target),
        all_selected_bytes_equal=True,
        parsed_manifests_equal=True,
        manual_files=manual_files,
        tool_files=tool_files,
        cost=cost,
    ), ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(dict(manual=str(manual), tool=str(target), paired_bytes=True), ensure_ascii=False))


if __name__ == '__main__':
    main()
