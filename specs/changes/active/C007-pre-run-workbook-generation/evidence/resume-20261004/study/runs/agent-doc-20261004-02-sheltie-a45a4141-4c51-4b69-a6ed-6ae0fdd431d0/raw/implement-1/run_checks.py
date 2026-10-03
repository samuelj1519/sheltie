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

RAW.mkdir(parents=True, exist_ok=True)
reply = json.loads((RUN / 'begin-implement-1-reply.json').read_text())['data']
input_hashes = {name: hashlib.sha256(Path(path).read_bytes()).hexdigest()
                for name, path in reply['inputs'].items() if path}
assert input_hashes == {
    'task': '1345a1ce70f26a1311de7d6b8a57b5786bd0ea154f4073a46eccd1fe461e4f64',
    'project': '188dac600eeccd1593012bbd6894dc966b8daee495ac1d33f0ce59ab5f7b87ec',
}
skill = Path('/Users/shushu/.agents/skills/tech-doc-style-chinese/SKILL.md')
assert hashlib.sha256(skill.read_bytes()).hexdigest() == '24c1c75482d47270d3c58bfdab24df3747334cc61ea8be94e98131236aa9b243'
write_new('actor.json', {
    'actor_id': '/root/c007_study_coordinator/run02_implement',
    'phase': 'implement#1.0',
    'identity_recorded_utc': now(),
    'model_declaration': 'gpt-6.1-sol/high inherited; no override',
    'runtime_model_session_metadata': 'unknown; Root independently verifies',
    'usage': 'unknown', 'fee': 'unknown',
    'repo': str(REPO), 'baseline_head': BASE, 'baseline_tree': BASE_TREE,
    'initial_status_verified_by_tool': 'clean before edits',
    'task_project_sha256': input_hashes,
    'brief': reply['brief_path'],
    'environment': {name: os.environ.get(name) for name in ['PATH', 'LANG', 'LC_ALL', 'RUSTC_WRAPPER', 'CARGO_TARGET_DIR', 'GIT_CONFIG_COUNT', 'GIT_INDEX_FILE', 'GIT_WORK_TREE', 'GIT_DIR']},
    'platform': platform.platform(),
    'commands': 'Every command below is preserved through frozen capture.execute; preceding input/source reads and apply_patch are in the actual worker tool transcript.',
    'earlier_diagnostic_failure': {
        'argv': ['zsh', '-lc', "rg -n 'enum|struct|Version|Verify|Start|Begin|Submit|home|request_id|artifact|revision' crates/sheltie-cli/src/cli.rs crates/sheltie-cli/src/main.rs Cargo.toml crates/sheltie-runtime/src/store* crates/sheltie-core/src/protocol*"],
        'cwd': str(REPO), 'exit_code': 1,
        'stdout_stderr_tool_combined': 'zsh:1: no matches found: crates/sheltie-core/src/protocol*\n',
        'separate_streams': 'unknown; original exec tool returned combined output',
        'impact': 'Read-only glob diagnostic failed; correct concrete file paths subsequently read. Not a required check and not hidden.'
    },
    'permissions': 'Only assigned allow_files, actual begin outputs.change/checks, and this new raw/implement-1 directory. No Sheltie commands, Rust build, installation, source Root modifications, or other arm access.',
})

_, identity = run('baseline-identity', ['git', 'show', '-s', '--format=%H%n%T', 'HEAD'])
assert identity.splitlines() == [BASE, BASE_TREE]
_, rs_files = run('source-file-list', ['git', 'ls-files', '*.rs'])
assert len(rs_files.splitlines()) == 192
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
