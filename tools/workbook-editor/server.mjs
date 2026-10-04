import http from 'node:http';
import { randomBytes } from 'node:crypto';
import { readFile, realpath, access } from 'node:fs/promises';
import { constants } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join, isAbsolute } from 'node:path';
import { zipSync } from 'fflate';
import { decodeEntries, readTree } from './lib/files.mjs';
import { checkWorkbook } from './lib/engine.mjs';
import { MAX_BODY } from './public/files.mjs';
const directory = dirname(fileURLToPath(import.meta.url));
const staticFiles = new Map([
  ['/', ['public/index.html', 'text/html; charset=utf-8']],
  ['/app.mjs', ['public/app.mjs', 'text/javascript; charset=utf-8']],
  ['/model.mjs', ['public/model.mjs', 'text/javascript; charset=utf-8']],
  ['/layout.mjs', ['public/layout.mjs', 'text/javascript; charset=utf-8']],
  ['/geometry.mjs', ['public/geometry.mjs', 'text/javascript; charset=utf-8']],
  ['/sources.mjs', ['public/sources.mjs', 'text/javascript; charset=utf-8']],
  ['/presentation.mjs', ['public/presentation.mjs', 'text/javascript; charset=utf-8']],
  ['/files.mjs', ['public/files.mjs', 'text/javascript; charset=utf-8']],
  ['/style.css', ['public/style.css', 'text/css; charset=utf-8']],
]);
const tomlFiles = ['index.js', 'parse.js', 'stringify.js', 'primitive.js', 'util.js', 'error.js', 'extract.js', 'date.js', 'struct.js'];
for (const file of tomlFiles) staticFiles.set(`/vendor/smol-toml/${file}`, [`node_modules/smol-toml/dist/${file}`, 'text/javascript; charset=utf-8']);
async function body(request) {
  const buffer = await new Promise((resolve, reject) => {
    let size = 0, finished = false;
    const parts = [];
    const fail = error => { if (!finished) { finished = true; parts.length = 0; reject(error); } };
    request.on('data', chunk => {
      if (finished) return;
      size += chunk.length;
      if (size > MAX_BODY) { const error = new Error('HTTP 请求体超过 24 MiB。'); error.status = 413; fail(error); return; }
      parts.push(chunk);
    });
    request.once('end', () => { if (!finished) { finished = true; resolve(Buffer.concat(parts)); } });
    request.once('error', fail);
    request.once('aborted', () => fail(new Error('请求断开或取消')));
  });
  try { return JSON.parse(buffer.toString('utf8')); }
  catch { throw new Error('请求体不是完整 JSON。'); }
}
export async function createEditor({ binary, port = 4311, processOptions, onRoot } = {}) {
  if (!binary || !isAbsolute(binary)) throw new Error('--sheltie 必须是可信引擎的绝对路径。');
  binary = await realpath(binary);
  await access(binary, constants.X_OK);
  if (!Number.isInteger(port) || port < 0 || port > 65535) throw new Error('端口必须为 0–65535。');
  const token = randomBytes(32).toString('hex');
  let origin;
  const active = new Set();
  const send = (response, status, value) => {
    if (response.destroyed) return;
    response.writeHead(status, { 'Content-Type': 'application/json; charset=utf-8' });
    response.end(JSON.stringify(value));
  };
  const server = http.createServer(async (request, response) => {
    response.setHeader('Cache-Control', 'no-store');
    response.setHeader('X-Content-Type-Options', 'nosniff');
    response.setHeader('Content-Security-Policy', "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' blob:; object-src 'none'; frame-ancestors 'none'; base-uri 'none'");
    if (request.headers.host !== new URL(origin).host) { send(response, 403, { ok: false, error: 'Host 不匹配本地监听地址。' }); return; }
    const path = request.url;
    const controller = new AbortController();
    request.on('aborted', () => controller.abort());
    response.on('close', () => { if (!response.writableEnded) controller.abort(); });
    try {
      if (request.method === 'GET' && path === '/api/session') { send(response, 200, { token }); return; }
      if (request.method === 'GET' && path === '/toml.mjs') { response.writeHead(200, { 'Content-Type': 'text/javascript; charset=utf-8' }); response.end("export { parse, stringify } from '/vendor/smol-toml/index.js';\n"); return; }
      if (request.method === 'GET' && staticFiles.has(path)) {
        const [file, type] = staticFiles.get(path);
        const bytes = await readFile(join(directory, file));
        response.writeHead(200, { 'Content-Type': type });
        response.end(bytes); return;
      }
      if (request.method === 'GET' && ['/api/sample/code-change', '/api/sample/spec-dev'].includes(path)) {
        const root = path.endsWith('/code-change') ? join(directory, '../../examples/code-change') : join(directory, '../../workbooks/spec-dev');
        const files = await readTree(root);
        send(response, 200, [...files].map(([p, bytes]) => [p, bytes.toString('base64')])); return;
      }
      if (request.method !== 'POST' || !['/api/check', '/api/export'].includes(path)) { send(response, 404, { ok: false, error: '没有此操作。' }); return; }
      if (request.headers.origin !== origin || request.headers['x-editor-token'] !== token) { send(response, 403, { ok: false, error: 'Origin 或会话 token 不匹配。' }); return; }
      if (request.headers['content-type'] !== 'application/json') { send(response, 415, { ok: false, error: 'Content-Type 必须为 application/json。' }); return; }
      const entries = await body(request);
      const files = decodeEntries(entries);
      const operation = checkWorkbook(files, binary, { signal: controller.signal, processOptions, onRoot });
      active.add(operation);
      let checked;
      try { checked = await operation; } finally { active.delete(operation); }
      if (!checked.ok) { send(response, 422, checked); return; }
      if (path === '/api/check') { send(response, 200, checked); return; }
      if (controller.signal.aborted) return;
      const zippable = Object.create(null);
      for (const [p, bytes] of files) zippable[p] = bytes;
      const zip = zipSync(zippable, { level: 6 });
      response.writeHead(200, { 'Content-Type': 'application/zip', 'Content-Disposition': 'attachment; filename="workbook.zip"', 'Content-Length': zip.byteLength });
      response.end(zip);
    } catch (error) { send(response, error.status ?? 400, { ok: false, error: error.message }); }
  });
  server.requestTimeout = 30000;
  server.headersTimeout = 10000;
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(port, '127.0.0.1', resolve); });
  origin = `http://127.0.0.1:${server.address().port}`;
  return { server, origin, token, close: async () => { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); await Promise.allSettled([...active]); } };
}
if (process.argv[1] && await realpath(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    if (args.length !== 4 || args[0] !== '--sheltie' || args[2] !== '--port') throw new Error('用法：node server.mjs --sheltie /absolute/path/sheltie --port 4311');
    const editor = await createEditor({ binary: args[1], port: Number(args[3]) });
    console.log(`Workbook 编辑器：${editor.origin}`);
    for (const event of ['SIGTERM', 'SIGINT']) process.once(event, async () => { await editor.close(); process.exit(0); });
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
