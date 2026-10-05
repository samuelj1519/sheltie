import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { graphPositions, routeEdges, graphBounds, CARD_WIDTH, CARD_HEIGHT, edgeKindLabel, pickRouteAtPoint, renderedCanvasTransform, visibleEdges } from '../public/geometry.mjs';
import { sourceChoices, describeSource, composeSource, declarationBoolean, declarationBytes } from '../public/sources.mjs';
import { CanvasView } from '../public/layout.mjs';
import { WorkbookModel } from '../public/model.mjs';
import { readTree } from '../lib/files.mjs';
const graph = { entry: 'a', nodes: [{ id: 'b' }, { id: 'c' }, { id: 'a' }, { id: 'isolated' }], edges: [{ from: 'a', to: 'b', kind: 'main' }, { from: 'b', to: 'c', kind: 'main' }, { from: 'c', to: 'b', kind: 'main' }, { from: 'c', to: 'a', kind: 'back' }] };
test('Main-flow layout follows actual hierarchy, terminates cycles/orphans, ignores array order, and rearranges deterministically', () => {
  const expected = [['a', { x: 0, y: 0 }], ['b', { x: 320, y: 0 }], ['c', { x: 640, y: 0 }], ['isolated', { x: 960, y: 0 }]];
  assert.deepEqual([...graphPositions(graph)], expected); assert.deepEqual([...graphPositions(graph)], expected);
  const invalid = { ...graph, entry: 'missing', edges: [...graph.edges, { from: 'bad', to: 'a', kind: 'main' }] };
  assert.equal(graphPositions(invalid).size, 4); assert.equal(invalid.entry, 'missing'); assert.equal(invalid.edges.length, 5);
});
test('Edge objects, ports, routes and labels share identity; reverse edges differ and invalid references add no nodes', () => {
  const routes = routeEdges(graph, graphPositions(graph)); assert.equal(routes.length, 4);
  assert.equal(routes[0].edge, graph.edges[0]); assert.equal(routes[2].edge, graph.edges[2]);
  assert.deepEqual(routes[0].start, { x: 240, y: 73.5 }); assert.deepEqual(routes[0].end, { x: 320, y: 78 });
  assert.equal(routes[0].d, 'M240,73.5 L280,73.5 L280,78 L320,78');
  assert.equal(routes[2].label.y, -45); assert.equal(routes[3].label.y, -75); assert.notEqual(routes[1].d, routes[2].d);
  assert.equal(routeEdges({ ...graph, edges: [...graph.edges, { from: 'a', to: 'missing' }] }, graphPositions(graph)).length, 4);
  assert.deepEqual([CARD_WIDTH, CARD_HEIGHT], [240, 156]); assert.equal(edgeKindLabel('alien'), 'Original type: alien');
});
test('Fit includes outer lanes and labels; prepare preserves dragged coordinates and arranging preserves the full Map bytes', async () => {
  const files = await readTree(fileURLToPath(new URL('../../../workbooks/spec-dev/', import.meta.url))); const model = new WorkbookModel(files), f = model.flow('flows/default.toml');
  const view = new CanvasView(); view.positions.set('plan', { x: 42, y: 99 }); view.prepare(f); assert.deepEqual(view.positions.get('plan'), { x: 42, y: 99 });
  view.arrange(f); const bounds = graphBounds(f, view.positions); assert.ok(bounds.minY < -200); assert.ok(bounds.maxY > CARD_HEIGHT + 200); view.fitGraph(f, 900, 700);
  assert.ok(bounds.minX * view.scale + view.x >= 39); assert.ok(bounds.minY * view.scale + view.y >= 39); assert.ok(bounds.maxX * view.scale + view.x <= 861); assert.ok(bounds.maxY * view.scale + view.y <= 661);
  const copy = model.snapshot(); assert.deepEqual([...copy.keys()], [...files.keys()]); for (const [path, bytes] of files) assert.deepEqual(Buffer.from(copy.get(path)), Buffer.from(bytes), path);
});
const flow = { nodes: [{ id: 'write', title: 'Write', outputs: [{ name: 'report', path: 'report.md' }] }, { id: 'empty', outputs: [] }, { id: 'consumer', outputs: [] }, ...['start', 'resource', 'engine'].map(id => ({ id, outputs: [{ name: 'wrong' }] }))] };
const files = new Map([['resources/reference.bin', Uint8Array.of(255)], ['instructions/brief.md', Uint8Array.of(0)], ['workbook.toml', Uint8Array.of(1)]]);
test('Four source types map exact declarations and all Workbook files, including paths outside resources', () => {
  for (const [raw, parsed] of [['start.task', { type: 'start', key: 'task' }], ['write.report', { type: 'step', node: 'write', output: 'report' }], ['resource.instructions/brief.md', { type: 'resource', path: 'instructions/brief.md' }], ['engine.stats', { type: 'stats' }]]) { assert.deepEqual(describeSource(raw, flow, files, 'consumer'), parsed); assert.equal(composeSource(parsed, flow, files, 'consumer'), raw); }
  assert.deepEqual(sourceChoices(flow, files, 'consumer').steps.map(n => n.id), ['write', 'empty']); assert.equal(sourceChoices(flow, files, 'consumer').resources.length, 3);
  assert.throws(() => sourceChoices(flow, files), /owning step/);
});
test('Missing/reserved/invalid/unknown sources stay custom without guesses; incomplete pickers do not commit', () => {
  for (const raw of ['write.missing', 'missing.report', 'empty.report', 'engine.wrong', 'resource.bad', 'start.', 'start..task', 'start.bad_key', 'start.' + 'a'.repeat(65), 'write.report.extra', undefined, 1.5]) { assert.deepEqual(describeSource(raw, flow, files, 'consumer'), { type: 'custom', raw }); assert.equal(composeSource({ type: 'custom', raw }, flow, files, 'consumer'), raw); }
  for (const draft of [{ type: 'step' }, { type: 'step', node: 'write' }, { type: 'step', node: 'empty', output: 'report' }, { type: 'resource' }, { type: 'start', key: '' }]) assert.equal(composeSource(draft, flow, files, 'consumer'), undefined);
});
test('Advanced tri-state and BigInt bytes survive; invalid large integers are rejected without rounding', () => {
  assert.equal(declarationBoolean(''), undefined); assert.equal(declarationBoolean('true'), true); assert.equal(declarationBoolean('false'), false); assert.equal(declarationBoolean('invalid'), 'invalid');
  assert.equal(declarationBytes(''), undefined); assert.equal(declarationBytes('18446744073709551615'), 18446744073709551615n); assert.equal(declarationBytes('0'), 0n);
  for (const text of ['1.0', '-1', '1e6', 'unknown']) assert.throws(() => declarationBytes(text), /integer/);
});
test('Source descriptions and incomplete choices do not mutate fields; source changes preserve required=false and unknown properties', () => {
  const rows = [{ name: 'a', from: 'write.report', extra: { x: 5n } }, { name: 'b', from: 'write.report', required: false, result: false, max_bytes: 18446744073709551615n }, { name: 'c', from: 'write.report', required: true, result: true }], before = structuredClone(rows);
  for (const row of rows) describeSource(row.from, flow, files, 'consumer'); assert.deepEqual(rows, before);
  const missing = composeSource({ type: 'step', node: 'empty' }, flow, files, 'consumer'); if (missing !== undefined) rows[1].from = missing; assert.deepEqual(rows, before);
  rows[1].from = composeSource({ type: 'stats' }, flow, files, 'consumer'); assert.deepEqual(rows[1], { ...before[1], from: 'engine.stats' }); assert.equal(rows[0].required, undefined); assert.deepEqual(rows[0].extra, { x: 5n });
});

test('Main edges that would cross a third card use independent outer lanes without changing kind', () => {
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

test('Initial 100% main view fits visible backbone only; cached cameras and manual positions remain intact', () => {
  const two = { entry: 'a', nodes: [{ id: 'a' }, { id: 'b' }], edges: [{ from: 'a', to: 'b', kind: 'main' }, { from: 'b', to: 'a', kind: 'back' }] };
  const view = CanvasView.load({ getItem: () => null }, 'new'); view.prepare(two); const routes = routeEdges(two, view.positions, visibleEdges(two)); view.initialCamera(two, routes);
  assert.equal(view.scale, 1); assert.equal(view.y, 40); assert.equal(view.x, 40);
  const bounds = graphBounds(two, view.positions, routes); assert.equal(bounds.minY + view.y, 40); assert.equal(bounds.minX + view.x, 40);
  const stored = JSON.stringify({ x: -27, y: 16, scale: .7, positions: [['a', { x: 19, y: 22 }]] });
  const cached = CanvasView.load({ getItem: () => stored }, 'existing'); cached.prepare(two); cached.initialCamera(two);
  assert.equal(cached.x, -27); assert.equal(cached.y, 16); assert.equal(cached.scale, .7); assert.deepEqual(cached.positions.get('a'), { x: 19, y: 22 });
});

test('Source picker excludes its owner; original self-references stay custom and incomplete choices do not guess outputs', () => {
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
test('Unknown prototype-like edge kinds remain literal without Object prototype lookup', () => {
  assert.equal(edgeKindLabel('constructor'), 'Original type: constructor');
  assert.equal(edgeKindLabel('__proto__'), 'Original type: __proto__');
  assert.equal(edgeKindLabel('toString'), 'Original type: toString');
  assert.equal(edgeKindLabel('main'), 'Continue');
});

test('Integer pointer positions select the nearest visible main edge independently of farther transparent-hit draw order', () => {
  const main = { index: 0, edge: { from: 'spec', to: 'plan', kind: 'main' }, points: [{ x: 240, y: 85 }, { x: 295, y: 85 }, { x: 295, y: 62.5 }, { x: 350, y: 62.5 }] };
  const back = { index: 23, edge: { from: 'escalate', to: 'plan', kind: 'back' }, points: [{ x: 334, y: -300 }, { x: 334, y: 200 }] };
  const transform = { x: 300.13555, y: 331.13235, scale: .546994885 };
  const picked = pickRouteAtPoint([main, back], { x: 485, y: 365 }, transform);
  assert.equal(picked.route, main); assert.deepEqual(picked.ambiguous, []);
  assert.equal(pickRouteAtPoint([back, main], { x: 485, y: 365 }, transform).route, main);
  assert.equal(pickRouteAtPoint([main, back], { x: 485.566821, y: 365.319534 }, transform).route, main);
});
test('Equal-distance intersections return every candidate explicitly; pan/zoom and endpoint distances remain accurate', () => {
  const horizontal = { index: 1, edge: { id: 'horizontal' }, points: [{ x: 0, y: 10 }, { x: 20, y: 10 }] };
  const vertical = { index: 2, edge: { id: 'vertical' }, points: [{ x: 15, y: 0 }, { x: 15, y: 20 }] };
  const tied = pickRouteAtPoint([horizontal, vertical], { x: 40, y: 40 }, { x: 10, y: 20, scale: 2 });
  assert.equal(tied.route, undefined); assert.deepEqual(tied.ambiguous, [horizontal, vertical]);
  assert.equal(pickRouteAtPoint([horizontal, vertical], { x: 36, y: 40 }, { x: 10, y: 20, scale: 2 }).route, horizontal);
  assert.equal(pickRouteAtPoint([horizontal, vertical], { x: 40, y: 34 }, { x: 10, y: 20, scale: 2 }).route, vertical);
  assert.equal(pickRouteAtPoint([horizontal, vertical], { x: 60, y: 40 }, { x: 10, y: 20, scale: 2 }).route, horizontal);
  assert.deepEqual(pickRouteAtPoint([], { x: 1, y: 2 }, { x: 0, y: 0, scale: 1 }), { route: undefined, ambiguous: [] });
});

test('Caller conversion after native scroll251 uses the actual world rectangle and does not confuse main/rework edges', () => {
  const scale = .546994885;
  const worldRect = { left: 300.13555, top: 80.13235, width: 5000 * scale };
  const transform = renderedCanvasTransform(worldRect, 5000);
  assert.deepEqual(transform, { x: 300.13555, y: 80.13235, scale });
  const main = { index: 0, edge: { from: 'spec', to: 'plan' }, points: [{ x: 100, y: 120 }, { x: 220, y: 120 }] };
  const back = { index: 23, edge: { from: 'escalate', to: 'plan' }, points: [{ x: 100, y: -339 }, { x: 220, y: -339 }] };
  assert.equal(pickRouteAtPoint([main, back], { x: 393, y: 146 }, transform).route, main);
});
test('Caller conversion uses the rendered rectangle for horizontal/vertical scroll, pan/zoom, and zero scroll', () => {
  assert.deepEqual(renderedCanvasTransform({ left: -140, top: 64, width: 1250 }, 5000), { x: -140, y: 64, scale: .25 });
  assert.deepEqual(renderedCanvasTransform({ left: 70, top: 110, width: 10000 }, 5000), { x: 70, y: 110, scale: 2 });
});

test('Zoom after native scroll remains centered on the pointer; zero-scroll consumers retain their behavior', () => {
  const view = new CanvasView(); view.x = 88.13555; view.y = 216.13235; view.scale = .546994885;
  const point = { x: (393 - 263.13555) / .546994885, y: (146 - 80.13235) / .546994885 };
  view.zoom(1.15, 181, 31, 37, 251);
  assert.ok(Math.abs(point.x * view.scale + 212 + view.x - 37 - 393) < 1e-10);
  assert.ok(Math.abs(point.y * view.scale + 115 + view.y - 251 - 146) < 1e-10);
});
