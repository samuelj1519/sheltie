import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import { mkdtemp, rm, access, stat, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { unzipSync } from 'fflate';
import { createEditor } from '../server.mjs';
import { newWorkbook, WorkbookModel } from '../public/model.mjs';
import { readTree, materialize } from '../lib/files.mjs';
import { removeOwnedRoot } from '../lib/engine.mjs';
import { MAX_BODY, MAX_BYTES, toWire } from '../public/files.mjs';
const binary = process.env.SHELTIE_EDITOR_ENGINE;
if (!binary) throw new Error('必须提供真实引擎');
function request(editor, { path = '/api/check', method = 'POST', headers = {}, payload = '[]', omit = [] } = {}) {
  const base = { Host: new URL(editor.origin).host, Origin: editor.origin, 'X-Editor-Token': editor.token, 'Content-Type': 'application/json' };
  for (const key of omit) delete base[key];
  return new Promise((resolve, reject) => {
    const req = http.request(editor.origin + path, { method, headers: { ...base, ...headers }, setHost: false }, response => {
      const chunks = []; response.on('data', data => chunks.push(data)); response.on('end', () => resolve({ status: response.statusCode, headers: response.headers, bytes: Buffer.concat(chunks) }));
    }); req.on('error', reject); req.end(method === 'GET' ? undefined : payload);
  });
}
function cli(argv, home) {
  const result = spawnSync(binary, argv, { env: { ...process.env, SHELTIE_HOME: home }, encoding: 'utf8', timeout: 30000, maxBuffer: 1048576 });
  console.log(JSON.stringify({ boundary: 'real-cli-zip', argv: [binary, ...argv], home, stdout: result.stdout, stderr: result.stderr, exit: result.status })); assert.equal(result.status, 0); return JSON.parse(result.stdout);
}
test('真实 HTTP Host/Origin/token 拒绝矩阵，固定静态路由和样例', async () => {
  const roots = [], editor = await createEditor({ binary, port: 0, onRoot: root => roots.push(root) });
  try {
    for (const item of [
      { omit: ['Host'] }, { headers: { Host: 'localhost:' + new URL(editor.origin).port } }, { headers: { Host: 'evil.test' } },
      { omit: ['Origin'] }, { headers: { Origin: 'null' } }, { headers: { Origin: 'https://evil.test' } }, { headers: { Origin: editor.origin + '/' } },
      { omit: ['X-Editor-Token'] }, { headers: { 'X-Editor-Token': 'wrong' } },
    ]) assert.equal((await request(editor, item)).status, item.omit?.includes('Host') ? 400 : 403, JSON.stringify(item));
    assert.equal(roots.length, 0);
    for (const path of ['/', '/app.mjs', '/model.mjs', '/layout.mjs', '/presentation.mjs', '/toml.mjs', '/vendor/smol-toml/date.js', '/vendor/smol-toml/struct.js']) assert.equal((await request(editor, { path, method: 'GET' })).status, 200, path);
    for (const name of ['geometry', 'sources']) {
      const served = await request(editor, { path: `/${name}.mjs`, method: 'GET' });
      assert.equal(served.status, 200); assert.match(served.headers['content-type'], /^text\/javascript/);
      assert.deepEqual(served.bytes, await readFile(new URL(`../public/${name}.mjs`, import.meta.url)));
    }
    const servedApp = await request(editor, { path: '/app.mjs', method: 'GET' });
    assert.match(servedApp.bytes.toString(), /from '\.\/geometry\.mjs'/); assert.match(servedApp.bytes.toString(), /from '\.\/sources\.mjs'/);
    for (const path of ['/../../etc/passwd', '/api/sample/../../etc/passwd', '/vendor/smol-toml/../../package.json', '/api/arbitrary']) assert.equal((await request(editor, { path, method: 'GET' })).status, 404);
    const sample = await request(editor, { path: '/api/sample/code-change', method: 'GET' }); assert.equal(sample.status, 200); const entries = JSON.parse(sample.bytes); assert.ok(entries.some(([p]) => p === 'flows/default.toml'));
    console.log(JSON.stringify({ boundary: 'real-http', origin: editor.origin, matrix: '9 forbidden before materialization, 8 existing static + 2 exact-byte production modules + fixed sample, 4 unknown routes' }));
  } finally { await editor.close(); }
});
test('真实 HTTP 逐项路径/base64/数量/16MiB 与24MiB流式限额；所有前检不物化', async () => {
  const roots = [], editor = await createEditor({ binary, port: 0, onRoot: root => roots.push(root) });
  try {
    const invalid = [{ x: '' }, [['../x', '']], [['a', ''], ['a', '']], [['Dir/a', ''], ['dir/b', '']], [['a', ''], ['a/b', '']], [['x', 'YR==']], Array.from({ length: 1025 }, (_, i) => [`f${i}`, '']), [['x', Buffer.alloc(MAX_BYTES + 1).toString('base64')]]];
    for (const payload of invalid) assert.equal((await request(editor, { payload: JSON.stringify(payload) })).status, 400);
    assert.equal((await request(editor, { payload: Buffer.alloc(MAX_BODY, 32) })).status, 400);
    assert.equal((await request(editor, { payload: Buffer.alloc(MAX_BODY + 1, 32) })).status, 413);
    assert.equal(roots.length, 0);
    const exactBytes = await request(editor, { payload: JSON.stringify([['large', Buffer.alloc(MAX_BYTES).toString('base64')]]) }); assert.equal(exactBytes.status, 422);
    const exactFiles = await request(editor, { payload: JSON.stringify(Array.from({ length: 1024 }, (_, i) => [`f${i}`, ''])) }); assert.equal(exactFiles.status, 422);
    for (const root of roots) await assert.rejects(access(root));
    console.log(JSON.stringify({ boundary: 'real-http-limits', precheckRoots: 0, exactLimitRootsRemoved: roots.length, invalidCases: invalid.length, exactBody: MAX_BODY, oneOverBody: MAX_BODY + 1 }));
  } finally { await editor.close(); }
});
test('真实 HTTP check/export 同字节 ZIP 解压后 add/show/verify；失败绝无成功 ZIP', async () => {
  const roots = [], editor = await createEditor({ binary, port: 0, onRoot: root => roots.push(root) });
  const root = await mkdtemp(join(tmpdir(), 'editor-zip-test-'));
  try {
    const files = await readTree(resolve('../../examples/code-change')); files.set('resources/binary', Buffer.from([0, 255, 1, 128, 13, 10]));
    const model = new WorkbookModel(files); model.edit('flows/default.toml', model.flow('flows/default.toml').nodes[0], 'title', '真实 HTTP 编辑');
    const exact = model.snapshot(), payload = JSON.stringify(toWire(exact));
    const checked = await request(editor, { payload }); assert.equal(checked.status, 200, checked.bytes.toString()); assert.equal(JSON.parse(checked.bytes).ok, true);
    const exported = await request(editor, { path: '/api/export', payload }); assert.equal(exported.status, 200, exported.bytes.toString()); assert.equal(exported.headers['content-type'], 'application/zip');
    const unzipped = unzipSync(exported.bytes); assert.deepEqual(Object.keys(unzipped).sort(), [...exact.keys()].sort()); for (const [path, bytes] of exact) assert.deepEqual(Buffer.from(unzipped[path]), Buffer.from(bytes), path);
    await materialize(new Map(Object.entries(unzipped)), join(root, 'unzipped'));
    const home = join(root, 'home'); const added = cli(['--json', 'workbook', 'add', join(root, 'unzipped')], home); assert.equal(added.data.id, 'code-change');
    const shown = cli(['--json', 'workbook', 'show', 'code-change@1.0.0'], home); assert.equal(shown.ok, true); assert.match(JSON.stringify(shown.data), /真实 HTTP 编辑/);
    const verified = cli(['--json', 'workbook', 'verify', 'code-change@1.0.0'], home); assert.equal(verified.ok, true); assert.match(JSON.stringify(verified.data), /"status":"ok"/);
    model.edit('flows/default.toml', model.flow('flows/default.toml'), 'entry', 'absent');
    const failed = await request(editor, { path: '/api/export', payload: JSON.stringify(toWire(model.snapshot())) }); assert.equal(failed.status, 422); assert.equal(failed.headers['content-type'], 'application/json; charset=utf-8'); assert.match(failed.bytes.toString(), /FLOW_INVALID/); assert.equal(JSON.parse(failed.bytes).engineError.code, 'FLOW_INVALID'); assert.equal(JSON.parse(failed.bytes).engineError.detail.path, 'entry');
    for (const owned of roots) await assert.rejects(access(owned));
    console.log(JSON.stringify({ boundary: 'real-http-zip', origin: editor.origin, zipBytes: exported.bytes.length, files: exact.size, ownedRootsRemoved: roots }));
  } finally { await editor.close(); await removeOwnedRoot(root); }
});
test('真实HTTP断开终止归属直接child，观察close后清自有根', async () => {
  const scripts = await mkdtemp(join(tmpdir(), 'editor-http-disconnect-'));
  let root, seenRoot, seenClose;
  const readyRoot = new Promise(resolve => { seenRoot = resolve; }), closed = new Promise(resolve => { seenClose = resolve; });
  const fake = join(scripts, 'fake');
  await writeFile(fake, '#!' + process.execPath + '\nconst fs=require("node:fs"),path=require("node:path");process.on("SIGTERM",()=>{});fs.writeFileSync(path.join(process.argv.at(-1),"..","ready"),String(process.pid));setInterval(()=>{},100);\n', { mode: 0o700 });
  const editor = await createEditor({ binary: fake, port: 0, onRoot: value => { root = value; seenRoot(value); }, processOptions: { timeoutMs: 5000, termGraceMs: 50, closeWaitMs: 1000, onClose: seenClose } });
  try {
    const req = http.request(editor.origin + '/api/check', { method: 'POST', headers: { Host: new URL(editor.origin).host, Origin: editor.origin, 'X-Editor-Token': editor.token, 'Content-Type': 'application/json' } });
    req.on('error', () => {}); req.end(JSON.stringify(toWire(newWorkbook().snapshot())));
    await readyRoot;
    let pid;
    for (let i = 0; i < 100; i++) { try { pid = Number(await readFile(join(root, 'ready'), 'utf8')); break; } catch { await new Promise(resolve => setTimeout(resolve, 10)); } }
    assert.ok(Number.isInteger(pid) && pid > 0); assert.equal((await stat(root)).mode & 0o777, 0o700);
    req.destroy(); const result = await closed; assert.equal(result.signal, 'SIGKILL');
    for (let i = 0; i < 100; i++) { try { await access(root); await new Promise(resolve => setTimeout(resolve, 10)); } catch { break; } }
    await assert.rejects(access(root)); assert.throws(() => process.kill(pid, 0), { code: 'ESRCH' });
    console.log(JSON.stringify({ boundary: 'fake-child-real-http-disconnect', argv: [fake, '--json', 'workbook', 'add', join(root, 'workbook')], pid, closed: result, removedRoot: root }));
  } finally { await editor.close(); await rm(scripts, { recursive: true, force: true }); }
});
