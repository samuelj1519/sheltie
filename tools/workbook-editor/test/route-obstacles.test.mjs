import test from 'node:test';
import assert from 'node:assert/strict';
import { routeEdges, CARD_WIDTH, CARD_HEIGHT, pickRouteAtPoint } from '../public/geometry.mjs';
const flow = { entry: 'a', nodes: ['a', 'b', 'c'].map(id => ({ id })), edges: [{ from: 'a', to: 'b', kind: 'main' }, { from: 'b', to: 'c', kind: 'main' }, { from: 'b', to: 'a', kind: 'back' }] };
function crosses(points, card) {
  return points.slice(1).some((p, i) => {
    const a = points[i];
    assert.ok(a.x === p.x || a.y === p.y, 'Every segment remains a clickable orthogonal path');
    return a.x === p.x ? a.x > card.x && a.x < card.x + CARD_WIDTH && Math.max(a.y, p.y) > card.y && Math.min(a.y, p.y) < card.y + CARD_HEIGHT : a.y > card.y && a.y < card.y + CARD_HEIGHT && Math.max(a.x, p.x) > card.x && Math.min(a.x, p.x) < card.x + CARD_WIDTH;
  });
}
function assertClear(pos) {
  const before = structuredClone([...pos]), original = structuredClone(flow), routes = routeEdges(flow, pos);
  for (const route of routes) for (const [id, card] of pos) if (id !== route.edge.from && id !== route.edge.to) assert.equal(crosses(route.points, card), false, `${route.index} crosses ${id}`);
  assert.deepEqual([...pos], before); assert.deepEqual(flow, original); assert.equal(routes.length, 3);
  return routes;
}
test('S1 counterexample: exit, vertical/horizontal sections and entry avoid a moved third card without changing layout or Flow', () => {
  const pos = new Map([['a', { x: 640, y: 0 }], ['b', { x: 0, y: 0 }], ['c', { x: 250, y: -200 }]]);
  const routes = assertClear(pos);
  assert.equal(routes[2].edge, flow.edges[2]);
  assert.deepEqual(routes[2].start, { x: 240, y: 87 }); assert.deepEqual(routes[2].end, { x: 640, y: 82.5 });
});
test('An upper card blocks the target vertical channel; routes still avoid it and nearest-line selection identifies the actual object', () => {
  const pos = new Map([['a', { x: 640, y: 0 }], ['b', { x: 0, y: 0 }], ['c', { x: 550, y: -200 }]]);
  const routes = assertClear(pos), back = routes[2];
  const segment = back.points.slice(1).map((p, i) => [back.points[i], p]).find(([a, b]) => Math.hypot(a.x - b.x, a.y - b.y) > 50 && a.y === b.y && a.y < -200);
  assert.ok(segment); const [a, b] = segment;
  const picked = pickRouteAtPoint([back], { x: (a.x + b.x) / 2, y: a.y }, { x: 0, y: 0, scale: 1 }); assert.equal(picked.route, back);
});
test('Main edges validate after same-side offsets: neutral y78 is clear, while actual y73.5/69 must avoid the third card', () => {
  const pos = new Map([['a', { x: 0, y: 0 }], ['b', { x: 640, y: 0 }], ['c', { x: 300, y: -81 }]]);
  const routes = assertClear(pos);
  assert.deepEqual(routes[0].start, { x: 240, y: 73.5 }); assert.deepEqual(routes[0].end, { x: 640, y: 69 });
});
test('Blocked upper/lower source and target sides use verified local detours without merely moving the outer lane', () => {
  const maze = { entry: 'a', nodes: ['a', 'b', 'c', 'd'].map(id => ({ id })), edges: [{ from: 'a', to: 'b', kind: 'main' }, { from: 'b', to: 'c', kind: 'main' }, { from: 'c', to: 'd', kind: 'main' }, { from: 'b', to: 'a', kind: 'back' }] };
  const pos = new Map([['a', { x: 300, y: 0 }], ['b', { x: 0, y: 0 }], ['c', { x: 210, y: -200 }], ['d', { x: 210, y: 200 }]]), before = structuredClone([...pos]);
  const routes = routeEdges(maze, pos);
  for (const route of routes) for (const [id, card] of pos) if (id !== route.edge.from && id !== route.edge.to) assert.equal(crosses(route.points, card), false, `${route.index} crosses ${id}`);
  assert.deepEqual([...pos], before); assert.equal(routes.length, 4);
  assert.ok(routes[3].points.some(point => point.y === -275));
});
