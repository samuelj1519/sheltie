import { mkdir, open, readdir, readFile, lstat } from 'node:fs/promises';
import { join } from 'node:path';
import { validateWire, validateMap, validateList } from '../public/files.mjs';
export function decodeEntries(entries) {
  validateWire(entries);
  const files = new Map(entries.map(([path, encoded]) => [path, Buffer.from(encoded, 'base64')]));
  validateMap(files);
  return files;
}
export async function materialize(files, root) {
  validateMap(files);
  await mkdir(root, { mode: 0o700 });
  for (const [path, bytes] of files) {
    const parts = path.split('/');
    if (parts.length > 1) await mkdir(join(root, ...parts.slice(0, -1)), { recursive: true, mode: 0o700 });
    const file = await open(join(root, path), 'wx', 0o600);
    try { await file.writeFile(bytes); } finally { await file.close(); }
  }
  await verifyMaterialized(files, root);
}
export async function readTree(root) {
  const files = new Map(), entries = [];
  async function visit(dir, prefix) {
    for (const name of await readdir(dir)) {
      const path = prefix + name;
      const full = join(dir, name);
      const stat = await lstat(full);
      if (stat.isDirectory()) await visit(full, path + '/');
      else if (stat.isFile()) entries.push([path, stat.size]);
      else throw new Error(`Materialized set contains a nonregular file: ${path}`);
    }
  }
  await visit(root, '');
  validateList(entries);
  for (const [path, size] of entries) {
    const bytes = await readFile(join(root, path));
    if (bytes.length !== size) throw new Error(`File changed while reading: ${path}`);
    files.set(path, bytes);
  }
  validateMap(files);
  return files;
}
export async function verifyMaterialized(expected, root) {
  const actual = await readTree(root);
  if (actual.size !== expected.size) throw new Error('Materialized file set differs.');
  for (const [path, bytes] of expected) {
    if (!actual.has(path) || !Buffer.from(bytes).equals(actual.get(path))) throw new Error(`Materialized paths or original bytes differ: ${path}`);
  }
}
