import test from 'node:test';
import assert from 'node:assert/strict';
import { parse } from 'smol-toml';
import { WorkbookModel, newWorkbook } from '../public/model.mjs';
import { readTree } from '../lib/files.mjs';
import { checkWorkbook } from '../lib/engine.mjs';
import { resolve } from 'node:path';
const engine = process.env.SHELTIE_EDITOR_ENGINE;
const text = value => new TextEncoder().encode(value);
test('Editing one field preserves the whole object, other Flows and resource bytes', async () => {
  const source = await readTree(resolve('../../examples/code-change'));
  source.set('resources/bin', Uint8Array.from([0, 1, 254, 255]));
  const manifest = parse(new TextDecoder().decode(source.get('workbook.toml')));
  source.set('workbook.toml', text(new TextDecoder().decode(source.get('workbook.toml')).replace('flows = ["flows/default.toml"]', 'flows = ["flows/default.toml", "flows/other.toml"]')));
  source.set('flows/other.toml', text('schema="flow/v1"\nid="other"\nentry="one"\n[[nodes]]\nid="one"\ntitle="Another Flow"\nexecutor="agent"\ninstruction={text="Preserve"}\n'));
  const model = new WorkbookModel(source);
  model.edit('flows/default.toml', model.flow('flows/default.toml').nodes[0], 'title', 'new-title');
  for (const [path, bytes] of source) if (path !== 'flows/default.toml') assert.deepEqual(model.snapshot().get(path), bytes, path);
  const edited = parse(model.text('flows/default.toml'));
  assert.equal(edited.nodes[0].tier, 'strong'); assert.equal(edited.nodes[0].max_visits, 3); assert.equal(edited.nodes[0].outputs[0].max_bytes, 262144); assert.equal(edited.nodes[2].inputs[1].result, true);
  assert.equal(manifest.id, 'code-change');
});
test('Adding an explicit edge adds no input; input edits add no edge; deletion preserves references for CLI refusal', async () => {
  const model = newWorkbook(); const path = 'flows/default.toml', flow = model.flow(path), first = flow.nodes[0];
  const second = model.addNode(path); model.addEdge(path, first.id, second.id, 'branch');
  assert.deepEqual(second.inputs, []); assert.equal(flow.edges.length, 1);
  model.edit(path, second, 'inputs', [{ name: 'task', from: 'start.task' }]); assert.equal(flow.edges.length, 1);
  model.deleteNode(path, first); assert.equal(flow.entry, 'step-1'); assert.equal(flow.edges[0].from, 'step-1');
  const result = await checkWorkbook(model.snapshot(), engine); assert.equal(result.ok, false); assert.match(result.process.stdout, /entry/);
});
test('Unknown fields at every TOML level survive title edits and are individually refused by the real CLI', async () => {
  for (const level of ['manifest', 'flow', 'node', 'instruction', 'input', 'output', 'edge', 'manifest-requires']) {
    const model = new WorkbookModel(await readTree(resolve('../../examples/code-change'))); const flow = model.flow('flows/default.toml');
    let target, path = 'flows/default.toml';
    if (level === 'manifest') { path = 'workbook.toml'; target = model.manifest; }
    if (level === 'flow') target = flow;
    if (level === 'node') target = flow.nodes[0];
    if (level === 'instruction') target = flow.nodes[0].instruction;
    if (level === 'input') target = flow.nodes[0].inputs[0];
    if (level === 'output') target = flow.nodes[0].outputs[0];
    if (level === 'edge') target = flow.edges[0];
    if (level === 'manifest-requires') { path = 'workbook.toml'; model.manifest.requires = [{ kind: 'skill', name: 'sample' }]; target = model.manifest.requires[0]; }
    model.edit(path, target, 'unknown_contract_field', { nested: [1, 2, 3] });
    model.edit('flows/default.toml', flow.nodes[0], 'title', 'title-only-change');
    if (path === 'workbook.toml') model.edit(path, model.manifest, 'name', 'name-only-change');
    assert.match(model.text(path), /unknown_contract_field/);
    const checked = await checkWorkbook(model.snapshot(), engine);
    console.log(JSON.stringify({ boundary: 'real-cli-unknown', level, checked }));
    assert.equal(checked.ok, false, level); assert.match(checked.process.stdout, /unknown field.*unknown_contract_field/);
  }
});
test('Unparseable/unrepresentable Flows preserve bytes and reject editing; HTML instructions remain literal text', () => {
  const base = newWorkbook().snapshot(); const invalid = text('not TOML = ['); base.set('flows/default.toml', invalid); const broken = new WorkbookModel(base);
  assert.throws(() => broken.flow('flows/default.toml'), /original bytes are preserved/); assert.deepEqual(broken.snapshot().get('flows/default.toml'), invalid);
  const model = newWorkbook(), node = model.flow('flows/default.toml').nodes[0]; model.edit('flows/default.toml', node.instruction, 'text', '<img src=x onerror=alert(1)><script>alert(1)</script>');
  assert.equal(parse(model.text('flows/default.toml')).nodes[0].instruction.text, '<img src=x onerror=alert(1)><script>alert(1)</script>');
});
test('Parsed unrepresentable known fields reject further editing without cleaning bytes', () => {
  const bytes = newWorkbook().snapshot();
  bytes.set('flows/default.toml', text('schema="flow/v1"\nid="default"\nentry="one"\n[[nodes]]\nid="one"\ntitle="Original text"\nexecutor="agent"\ninstruction={text="Original text"}\ninputs="cannot represent"\n'));
  const model = new WorkbookModel(bytes); const original = bytes.get('flows/default.toml');
  assert.throws(() => model.edit('flows/default.toml', model.documents.get('flows/default.toml').nodes[0], 'title', 'changed-title'), /cannot be represented/);
  assert.deepEqual(model.snapshot().get('flows/default.toml'), original);
});
test('Editing parsed TOML through instructions reparses it; subsequent property edits preserve the new bytes', () => {
  const model = newWorkbook();
  model.writeText('flows/default.toml', 'schema="flow/v1"\nid="default"\nentry="one"\n[[nodes]]\nid="one"\ntitle="text-edited"\nexecutor="agent"\ninstruction={text="Original text"}\nunknown=77\n');
  const node = model.flow('flows/default.toml').nodes[0]; assert.equal(node.title, 'text-edited');
  model.edit('flows/default.toml', node, 'title', 'property-edited'); assert.equal(parse(model.text('flows/default.toml')).nodes[0].unknown, 77);
});
test('Integral floats never become valid integers; unrepresentable advanced fields retain bytes and reject editing', async () => {
  for (const line of ['max_visits=1.0', 'max_retries=1.0', 'outputs=[{name="report",path="report.md",max_bytes=1.0}]']) {
    const files = newWorkbook().snapshot(); files.set('flows/default.toml', text('schema="flow/v1"\nid="default"\nentry="one"\n[[nodes]]\nid="one"\ntitle="Original text"\nexecutor="agent"\ninstruction={text="Original text"}\n' + line + '\n'));
    const model = new WorkbookModel(files), original = model.snapshot().get('flows/default.toml');
    assert.throws(() => model.edit('flows/default.toml', model.documents.get('flows/default.toml').nodes[0], 'title', 'changed-title'), /cannot be represented/);
    assert.deepEqual(model.snapshot().get('flows/default.toml'), original);
    const checked = await checkWorkbook(model.snapshot(), engine); assert.equal(checked.ok, false); assert.match(checked.process.stdout, /float|floating/);
    console.log(JSON.stringify({ boundary: 'real-cli-integer-float', line, checked }));
  }
});
test('Unknown objects preserve integer/float distinctions and large integers; title edits do not clean nested numbers', async () => {
  const files = newWorkbook().snapshot();
  files.set('flows/default.toml', text('schema="flow/v1"\nid="default"\nentry="one"\n[[nodes]]\nid="one"\ntitle="Original text"\nexecutor="agent"\ninstruction={text="Original text"}\nmax_visits=2\nunknown={float=1.0,integer=1,huge=9223372036854775807,array=[1.0,1]}\n'));
  const model = new WorkbookModel(files), node = model.flow('flows/default.toml').nodes[0];
  assert.equal(node.max_visits, 2n); model.edit('flows/default.toml', node, 'title', 'changed-title');
  const full = parse(model.text('flows/default.toml'), { integersAsBigInt: true });
  assert.equal(full.nodes[0].unknown.float, 1); assert.equal(full.nodes[0].unknown.integer, 1n); assert.equal(full.nodes[0].unknown.huge, 9223372036854775807n); assert.deepEqual(full.nodes[0].unknown.array, [1, 1n]); assert.equal(full.nodes[0].max_visits, 2n);
  const checked = await checkWorkbook(model.snapshot(), engine); assert.equal(checked.ok, false); assert.match(checked.process.stdout, /unknown field.*unknown/);
  const legal = newWorkbook(); const legalNode = legal.flow('flows/default.toml').nodes[0]; legal.edit('flows/default.toml', legalNode, 'max_visits', 3n); legal.edit('flows/default.toml', legalNode, 'max_retries', 0n); legal.edit('flows/default.toml', legalNode, 'outputs', [{ name: 'report', path: 'report.md', max_bytes: 1234n, result: true }]);
  const accepted = await checkWorkbook(legal.snapshot(), engine); assert.equal(accepted.ok, true); console.log(JSON.stringify({ boundary: 'real-cli-integer-preservation', rejected: checked, accepted }));
});

test('Repeated endpoint additions across kinds return the original edge and preserve unknown objects and bytes', () => {
  const model = newWorkbook(), path = 'flows/default.toml', first = model.flow(path).nodes[0], second = model.addNode(path);
  const edge = model.addEdge(path, first.id, second.id, 'main');
  model.edit(path, edge, 'unknown', { keep: ['original', 7n] });
  const before = model.snapshot();
  for (const kind of ['main', 'back', 'branch', 're_review']) {
    assert.equal(model.addEdge(path, first.id, second.id, kind), edge);
    assert.equal(edge.kind, 'main');
    assert.deepEqual(edge.unknown, { keep: ['original', 7n] });
    assert.deepEqual(model.snapshot(), before);
  }
  assert.equal(model.flow(path).edges.length, 1);
  assert.deepEqual(second.inputs, []);
});
test('Endpoint edits reject cross-kind duplicates and self-loops while preserving endpoints and all bytes', () => {
  const model = newWorkbook(), path = 'flows/default.toml', first = model.flow(path).nodes[0], second = model.addNode(path), third = model.addNode(path);
  model.addEdge(path, first.id, second.id, 'branch');
  const edge = model.addEdge(path, first.id, third.id, 'main');
  const reverse = model.addEdge(path, second.id, first.id, 'back');
  const before = model.snapshot();
  assert.throws(() => model.edit(path, edge, 'to', second.id), /duplicates/);
  assert.equal(edge.to, third.id); assert.deepEqual(model.snapshot(), before);
  assert.throws(() => model.edit(path, edge, 'to', first.id), /self-loop/);
  assert.equal(edge.to, third.id); assert.deepEqual(model.snapshot(), before);
  assert.throws(() => model.edit(path, reverse, 'from', first.id), /self-loop/);
  assert.equal(reverse.from, second.id); assert.deepEqual(model.snapshot(), before);
  model.edit(path, edge, 'from', second.id);
  assert.equal(edge.from, second.id); assert.equal(edge.to, third.id);
});
test('Imported duplicate/self-loop bytes remain intact; repeated additions do not clean them and the real CLI refuses them', async () => {
  for (const extra of ['[[edges]]\nfrom="step-1"\nto="step-2"\nkind="branch"\n', '[[edges]]\nfrom="step-1"\nto="step-1"\nkind="main"\n']) {
    const model = newWorkbook(), path = 'flows/default.toml'; model.addNode(path); model.addEdge(path, 'step-1', 'step-2');
    const files = model.snapshot(); files.set(path, text(model.text(path) + '\n' + extra));
    const imported = new WorkbookModel(files), before = imported.snapshot(), edges = imported.flow(path).edges;
    assert.equal(edges.length, 2);
    assert.equal(imported.addEdge(path, 'step-1', 'step-2', 'back'), edges[0]);
    assert.deepEqual(imported.snapshot(), before); assert.equal(edges.length, 2);
    const checked = await checkWorkbook(imported.snapshot(), engine);
    assert.equal(checked.ok, false); assert.match(checked.process.stdout, /FLOW_INVALID/);
    console.log(JSON.stringify({ boundary: 'real-cli-imported-invalid-edge', extra, checked }));
  }
});
test('Repeated endpoint additions pass the real CLI; reverse back edges remain legal and add no inputs', async () => {
  const model = newWorkbook(), path = 'flows/default.toml', first = model.flow(path).nodes[0], second = model.addNode(path);
  const edge = model.addEdge(path, first.id, second.id);
  const before = model.snapshot();
  for (let i = 0; i < 12; i++) assert.equal(model.addEdge(path, first.id, second.id), edge);
  assert.deepEqual(model.snapshot(), before);
  const checked = await checkWorkbook(model.snapshot(), engine);
  assert.equal(checked.ok, true);
  model.edit(path, first, 'max_visits', 2n); model.edit(path, second, 'max_visits', 2n);
  const terminal = model.addNode(path); model.addEdge(path, second.id, terminal.id);
  const reverse = model.addEdge(path, second.id, first.id, 'back');
  assert.notEqual(reverse, edge); assert.equal(model.flow(path).edges.length, 3); assert.deepEqual(second.inputs, []);
  const reversed = await checkWorkbook(model.snapshot(), engine);
  assert.equal(reversed.ok, true);
  console.log(JSON.stringify({ boundary: 'real-cli-repeat-and-reverse-edge', checked, reversed }));
});
