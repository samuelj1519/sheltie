import { WorkbookModel } from './model.mjs';
const key = value => typeof value === 'string' && value.length <= 64 && /^[a-z0-9]+(-[a-z0-9]+)*$/.test(value);
export function sourceChoices(flow, files, owningNodeId) {
  if (typeof owningNodeId !== 'string') throw new Error('来源选择必须指定所属步骤。');
  return { steps: (flow.nodes ?? []).filter(n => n.id !== owningNodeId && key(n.id) && !['start', 'resource', 'engine'].includes(n.id)), resources: [...files.keys()].sort() };
}
export function describeSource(value, flow, files, owningNodeId) {
  if (typeof value !== 'string') return { type: 'custom', raw: value };
  if (value === 'engine.stats') return { type: 'stats' };
  const start = /^start\.([a-z0-9]+(?:-[a-z0-9]+)*)$/.exec(value);
  if (start && key(start[1])) return { type: 'start', key: start[1] };
  const choices = sourceChoices(flow, files, owningNodeId);
  if (value.startsWith('resource.') && choices.resources.includes(value.slice(9))) return { type: 'resource', path: value.slice(9) };
  const parts = value.split('.');
  if (parts.length === 2 && key(parts[0]) && key(parts[1]) && choices.steps.some(n => n.id === parts[0] && (n.outputs ?? []).some(o => o.name === parts[1]))) return { type: 'step', node: parts[0], output: parts[1] };
  return { type: 'custom', raw: value };
}
export function composeSource(draft, flow, files, owningNodeId) {
  const choices = sourceChoices(flow, files, owningNodeId);
  if (draft.type === 'start') return key(draft.key) ? `start.${draft.key}` : undefined;
  if (draft.type === 'stats') return 'engine.stats';
  if (draft.type === 'resource') return choices.resources.includes(draft.path) ? `resource.${draft.path}` : undefined;
  if (draft.type === 'step') return choices.steps.some(n => n.id === draft.node && (n.outputs ?? []).some(o => key(o.name) && o.name === draft.output)) ? `${draft.node}.${draft.output}` : undefined;
  if (draft.type === 'custom') return draft.raw;
  return undefined;
}
export function declarationBoolean(value) { return value === '' ? undefined : value === 'true' ? true : value === 'false' ? false : value; }
export function declarationBytes(value) {
  if (value === '') return undefined;
  if (!/^\d+$/.test(value)) throw new Error('大小上限请输入完整的非负整数字节数。');
  return BigInt(value);
}

const referencePrefix = 'resources/sheltie-editor-ref-';
const referenceHeader = 'Sheltie editor fixed reference v1\n';
const uuidPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
export function fixedReference(files, from, flowPath, ownerId) {
  if (typeof from !== 'string' || !from.startsWith(`resource.${referencePrefix}`)) return undefined;
  const path = from.slice(9), id = path.slice(referencePrefix.length, -4);
  if (!path.endsWith('.txt') || !uuidPattern.test(id) || !files.has(path)) return undefined;
  try {
    const text = new TextDecoder('utf-8', { fatal: true }).decode(files.get(path));
    if (!text.startsWith(referenceHeader)) return undefined;
    const data = JSON.parse(text.slice(referenceHeader.length));
    if (!data || typeof data !== 'object' || Array.isArray(data) || Object.keys(data).sort().join(',') !== 'id,location,owner,schema' || data.schema !== 'sheltie-editor-reference/v1' || data.id !== id || typeof data.location !== 'string' || !data.location.trim()) return undefined;
    if (!data.owner || typeof data.owner !== 'object' || Array.isArray(data.owner) || Object.keys(data.owner).sort().join(',') !== 'flow,node' || data.owner.flow !== flowPath || data.owner.node !== ownerId) return undefined;
    return { path, location: data.location };
  } catch { return undefined; }
}
function inputReason(row, flow, files, ownerId) {
  if (typeof row.name !== 'string') return '资料名称必须是文本。';
  if (row.required !== undefined && typeof row.required !== 'boolean') return '提供要求无法表达。';
  const source = describeSource(row.from, flow, files, ownerId);
  if (source.type === 'custom') return '来源缺失、指向自身或无法识别。';
  if (source.type !== 'step' && row.required === false) return '该来源不能声明为可选资料。';
  if (source.type === 'step') {
    const output = flow.nodes.find(n => n.id === source.node)?.outputs?.find(o => o.name === source.output);
    if (output?.required === false && row.required !== false) return '可选输出不能作为必需资料。';
    const visited = new Set([source.node]), queue = [source.node];
    for (let i = 0; i < queue.length; i++) for (const edge of flow.edges ?? []) if (edge.from === queue[i] && !visited.has(edge.to)) { visited.add(edge.to); queue.push(edge.to); }
    if (!visited.has(ownerId)) return '该来源不能沿流程到达本步骤。';
  }
  return '';
}
export function predecessorMaterials(flow, files, ownerId) {
  const predecessorIds = new Set((flow.edges ?? []).filter(e => e.to === ownerId && e.from !== ownerId).map(e => e.from));
  const currentInputs = flow.nodes?.find(n => n.id === ownerId)?.inputs ?? [], result = [];
  for (const node of flow.nodes ?? []) if (predecessorIds.has(node.id)) {
    for (const [kind, rows] of [['output', node.outputs ?? []], ['input', node.inputs ?? []]]) rows.forEach((row, index) => {
      const item = { key: `${kind}:${node.id}:${index}`, kind, nodeId: node.id, nodeTitle: node.title || node.id, name: row.name, from: kind === 'output' ? `${node.id}.${row.name}` : row.from };
      if (row.required !== undefined) item.required = row.required;
      item.reason = inputReason(item, flow, files, ownerId);
      item.usedNames = typeof item.from === 'string' ? currentInputs.filter(input => input.from === item.from).map(input => input.name) : [];
      result.push(item);
    });
  }
  return result;
}
export function suggestedInputName(name, from, rows) {
  const base = typeof name === 'string' ? name : '资料';
  if (!rows.some(row => row.name === base && row.from !== from)) return base;
  let index = 2, candidate;
  do { candidate = `${base}-${index++}`; } while (rows.some(row => row.name === candidate));
  return candidate;
}
export function planInputRows(existing, requested) {
  const additions = [], skipped = [], conflicts = [];
  for (const input of requested) {
    if (typeof input.name !== 'string') { conflicts.push({ name: input.name, reason: '名称必须是文本。' }); continue; }
    const same = [...existing, ...additions].find(row => row.name === input.name);
    if (same) { if (same.from === input.from) skipped.push(input); else conflicts.push({ name: input.name, reason: '名称已被另一份资料使用，请改名。' }); continue; }
    const row = { name: input.name, from: input.from }; if (input.required !== undefined) row.required = input.required;
    additions.push(row);
  }
  return { additions, skipped, conflicts };
}

function writeReference(model, flowPath, ownerId, location, uuid) {
  if (typeof location !== 'string' || !location.trim()) throw new Error('请输入非空的引用位置。');
  let id, path;
  for (let attempt = 0; attempt < 50; attempt++) {
    id = uuid(); if (!uuidPattern.test(id)) throw new Error('生成的引用标识不合规。');
    path = `${referencePrefix}${id}.txt`; if (!model.files.has(path)) break;
    path = undefined;
  }
  if (!path) throw new Error('不能生成新的引用文件；原方法保持不变。');
  const data = { schema: 'sheltie-editor-reference/v1', id, owner: { flow: flowPath, node: ownerId }, location };
  model.writeText(path, referenceHeader + JSON.stringify(data) + '\n');
  return `resource.${path}`;
}
export function addMaterialInputs(model, flowPath, ownerId, requests, uuid = () => crypto.randomUUID()) {
  const candidate = new WorkbookModel(model.snapshot()), flow = candidate.flow(flowPath), node = flow.nodes.find(n => n.id === ownerId);
  if (!node) throw new Error('所属步骤不存在。');
  const materials = predecessorMaterials(flow, candidate.files, ownerId), rows = node.inputs ?? [], inputs = [];
  for (const request of requests) {
    if (request.location !== undefined) {
      const same = [...rows, ...inputs].find(row => row.name === request.name && fixedReference(candidate.files, row.from, flowPath, ownerId)?.location === request.location);
      if (same) continue;
      if ([...rows, ...inputs].some(row => row.name === request.name)) throw new Error(`资料名称 ${request.name} 已存在，请改名。`);
      if (typeof request.name !== 'string') throw new Error('资料名称必须是文本。');
      inputs.push({ name: request.name, from: writeReference(candidate, flowPath, ownerId, request.location, uuid) });
    } else {
      const item = materials.find(m => m.key === request.key);
      if (!item || item.reason) throw new Error(item?.reason || '所选资料已不存在。');
      inputs.push({ ...item, name: request.name });
    }
  }
  const planned = planInputRows(rows, inputs);
  if (planned.conflicts.length) throw new Error(planned.conflicts.map(c => `${c.name}：${c.reason}`).join('\n'));
  if (!planned.additions.length) return { model, added: 0, skipped: planned.skipped.length };
  node.inputs = [...rows, ...planned.additions]; candidate.commit(flowPath); candidate.snapshot();
  return { model: candidate, added: planned.additions.length, skipped: planned.skipped.length };
}
export function editFixedReference(model, flowPath, ownerId, rowIndex, location, uuid = () => crypto.randomUUID()) {
  const node = model.flow(flowPath).nodes.find(n => n.id === ownerId), row = node?.inputs?.[rowIndex];
  const reference = fixedReference(model.files, row?.from, flowPath, ownerId);
  if (!reference) throw new Error('这是普通参考文件来源，不能作为专用位置引用编辑。');
  if (reference.location === location) return model;
  const candidate = new WorkbookModel(model.snapshot()), next = candidate.flow(flowPath).nodes.find(n => n.id === ownerId);
  next.inputs[rowIndex].from = writeReference(candidate, flowPath, ownerId, location, uuid); candidate.commit(flowPath); candidate.snapshot();
  return candidate;
}

export function bindingSummary(row, kind, flow, files, flowPath, ownerId) {
  if (kind === 'outputs') return row.path === undefined ? '尚未声明文件名' : String(row.path);
  const reference = fixedReference(files, row.from, flowPath, ownerId); if (reference) return `固定引用：${reference.location}`;
  const source = describeSource(row.from, flow, files, ownerId);
  if (source.type === 'start') return `开始时提供 · ${source.key}`;
  if (source.type === 'step') return `${flow.nodes.find(n => n.id === source.node)?.title || source.node} · ${source.output}`;
  if (source.type === 'resource') return `参考文件 · ${source.path}`;
  if (source.type === 'stats') return '运行统计';
  return `原来源 · ${row.from === undefined ? '尚未声明' : String(row.from)}`;
}
export function projectBindingGroups(node, kind, flow, files, flowPath, query = '') {
  if (kind !== 'inputs' && kind !== 'outputs') throw new Error('请选择输入或输出。');
  const groups = new Map(), search = query.toLowerCase();
  (node[kind] ?? []).forEach((row, index) => {
    const summary = bindingSummary(row, kind, flow, files, flowPath, node.id);
    if (!`${row.name ?? ''} ${row.from ?? row.path ?? ''} ${summary}`.toLowerCase().includes(search)) return;
    let label;
    if (kind === 'outputs') { const slash = typeof row.path === 'string' ? row.path.lastIndexOf('/') : -1; label = slash < 0 ? '输出文件' : `文件夹 · ${row.path.slice(0, slash)}`; }
    else if (fixedReference(files, row.from, flowPath, node.id)) label = '固定引用';
    else {
      const source = describeSource(row.from, flow, files, node.id);
      if (source.type === 'step') label = `前置步骤 · ${flow.nodes.find(n => n.id === source.node)?.title || source.node}`;
      else label = { start: '开始时提供', resource: '方法参考文件', stats: '运行统计', custom: '自定义或原始来源' }[source.type];
    }
    if (!groups.has(label)) groups.set(label, []);
    groups.get(label).push({ index, row, summary });
  });
  return [...groups].map(([label, rows]) => ({ label, rows }));
}

export function pruneUsedStaged(staged, materials) {
  const used = new Set(materials.filter(item => item.usedNames.length).map(item => item.key)), before = staged.length;
  for (let i = staged.length - 1; i >= 0; i--) if (used.has(staged[i].key)) staged.splice(i, 1);
  return before - staged.length;
}
export function removeStagedChoice(staged, item) {
  const index = staged.indexOf(item); if (index < 0) return false; staged.splice(index, 1); return true;
}
