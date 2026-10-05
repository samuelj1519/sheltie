import test from 'node:test';
import assert from 'node:assert/strict';
import { resolve } from 'node:path';
import { CanvasView } from '../public/layout.mjs';
import { WorkbookModel } from '../public/model.mjs';
import { readTree } from '../lib/files.mjs';
function storage() {
  const values = new Map();
  return { getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value), values };
}
test('Real app layout drag/pan/zoom/fit/save/load preserves every Workbook file byte', async () => {
  const source = await readTree(resolve('../../examples/code-change'));
  source.set('resources/binary', Buffer.from([0, 255, 13, 10, 128]));
  const model = new WorkbookModel(source), cache = storage(), key = 'layout:code-change:default';
  const view = CanvasView.load(cache, key);
  assert.deepEqual(view.position('implement', 0), { x: 0, y: 0 });
  view.position('review', 1); view.position('deliver', 2);
  view.move({ type: 'node', id: 'implement', startX: 100, startY: 200, x: 0, y: 0 }, 135, 250);
  assert.deepEqual(view.positions.get('implement'), { x: 35, y: 50 });
  view.zoom(2, 100, 100); assert.equal(view.scale, 2); assert.equal(view.x, -10); assert.equal(view.y, -10);
  view.move({ type: 'pan', startX: 50, startY: 60, x: -10, y: -10 }, 70, 80);
  assert.equal(view.x, 10); assert.equal(view.y, 10);
  view.save(cache, key);
  assert.deepEqual(JSON.parse(cache.values.get(key)), { x: 10, y: 10, scale: 2, positions: [['implement', { x: 35, y: 50 }], ['review', { x: 280, y: 0 }], ['deliver', { x: 560, y: 0 }]] });
  const reopened = CanvasView.load(cache, key);
  assert.deepEqual(reopened.positions.get('implement'), { x: 35, y: 50 }); assert.equal(reopened.scale, 2);
  reopened.fit(['implement', 'review', 'deliver'], 1000, 600);
  assert.equal(reopened.scale, 1); assert.equal(reopened.x, 40); assert.equal(reopened.y, 40);
  reopened.save(cache, key);
  const after = model.snapshot();
  assert.deepEqual([...after.keys()].sort(), [...source.keys()].sort());
  for (const [path, expected] of source) assert.deepEqual(Buffer.from(after.get(path)), Buffer.from(expected), path);
});
test('Corrupt layout cache and invalid coordinates fall back to the default view without writing the Workbook', () => {
  const cache = storage();
  cache.setItem('bad', '{'); const fallback = CanvasView.load(cache, 'bad'); assert.equal(fallback.x, 45); assert.equal(fallback.scale, 1); assert.equal(fallback.positions.size, 0);
  cache.setItem('bad-scale', '{"x":1,"y":2,"scale":3,"positions":[]}'); assert.equal(CanvasView.load(cache, 'bad-scale').scale, 1);
  cache.setItem('partial', '{"x":1,"y":2,"scale":1,"positions":[["valid",{"x":7,"y":8}],["invalid",{"x":"HTML","y":9}]]}');
  const partial = CanvasView.load(cache, 'partial'); assert.deepEqual([...partial.positions], [['valid', { x: 7, y: 8 }]]);
  assert.throws(() => partial.move({ type: 'unknown', startX: 0, startY: 0 }, 1, 1), /[Uu]nknown/);
});
