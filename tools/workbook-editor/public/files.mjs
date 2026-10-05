export const MAX_FILES = 1024;
export const MAX_BYTES = 16 * 1024 * 1024;
export const MAX_BODY = 24 * 1024 * 1024;
const utf8 = new TextEncoder();
const fold = value => value.normalize('NFC').replace(/[A-Z]/g, c => c.toLowerCase());
export function validateList(items) {
  if (!Array.isArray(items) || items.length === 0 || items.length > MAX_FILES) throw new Error('File count must be 1–1024.');
  const prefixes = new Map();
  let total = 0;
  for (const item of items) {
    const [path, size] = item;
    if (typeof path !== 'string' || !path || utf8.encode(path).length > 4096 || /[\\\0:]/u.test(path) || path.startsWith('/') || path.split('/').some(p => !p || p === '.' || p === '..')) throw new Error(`Unsafe relative path: ${String(path)}`);
    if (!Number.isSafeInteger(size) || size < 0 || size > MAX_BYTES || (total += size) > MAX_BYTES) throw new Error('Total file size exceeds 16 MiB.');
    const parts = path.split('/');
    for (let i = 1; i <= parts.length; i++) {
      const spelling = parts.slice(0, i).join('/');
      const key = fold(spelling);
      const type = i === parts.length ? 'file' : 'directory';
      const previous = prefixes.get(key);
      if (previous && (previous.spelling !== spelling || previous.type !== type || type === 'file')) throw new Error(`Duplicate, directory alias or file-prefix conflict: ${path} (${previous.spelling})`);
      prefixes.set(key, { spelling, type });
    }
  }
  return total;
}
export function decodedSize(base64) {
  if (typeof base64 !== 'string' || base64.length > Math.ceil(MAX_BYTES / 3) * 4 || base64.length % 4 !== 0 || /[^A-Za-z0-9+/=]/.test(base64)) throw new Error('Invalid base64 encoding.');
  const padding = base64.endsWith('==') ? 2 : base64.endsWith('=') ? 1 : 0;
  if (base64.includes('=') && base64.indexOf('=') !== base64.length - padding) throw new Error('Invalid base64 padding position.');
  const size = base64.length / 4 * 3 - padding;
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  if ((base64.endsWith('==') && (alphabet.indexOf(base64.at(-3)) & 15)) || (base64.endsWith('=') && !base64.endsWith('==') && (alphabet.indexOf(base64.at(-2)) & 3))) throw new Error('Invalid base64 padding bits.');
  return size;
}
export function validateWire(entries) {
  if (!Array.isArray(entries) || entries.some(e => !Array.isArray(e) || e.length !== 2)) throw new Error('Files must be transmitted as entry arrays [[path, base64], …].');
  if (entries.length === 0 || entries.length > MAX_FILES) throw new Error('File count must be 1–1024.');
  const sizes = entries.map(([p, b]) => [p, decodedSize(b)]);
  validateList(sizes);
  return sizes;
}
export function validateMap(files) {
  if (!(files instanceof Map) || [...files.values()].some(v => !(v instanceof Uint8Array))) throw new Error('The file set must be a Map of original bytes.');
  validateList([...files].map(([p, b]) => [p, b.byteLength]));
}
export async function readDirectory(fileList) {
  const files = Array.from(fileList);
  const paths = files.map(f => {
    if (typeof f.webkitRelativePath !== 'string' || !f.webkitRelativePath.includes('/')) throw new Error('Choose a Workbook directory.');
    return f.webkitRelativePath.slice(f.webkitRelativePath.indexOf('/') + 1);
  });
  const roots = new Set(files.map(f => f.webkitRelativePath.split('/')[0]));
  if (roots.size !== 1) throw new Error('Open one directory at a time.');
  validateList(files.map((f, i) => [paths[i], f.size]));
  const result = new Map();
  for (let i = 0; i < files.length; i++) {
    const bytes = new Uint8Array(await files[i].arrayBuffer());
    if (bytes.byteLength !== files[i].size) throw new Error(`File size changed while reading: ${paths[i]}`);
    result.set(paths[i], bytes);
  }
  validateMap(result);
  return result;
}
export function toWire(files) {
  validateMap(files);
  return [...files].map(([path, bytes]) => {
    let text = '';
    for (let i = 0; i < bytes.length; i += 8192) text += String.fromCharCode(...bytes.subarray(i, i + 8192));
    return [path, btoa(text)];
  });
}
