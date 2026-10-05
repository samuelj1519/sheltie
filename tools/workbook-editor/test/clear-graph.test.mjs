import test from 'node:test';
import assert from 'node:assert/strict';
import { graphPositions, routeEdges, CARD_WIDTH, CARD_HEIGHT } from '../public/geometry.mjs';
const chain = { entry: 'a', nodes: ['d', 'b', 'a', 'c'].map(id => ({ id })), edges: [{ from: 'a', to: 'b', kind: 'main' }, { from: 'b', to: 'c', kind: 'main' }, { from: 'c', to: 'd', kind: 'main' }] };
test('First-visit main order uses compact three-step rows; step four wraps, independent of nodes array order', () => {
  const pos = graphPositions(chain);
  assert.deepEqual(pos.get('a'), { x: 0, y: 0 });
  assert.deepEqual(pos.get('b'), { x: 320, y: 0 });
  assert.deepEqual(pos.get('c'), { x: 640, y: 0 });
  assert.deepEqual(pos.get('d'), { x: 640, y: 216 });
});
test('Row and turn ports follow actual direction; highlights reuse the same route', () => {
  const pos = new Map([['a', { x: 640, y: 0 }], ['b', { x: 640, y: 216 }], ['c', { x: 320, y: 216 }], ['d', { x: 0, y: 216 }]]);
  const routes = routeEdges(chain, pos);
  assert.deepEqual(routes[0].start, { x: 760, y: CARD_HEIGHT });
  assert.deepEqual(routes[0].end, { x: 760, y: 216 });
  assert.deepEqual(routes[1].start, { x: 640, y: 294 });
  assert.deepEqual(routes[1].end, { x: 320 + CARD_WIDTH, y: 294 });
});
import { predecessorMaterials, planInputRows, suggestedInputName } from '../public/sources.mjs';
test('Input names require only text and node-local uniqueness; Chinese and underscore choices are preserved', () => {
  const flow = { nodes: [{ id: 'up', inputs: [{ name: 'plan_tpl', from: 'start.task' }, { name: 'original-requirements', from: 'engine.stats' }], outputs: [{ name: 'draft', path: 'draft.md' }] }, { id: 'down' }], edges: [{ from: 'up', to: 'down', kind: 'main' }] };
  const files = new Map();
  const inputs = predecessorMaterials(flow, files, 'down').filter(m => m.kind === 'input');
  assert.deepEqual(inputs.map(m => [m.name, m.reason]), [['plan_tpl', ''], ['original-requirements', '']]);
  assert.deepEqual(planInputRows([], [{ name: '中文_方案', from: 'start.task' }]), { additions: [{ name: '中文_方案', from: 'start.task' }], skipped: [], conflicts: [] });
  assert.equal(suggestedInputName('中文_方案', 'start.task', []), '中文_方案');
});
import { readTree } from '../lib/files.mjs';
import { WorkbookModel, newWorkbook } from '../public/model.mjs';
import { mainSkeleton, visibleEdges, graphBounds, pickRouteAtPoint } from '../public/geometry.mjs';
import { addMaterialInputs, fixedReference, editFixedReference } from '../public/sources.mjs';
import { fileURLToPath } from 'node:url';
import { checkWorkbook } from '../lib/engine.mjs';
const id1 = '11111111-1111-4111-8111-111111111111', id2 = '22222222-2222-4222-8222-222222222222';
function fixture() {
  const model = newWorkbook(), path = 'flows/default.toml', owner = model.addNode(path), first = model.flow(path).nodes[0];
  first.inputs = [{ name: 'plan_tpl', from: 'start.task' }, { name: 'original-requirements', from: 'engine.stats' }];
  first.outputs = [{ name: 'draft', path: 'draft.md' }, { name: 'optional', path: 'optional.md', required: false }, { name: 'approved', path: 'approved.md', required: true }];
  model.addEdge(path, first.id, owner.id); model.commit(path);
  return { model, path, owner: owner.id };
}
function assertBytes(actual, expected) {
  assert.deepEqual([...actual.keys()].sort(), [...expected.keys()].sort());
  for (const [path, bytes] of expected) assert.deepEqual(Buffer.from(actual.get(path)), Buffer.from(bytes), path);
}
function crosses(points, p) {
  return points.slice(1).some((b, i) => {
    const a = points[i];
    return a.x === b.x ? a.x > p.x + 1 && a.x < p.x + CARD_WIDTH - 1 && Math.max(a.y, b.y) > p.y + 1 && Math.min(a.y, b.y) < p.y + CARD_HEIGHT - 1 : a.y > p.y + 1 && a.y < p.y + CARD_HEIGHT - 1 && Math.max(a.x, b.x) > p.x + 1 && Math.min(a.x, b.x) < p.x + CARD_WIDTH - 1;
  });
}
test('Real spec graph has 11 nodes/25 edges and a 9-node/8-edge backbone; all/adjacent views share edge objects and avoid other cards', async () => {
  const files = await readTree(fileURLToPath(new URL('../../../workbooks/spec-dev/', import.meta.url))), model = new WorkbookModel(files), flow = model.flow('flows/default.toml');
  assert.deepEqual(mainSkeleton(flow).nodes, ['spec', 'plan', 'plan-review', 'scaffold', 'implement', 'verify', 'review', 'deliver', 'retro']);
  assert.equal(mainSkeleton(flow).edges.length, 8); assert.equal(flow.nodes.length, 11); assert.equal(flow.edges.length, 25);
  assert.equal(visibleEdges(flow).length, 8); assert.deepEqual(visibleEdges(flow, 'all'), flow.edges);
  const edge = flow.edges[3]; assert.ok(!visibleEdges(flow).includes(edge)); assert.ok(visibleEdges(flow, 'main', undefined, edge).includes(edge));
  const adjacent = visibleEdges(flow, 'main', 'escalate'); assert.ok(flow.edges.filter(e => e.from === 'escalate' || e.to === 'escalate').every(e => adjacent.includes(e)));
  const positions = graphPositions(flow);
  for (const offset of [null, { x: -40, y: 20 }]) {
    if (offset) positions.set('spec', offset);
    for (const route of routeEdges(flow, positions)) for (const node of flow.nodes) if (node.id !== route.edge.from && node.id !== route.edge.to) assert.equal(crosses(route.points, positions.get(node.id)), false, `${route.index}:${node.id}`);
  }
  assertBytes(model.snapshot(), files);
});
test('Hidden closer routes do not participate in hit tests or fit; focusing a hidden list route changes only view selection', () => {
  const shown = { index: 0, edge: { id: 'shown' }, points: [{ x: 0, y: 10 }, { x: 20, y: 10 }] }, hidden = { index: 1, edge: { id: 'hidden' }, points: [{ x: 0, y: 11 }, { x: 20, y: 11 }] };
  assert.equal(pickRouteAtPoint([shown], { x: 10, y: 11 }, { x: 0, y: 0, scale: 1 }).route, shown);
  assert.equal(pickRouteAtPoint([shown, hidden], { x: 10, y: 11 }, { x: 0, y: 0, scale: 1 }).route, hidden);
  const pos = graphPositions(chain), main = routeEdges(chain, pos, visibleEdges(chain));
  const bounds = graphBounds(chain, pos, main); assert.equal(bounds.minY, 0); assert.equal(bounds.maxY, 372);
  const before = structuredClone([...pos]); graphBounds({ nodes: [chain.nodes[0], chain.nodes[1]] }, pos, [main[1]], main[1].edge); assert.deepEqual([...pos], before);
});
test('Direct predecessor outputs and inherited inputs preserve omitted/false/true required and do not copy terminal properties', () => {
  const { model, path, owner } = fixture(), f = model.flow(path), first = f.nodes[0];
  first.outputs[0].result = true; first.outputs[0].max_bytes = 17n; first.inputs[0].result = true;
  const options = predecessorMaterials(f, model.files, owner);
  assert.deepEqual(options.map(o => [o.key, o.name, o.from, o.required]), [['output:step-1:0', 'draft', 'step-1.draft', undefined], ['output:step-1:1', 'optional', 'step-1.optional', false], ['output:step-1:2', 'approved', 'step-1.approved', true], ['input:step-1:0', 'plan_tpl', 'start.task', undefined], ['input:step-1:1', 'original-requirements', 'engine.stats', undefined]]);
  const planned = planInputRows([], options);
  assert.deepEqual(planned.additions, [{ name: 'draft', from: 'step-1.draft' }, { name: 'optional', from: 'step-1.optional', required: false }, { name: 'approved', from: 'step-1.approved', required: true }, { name: 'plan_tpl', from: 'start.task' }, { name: 'original-requirements', from: 'engine.stats' }]);
  assert.ok(planned.additions.every(r => !Object.hasOwn(r, 'result') && !Object.hasOwn(r, 'max_bytes')));
});
test('Invalid self/missing/optional choices cannot be added; imported invalid rows are not repaired', () => {
  const { model, path, owner } = fixture(), flow = model.flow(path), original = flow.nodes[0];
  original.inputs.push({ name: 'self', from: 'step-2.missing' }, { name: 'missing', from: 'unknown.output' }, { name: 'bad-optional', from: 'start.task', required: false }); model.commit(path);
  const bytes = model.snapshot(), options = predecessorMaterials(flow, model.files, owner), denied = options.filter(o => ['self', 'missing', 'bad-optional'].includes(o.name));
  assert.ok(denied.every(o => o.reason));
  for (const option of denied) assert.throws(() => addMaterialInputs(model, path, owner, [{ key: option.key, name: option.name }]), /Source|source|declaration/);
  assertBytes(model.snapshot(), bytes);
});
test('Conflict preview does not overwrite rows; identical name/from pairs deduplicate and valid names may be customized', () => {
  const existing = [{ name: '计划_模板', from: 'start.task', required: false, result: true, extra: 9n }], before = structuredClone(existing);
  assert.equal(suggestedInputName('计划_模板', 'engine.stats', existing), '计划_模板-2');
  assert.deepEqual(planInputRows(existing, [{ name: '计划_模板', from: 'start.task' }, { name: '计划_模板', from: 'engine.stats' }]), { additions: [], skipped: [{ name: '计划_模板', from: 'start.task' }], conflicts: [{ name: '计划_模板', reason: 'This name is already used by another item; choose a different name.' }] });
  assert.deepEqual(existing, before);
});
test('Committed fixed references create owned resources without reading absolute/relative/URL locations; rename changes only name and COW preserves shared files', () => {
  const { model, path, owner } = fixture(), before = model.snapshot();
  const locations = ['/Users/person/a.txt', '../local/relative.md', 'https://example.invalid/never-fetch?q=private']; let count = 0;
  const ids = [id1, id2, '33333333-3333-4333-8333-333333333333', '44444444-4444-4444-8444-444444444444'];
  const result = addMaterialInputs(model, path, owner, locations.map((location, index) => ({ name: `reference_${index}`, location })), () => ids[count++]);
  assertBytes(model.snapshot(), before); assert.equal(result.added, 3);
  const rows = result.model.flow(path).nodes[1].inputs;
  assert.deepEqual(rows.map(r => fixedReference(result.model.files, r.from, path, owner)?.location), locations);
  const originalRef = rows[0].from, resource = originalRef.slice(9), resourceBytes = Buffer.from(result.model.files.get(resource));
  result.model.edit(path, rows[0], 'name', '中文新名称'); assert.deepEqual(Buffer.from(result.model.files.get(resource)), resourceBytes);
  assert.equal(editFixedReference(result.model, path, owner, 0, locations[0]), result.model);
  rows.push({ name: 'alias', from: originalRef }); result.model.commit(path);
  const changed = editFixedReference(result.model, path, owner, 0, 'relative/new-location', () => ids[count++]);
  assert.equal(changed.flow(path).nodes[1].inputs[3].from, originalRef); assert.notEqual(changed.flow(path).nodes[1].inputs[0].from, originalRef); assert.deepEqual(Buffer.from(changed.files.get(resource)), resourceBytes);
  assert.equal(fixedReference(changed.files, changed.flow(path).nodes[1].inputs[0].from, path, owner).location, 'relative/new-location');
});
test('Invalid ownership markers, UTF-8, or UUID context fall back to ordinary resources without overwriting existing files', () => {
  const { model, path, owner } = fixture();
  const file = `resources/sheltie-editor-ref-${id1}.txt`, from = `resource.${file}`;
  for (const body of ['fake marker', 'Sheltie editor fixed reference v1\n{"schema":"sheltie-editor-reference/v1"}', 'Sheltie editor fixed reference v1\n' + JSON.stringify({ schema: 'sheltie-editor-reference/v1', id: id1, owner: { flow: path, node: 'step-1' }, location: 'x' }), 'Sheltie editor fixed reference v1\n' + JSON.stringify({ schema: 'sheltie-editor-reference/v1', id: id1, owner: { flow: path, node: owner }, location: 'x', injected: true })]) {
    model.writeText(file, body); assert.equal(fixedReference(model.files, from, path, owner), undefined);
  }
  const original = model.snapshot(); let calls = 0;
  const added = addMaterialInputs(model, path, owner, [{ name: 'new-reference', location: 'x' }], () => [id1, id2][calls++]);
  assert.equal(calls, 2); assert.deepEqual(Buffer.from(added.model.files.get(file)), Buffer.from(original.get(file)));
  assert.equal(added.model.flow(path).nodes[1].inputs[0].from, `resource.resources/sheltie-editor-ref-${id2}.txt`);
  const invalidBytes = new Map(original); invalidBytes.set(file, Uint8Array.of(255)); assert.equal(fixedReference(invalidBytes, from, path, owner), undefined);
});
test('Batch failure or excess file count preserves the original Map and rows without orphan reference files', () => {
  const { model, path, owner } = fixture(), node = model.flow(path).nodes[1]; node.inputs = [{ name: 'existing', from: 'start.task' }]; model.commit(path); const original = model.snapshot();
  assert.throws(() => addMaterialInputs(model, path, owner, [{ name: 'first', location: 'x' }, { name: 'existing', location: 'y' }], () => id1), /already exists/); assertBytes(model.snapshot(), original);
  const full = new Map(original); for (let i = full.size; i < 1024; i++) full.set(`resources/f${i}`, Uint8Array.of(0)); const limited = new WorkbookModel(full);
  assert.throws(() => addMaterialInputs(limited, path, owner, [{ name: 'ref', location: 'x' }], () => id1), /1024/); assertBytes(limited.snapshot(), full);
});
test('Real CLI accepts Chinese/underscore/optional inputs and external-location resources after rename and COW edits', async () => {
  const engine = process.env.SHELTIE_EDITOR_ENGINE; assert.ok(engine, 'A real trusted engine is required');
  const { model, path, owner } = fixture();
  const requests = [{ key: 'output:step-1:0', name: 'implementation_report' }, { key: 'output:step-1:1', name: 'optional-feedback' }, { key: 'output:step-1:2', name: 'approved' }, { key: 'input:step-1:0', name: 'plan_tpl' }, { key: 'input:step-1:1', name: 'original-requirements' }, { name: 'external-path', location: '/Users/person/original.txt' }, { name: 'relative-reference', location: '../relative/file.md' }, { name: 'remote-link', location: 'https://example.invalid/never-fetch' }]; let count = 1;
  const uuid = () => `${String(count++).padStart(8, '0')}-1111-4111-8111-111111111111`;
  const added = addMaterialInputs(model, path, owner, requests, uuid).model;
  assert.equal(added.flow(path).nodes[1].inputs[1].required, false);
  const first = await checkWorkbook(added.snapshot(), engine); console.log(JSON.stringify({ boundary: 'real-cli-unified-inputs', result: first })); assert.equal(first.ok, true);
  const edited = editFixedReference(added, path, owner, 5, 'new relative/location', uuid); const second = await checkWorkbook(edited.snapshot(), engine); console.log(JSON.stringify({ boundary: 'real-cli-cow-location', result: second })); assert.equal(second.ok, true);
  const invalid = new WorkbookModel(added.snapshot()); invalid.edit(path, invalid.flow(path).nodes[1].inputs[1], 'required', true); const rejected = await checkWorkbook(invalid.snapshot(), engine); console.log(JSON.stringify({ boundary: 'real-cli-optional-not-required', result: rejected })); assert.equal(rejected.ok, false); assert.equal(rejected.engineError.code, 'FLOW_INVALID'); assert.equal(rejected.engineError.detail.rule, '5');
});
test('Incoming and outgoing edges on the same physical card side share ports; rework does not overlap the whole main route', async () => {
  const model = new WorkbookModel(await readTree(fileURLToPath(new URL('../../../workbooks/spec-dev/', import.meta.url)))), flow = model.flow('flows/default.toml'), routes = routeEdges(flow, graphPositions(flow));
  const planRight = [routes[1].start.y, routes[4].end.y, routes[23].end.y];
  const reviewLeft = [routes[1].end.y, routes[3].start.y, routes[4].start.y];
  assert.equal(new Set(planRight).size, 3); assert.equal(new Set(reviewLeft).size, 3);
  assert.ok(routes[1].start.y !== routes[4].end.y); assert.ok(routes[1].end.y !== routes[3].start.y);
});
