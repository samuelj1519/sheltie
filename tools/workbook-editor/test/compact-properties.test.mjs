import test from 'node:test';
import assert from 'node:assert/strict';
import { readTree } from '../lib/files.mjs';
import { WorkbookModel } from '../public/model.mjs';
import { predecessorMaterials } from '../public/sources.mjs';
import { fileURLToPath } from 'node:url';
test('真实plan12输入已有source按from投影勾选与别名，而不是等待新staged', async () => {
  const model = new WorkbookModel(await readTree(fileURLToPath(new URL('../../../workbooks/spec-dev/', import.meta.url)))), flow = model.flow('flows/default.toml'), plan = flow.nodes.find(n => n.id === 'plan');
  assert.equal(plan.inputs.length, 12); assert.equal(plan.outputs.length, 2);
  const options = predecessorMaterials(flow, model.files, 'plan');
  assert.deepEqual(options.find(o => o.kind === 'output' && o.from === 'spec.spec').usedNames, ['spec']);
  assert.deepEqual(options.find(o => o.kind === 'output' && o.from === 'plan-review.reviewed-plan').usedNames, ['previous_plan']);
  assert.deepEqual(options.find(o => o.kind === 'output' && o.from === 'plan-review.reviewed-tasks').usedNames, ['previous_tasks']);
});
import { projectBindingGroups, bindingSummary, addMaterialInputs } from '../public/sources.mjs';
import { newWorkbook } from '../public/model.mjs';
import { checkWorkbook } from '../lib/engine.mjs';
function sourceFixture() {
  const model = newWorkbook(), path = 'flows/default.toml', node = model.addNode(path), upstream = model.flow(path).nodes[0];
  upstream.outputs = [{ name: 'draft', path: 'draft.md' }]; upstream.inputs = [{ name: 'task', from: 'start.task' }]; model.addEdge(path, upstream.id, node.id); model.commit(path); return { model, path, ownerId: node.id };
}
function exactBytes(actual, expected) {
  assert.deepEqual([...actual.keys()].sort(), [...expected.keys()].sort());
  for (const [path, bytes] of expected) assert.deepEqual(Buffer.from(actual.get(path)), Buffer.from(bytes), path);
}
test('同from多个别名/required精确派生；改名、移除一个及最后一个后刷新而不合并声明', () => {
  const { model, path, ownerId } = sourceFixture(), flow = model.flow(path), owner = flow.nodes[1];
  owner.inputs = [{ name: '第一名', from: 'step-1.draft', required: false, result: false }, { name: 'second_alias', from: 'step-1.draft', required: true }, { name: 'same-name-other-source', from: 'start.task' }]; model.commit(path);
  const usage = () => predecessorMaterials(flow, model.files, ownerId).find(o => o.from === 'step-1.draft').usedNames;
  assert.deepEqual(usage(), ['第一名', 'second_alias']); assert.equal(owner.inputs.length, 3);
  model.edit(path, owner.inputs[0], 'name', '改名_保持来源'); assert.deepEqual(usage(), ['改名_保持来源', 'second_alias']); assert.deepEqual(owner.inputs[0], { name: '改名_保持来源', from: 'step-1.draft', required: false, result: false });
  owner.inputs.splice(0, 1); model.commit(path); assert.deepEqual(usage(), ['second_alias']); assert.equal(owner.inputs[0].required, true);
  owner.inputs.splice(0, 1); model.commit(path); assert.deepEqual(usage(), []); assert.equal(owner.inputs[0].from, 'start.task');
});
test('多个前置候选共享from都显示同实际使用名，不以候选name或required来猜勾选', () => {
  const flow = { nodes: [{ id: 'up', outputs: [{ name: 'draft', path: 'draft.md' }] }, { id: 'middle', inputs: [{ name: 'renamed', from: 'up.draft', required: false }] }, { id: 'owner', inputs: [{ name: 'actual_name', from: 'up.draft', required: true }, { name: 'optional_alias', from: 'up.draft', required: false }, { name: 'draft', from: 'start.task' }] }], edges: [{ from: 'up', to: 'owner', kind: 'main' }, { from: 'middle', to: 'owner', kind: 'branch' }] };
  const before = structuredClone(flow), options = predecessorMaterials(flow, new Map(), 'owner').filter(o => o.from === 'up.draft');
  assert.equal(options.length, 2); assert.ok(options.every(o => JSON.stringify(o.usedNames) === JSON.stringify(['actual_name', 'optional_alias']))); assert.deepEqual(flow, before);
  flow.nodes[2].inputs = [{ name: 'draft', from: 'start.task' }]; assert.ok(predecessorMaterials(flow, new Map(), 'owner').filter(o => o.from === 'up.draft').every(o => o.usedNames.length === 0));
});
test('非法原source也精确显示已有状态，unknown/三态/大整数和浮点原字节不被projection改写', () => {
  const { model, path, ownerId } = sourceFixture(), flow = model.flow(path), upstream = flow.nodes[0], owner = flow.nodes[1];
  upstream.inputs.push({ name: 'broken', from: 'missing.out', required: false });
  owner.inputs = [{ name: '原引用', from: 'missing.out', required: false, result: false, untouched: { integer: 9007199254740993n, float: 1.0 } }, { name: 'missing-value' }]; model.commit(path); const bytes = model.snapshot();
  const option = predecessorMaterials(flow, model.files, ownerId).find(o => o.from === 'missing.out'); assert.deepEqual(option.usedNames, ['原引用']); assert.ok(option.reason);
  const groups = projectBindingGroups(owner, 'inputs', flow, model.files, path);
  assert.equal(groups[0].label, '自定义或原始来源'); assert.equal(groups[0].rows.length, 2); assert.equal(groups[0].rows[0].row, owner.inputs[0]); exactBytes(model.snapshot(), bytes);
});
test('分组与搜索保首次来源组/原row索引，输入输出切换只投影原27样例文件', async () => {
  const model = new WorkbookModel(await readTree(fileURLToPath(new URL('../../../workbooks/spec-dev/', import.meta.url)))), flow = model.flow('flows/default.toml'), node = flow.nodes.find(n => n.id === 'plan'), before = model.snapshot();
  const groups = projectBindingGroups(node, 'inputs', flow, model.files, 'flows/default.toml');
  assert.equal(groups.reduce((n, g) => n + g.rows.length, 0), 12); assert.equal(groups[0].label, '前置步骤 · 写需求规格'); assert.equal(groups[0].rows[0].index, 0);
  const matched = projectBindingGroups(node, 'inputs', flow, model.files, 'flows/default.toml', 'PREVIOUS_PLAN'); assert.equal(matched.length, 1); assert.equal(matched[0].rows[0].row.name, 'previous_plan'); assert.equal(matched[0].rows[0].index, 7);
  const outputs = projectBindingGroups(node, 'outputs', flow, model.files, 'flows/default.toml'); assert.deepEqual(outputs[0].rows.map(r => [r.index, r.row.name, r.summary]), [[0, 'plan', 'plan.md'], [1, 'tasks', 'tasks.md']]);
  assert.equal(projectBindingGroups(node, 'outputs', flow, model.files, 'flows/default.toml', 'none').length, 0); assert.equal(before.size, 27); exactBytes(model.snapshot(), before);
});
test('分组真实继承资料后已用状态刷新，单项name/path编辑保存与重新解析保from和高级声明', async () => {
  const engine = process.env.SHELTIE_EDITOR_ENGINE; assert.ok(engine, '需要真实固定引擎');
  const { model, path, ownerId } = sourceFixture();
  const added = addMaterialInputs(model, path, ownerId, [{ key: 'output:step-1:0', name: '实现_报告' }, { key: 'input:step-1:0', name: '原始任务' }]).model, flow = added.flow(path), owner = flow.nodes[1];
  assert.deepEqual(predecessorMaterials(flow, added.files, ownerId).find(o => o.from === 'step-1.draft').usedNames, ['实现_报告']);
  added.edit(path, owner.inputs[0], 'name', '重新命名'); assert.equal(owner.inputs[0].from, 'step-1.draft');
  owner.outputs = [{ name: 'report', path: 'old.md', required: true, result: true, max_bytes: 12345n }]; added.commit(path); added.edit(path, owner.outputs[0], 'path', 'reports/new.md');
  const reopened = new WorkbookModel(added.snapshot()), stored = reopened.flow(path).nodes[1]; assert.deepEqual(stored.inputs, [{ name: '重新命名', from: 'step-1.draft' }, { name: '原始任务', from: 'start.task' }]); assert.deepEqual(stored.outputs[0], { name: 'report', path: 'reports/new.md', required: true, result: true, max_bytes: 12345n });
  const result = await checkWorkbook(reopened.snapshot(), engine); console.log(JSON.stringify({ boundary: 'real-cli-compact-single-edit', result })); assert.equal(result.ok, true);
});

import { pruneUsedStaged, removeStagedChoice } from '../public/sources.mjs';
test('已有来源刷新剔除staged A后，旧A chip回调不能删另一待添加B', () => {
  const a = { key: 'output:up:0', name: 'A' }, b = { key: 'output:up:1', name: 'B' }, staged = [a, b];
  assert.equal(pruneUsedStaged(staged, [{ key: a.key, usedNames: ['已有A'] }, { key: b.key, usedNames: [] }]), 1);
  assert.deepEqual(staged, [b]); removeStagedChoice(staged, a); assert.deepEqual(staged, [b]);
  removeStagedChoice(staged, b); assert.deepEqual(staged, []);
});
