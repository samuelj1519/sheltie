import { WorkbookModel, newWorkbook } from './model.mjs';
import { readDirectory, toWire } from './files.mjs';
import { CanvasView } from './layout.mjs';
import { syncTextPreviews, describeFailure } from './presentation.mjs';
const $ = id => document.getElementById(id);
let model = newWorkbook(), flowPath = model.manifest.flows[0], selection = { type: 'manifest' }, token, dirty = false, busy = false;
let view = new CanvasView(), drag;
const svgNS = 'http://www.w3.org/2000/svg';
function element(tag, text, className) {
  const e = document.createElement(tag);
  if (text !== undefined) e.textContent = String(text);
  if (className) e.className = className;
  return e;
}
function safe(fn) { return (...args) => { try { const result = fn(...args); if (result?.catch) result.catch(showError); } catch (error) { showError(error); } }; }
function button(text, fn, className) { const b = element('button', text, className); b.type = 'button'; b.addEventListener('click', safe(fn)); return b; }
function status(text, error = false) { $('status').textContent = text; $('status').style.color = error ? 'var(--red)' : 'var(--text)'; }
function details(text) { $('error-text').textContent = text; }
function clearFailure() { $('error-summary').hidden = true; $('error-links').replaceChildren(); }
function openErrorCandidate(path, fieldPath) {
  if (path === 'workbook.toml') selection = { type: 'manifest' };
  else {
    flowPath = path; loadView();
    const index = /^nodes\[(\d+)\]/.exec(fieldPath)?.[1];
    const node = index === undefined ? undefined : flow().nodes[Number(index)];
    selection = node ? { type: 'node', node } : { type: 'flow' };
  }
  render();
  const key = fieldPath.split('.').at(-1);
  for (const input of $('property-content').querySelectorAll('[data-field-key]')) if (input.dataset.fieldKey === key) { input.focus(); break; }
  status(`已打开候选 ${path}，请核对引擎字段 ${fieldPath || '未提供'}。`, true);
}
function showFailure(result) {
  const issue = describeFailure(result, model.manifest.flows);
  $('error-summary').hidden = false;
  $('error-message').textContent = issue.message;
  $('error-fields').replaceChildren();
  for (const [label, value] of [['错误码', issue.code], ['规则', issue.rule], ['字段', issue.path], ['原因', issue.reason]]) {
    if (value) $('error-fields').append(element('dt', label), element('dd', value));
  }
  $('error-next').textContent = `下一步：${issue.next}`;
  $('error-file-note').textContent = issue.fileNote;
  $('error-links').replaceChildren();
  for (const path of issue.candidates) if (model.files.has(path)) $('error-links').append(button(`打开候选 ${path}`, () => openErrorCandidate(path, issue.path)));
  details(JSON.stringify(result, null, 2)); $('error-details').open = false;
  status(`检查失败：${issue.message}`, true);
}
function showError(error) { showFailure({ error: error.message ?? String(error) }); details(error.stack ?? String(error)); }
function changed() {
  dirty = true; clearFailure();
  syncTextPreviews(model, $('property-content'));
  status('草稿已修改。检查结构后保存新副本。');
}
function edit(path, target, key, value) { model.edit(path, target, key, value); changed(); renderNavigation(); renderCanvas(); }
function field(parent, label, value, onChange, { type = 'text', help, options, optional = false, mono = false, fieldKey } = {}) {
  if (value !== undefined && !['string', 'number', 'bigint', 'boolean'].includes(typeof value)) throw new Error(`${label} 的值不能由表单表示；原字节保留。`);
  const wrap = element('label', undefined, 'field'); wrap.append(element('span', label));
  const input = element(options ? 'select' : type === 'textarea' ? 'textarea' : 'input');
  if (options) {
    for (const [key, text] of options) { const option = element('option', text); option.value = key; input.append(option); }
    const val = value === undefined ? '' : String(value);
    if (![...input.options].some(o => o.value === val)) { const option = element('option', `原值：${val}`); option.value = val; input.append(option); }
  } else if (type !== 'textarea') input.type = type;
  input.value = value === undefined ? '' : String(value);
  if (fieldKey) input.dataset.fieldKey = fieldKey;
  if (mono) input.classList.add('mono');
  input.addEventListener('change', safe(() => {
    let value = input.value;
    if (optional && value === '') value = undefined;
    else if (type === 'number') { value = Number(value); if (!Number.isSafeInteger(value)) throw new Error(`${label} 必须为整数。`); value = BigInt(value); }
    onChange(value);
  }));
  wrap.append(input);
  if (help) wrap.append(element('small', help, 'field-help'));
  parent.append(wrap); return input;
}
const boolOptions = [['', '未声明（使用默认值）'], ['true', '是'], ['false', '否']];
function booleanField(parent, label, value, onChange, help) { field(parent, label, value, v => onChange(v === '' ? undefined : v === 'true'), { options: boolOptions, help }); }
function advanced(parent, label = '高级设置') { const d = element('details'); d.append(element('summary', label)); parent.append(d); return d; }
function section(parent, title, add) { const head = element('div', undefined, 'section-label'); head.append(element('h3', title)); if (add) head.append(button('＋ 添加', add)); parent.append(head); }
function flow() { return model.flow(flowPath); }
function layoutKey() { return `sheltie-editor-layout:${model.manifest.id}:${flowPath}`; }
function loadView() { view = CanvasView.load(localStorage, layoutKey()); }
function saveView() { try { view.save(localStorage, layoutKey()); } catch { status('布局无法保存在浏览器；Workbook 草稿仍可保存。'); } }
function position(node, index) { return view.position(node.id, index); }
function svg(tag, attributes = {}, text) { const e = document.createElementNS(svgNS, tag); for (const [k, v] of Object.entries(attributes)) e.setAttribute(k, String(v)); if (text !== undefined) e.textContent = String(text); return e; }
function renderNavigation() {
  $('method').replaceChildren(button(model.manifest.name || '未命名方法', () => { selection = { type: 'manifest' }; renderProperties(); }, `nav-item ${selection.type === 'manifest' ? 'active' : ''}`));
  $('flows').replaceChildren();
  for (const path of model.manifest.flows) {
    const b = button(path, () => { flowPath = path; selection = { type: 'flow' }; loadView(); render(); }, `nav-item ${path === flowPath ? 'active' : ''}`);
    const parsed = model.documents.get(path); b.append(element('small', model.errors.has(path) ? '无法解析 · 原字节保留' : `Flow ${parsed?.id ?? ''}`)); $('flows').append(b);
  }
  $('node-list').replaceChildren();
  try { for (const node of flow().nodes) $('node-list').append(button(`${node.title ?? node.id}`, () => selectNode(node), `nav-item ${selection.node === node ? 'active' : ''}`)); } catch { /* 无法表示的 Flow 在属性面板显示原错误。 */ }
  $('flow-label').textContent = flowPath;
}
function selectNode(node) { selection = { type: 'node', node }; renderNavigation(); renderCanvas(); renderProperties(); }
function selectEdge(edge) { selection = { type: 'edge', edge }; renderNavigation(); renderCanvas(); renderProperties(); }
function renderCanvas() {
  $('nodes').replaceChildren(); $('edges').replaceChildren();
  $('world').style.transform = `translate(${view.x}px,${view.y}px) scale(${view.scale})`;
  $('zoom-label').textContent = `${Math.round(view.scale * 100)}%`;
  let current; try { current = flow(); } catch (error) { status(`${flowPath}：${error.message}`, true); return; }
  const defs = svg('defs'), marker = svg('marker', { id: 'arrow', viewBox: '0 0 10 10', refX: 9, refY: 5, markerWidth: 7, markerHeight: 7, orient: 'auto-start-reverse' }); marker.append(svg('path', { d: 'M 0 0 L 10 5 L 0 10 z', fill: '#7994b6' })); defs.append(marker); $('edges').append(defs);
  for (const edge of current.edges ?? []) {
    const from = current.nodes.find(n => n.id === edge.from), to = current.nodes.find(n => n.id === edge.to);
    if (!from || !to) continue;
    const a = position(from, current.nodes.indexOf(from)), b = position(to, current.nodes.indexOf(to));
    const x1 = a.x + 206, y1 = a.y + 57, x2 = b.x, y2 = b.y + 57, bend = Math.max(65, Math.abs(x2 - x1) * .45);
    const d = `M${x1},${y1} C${x1 + bend},${y1} ${x2 - bend},${y2} ${x2},${y2}`;
    const group = svg('g'); const hit = svg('path', { d, class: 'edge-hit', tabindex: 0, role: 'button', 'aria-label': `${edge.from} 到 ${edge.to}，${edge.kind}` });
    group.addEventListener('pointerdown', event => event.stopPropagation()); group.addEventListener('click', () => selectEdge(edge)); hit.addEventListener('keydown', event => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); selectEdge(edge); } });
    group.append(hit, svg('path', { d, class: `edge ${selection.edge === edge ? 'active' : ''}`, 'marker-end': 'url(#arrow)', 'pointer-events': 'none' }), svg('text', { x: (x1 + x2) / 2, y: (y1 + y2) / 2 - 9, 'text-anchor': 'middle' }, edge.kind)); $('edges').append(group);
  }
  current.nodes.forEach((node, index) => {
    const pos = position(node, index);
    const b = button('', () => selectNode(node), `node ${selection.node === node ? 'selected' : ''} ${node.gate ? 'gated' : ''}`);
    b.style.left = `${pos.x}px`; b.style.top = `${pos.y}px`;
    const head = element('span', undefined, 'node-head'); head.append(element('span', node.id)); if (node.id === current.entry) head.append(element('span', '入口', 'entry-mark'));
    b.append(head, element('span', node.title, 'node-title'), element('span', `${node.executor === 'human' ? '人工' : 'agent'}${node.gate ? ' · 门槛' : ''} · ${(node.inputs ?? []).length} 输入 / ${(node.outputs ?? []).length} 输出`, 'node-meta'));
    b.addEventListener('pointerdown', event => {
      if (event.button !== 0) return;
      event.stopPropagation(); selectNode(node); $('viewport').setPointerCapture(event.pointerId);
      drag = { type: 'node', id: node.id, startX: event.clientX, startY: event.clientY, x: pos.x, y: pos.y };
    });
    $('nodes').append(b);
  });
}
function renderBindings(parent, node, key) {
  const output = key === 'outputs', title = output ? '输出' : '输入';
  if (node[key] !== undefined && (!Array.isArray(node[key]) || node[key].some(row => !row || typeof row !== 'object' || Array.isArray(row)))) throw new Error(`${key} 不能由表单表示；原字节保留。`);
  section(parent, title, () => {
    node[key] ??= []; let number = 1; while (node[key].some(row => row.name === `${output ? 'output' : 'input'}-${number}`)) number++;
    node[key].push(output ? { name: `output-${number}`, path: `output-${number}.md` } : { name: `input-${number}`, from: 'start.task' }); model.commit(flowPath); changed(); render();
  });
  if (!output) parent.append(element('p', '输入来源独立于显式边；修改绑定不会自动连线。', 'field-help'));
  (node[key] ?? []).forEach((row, index) => {
    const box = element('div', undefined, 'binding-row'); parent.append(box);
    field(box, '名称 name', row.name, v => edit(flowPath, row, 'name', v), { mono: true });
    field(box, output ? '输出路径 path' : '来源 from', row[output ? 'path' : 'from'], v => edit(flowPath, row, output ? 'path' : 'from', v), { mono: true, help: output ? '相对 outputs/；例如 report.md。' : 'start.task、resource.resources/file、engine.stats 或 node.output。' });
    booleanField(box, '必需 required（默认是）', row.required, v => edit(flowPath, row, 'required', v));
    if (output) field(box, '最大字节 max_bytes（默认 1048576）', row.max_bytes, v => edit(flowPath, row, 'max_bytes', v), { type: 'number', optional: true });
    booleanField(box, '选为最终成果 result（默认否）', row.result, v => edit(flowPath, row, 'result', v), '只允许终点的必需项。');
    box.append(button('删除此声明', () => { node[key].splice(index, 1); model.commit(flowPath); changed(); render(); }, 'danger'));
  });
}
function instructionForm(parent, node) {
  if (!node.instruction || typeof node.instruction !== 'object' || Array.isArray(node.instruction)) throw new Error('instruction 无法表示，原字节保留。');
  const instruction = node.instruction;
  if ((instruction.file === undefined) === (instruction.text === undefined)) throw new Error('instruction 必须恰有 file 或 text；原字节保留，不能静默选择。');
  field(parent, '说明方式', instruction.file !== undefined ? 'file' : 'text', value => {
    if (value === 'file') { delete instruction.text; instruction.file = `instructions/${node.id}.md`; }
    else { let text = '说明需要读取什么、完成什么，以及输出要求。'; if (instruction.file && model.files.has(instruction.file)) text = model.text(instruction.file); delete instruction.file; instruction.text = text; }
    model.commit(flowPath); changed(); renderProperties();
  }, { options: [['text', '内嵌文本 text'], ['file', '说明文件 file']] });
  if (instruction.file !== undefined) {
    field(parent, '说明文件路径', instruction.file, value => { edit(flowPath, instruction, 'file', value); renderProperties(); }, { mono: true });
    if (model.files.has(instruction.file)) field(parent, '说明文件原文', model.text(instruction.file), value => { model.writeText(instruction.file, value); changed(); if (model.documents.has(instruction.file)) { const id = node.id; try { const refreshed = flow().nodes.find(n => n.id === id); selection = refreshed ? { type: 'node', node: refreshed } : { type: 'flow' }; } catch { selection = { type: 'flow' }; } render(); } }, { type: 'textarea', help: '仅修改草稿中的此文件；共用该文件的节点也会看到修改。' });
    else { parent.append(element('p', '此说明文件尚不存在。创建文件后编辑，或切换为内嵌文本。', 'warn')); parent.append(button('创建说明文件', () => { model.writeText(instruction.file, '说明需要读取什么、完成什么，以及输出要求。\n'); changed(); renderProperties(); })); }
  } else field(parent, '说明文本', instruction.text, value => edit(flowPath, instruction, 'text', value), { type: 'textarea' });
}
function nodeForm(parent, node) {
  field(parent, '节点 ID', node.id, value => edit(flowPath, node, 'id', value), { mono: true, fieldKey: 'id', help: '改 ID 后需手动更新入口、边和输入引用；检查会定位遗留引用。' });
  field(parent, '标题', node.title, value => edit(flowPath, node, 'title', value), { fieldKey: 'title' });
  field(parent, '执行者 executor', node.executor, value => edit(flowPath, node, 'executor', value), { options: [['agent', 'agent'], ['human', '人工 human']] });
  parent.append(button('设为入口', () => edit(flowPath, flow(), 'entry', node.id)));
  section(parent, '说明'); instructionForm(parent, node);
  renderBindings(parent, node, 'inputs'); renderBindings(parent, node, 'outputs');
  const extra = advanced(parent);
  field(extra, '档位 tier', node.tier, value => edit(flowPath, node, 'tier', value === '' ? undefined : value), { options: [['', '未声明（standard）'], ['standard', 'standard'], ['strong', 'strong']], help: '人工执行者不得声明 tier；可选择「未声明」。' });
  booleanField(extra, '门槛 gate（默认否）', node.gate, value => edit(flowPath, node, 'gate', value));
  field(extra, '最多到达 max_visits（默认 1）', node.max_visits, value => edit(flowPath, node, 'max_visits', value), { type: 'number', optional: true });
  field(extra, '失败重试 max_retries（默认 1）', node.max_retries, value => edit(flowPath, node, 'max_retries', value), { type: 'number', optional: true });
  if (node.requires !== undefined && (!Array.isArray(node.requires) || node.requires.some(v => typeof v !== 'string'))) throw new Error('requires 无法表示，原字节保留。');
  field(extra, '宿主资源 requires（每行一个 kind:name）', node.requires?.join('\n'), value => edit(flowPath, node, 'requires', value === undefined ? undefined : value.split('\n').map(v => v.trim()).filter(Boolean)), { type: 'textarea', optional: true, mono: true, help: '需先在方法的高级设置声明资源；不会安装到宿主。' });
  parent.append(button('删除节点，保留引用供检查', () => { model.deleteNode(flowPath, node); selection = { type: 'flow' }; changed(); render(); }, 'danger'));
}
function manifestForm(parent) {
  for (const [key, label] of [['id', '方法 ID'], ['version', '版本'], ['name', '名称'], ['description', '描述']]) field(parent, label, model.manifest[key], value => edit('workbook.toml', model.manifest, key, value), { type: key === 'description' ? 'textarea' : 'text', optional: key === 'description', mono: ['id', 'version'].includes(key) });
  parent.append(element('p', `${model.files.size} 个文件。未编辑 Flow、说明与资源保留原字节。`, 'field-help'));
  const extra = advanced(parent, '高级：宿主资源声明 requires');
  const rows = model.manifest.requires;
  if (rows !== undefined && (!Array.isArray(rows) || rows.some(row => !row || typeof row !== 'object' || Array.isArray(row)))) throw new Error('manifest requires 无法由界面表示；原字节保留。');
  extra.append(button('＋ 添加资源声明', () => { model.manifest.requires ??= []; model.manifest.requires.push({ kind: 'skill', name: 'resource-name' }); model.commit('workbook.toml'); changed(); renderProperties(); }));
  (rows ?? []).forEach((row, index) => {
    const box = element('div', undefined, 'binding-row'); extra.append(box);
    field(box, '类型 kind', row.kind, value => edit('workbook.toml', row, 'kind', value), { options: [['skill', 'skill'], ['agent', 'agent'], ['mcp', 'mcp']] });
    for (const key of ['name', 'version', 'digest', 'source']) field(box, key, row[key], value => edit('workbook.toml', row, key, value), { mono: true, optional: key !== 'name' });
    box.append(button('删除资源声明', () => { rows.splice(index, 1); model.commit('workbook.toml'); changed(); renderProperties(); }, 'danger'));
  });
}
function renderProperties() {
  const parent = $('property-content'); parent.replaceChildren();
  try {
    $('property-title').textContent = selection.type === 'manifest' ? '方法属性' : selection.type === 'node' ? '节点属性' : selection.type === 'edge' ? '显式边属性' : 'Flow 属性';
    if (selection.type === 'manifest') manifestForm(parent);
    else if (selection.type === 'node') nodeForm(parent, selection.node);
    else if (selection.type === 'edge') {
      const edge = selection.edge;
      const options = flow().nodes.map(node => [String(node.id), `${node.id} · ${node.title}`]);
      field(parent, '起点 from', edge.from, value => edit(flowPath, edge, 'from', value), { options });
      field(parent, '终点 to', edge.to, value => edit(flowPath, edge, 'to', value), { options });
      field(parent, '边类型 kind', edge.kind, value => edit(flowPath, edge, 'kind', value), { options: ['main', 'back', 'branch', 're_review'].map(v => [v, v]) });
      parent.append(element('p', '显式边只表示可走的下一步，不自动创建输入绑定。', 'field-help'));
      parent.append(button('删除显式边', () => { const edges = flow().edges; edges.splice(edges.indexOf(edge), 1); model.commit(flowPath); selection = { type: 'flow' }; changed(); render(); }, 'danger'));
    } else {
      const current = flow();
      field(parent, 'Flow ID', current.id, value => edit(flowPath, current, 'id', value), { mono: true });
      field(parent, '入口节点 entry', current.entry, value => edit(flowPath, current, 'entry', value), { fieldKey: 'entry', options: current.nodes.map(n => [String(n.id), `${n.id} · ${n.title}`]) });
      parent.append(element('p', '选中画布节点编辑说明、输入与输出。选中连线编辑显式边。', 'field-help'));
    }
    const raw = advanced(parent, '查看当前 TOML 原文');
    const path = selection.type === 'manifest' ? 'workbook.toml' : flowPath;
    const pre = element('pre', model.text(path)); pre.dataset.previewPath = path; raw.append(pre);
  } catch (error) { parent.append(element('p', error.message, 'warn')); if (model.files.has(flowPath)) parent.append(element('pre', new TextDecoder().decode(model.files.get(flowPath)))); }
}
function render() { renderNavigation(); renderCanvas(); renderProperties(); }
function replaceModel(next) { model = next; flowPath = model.manifest.flows[0]; selection = { type: 'manifest' }; dirty = false; loadView(); render(); clearFailure(); details(''); $('error-details').open = false; status('草稿已打开。选择节点编辑；保存前会重新检查当前文件。'); }
function confirmReplace() { return !dirty || confirm('当前草稿尚未保存新副本。继续会丢弃本页草稿，是否继续？'); }
async function sample(name) {
  if (!confirmReplace()) return;
  const response = await fetch(`/api/sample/${name}`); if (!response.ok) throw new Error('样例读取失败。');
  const entries = await response.json(); const files = new Map(entries.map(([p, encoded]) => [p, Uint8Array.from(atob(encoded), c => c.charCodeAt(0))])); replaceModel(new WorkbookModel(files));
}
async function operation(kind) {
  if (busy || !token) return;
  const entries = toWire(model.snapshot()); busy = true; for (const id of ['new', 'open', 'check', 'save']) $(id).disabled = true; document.querySelector('.workspace').inert = true;
  status(kind === 'export' ? '正在检查当前字节并生成完整新副本…' : '正在用本次临时 Home 检查结构…');
  try {
    const response = await fetch(`/api/${kind}`, { method: 'POST', headers: { 'Content-Type': 'application/json', 'X-Editor-Token': token }, body: JSON.stringify(entries) });
    if (!response.ok) { showFailure(await response.json()); return; }
    clearFailure();
    if (kind === 'check') { const result = await response.json(); details(JSON.stringify(result, null, 2)); $('error-details').open = false; status(`结构检查通过：${result.result.data.id}@${result.result.data.version}。保存时会重新校验。`); }
    else {
      const blob = await response.blob(); const url = URL.createObjectURL(blob), link = element('a'); link.href = url; link.download = `${model.manifest.id}-${model.manifest.version}.zip`; document.body.append(link); link.click(); link.remove(); setTimeout(() => URL.revokeObjectURL(url), 60000); dirty = false; details(''); $('error-details').open = false; status('完整新副本已下载。解压后选择目录重新打开；原目录未写入。');
    }
  } finally { busy = false; for (const id of ['new', 'open', 'check', 'save']) $(id).disabled = false; document.querySelector('.workspace').inert = false; }
}
$('new').addEventListener('click', safe(() => { if (confirmReplace()) { replaceModel(newWorkbook()); dirty = true; } }));
$('open').addEventListener('click', () => { if (confirmReplace()) $('directory').click(); });
$('directory').addEventListener('change', safe(async () => { try { if ($('directory').files.length) replaceModel(new WorkbookModel(await readDirectory($('directory').files))); } finally { $('directory').value = ''; } }));
$('sample-code').addEventListener('click', safe(() => sample('code-change'))); $('sample-spec').addEventListener('click', safe(() => sample('spec-dev')));
$('check').addEventListener('click', safe(() => operation('check'))); $('save').addEventListener('click', safe(() => operation('export')));
$('add-node').addEventListener('click', safe(() => { const node = model.addNode(flowPath); changed(); selectNode(node); }));
$('add-edge').addEventListener('click', safe(() => { const nodes = flow().nodes; if (nodes.length < 2) throw new Error('先增加至少两个节点，再添加显式边。'); const from = selection.node ?? nodes[0]; const to = nodes.find(n => n !== from); const edge = model.addEdge(flowPath, from.id, to.id); changed(); selectEdge(edge); }));
$('add-flow').addEventListener('click', safe(() => {
  const id = prompt('新 Flow ID（小写字母、数字和单个连字符）', 'new-flow'); if (id === null) return;
  if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(id)) throw new Error('Flow ID 格式不正确。');
  const path = `flows/${id}.toml`; if (model.files.has(path) || [...model.documents.values()].some(d => d.schema === 'flow/v1' && d.id === id)) throw new Error('Flow 路径或 ID 已存在。');
  const source = newWorkbook(); const definition = source.flow('flows/default.toml'); definition.id = id; source.commit('flows/default.toml'); model.writeText(path, source.text('flows/default.toml')); model.load(path); model.manifest.flows.push(path); model.commit('workbook.toml'); flowPath = path; selection = { type: 'flow' }; changed(); loadView(); render();
}));
function zoom(factor, px = $('viewport').clientWidth / 2, py = $('viewport').clientHeight / 2) { view.zoom(factor, px, py); renderCanvas(); saveView(); }
$('zoom-in').addEventListener('click', () => zoom(1.15)); $('zoom-out').addEventListener('click', () => zoom(1 / 1.15));
$('reset-view').addEventListener('click', safe(() => { view.fit(flow().nodes.map(n => n.id), $('viewport').clientWidth, $('viewport').clientHeight); renderCanvas(); saveView(); }));
$('viewport').addEventListener('wheel', event => { event.preventDefault(); const rect = $('viewport').getBoundingClientRect(); zoom(event.deltaY < 0 ? 1.1 : 1 / 1.1, event.clientX - rect.left, event.clientY - rect.top); }, { passive: false });
$('viewport').addEventListener('pointerdown', event => { if (event.button !== 0 || event.target.closest('button')) return; $('viewport').setPointerCapture(event.pointerId); drag = { type: 'pan', startX: event.clientX, startY: event.clientY, x: view.x, y: view.y }; });
$('viewport').addEventListener('pointermove', event => { if (!drag || busy) return; view.move(drag, event.clientX, event.clientY); renderCanvas(); });
for (const event of ['pointerup', 'pointercancel']) $('viewport').addEventListener(event, () => { if (drag) { drag = null; saveView(); } });
$('toggle-nav').addEventListener('click', () => { document.body.classList.toggle('hide-nav'); document.body.classList.toggle('show-nav'); }); $('toggle-props').addEventListener('click', () => { document.body.classList.toggle('hide-props'); document.body.classList.toggle('show-props'); });
window.addEventListener('beforeunload', event => { if (dirty) { event.preventDefault(); event.returnValue = ''; } });
loadView(); render(); $('check').disabled = true; $('save').disabled = true;
try { const response = await fetch('/api/session'); if (!response.ok) throw new Error('本地服务会话不可用，请确认以 127.0.0.1 地址打开页面。'); token = (await response.json()).token; $('check').disabled = false; $('save').disabled = false; status('新草稿已就绪。增加节点与显式边，或打开目录／固定样例。'); } catch (error) { showError(error); }
