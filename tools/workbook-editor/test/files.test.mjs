import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { validateList, decodedSize, readDirectory, MAX_BYTES, MAX_FILES } from '../public/files.mjs';
import { decodeEntries, materialize, verifyMaterialized } from '../lib/files.mjs';
test('数量/总大小恰好上限接受，多一个拒绝', () => {
  assert.equal(validateList([['x', MAX_BYTES]]), MAX_BYTES);
  assert.throws(() => validateList([['x', MAX_BYTES + 1]]), /16 MiB/);
  assert.equal(validateList(Array.from({ length: MAX_FILES }, (_, i) => [`f${i}`, 0])), 0);
  assert.throws(() => validateList(Array.from({ length: MAX_FILES + 1 }, (_, i) => [`f${i}`, 0])), /1024/);
});
test('路径逃逸、重复、ASCII/NFC/目录别名和前缀冲突均拒绝', () => {
  for (const path of ['/x', '../x', 'a/../b', 'a//b', './a', 'C:x', 'a\\b', 'a\0b', 'a/']) assert.throws(() => validateList([[path, 0]]), /路径/);
  for (const paths of [['a', 'a'], ['A', 'a'], ['é', 'e\u0301'], ['Dir/a', 'dir/b'], ['a', 'a/b'], ['a/b', 'a']]) assert.throws(() => validateList(paths.map(p => [p, 0])), /冲突/);
  assert.equal(validateList([['dir/a', 1], ['dir/b', 2]]), 3);
  assert.equal(validateList([['a'.repeat(4096), 0]]), 0);
  assert.throws(() => validateList([['a'.repeat(4097), 0]]), /路径/);
});
test('严格 base64 与逐项数组；解码前拒绝大小和同名', () => {
  assert.equal(decodedSize('AAEC/w=='), 4);
  for (const value of ['A', 'YQ', 'YQ===', 'Y Q==', 'YQ==\n', '_w==', 'YR==', 'YWJ=']) assert.throws(() => decodedSize(value), /base64/);
  assert.throws(() => decodeEntries({ x: 'YQ==' }), /逐项/);
  assert.throws(() => decodeEntries([['a', 'YQ=='], ['a', 'Yg==']]), /冲突/);
  assert.deepEqual([...decodeEntries([['bin', 'AAEC/w==']]).values()][0], Buffer.from([0, 1, 2, 255]));
});
test('浏览器文件清单超限时从不读取；真实读取后大小再次核对', async () => {
  let reads = 0;
  const oversized = { webkitRelativePath: 'book/x', size: MAX_BYTES + 1, arrayBuffer: async () => { reads++; return new ArrayBuffer(0); } };
  await assert.rejects(readDirectory([oversized]), /16 MiB/); assert.equal(reads, 0);
  await assert.rejects(readDirectory([{ ...oversized, size: 1 }]), /大小改变/); assert.equal(reads, 1);
  const files = await readDirectory([{ webkitRelativePath: 'book/x', size: 2, arrayBuffer: async () => Uint8Array.from([0, 255]).buffer }]);
  assert.deepEqual(files.get('x'), Uint8Array.from([0, 255]));
});
test('物化独占创建并核实际文件集合和原字节', async () => {
  const root = await mkdtemp(join(tmpdir(), 'editor-files-test-'));
  try {
    const expected = new Map([['workbook.toml', Buffer.from('literal')], ['resources/binary', Buffer.from([0, 255])]]);
    await materialize(expected, join(root, 'book'));
    await writeFile(join(root, 'book/resources/binary'), Buffer.from([0, 254]));
    await assert.rejects(verifyMaterialized(expected, join(root, 'book')), /原字节/);
    await writeFile(join(root, 'book/resources/binary'), Buffer.from([0, 255]));
    await writeFile(join(root, 'book/extra'), 'extra');
    await assert.rejects(verifyMaterialized(expected, join(root, 'book')), /集合/);
  } finally { await rm(root, { recursive: true, force: true }); }
});
