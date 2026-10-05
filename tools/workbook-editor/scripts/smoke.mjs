import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { resolve } from 'node:path';
import assert from 'node:assert/strict';
import { unzipSync } from 'fflate';
import { newWorkbook } from '../public/model.mjs';
import { toWire } from '../public/files.mjs';
const binary = process.env.SHELTIE_EDITOR_ENGINE;
if (!binary) throw new Error('SHELTIE_EDITOR_ENGINE requires a trusted absolute path');
const argv = [resolve('server.mjs'), '--sheltie', binary, '--port', '4311'];
const child = spawn(process.execPath, argv, { stdio: ['ignore', 'pipe', 'pipe'] });
const closed = once(child, 'close');
let stdout = '', stderr = '';
child.stdout.on('data', b => { stdout += b.toString(); }); child.stderr.on('data', b => { stderr += b.toString(); });
let deadline;
try {
  const origin = await new Promise((resolve, reject) => {
    deadline = setTimeout(() => reject(new Error('Startup did not return a local URL')), 10000);
    child.stdout.on('data', () => { const match = stdout.match(/http:\/\/127\.0\.0\.1:\d+/); if (match) resolve(match[0]); }); child.once('error', reject); child.once('exit', code => { if (code !== null) reject(new Error('Service exited early: ' + code + ' ' + stderr)); });
  }); clearTimeout(deadline);
  const html = await fetch(origin); assert.equal(html.status, 200); const page = await html.text(); assert.match(page, /id="viewport"/); assert.match(page, /<html lang="en">/); assert.match(page, /New Workbook/); assert.match(page, /Save new ZIP copy/);
  const session = await (await fetch(origin + '/api/session')).json();
  const model = newWorkbook(), payload = JSON.stringify(toWire(model.snapshot()));
  const headers = { 'Content-Type': 'application/json', Origin: origin, 'X-Editor-Token': session.token };
  const checked = await fetch(origin + '/api/check', { method: 'POST', headers, body: payload }); const result = await checked.json(); assert.equal(checked.status, 200); assert.equal(result.ok, true);
  const exported = await fetch(origin + '/api/export', { method: 'POST', headers, body: payload }); assert.equal(exported.status, 200); const zip = new Uint8Array(await exported.arrayBuffer()); const files = unzipSync(zip);
  for (const [path, bytes] of model.snapshot()) assert.deepEqual(Buffer.from(files[path]), Buffer.from(bytes));
  console.log(JSON.stringify({ boundary: 'standalone-entry-smoke', argv: [process.execPath, ...argv], pid: child.pid, origin, check: result, zipBytes: zip.length, files: Object.keys(files) }));
} finally {
  clearTimeout(deadline); child.kill('SIGTERM'); const [code, signal] = await closed; console.log(JSON.stringify({ boundary: 'standalone-entry-close', pid: child.pid, code, signal, stdout, stderr })); assert.equal(code, 0);
}
