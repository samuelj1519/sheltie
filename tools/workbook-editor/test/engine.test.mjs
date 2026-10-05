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
if (!engine) throw new Error('SHELTIE_EDITOR_ENGINE must point to this verified engine; real CLI tests cannot be skipped.');
export function cli(argv, home) {
  const result = spawnSync(engine, argv, { env: { ...process.env, SHELTIE_HOME: home }, encoding: 'utf8', timeout: 30000, maxBuffer: 1048576 });
  console.log(JSON.stringify({ boundary: 'real-cli', argv: [engine, ...argv], home, stdout: result.stdout, stderr: result.stderr, exit: result.status, signal: result.signal }));
  return result;
}
test('Real CLI loads two-step and code-change/spec-dev; invalid entry and unknown fields are rejected', async () => {
  for (const path of ['examples/two-step', 'examples/code-change', 'workbooks/spec-dev']) {
    const checked = await checkWorkbook(await readTree(resolve('../..', path)), engine);
    console.log(JSON.stringify({ boundary: 'real-cli', checked }));
    assert.equal(checked.ok, true, path + ': ' + JSON.stringify(checked));
  }
  const missing = newWorkbook(); missing.edit('flows/default.toml', missing.flow('flows/default.toml'), 'entry', 'absent');
  const rejected = await checkWorkbook(missing.snapshot(), engine); assert.equal(rejected.ok, false); assert.match(rejected.process.stdout, /FLOW_INVALID/);
  const unknown = newWorkbook(); unknown.edit('flows/default.toml', unknown.flow('flows/default.toml').nodes[0], 'mystery', 7); unknown.edit('flows/default.toml', unknown.flow('flows/default.toml').nodes[0], 'title', 'changed-title');
  const denied = await checkWorkbook(unknown.snapshot(), engine); assert.equal(denied.ok, false); assert.match(denied.process.stdout, /unknown field.*mystery/);
  console.log(JSON.stringify({ boundary: 'real-cli-negative', rejected, denied }));
});
test('Valid advanced declarations, gates, limits, requires, results and loops pass the real CLI', async () => {
  const files = await readTree(resolve('../../examples/code-change'));
  const { WorkbookModel } = await import('../public/model.mjs');
  const model = new WorkbookModel(files), flow = model.flow('flows/default.toml');
  model.edit('workbook.toml', model.manifest, 'requires', [{ kind: 'skill', name: 'sample', version: '^1', source: 'https://example.com/skill' }]);
  model.edit('flows/default.toml', flow.nodes[0], 'requires', ['skill:sample']);
  model.edit('flows/default.toml', flow.nodes[0], 'gate', true);
  const checked = await checkWorkbook(model.snapshot(), engine); console.log(JSON.stringify({ boundary: 'real-cli-advanced', checked })); assert.equal(checked.ok, true);
});
test('CLI failure and TERM→KILL timeout clean owned roots after exit/close; external sentinels remain intact', async () => {
  const scripts = await mkdtemp(join(tmpdir(), 'editor-child-test-'));
  try {
    const sentinel = join(scripts, 'sentinel'); await writeFile(sentinel, 'keep');
    const fake = join(scripts, 'fake'); await writeFile(fake, '#!' + process.execPath + '\nprocess.on("SIGTERM",()=>{});process.stdout.write("READY\\n");setInterval(()=>{},100);\n', { mode: 0o700 });
    let root;
    // Allow interpreter startup and prove the TERM handler is ready before exercising KILL.
    const checked = await checkWorkbook(newWorkbook().snapshot(), fake, { onRoot: value => { root = value; }, processOptions: { timeoutMs: 5000, termGraceMs: 80, closeWaitMs: 1000 } });
    console.log(JSON.stringify({ boundary: 'fake-child-timeout', checked, root }));
    assert.equal(checked.ok, false); assert.match(checked.error, /timed out/); assert.equal(checked.process.closed, true); assert.equal(checked.process.exited, true); assert.equal(checked.process.stdout, 'READY\n'); assert.equal(checked.process.signal, 'SIGKILL'); await assert.rejects(access(root)); assert.equal((await stat(sentinel)).size, 4);
    await writeFile(fake, '#!/usr/bin/env node\nprocess.stderr.write("bad");process.exit(7);\n', { mode: 0o700 });
    const failed = await checkWorkbook(newWorkbook().snapshot(), fake, { onRoot: value => { root = value; } }); assert.equal(failed.process.code, 7); await assert.rejects(access(root));
  } finally { await rm(scripts, { recursive: true, force: true }); }
});
test('Each stream limit and cancellation terminates the direct child', async () => {
  for (const stream of ['stdout', 'stderr']) {
    const result = await runChild(process.execPath, ['-e', `process.${stream}.write('x'.repeat(4097));setInterval(()=>{},100);`], { home: '/unused', outputLimit: 4096 });
    assert.match(result.reason, /exceeds/); assert.equal(result[stream].length, 4096); assert.equal(result.closed, true); assert.equal(result.exited, true);
  }
  const controller = new AbortController();
  const running = runChild(process.execPath, ['-e', 'setInterval(()=>{},100)'], { home: '/unused', signal: controller.signal });
  controller.abort(); const result = await running; assert.match(result.reason, /cancelled/); assert.equal(result.closed, true);
});
test('Each stream accepts exactly 1MiB and stops at +1 byte; incomplete JSON never qualifies as success', async () => {
  for (const stream of ['stdout', 'stderr']) {
    const accepted = await runChild(process.execPath, ['-e', `process.${stream}.write('x'.repeat(1048576))`], { home: '/unused' });
    assert.equal(accepted.reason, null); assert.equal(accepted.code, 0); assert.equal(accepted[stream].length, 1048576);
    const rejected = await runChild(process.execPath, ['-e', `process.${stream}.write('x'.repeat(1048577));setInterval(()=>{},100)`], { home: '/unused' });
    assert.match(rejected.reason, /exceeds 1048576/); assert.equal(rejected.closed, true);
  }
  const root = await mkdtemp(join(tmpdir(), 'editor-incomplete-test-'));
  try {
    const fake = join(root, 'fake');
    for (const output of ['{', '{"ok":true}']) {
      await writeFile(fake, '#!' + process.execPath + '\nprocess.stdout.write(' + JSON.stringify(output) + ');\n', { mode: 0o700 });
      const checked = await checkWorkbook(newWorkbook().snapshot(), fake); assert.equal(checked.ok, false); assert.match(checked.error, /JSON|complete success/);
    }
  } finally { await rm(root, { recursive: true, force: true }); }
});
test('Direct-child exit without stream closure fails and preserves the root until actual close', async () => {
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
