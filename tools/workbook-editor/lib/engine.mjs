import { spawn } from 'node:child_process';
import { mkdtemp, chmod, rm, readdir, lstat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { materialize } from './files.mjs';
export async function removeOwnedRoot(root) {
  async function writable(path) {
    const info = await lstat(path);
    if (info.isSymbolicLink()) throw new Error(`Symbolic link in the owned root; preserved for inspection: ${root}`);
    if (info.isDirectory()) {
      await chmod(path, 0o700);
      for (const name of await readdir(path)) await writable(join(path, name));
    } else if (info.isFile()) await chmod(path, 0o600);
    else throw new Error(`Nonregular file in the owned root; preserved for inspection: ${root}`);
  }
  await writable(root);
  await rm(root, { recursive: true });
}
export function runChild(binary, argv, { home, signal, timeoutMs = 30000, outputLimit = 1048576, termGraceMs = 1000, closeWaitMs = 5000, onClose } = {}) {
  return new Promise(resolve => {
    const chunks = { stdout: [], stderr: [] };
    const counts = { stdout: 0, stderr: 0 };
    let reason = null, closed = false, exited = false, killTimer, closeTimer;
    const child = spawn(binary, argv, { env: { ...process.env, SHELTIE_HOME: home }, shell: false, stdio: ['ignore', 'pipe', 'pipe'] });
    const finish = (code, childSignal, confirmed) => {
      if (closed) return;
      closed = true;
      clearTimeout(timeout); clearTimeout(killTimer); clearTimeout(closeTimer);
      signal?.removeEventListener('abort', onAbort);
      resolve({ argv: [binary, ...argv], pid: child.pid ?? null, code, signal: childSignal, reason, exited, closed: confirmed, stdout: Buffer.concat(chunks.stdout).toString('utf8'), stderr: Buffer.concat(chunks.stderr).toString('utf8') });
    };
    const stop = why => {
      if (reason || closed) return;
      reason = why;
      child.kill('SIGTERM');
      killTimer = setTimeout(() => {
        if (!closed) child.kill('SIGKILL');
        closeTimer = setTimeout(() => finish(null, null, false), closeWaitMs);
      }, termGraceMs);
    };
    const onAbort = () => stop('Request disconnected or cancelled');
    const timeout = setTimeout(() => stop('CLI timed out'), timeoutMs);
    for (const stream of ['stdout', 'stderr']) {
      child[stream].on('data', data => {
        const available = Math.max(0, outputLimit - counts[stream]);
        if (available) chunks[stream].push(data.subarray(0, available));
        counts[stream] += data.length;
        if (counts[stream] > outputLimit) stop(`${stream} exceeds ${outputLimit} bytes`);
      });
    }
    child.on('error', error => { reason ??= `CLI startup failed: ${error.message}`; });
    child.on('exit', () => { exited = true; });
    child.on('close', (code, childSignal) => { onClose?.({ code, signal: childSignal }); finish(code, childSignal, true); });
    signal?.addEventListener('abort', onAbort, { once: true });
    if (signal?.aborted) onAbort();
  });
}
export async function checkWorkbook(files, binary, { signal, processOptions = {}, onRoot } = {}) {
  const root = await mkdtemp(join(tmpdir(), 'sheltie-editor-'));
  await chmod(root, 0o700);
  onRoot?.(root);
  let processResult, engineError, preserve = false;
  try {
    const workbook = join(root, 'workbook');
    const home = join(root, 'home');
    await materialize(files, workbook);
    if (signal?.aborted) throw new Error('Request disconnected or cancelled');
    processResult = await runChild(binary, ['--json', 'workbook', 'add', workbook], { ...processOptions, home, signal });
    if (!processResult.closed) { preserve = true; throw new Error(`Cannot confirm direct-child exit and stream closure; preserving ${root}`); }
    if (processResult.reason) throw new Error(processResult.reason);
    if (processResult.code !== 0) {
      try {
        const failure = JSON.parse(processResult.stdout);
        if (failure.ok === false && typeof failure.error?.code === 'string' && typeof failure.error?.message === 'string') engineError = failure.error;
      } catch { /* An incomplete failure stream remains a CLI failure; do not fabricate engine fields. */ }
      throw new Error(engineError?.message ?? 'The engine rejected the Workbook');
    }
    let result;
    try { result = JSON.parse(processResult.stdout); } catch { throw new Error('The engine did not return complete JSON'); }
    if (result.ok !== true || typeof result.request_id !== 'string' || !Array.isArray(result.next) || !result.data || typeof result.data.id !== 'string' || typeof result.data.version !== 'string' || !/^[0-9a-f]{64}$/.test(result.data.digest) || !Array.isArray(result.data.flows) || result.data.flows.some(v => typeof v !== 'string') || !Array.isArray(result.data.requires) || typeof result.data.replayed !== 'boolean') throw new Error('The engine did not return a complete success result');
    return { ok: true, result, process: processResult };
  } catch (error) {
    return { ok: false, error: error.message, process: processResult ?? null, ...(engineError ? { engineError } : {}), ...(preserve ? { residualRoot: root } : {}) };
  } finally {
    if (!preserve) await removeOwnedRoot(root);
  }
}
