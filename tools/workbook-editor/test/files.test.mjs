import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { validateList, decodedSize, readDirectory, MAX_BYTES, MAX_FILES } from '../public/files.mjs';
import { decodeEntries, materialize, verifyMaterialized } from '../lib/files.mjs';
test('Count/total-size exact limits are accepted and +1 is rejected', () => {
  assert.equal(validateList([['x', MAX_BYTES]]), MAX_BYTES);
  assert.throws(() => validateList([['x', MAX_BYTES + 1]]), /16 MiB/);
  assert.equal(validateList(Array.from({ length: MAX_FILES }, (_, i) => [`f${i}`, 0])), 0);
  assert.throws(() => validateList(Array.from({ length: MAX_FILES + 1 }, (_, i) => [`f${i}`, 0])), /1024/);
});
test('Reject escaping, duplicate, ASCII/NFC/directory aliases and prefix conflicts', () => {
  for (const path of ['/x', '../x', 'a/../b', 'a//b', './a', 'C:x', 'a\\b', 'a\0b', 'a/']) assert.throws(() => validateList([[path, 0]]), /path/);
  for (const paths of [['a', 'a'], ['A', 'a'], ['é', 'e\u0301'], ['Dir/a', 'dir/b'], ['a', 'a/b'], ['a/b', 'a']]) assert.throws(() => validateList(paths.map(p => [p, 0])), /conflict/);
  assert.equal(validateList([['dir/a', 1], ['dir/b', 2]]), 3);
  assert.equal(validateList([['a'.repeat(4096), 0]]), 0);
  assert.throws(() => validateList([['a'.repeat(4097), 0]]), /path/);
});
test('Strict base64 and entry arrays reject size and duplicate paths before decoding', () => {
  assert.equal(decodedSize('AAEC/w=='), 4);
  for (const value of ['A', 'YQ', 'YQ===', 'Y Q==', 'YQ==\n', '_w==', 'YR==', 'YWJ=']) assert.throws(() => decodedSize(value), /base64/);
  assert.throws(() => decodeEntries({ x: 'YQ==' }), /entry arrays/);
  assert.throws(() => decodeEntries([['a', 'YQ=='], ['a', 'Yg==']]), /conflict/);
  assert.deepEqual([...decodeEntries([['bin', 'AAEC/w==']]).values()][0], Buffer.from([0, 1, 2, 255]));
});
test('Oversized browser file lists are never read; actual read size is checked again', async () => {
  let reads = 0;
  const oversized = { webkitRelativePath: 'book/x', size: MAX_BYTES + 1, arrayBuffer: async () => { reads++; return new ArrayBuffer(0); } };
  await assert.rejects(readDirectory([oversized]), /16 MiB/); assert.equal(reads, 0);
  await assert.rejects(readDirectory([{ ...oversized, size: 1 }]), /size changed/); assert.equal(reads, 1);
  const files = await readDirectory([{ webkitRelativePath: 'book/x', size: 2, arrayBuffer: async () => Uint8Array.from([0, 255]).buffer }]);
  assert.deepEqual(files.get('x'), Uint8Array.from([0, 255]));
});
test('Exclusive materialization verifies the actual file set and original bytes', async () => {
  const root = await mkdtemp(join(tmpdir(), 'editor-files-test-'));
  try {
    const expected = new Map([['workbook.toml', Buffer.from('literal')], ['resources/binary', Buffer.from([0, 255])]]);
    await materialize(expected, join(root, 'book'));
    await writeFile(join(root, 'book/resources/binary'), Buffer.from([0, 254]));
    await assert.rejects(verifyMaterialized(expected, join(root, 'book')), /original bytes/);
    await writeFile(join(root, 'book/resources/binary'), Buffer.from([0, 255]));
    await writeFile(join(root, 'book/extra'), 'extra');
    await assert.rejects(verifyMaterialized(expected, join(root, 'book')), /file set/);
  } finally { await rm(root, { recursive: true, force: true }); }
});
