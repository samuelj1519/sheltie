import test from 'node:test';
import assert from 'node:assert/strict';
import { routeEdges, CARD_WIDTH, CARD_HEIGHT, pickRouteAtPoint } from '../public/geometry.mjs';
const flow = { entry: 'a', nodes: ['a', 'b', 'c'].map(id => ({ id })), edges: [{ from: 'a', to: 'b', kind: 'main' }, { from: 'b', to: 'c', kind: 'main' }, { from: 'b', to: 'a', kind: 'back' }] };
function crosses(points, card) {
  return points.slice(1).some((p, i) => {
    const a = points[i];
    assert.ok(a.x === p.x || a.y === p.y, '所有段仍为可点击的正交路径');
    return a.x === p.x ? a.x > card.x && a.x < card.x + CARD_WIDTH && Math.max(a.y, p.y) > card.y && Math.min(a.y, p.y) < card.y + CARD_HEIGHT : a.y > card.y && a.y < card.y + CARD_HEIGHT && Math.max(a.x, p.x) > card.x && Math.min(a.x, p.x) < card.x + CARD_WIDTH;
  });
}
function assertClear(pos) {
  const before = structuredClone([...pos]), original = structuredClone(flow), routes = routeEdges(flow, pos);
  for (const route of routes) for (const [id, card] of pos) if (id !== route.edge.from && id !== route.edge.to) assert.equal(crosses(route.points, card), false, `${route.index}穿${id}`);
  assert.deepEqual([...pos], before); assert.deepEqual(flow, original); assert.equal(routes.length, 3);
  return routes;
}
test('S1真实反例：完整外侧出口/竖段/横段/入口避开拖动第三卡片，不改布局或Flow', () => {
  const pos = new Map([['a', { x: 640, y: 0 }], ['b', { x: 0, y: 0 }], ['c', { x: 250, y: -200 }]]);
  const routes = assertClear(pos);
  assert.equal(routes[2].edge, flow.edges[2]);
  assert.deepEqual(routes[2].start, { x: 240, y: 87 }); assert.deepEqual(routes[2].end, { x: 640, y: 82.5 });
});
test('第二种摆放：目标入口方向的竖通道被上方卡片覆盖，同样避障且最近线同实际对象', () => {
  const pos = new Map([['a', { x: 640, y: 0 }], ['b', { x: 0, y: 0 }], ['c', { x: 550, y: -200 }]]);
  const routes = assertClear(pos), back = routes[2];
  const segment = back.points.slice(1).map((p, i) => [back.points[i], p]).find(([a, b]) => Math.hypot(a.x - b.x, a.y - b.y) > 50 && a.y === b.y && a.y < -200);
  assert.ok(segment); const [a, b] = segment;
  const picked = pickRouteAtPoint([back], { x: (a.x + b.x) / 2, y: a.y }, { x: 0, y: 0, scale: 1 }); assert.equal(picked.route, back);
});
test('同侧offset后的主边也最终验证：中性y78未撞，真实y73.5/69不能穿第三卡片', () => {
  const pos = new Map([['a', { x: 0, y: 0 }], ['b', { x: 640, y: 0 }], ['c', { x: 300, y: -81 }]]);
  const routes = assertClear(pos);
  assert.deepEqual(routes[0].start, { x: 240, y: 73.5 }); assert.deepEqual(routes[0].end, { x: 640, y: 69 });
});
test('源/目标上下两侧都受阻时采用已验证局部绕行，不靠把外lane移更远', () => {
  const maze = { entry: 'a', nodes: ['a', 'b', 'c', 'd'].map(id => ({ id })), edges: [{ from: 'a', to: 'b', kind: 'main' }, { from: 'b', to: 'c', kind: 'main' }, { from: 'c', to: 'd', kind: 'main' }, { from: 'b', to: 'a', kind: 'back' }] };
  const pos = new Map([['a', { x: 300, y: 0 }], ['b', { x: 0, y: 0 }], ['c', { x: 210, y: -200 }], ['d', { x: 210, y: 200 }]]), before = structuredClone([...pos]);
  const routes = routeEdges(maze, pos);
  for (const route of routes) for (const [id, card] of pos) if (id !== route.edge.from && id !== route.edge.to) assert.equal(crosses(route.points, card), false, `${route.index}穿${id}`);
  assert.deepEqual([...pos], before); assert.equal(routes.length, 4);
  assert.ok(routes[3].points.some(point => point.y === -275));
});
