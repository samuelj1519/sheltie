import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm, writeFile, access, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { checkWorkbook, runChild } from '../lib/engine.mjs';
import { materialize, readTree } from '../lib/files.mjs';
import { newWorkbook } from '../public/model.mjs';
const engine = process.env.SHELTIE_EDITOR_ENGINE;
if (!engine) throw new Error('SHELTIE_EDITOR_ENGINE 必须指向本次可信引擎；不能跳过真实 CLI 测试。');
export function cli(argv, home) {
  const result = spawnSync(engine, argv, { env: { ...process.env, SHELTIE_HOME: home }, encoding: 'utf8', timeout: 30000, maxBuffer: 1048576 });
  console.log(JSON.stringify({ boundary: 'real-cli', argv: [engine, ...argv], home, stdout: result.stdout, stderr: result.stderr, exit: result.status, signal: result.signal }));
  return result;
}
test('真实 CLI 装入两步和现有 code-change/spec-dev，非法入口/未知字段拒绝', async () => {
  for (const path of ['examples/two-step', 'examples/code-change', 'workbooks/spec-dev']) {
    const checked = await checkWorkbook(await readTree(resolve('../..', path)), engine);
    console.log(JSON.stringify({ boundary: 'real-cli', checked }));
    assert.equal(checked.ok, true, path + ': ' + JSON.stringify(checked));
  }
  const missing = newWorkbook(); missing.edit('flows/default.toml', missing.flow('flows/default.toml'), 'entry', 'absent');
  const rejected = await checkWorkbook(missing.snapshot(), engine); assert.equal(rejected.ok, false); assert.match(rejected.process.stdout, /FLOW_INVALID/);
  const unknown = newWorkbook(); unknown.edit('flows/default.toml', unknown.flow('flows/default.toml').nodes[0], 'mystery', 7); unknown.edit('flows/default.toml', unknown.flow('flows/default.toml').nodes[0], 'title', '改标题');
  const denied = await checkWorkbook(unknown.snapshot(), engine); assert.equal(denied.ok, false); assert.match(denied.process.stdout, /unknown field.*mystery/);
  console.log(JSON.stringify({ boundary: 'real-cli-negative', rejected, denied }));
});
test('合法高级声明、门槛/额度/requires/结果/回环通过真实 CLI', async () => {
  const files = await readTree(resolve('../../examples/code-change'));
  const { WorkbookModel } = await import('../public/model.mjs');
  const model = new WorkbookModel(files), flow = model.flow('flows/default.toml');
  model.edit('workbook.toml', model.manifest, 'requires', [{ kind: 'skill', name: 'sample', version: '^1', source: 'https://example.com/skill' }]);
  model.edit('flows/default.toml', flow.nodes[0], 'requires', ['skill:sample']);
  model.edit('flows/default.toml', flow.nodes[0], 'gate', true);
  const checked = await checkWorkbook(model.snapshot(), engine); console.log(JSON.stringify({ boundary: 'real-cli-advanced', checked })); assert.equal(checked.ok, true);
});
test('CLI 失败和 TERM→KILL 超时观察退出/close 后清自有根；外部哨兵不改', async () => {
  const scripts = await mkdtemp(join(tmpdir(), 'editor-child-test-'));
  try {
    const sentinel = join(scripts, 'sentinel'); await writeFile(sentinel, 'keep');
    const fake = join(scripts, 'fake'); await writeFile(fake, '#!' + process.execPath + '\nprocess.on("SIGTERM",()=>{});setInterval(()=>{},100);\n', { mode: 0o700 });
    let root;
    const checked = await checkWorkbook(newWorkbook().snapshot(), fake, { onRoot: value => { root = value; }, processOptions: { timeoutMs: 1000, termGraceMs: 80, closeWaitMs: 1000 } });
    console.log(JSON.stringify({ boundary: 'fake-child-timeout', checked, root }));
    assert.equal(checked.ok, false); assert.match(checked.error, /超时/); assert.equal(checked.process.closed, true); assert.equal(checked.process.exited, true); assert.equal(checked.process.signal, 'SIGKILL'); await assert.rejects(access(root)); assert.equal((await stat(sentinel)).size, 4);
    await writeFile(fake, '#!/usr/bin/env node\nprocess.stderr.write("bad");process.exit(7);\n', { mode: 0o700 });
    const failed = await checkWorkbook(newWorkbook().snapshot(), fake, { onRoot: value => { root = value; } }); assert.equal(failed.process.code, 7); await assert.rejects(access(root));
  } finally { await rm(scripts, { recursive: true, force: true }); }
});
test('各输出流超限/取消均终止直接 child', async () => {
  for (const stream of ['stdout', 'stderr']) {
    const result = await runChild(process.execPath, ['-e', `process.${stream}.write('x'.repeat(4097));setInterval(()=>{},100);`], { home: '/unused', outputLimit: 4096 });
    assert.match(result.reason, /超过/); assert.equal(result[stream].length, 4096); assert.equal(result.closed, true); assert.equal(result.exited, true);
  }
  const controller = new AbortController();
  const running = runChild(process.execPath, ['-e', 'setInterval(()=>{},100)'], { home: '/unused', signal: controller.signal });
  controller.abort(); const result = await running; assert.match(result.reason, /取消/); assert.equal(result.closed, true);
});
test('默认各流1MiB恰好接受，多一字节停止；不完整JSON不授成功', async () => {
  for (const stream of ['stdout', 'stderr']) {
    const accepted = await runChild(process.execPath, ['-e', `process.${stream}.write('x'.repeat(1048576))`], { home: '/unused' });
    assert.equal(accepted.reason, null); assert.equal(accepted.code, 0); assert.equal(accepted[stream].length, 1048576);
    const rejected = await runChild(process.execPath, ['-e', `process.${stream}.write('x'.repeat(1048577));setInterval(()=>{},100)`], { home: '/unused' });
    assert.match(rejected.reason, /超过 1048576/); assert.equal(rejected.closed, true);
  }
  const root = await mkdtemp(join(tmpdir(), 'editor-incomplete-test-'));
  try {
    const fake = join(root, 'fake');
    for (const output of ['{', '{"ok":true}']) {
      await writeFile(fake, '#!' + process.execPath + '\nprocess.stdout.write(' + JSON.stringify(output) + ');\n', { mode: 0o700 });
      const checked = await checkWorkbook(newWorkbook().snapshot(), fake); assert.equal(checked.ok, false); assert.match(checked.error, /JSON|完整成功/);
    }
  } finally { await rm(root, { recursive: true, force: true }); }
});
test('直接child退出而流未关闭时准确失败保留自有根，实际close之后才能清理', async () => {
  const scripts = await mkdtemp(join(tmpdir(), 'editor-held-stream-test-'));
  let root, observedClose;
  const closed = new Promise(resolve => { observedClose = resolve; });
  try {
    const fake = join(scripts, 'fake');
    await writeFile(fake, '#!' + process.execPath + '\nconst {spawn}=require("node:child_process");spawn(process.execPath,["-e","setTimeout(()=>{},1200)"],{stdio:["ignore","inherit","inherit"]});process.exit(0);\n', { mode: 0o700 });
    const checked = await checkWorkbook(newWorkbook().snapshot(), fake, { onRoot: value => { root = value; }, processOptions: { timeoutMs: 400, termGraceMs: 50, closeWaitMs: 50, onClose: observedClose } });
    assert.equal(checked.ok, false); assert.equal(checked.process.exited, true); assert.equal(checked.process.closed, false); assert.equal(checked.residualRoot, root); await access(root);
    console.log(JSON.stringify({ boundary: 'fake-held-stream', checked }));
    await closed;
    const { removeOwnedRoot } = await import('../lib/engine.mjs'); await removeOwnedRoot(root); root = null;
  } finally { await rm(scripts, { recursive: true, force: true }); }
});
