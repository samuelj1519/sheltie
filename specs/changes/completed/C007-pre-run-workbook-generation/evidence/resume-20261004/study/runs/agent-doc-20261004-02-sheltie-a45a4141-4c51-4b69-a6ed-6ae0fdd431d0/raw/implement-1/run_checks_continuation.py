import datetime
import hashlib
import importlib.util
import json
import os
import platform
from pathlib import Path

STUDY = Path('/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study')
RUN = STUDY / 'runs/agent-doc-20261004-02-sheltie-a45a4141-4c51-4b69-a6ed-6ae0fdd431d0'
RAW = RUN / 'raw/implement-1'
REPO = Path('/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/source-start/sheltie')
BASE = '351feb7ac22c21317a686693b732d5ae0c4b4bcc'
BASE_TREE = '80ea3046b1b3575f07e68313444c051ee7c5b7db'
DEADLINE = '2026-10-03T20:32:28.417368+00:00'
ALLOWED = ['README.md', 'specs/guides/source-quick-start.md']

spec = importlib.util.spec_from_file_location('frozen_capture', STUDY / 'capture.py')
capture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(capture)

def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()

def write_new(name, value):
    with (RAW / name).open('x') as output:
        json.dump(value, output, ensure_ascii=False, indent=2)
        output.write('\n')

def run(label, argv):
    record, stdout, stderr = capture.execute(RAW, label, argv, REPO, 120, DEADLINE)
    if record['exit_code'] != 0:
        raise RuntimeError('Actual failure retained: ' + label)
    return record, stdout.decode()


_, current = run('baseline-confirm-after-diagnostic', ['git', 'show', '-s', '--format=%H%n%T', 'HEAD'])
assert current.splitlines() == [BASE, BASE_TREE]
_, status = run('before-stage-status', ['git', 'status', '--porcelain=v1', '--untracked-files=all'])
assert status.splitlines() == [' M README.md', '?? specs/guides/source-quick-start.md'], status
run('stage-authorized-files', ['git', 'add', '--', *ALLOWED])
_, changed = run('staged-paths', ['git', 'diff', '--cached', '--name-only'])
assert changed.splitlines() == ALLOWED
_, tree = run('staged-tree-before-checks', ['git', 'write-tree'])
tree = tree.strip()
hashes = {name: hashlib.sha256((REPO / name).read_bytes()).hexdigest() for name in ALLOWED}
write_new('checked-input.json', {'recorded_utc': now(), 'baseline_head': BASE, 'baseline_tree': BASE_TREE,
                              'staged_tree': tree, 'allow_files_sha256': hashes, 'required_argv': [
                                  ['scripts/check-docs.sh', *ALLOWED], ['git', 'diff', '--cached', '--check']]})
checks = []
for label, argv in [
    ('required-docs', ['scripts/check-docs.sh', *ALLOWED]),
    ('required-staged-diff', ['git', 'diff', '--cached', '--check']),
]:
    record, stdout = run(label, argv)
    checks.append({'label': label, 'record_path': str(RAW / (label + '.json')), 'exit_code': record['exit_code'], 'staged_tree': tree})
_, after_tree = run('staged-tree-after-checks', ['git', 'write-tree'])
assert after_tree.strip() == tree
assert hashes == {name: hashlib.sha256((REPO / name).read_bytes()).hexdigest() for name in ALLOWED}
message = RAW / 'commit-message.txt'
with message.open('x') as output:
    output.write('docs(study): 补充当前源码快速开始入口\n\n'
                 '为首次使用者补齐源码候选的独立构建、显式 Home 与 code-change 全流程，保留 v0.2.0 安装历史。\n\n'
                 '已运行授权文件的 check-docs 与 staged diff 空白检查；两项退出码均为 0，原件绑定同一 staged tree。源码构建和流程演练未执行。\n\n'
                 'Change: C007\nTask: C007-T05\nSample: source-start\nArm: sheltie\n'
                 'Agent: /root/c007_study_coordinator/run02_implement\n')
run('candidate-commit', ['git', '-c', 'core.hooksPath=/dev/null', '-c', 'commit.gpgsign=false', 'commit', '--file', str(message)])
_, identity = run('candidate-identity', ['git', 'show', '-s', '--format=%H%n%T', 'HEAD'])
head, committed_tree = identity.splitlines()
assert committed_tree == tree
_, paths = run('candidate-patch-paths', ['git', 'diff', '--name-only', BASE, 'HEAD'])
assert paths.splitlines() == ALLOWED
_, status = run('candidate-status', ['git', 'status', '--porcelain=v1', '--untracked-files=all'])
assert status == ''
_, stat = run('candidate-stat', ['git', 'show', '--stat', '--oneline', 'HEAD'])
assert hashes == {name: hashlib.sha256((REPO / name).read_bytes()).hexdigest() for name in ALLOWED}
write_new('candidate.json', {'conclusion': 'reviewable; required mechanical checks completed', 'end_utc': now(),
                           'head': head, 'tree': tree, 'baseline_head': BASE, 'baseline_tree': BASE_TREE,
                           'allow_files_sha256': hashes, 'changed_paths': ALLOWED, 'status': 'clean',
                           'checks': checks, 'quality': 'not_run', 'acceptance': 'not_run',
                           'complete_patch_and_independent_apply': 'not_run; assigned to deliver'})
print(json.dumps({'head': head, 'tree': tree, 'checks': checks, 'raw': str(RAW)}, ensure_ascii=False))
