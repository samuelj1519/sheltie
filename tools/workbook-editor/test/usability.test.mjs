import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { graphPositions, routeEdges, graphBounds, CARD_WIDTH, CARD_HEIGHT, edgeKindLabel, pickRouteAtPoint, renderedCanvasTransform, visibleEdges } from '../public/geometry.mjs';
import { sourceChoices, describeSource, composeSource, declarationBoolean, declarationBytes } from '../public/sources.mjs';
import { CanvasView } from '../public/layout.mjs';
import { WorkbookModel } from '../public/model.mjs';
import { readTree } from '../lib/files.mjs';
const graph = { entry: 'a', nodes: [{ id: 'b' }, { id: 'c' }, { id: 'a' }, { id: 'isolated' }], edges: [{ from: 'a', to: 'b', kind: 'main' }, { from: 'b', to: 'c', kind: 'main' }, { from: 'c', to: 'b', kind: 'main' }, { from: 'c', to: 'a', kind: 'back' }] };
test('主流程按真实main层次排版，循环/孤点终止，非数组顺序，重复整理确定', () => {
  const expected = [['a', { x: 0, y: 0 }], ['b', { x: 320, y: 0 }], ['c', { x: 640, y: 0 }], ['isolated', { x: 960, y: 0 }]];
  assert.deepEqual([...graphPositions(graph)], expected); assert.deepEqual([...graphPositions(graph)], expected);
  const invalid = { ...graph, entry: 'missing', edges: [...graph.edges, { from: 'bad', to: 'a', kind: 'main' }] };
  assert.equal(graphPositions(invalid).size, 4); assert.equal(invalid.entry, 'missing'); assert.equal(invalid.edges.length, 5);
});
test('实际edge对象/端口/路径/标签同对象，往返不共线，坏引用不补节点', () => {
  const routes = routeEdges(graph, graphPositions(graph)); assert.equal(routes.length, 4);
  assert.equal(routes[0].edge, graph.edges[0]); assert.equal(routes[2].edge, graph.edges[2]);
  assert.deepEqual(routes[0].start, { x: 240, y: 73.5 }); assert.deepEqual(routes[0].end, { x: 320, y: 78 });
  assert.equal(routes[0].d, 'M240,73.5 L280,73.5 L280,78 L320,78');
  assert.equal(routes[2].label.y, -45); assert.equal(routes[3].label.y, -75); assert.notEqual(routes[1].d, routes[2].d);
  assert.equal(routeEdges({ ...graph, edges: [...graph.edges, { from: 'a', to: 'missing' }] }, graphPositions(graph)).length, 4);
  assert.deepEqual([CARD_WIDTH, CARD_HEIGHT], [240, 156]); assert.equal(edgeKindLabel('alien'), '原类型：alien');
});
test('fit包含外侧通道和标签；prepare保留拖动坐标；整理完整Map原字节不变', async () => {
  const files = await readTree(fileURLToPath(new URL('../../../workbooks/spec-dev/', import.meta.url))); const model = new WorkbookModel(files), f = model.flow('flows/default.toml');
  const view = new CanvasView(); view.positions.set('plan', { x: 42, y: 99 }); view.prepare(f); assert.deepEqual(view.positions.get('plan'), { x: 42, y: 99 });
  view.arrange(f); const bounds = graphBounds(f, view.positions); assert.ok(bounds.minY < -200); assert.ok(bounds.maxY > CARD_HEIGHT + 200); view.fitGraph(f, 900, 700);
  assert.ok(bounds.minX * view.scale + view.x >= 39); assert.ok(bounds.minY * view.scale + view.y >= 39); assert.ok(bounds.maxX * view.scale + view.x <= 861); assert.ok(bounds.maxY * view.scale + view.y <= 661);
  const copy = model.snapshot(); assert.deepEqual([...copy.keys()], [...files.keys()]); for (const [path, bytes] of files) assert.deepEqual(Buffer.from(copy.get(path)), Buffer.from(bytes), path);
});
const flow = { nodes: [{ id: 'write', title: '编写', outputs: [{ name: 'report', path: 'report.md' }] }, { id: 'empty', outputs: [] }, { id: 'consumer', outputs: [] }, ...['start', 'resource', 'engine'].map(id => ({ id, outputs: [{ name: 'wrong' }] }))] };
const files = new Map([['resources/reference.bin', Uint8Array.of(255)], ['instructions/brief.md', Uint8Array.of(0)], ['workbook.toml', Uint8Array.of(1)]]);
test('四种来源精确映射实际声明及全部Workbook文件，包括resources以外路径', () => {
  for (const [raw, parsed] of [['start.task', { type: 'start', key: 'task' }], ['write.report', { type: 'step', node: 'write', output: 'report' }], ['resource.instructions/brief.md', { type: 'resource', path: 'instructions/brief.md' }], ['engine.stats', { type: 'stats' }]]) { assert.deepEqual(describeSource(raw, flow, files, 'consumer'), parsed); assert.equal(composeSource(parsed, flow, files, 'consumer'), raw); }
  assert.deepEqual(sourceChoices(flow, files, 'consumer').steps.map(n => n.id), ['write', 'empty']); assert.equal(sourceChoices(flow, files, 'consumer').resources.length, 3);
  assert.throws(() => sourceChoices(flow, files), /所属步骤/);
});
test('缺声明、保留字、非法键、未知原值保持自定义，不猜来源；未完成picker不提交', () => {
  for (const raw of ['write.missing', 'missing.report', 'empty.report', 'engine.wrong', 'resource.bad', 'start.', 'start..task', 'start.bad_key', 'start.' + 'a'.repeat(65), 'write.report.extra', undefined, 1.5]) { assert.deepEqual(describeSource(raw, flow, files, 'consumer'), { type: 'custom', raw }); assert.equal(composeSource({ type: 'custom', raw }, flow, files, 'consumer'), raw); }
  for (const draft of [{ type: 'step' }, { type: 'step', node: 'write' }, { type: 'step', node: 'empty', output: 'report' }, { type: 'resource' }, { type: 'start', key: '' }]) assert.equal(composeSource(draft, flow, files, 'consumer'), undefined);
});
test('高级三态与BigInt字节保真，非法大数输入拒绝而不舍入', () => {
  assert.equal(declarationBoolean(''), undefined); assert.equal(declarationBoolean('true'), true); assert.equal(declarationBoolean('false'), false); assert.equal(declarationBoolean('invalid'), 'invalid');
  assert.equal(declarationBytes(''), undefined); assert.equal(declarationBytes('18446744073709551615'), 18446744073709551615n); assert.equal(declarationBytes('0'), 0n);
  for (const text of ['1.0', '-1', '1e6', 'unknown']) assert.throws(() => declarationBytes(text), /整数/);
});
test('来源描述投影和未完成选择不改字段，变更来源不洗白required=false或未知属性', () => {
  const rows = [{ name: 'a', from: 'write.report', extra: { x: 5n } }, { name: 'b', from: 'write.report', required: false, result: false, max_bytes: 18446744073709551615n }, { name: 'c', from: 'write.report', required: true, result: true }], before = structuredClone(rows);
  for (const row of rows) describeSource(row.from, flow, files, 'consumer'); assert.deepEqual(rows, before);
  const missing = composeSource({ type: 'step', node: 'empty' }, flow, files, 'consumer'); if (missing !== undefined) rows[1].from = missing; assert.deepEqual(rows, before);
  rows[1].from = composeSource({ type: 'stats' }, flow, files, 'consumer'); assert.deepEqual(rows[1], { ...before[1], from: 'engine.stats' }); assert.equal(rows[0].required, undefined); assert.deepEqual(rows[0].extra, { x: 5n });
});

test('主边跳层或手动移动后直连会穿第三卡片时，走外侧独立通道而不改kind', () => {
  const flow = { entry: 'a', nodes: [{ id: 'a' }, { id: 'middle' }, { id: 'b' }], edges: [{ from: 'a', to: 'b', kind: 'main' }] };
  const positions = new Map([['a', { x: 0, y: 0 }], ['middle', { x: 350, y: 0 }], ['b', { x: 700, y: 0 }]]);
  positions.set('deleted', { x: 4000, y: -4000 });
  const route = routeEdges(flow, positions)[0];
  assert.equal(graphBounds(flow, positions).maxX, 940);
  assert.equal(route.label.y, -45); assert.equal(route.edge.kind, 'main');
  assert.equal(route.points[1].x, 260); assert.equal(route.points[4].x, 680);
  assert.ok(route.points[2].y < 0); assert.ok(route.points[3].y < 0);
  const before = structuredClone(flow); routeEdges(flow, positions); assert.deepEqual(flow, before);
});

test('初次100%主视图只用可见骨架边界，已有缓存camera和手动位置不覆盖', () => {
  const two = { entry: 'a', nodes: [{ id: 'a' }, { id: 'b' }], edges: [{ from: 'a', to: 'b', kind: 'main' }, { from: 'b', to: 'a', kind: 'back' }] };
  const view = CanvasView.load({ getItem: () => null }, 'new'); view.prepare(two); const routes = routeEdges(two, view.positions, visibleEdges(two)); view.initialCamera(two, routes);
  assert.equal(view.scale, 1); assert.equal(view.y, 40); assert.equal(view.x, 40);
  const bounds = graphBounds(two, view.positions, routes); assert.equal(bounds.minY + view.y, 40); assert.equal(bounds.minX + view.x, 40);
  const stored = JSON.stringify({ x: -27, y: 16, scale: .7, positions: [['a', { x: 19, y: 22 }]] });
  const cached = CanvasView.load({ getItem: () => stored }, 'existing'); cached.prepare(two); cached.initialCamera(two);
  assert.equal(cached.x, -27); assert.equal(cached.y, 16); assert.equal(cached.scale, .7); assert.deepEqual(cached.positions.get('a'), { x: 19, y: 22 });
});

test('来源picker排除所属步骤：自身原引用显示自定义且原文保留，未完成不猜输出', () => {
  const owned = { nodes: [{ id: 'review', outputs: [{ name: 'review', path: 'review.md' }] }, { id: 'write', outputs: [{ name: 'change', path: 'change.md' }] }] };
  assert.deepEqual(sourceChoices(owned, files, 'review').steps.map(n => n.id), ['write']);
  assert.deepEqual(describeSource('review.review', owned, files, 'review'), { type: 'custom', raw: 'review.review' });
  assert.equal(composeSource({ type: 'step', node: 'review', output: 'review' }, owned, files, 'review'), undefined);
  assert.equal(composeSource({ type: 'step', node: 'write' }, owned, files, 'review'), undefined);
  assert.equal(composeSource({ type: 'step', node: 'write', output: 'change' }, owned, files, 'review'), 'write.change');
  const row = { name: 'previous-review', from: 'review.review', required: false, result: false, unknown: { untouched: 23n } }, before = structuredClone(row);
  describeSource(row.from, owned, files, 'review');
  const draft = composeSource({ type: 'step', node: 'review', output: 'review' }, owned, files, 'review');
  if (draft !== undefined) row.from = draft;
  assert.deepEqual(row, before);
  assert.equal(composeSource({ type: 'custom', raw: row.from }, owned, files, 'review'), 'review.review');
});
test('未知edge kind原型名称保持字面原值可识别，不误读Object原型', () => {
  assert.equal(edgeKindLabel('constructor'), '原类型：constructor');
  assert.equal(edgeKindLabel('__proto__'), '原类型：__proto__');
  assert.equal(edgeKindLabel('toString'), '原类型：toString');
  assert.equal(edgeKindLabel('main'), '继续');
});

test('真实整数鼠标位置按最近可见线选主边，不随更远透明hit的绘制顺序变化', () => {
  const main = { index: 0, edge: { from: 'spec', to: 'plan', kind: 'main' }, points: [{ x: 240, y: 85 }, { x: 295, y: 85 }, { x: 295, y: 62.5 }, { x: 350, y: 62.5 }] };
  const back = { index: 23, edge: { from: 'escalate', to: 'plan', kind: 'back' }, points: [{ x: 334, y: -300 }, { x: 334, y: 200 }] };
  const transform = { x: 300.13555, y: 331.13235, scale: .546994885 };
  const picked = pickRouteAtPoint([main, back], { x: 485, y: 365 }, transform);
  assert.equal(picked.route, main); assert.deepEqual(picked.ambiguous, []);
  assert.equal(pickRouteAtPoint([back, main], { x: 485, y: 365 }, transform).route, main);
  assert.equal(pickRouteAtPoint([main, back], { x: 485.566821, y: 365.319534 }, transform).route, main);
});
test('真实交点等距明确给全部候选而不静默选后画边，平移缩放与端点距离准确', () => {
  const horizontal = { index: 1, edge: { id: 'horizontal' }, points: [{ x: 0, y: 10 }, { x: 20, y: 10 }] };
  const vertical = { index: 2, edge: { id: 'vertical' }, points: [{ x: 15, y: 0 }, { x: 15, y: 20 }] };
  const tied = pickRouteAtPoint([horizontal, vertical], { x: 40, y: 40 }, { x: 10, y: 20, scale: 2 });
  assert.equal(tied.route, undefined); assert.deepEqual(tied.ambiguous, [horizontal, vertical]);
  assert.equal(pickRouteAtPoint([horizontal, vertical], { x: 36, y: 40 }, { x: 10, y: 20, scale: 2 }).route, horizontal);
  assert.equal(pickRouteAtPoint([horizontal, vertical], { x: 40, y: 34 }, { x: 10, y: 20, scale: 2 }).route, vertical);
  assert.equal(pickRouteAtPoint([horizontal, vertical], { x: 60, y: 40 }, { x: 10, y: 20, scale: 2 }).route, horizontal);
  assert.deepEqual(pickRouteAtPoint([], { x: 1, y: 2 }, { x: 0, y: 0, scale: 1 }), { route: undefined, ambiguous: [] });
});

test('真实native scroll251后的caller换算来自实际world矩形，主边不误选返工', () => {
  const scale = .546994885;
  const worldRect = { left: 300.13555, top: 80.13235, width: 5000 * scale };
  const transform = renderedCanvasTransform(worldRect, 5000);
  assert.deepEqual(transform, { x: 300.13555, y: 80.13235, scale });
  const main = { index: 0, edge: { from: 'spec', to: 'plan' }, points: [{ x: 100, y: 120 }, { x: 220, y: 120 }] };
  const back = { index: 23, edge: { from: 'escalate', to: 'plan' }, points: [{ x: 100, y: -339 }, { x: 220, y: -339 }] };
  assert.equal(pickRouteAtPoint([main, back], { x: 393, y: 146 }, transform).route, main);
});
test('caller完整水平/垂直native scroll、平移缩放及零滚动按渲染矩形转换', () => {
  assert.deepEqual(renderedCanvasTransform({ left: -140, top: 64, width: 1250 }, 5000), { x: -140, y: 64, scale: .25 });
  assert.deepEqual(renderedCanvasTransform({ left: 70, top: 110, width: 10000 }, 5000), { x: 70, y: 110, scale: 2 });
});

test('native scroll后的缩放仍以鼠标实际屏幕位置为中心，零scroll旧消费者保持', () => {
  const view = new CanvasView(); view.x = 88.13555; view.y = 216.13235; view.scale = .546994885;
  const point = { x: (393 - 263.13555) / .546994885, y: (146 - 80.13235) / .546994885 };
  view.zoom(1.15, 181, 31, 37, 251);
  assert.ok(Math.abs(point.x * view.scale + 212 + view.x - 37 - 393) < 1e-10);
  assert.ok(Math.abs(point.y * view.scale + 115 + view.y - 251 - 146) < 1e-10);
});
