import test from 'node:test';
import assert from 'node:assert/strict';
import { parse } from 'smol-toml';
import { WorkbookModel, newWorkbook } from '../public/model.mjs';
import { readTree } from '../lib/files.mjs';
import { checkWorkbook } from '../lib/engine.mjs';
import { resolve } from 'node:path';
const engine = process.env.SHELTIE_EDITOR_ENGINE;
const text = value => new TextEncoder().encode(value);
test('改一个字段保留完整对象/其他 Flow/资源原字节', async () => {
  const source = await readTree(resolve('../../examples/code-change'));
  source.set('resources/bin', Uint8Array.from([0, 1, 254, 255]));
  const manifest = parse(new TextDecoder().decode(source.get('workbook.toml')));
  source.set('workbook.toml', text(new TextDecoder().decode(source.get('workbook.toml')).replace('flows = ["flows/default.toml"]', 'flows = ["flows/default.toml", "flows/other.toml"]')));
  source.set('flows/other.toml', text('schema="flow/v1"\nid="other"\nentry="one"\n[[nodes]]\nid="one"\ntitle="另一 Flow"\nexecutor="agent"\ninstruction={text="保持"}\n'));
  const model = new WorkbookModel(source);
  model.edit('flows/default.toml', model.flow('flows/default.toml').nodes[0], 'title', '新标题');
  for (const [path, bytes] of source) if (path !== 'flows/default.toml') assert.deepEqual(model.snapshot().get(path), bytes, path);
  const edited = parse(model.text('flows/default.toml'));
  assert.equal(edited.nodes[0].tier, 'strong'); assert.equal(edited.nodes[0].max_visits, 3); assert.equal(edited.nodes[0].outputs[0].max_bytes, 262144); assert.equal(edited.nodes[2].inputs[1].result, true);
  assert.equal(manifest.id, 'code-change');
});
test('新增显式边不添加 input；编辑 input 不添加 edge；删除保留引用给 CLI 拒绝', async () => {
  const model = newWorkbook(); const path = 'flows/default.toml', flow = model.flow(path), first = flow.nodes[0];
  const second = model.addNode(path); model.addEdge(path, first.id, second.id, 'branch');
  assert.deepEqual(second.inputs, []); assert.equal(flow.edges.length, 1);
  model.edit(path, second, 'inputs', [{ name: 'task', from: 'start.task' }]); assert.equal(flow.edges.length, 1);
  model.deleteNode(path, first); assert.equal(flow.entry, 'step-1'); assert.equal(flow.edges[0].from, 'step-1');
  const result = await checkWorkbook(model.snapshot(), engine); assert.equal(result.ok, false); assert.match(result.process.stdout, /entry/);
});
test('所有 TOML 层级未知字段改标题后仍保留，真实 CLI 逐个拒绝', async () => {
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
    model.edit('flows/default.toml', flow.nodes[0], 'title', '仅改标题');
    if (path === 'workbook.toml') model.edit(path, model.manifest, 'name', '仅改名称');
    assert.match(model.text(path), /unknown_contract_field/);
    const checked = await checkWorkbook(model.snapshot(), engine);
    console.log(JSON.stringify({ boundary: 'real-cli-unknown', level, checked }));
    assert.equal(checked.ok, false, level); assert.match(checked.process.stdout, /unknown field.*unknown_contract_field/);
  }
});
test('不能解析或不能表示的 Flow 保留原字节并拒绝编辑；HTML说明是字面文本', () => {
  const base = newWorkbook().snapshot(); const invalid = text('not TOML = ['); base.set('flows/default.toml', invalid); const broken = new WorkbookModel(base);
  assert.throws(() => broken.flow('flows/default.toml'), /原字节保留/); assert.deepEqual(broken.snapshot().get('flows/default.toml'), invalid);
  const model = newWorkbook(), node = model.flow('flows/default.toml').nodes[0]; model.edit('flows/default.toml', node.instruction, 'text', '<img src=x onerror=alert(1)><script>alert(1)</script>');
  assert.equal(parse(model.text('flows/default.toml')).nodes[0].instruction.text, '<img src=x onerror=alert(1)><script>alert(1)</script>');
});
test('已解析但不能表示的已知字段拒绝后续编辑，不清洗字节', () => {
  const bytes = newWorkbook().snapshot();
  bytes.set('flows/default.toml', text('schema="flow/v1"\nid="default"\nentry="one"\n[[nodes]]\nid="one"\ntitle="原文"\nexecutor="agent"\ninstruction={text="原文"}\ninputs="cannot represent"\n'));
  const model = new WorkbookModel(bytes); const original = bytes.get('flows/default.toml');
  assert.throws(() => model.edit('flows/default.toml', model.documents.get('flows/default.toml').nodes[0], 'title', '改标题'), /无法由界面表示/);
  assert.deepEqual(model.snapshot().get('flows/default.toml'), original);
});
test('通过说明编辑同一已解析TOML时重新解析，后续属性不会覆盖新字节', () => {
  const model = newWorkbook();
  model.writeText('flows/default.toml', 'schema="flow/v1"\nid="default"\nentry="one"\n[[nodes]]\nid="one"\ntitle="文本修改"\nexecutor="agent"\ninstruction={text="原文"}\nunknown=77\n');
  const node = model.flow('flows/default.toml').nodes[0]; assert.equal(node.title, '文本修改');
  model.edit('flows/default.toml', node, 'title', '属性修改'); assert.equal(parse(model.text('flows/default.toml')).nodes[0].unknown, 77);
});
test('整数型float不变成合法integer；高级整数字段无法表示时保留原字节拒绝编辑', async () => {
  for (const line of ['max_visits=1.0', 'max_retries=1.0', 'outputs=[{name="report",path="report.md",max_bytes=1.0}]']) {
    const files = newWorkbook().snapshot(); files.set('flows/default.toml', text('schema="flow/v1"\nid="default"\nentry="one"\n[[nodes]]\nid="one"\ntitle="原文"\nexecutor="agent"\ninstruction={text="原文"}\n' + line + '\n'));
    const model = new WorkbookModel(files), original = model.snapshot().get('flows/default.toml');
    assert.throws(() => model.edit('flows/default.toml', model.documents.get('flows/default.toml').nodes[0], 'title', '改标题'), /无法由界面表示/);
    assert.deepEqual(model.snapshot().get('flows/default.toml'), original);
    const checked = await checkWorkbook(model.snapshot(), engine); assert.equal(checked.ok, false); assert.match(checked.process.stdout, /float|floating/);
    console.log(JSON.stringify({ boundary: 'real-cli-integer-float', line, checked }));
  }
});
test('完整未知对象保留整数/float区别与大整数；编辑普通标题不洗白嵌套数值', async () => {
  const files = newWorkbook().snapshot();
  files.set('flows/default.toml', text('schema="flow/v1"\nid="default"\nentry="one"\n[[nodes]]\nid="one"\ntitle="原文"\nexecutor="agent"\ninstruction={text="原文"}\nmax_visits=2\nunknown={float=1.0,integer=1,huge=9223372036854775807,array=[1.0,1]}\n'));
  const model = new WorkbookModel(files), node = model.flow('flows/default.toml').nodes[0];
  assert.equal(node.max_visits, 2n); model.edit('flows/default.toml', node, 'title', '改标题');
  const full = parse(model.text('flows/default.toml'), { integersAsBigInt: true });
  assert.equal(full.nodes[0].unknown.float, 1); assert.equal(full.nodes[0].unknown.integer, 1n); assert.equal(full.nodes[0].unknown.huge, 9223372036854775807n); assert.deepEqual(full.nodes[0].unknown.array, [1, 1n]); assert.equal(full.nodes[0].max_visits, 2n);
  const checked = await checkWorkbook(model.snapshot(), engine); assert.equal(checked.ok, false); assert.match(checked.process.stdout, /unknown field.*unknown/);
  const legal = newWorkbook(); const legalNode = legal.flow('flows/default.toml').nodes[0]; legal.edit('flows/default.toml', legalNode, 'max_visits', 3n); legal.edit('flows/default.toml', legalNode, 'max_retries', 0n); legal.edit('flows/default.toml', legalNode, 'outputs', [{ name: 'report', path: 'report.md', max_bytes: 1234n, result: true }]);
  const accepted = await checkWorkbook(legal.snapshot(), engine); assert.equal(accepted.ok, true); console.log(JSON.stringify({ boundary: 'real-cli-integer-preservation', rejected: checked, accepted }));
});
