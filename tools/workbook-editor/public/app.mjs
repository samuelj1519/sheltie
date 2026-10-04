import { WorkbookModel, newWorkbook } from './model.mjs';
import { readDirectory, toWire } from './files.mjs';
import { CanvasView } from './layout.mjs';
import { CARD_WIDTH, CARD_HEIGHT, routeEdges, edgeKindLabel, pickRouteAtPoint, renderedCanvasTransform, visibleEdges, mainSkeleton } from './geometry.mjs';
import { sourceChoices, describeSource, composeSource, declarationBoolean, declarationBytes, predecessorMaterials, suggestedInputName, planInputRows, addMaterialInputs, fixedReference, editFixedReference, bindingSummary, projectBindingGroups, pruneUsedStaged, removeStagedChoice } from './sources.mjs';
import { syncTextPreviews, describeFailure } from './presentation.mjs';
const $ = id => document.getElementById(id);
let model = newWorkbook(), flowPath = model.manifest.flows[0], selection = { type: 'manifest' }, token, dirty = false, busy = false;
let view = new CanvasView(), drag, graphMode = 'main', hoveredEdge, refreshBindingPane, refreshMaterialOptions, bindingState;
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
  refreshBindingPane?.(); refreshMaterialOptions?.();
  for (const place of $('property-content').querySelectorAll('[data-input-source-index]')) {
    const node = selection.node, row = node?.inputs?.[Number(place.dataset.inputSourceIndex)]; if (!row) continue;
    if (place.dataset.from !== String(row.from)) renderInputSource(place, row, node);
    else if (place.querySelector('.source-summary')) place.querySelector('.source-summary').textContent = sourceSummary(row, node);
  }
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
  let acceptedValue = input.value;
  input.addEventListener('change', safe(() => {
    let value = input.value;
    if (optional && value === '') value = undefined;
    else if (type === 'number') { value = Number(value); if (!Number.isSafeInteger(value)) throw new Error(`${label} 必须为整数。`); value = BigInt(value); }
    try { onChange(value); acceptedValue = input.value; }
    catch (error) { input.value = acceptedValue; throw error; }
  }));
  wrap.append(input);
  if (help) wrap.append(element('small', help, 'field-help'));
  parent.append(wrap); return input;
}
const boolOptions = [['', '未声明（使用默认值）'], ['true', '是'], ['false', '否']];
let suppressClick = false;
function booleanField(parent, label, value, onChange, help) { field(parent, label, value, v => onChange(declarationBoolean(v)), { options: boolOptions, help }); }
function advanced(parent, label = '高级设置') { const d = element('details'); d.append(element('summary', label)); parent.append(d); return d; }
function section(parent, title, add) { const head = element('div', undefined, 'section-label'); head.append(element('h3', title)); if (add) head.append(button('＋ 添加', add)); parent.append(head); }
function flow() { return model.flow(flowPath); }
function layoutKey() { return `sheltie-editor-layout:${model.manifest.id}:${flowPath}`; }
function loadView() { view = CanvasView.load(localStorage, layoutKey()); try { view.prepare(flow()); view.initialCamera(flow(), shownRoutes()); } catch { /* 原错误由属性面板说明。 */ } }
function saveView() { try { view.save(localStorage, layoutKey()); } catch { status('布局无法保存在浏览器；Workbook 草稿仍可保存。'); } }
function position(node, index) { return view.position(node.id, index); }
function shownEdges() { return visibleEdges(flow(), graphMode, selection.node?.id, selection.edge); }
function shownRoutes() { return routeEdges(flow(), view.positions, shownEdges()); }
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
  $('edge-list').replaceChildren();
  try { for (const edge of flow().edges ?? []) $('edge-list').append(button(edgeDescription(edge), () => selectEdge(edge, true), `nav-item edge-list-item ${selection.edge === edge ? 'active' : ''}`)); } catch { /* 原字节保留。 */ }
  $('flow-label').textContent = flowPath;
}
function selectNode(node) { selection = { type: 'node', node }; renderNavigation(); renderCanvas(); renderProperties(); }
function nodeTitle(id) { const node = flow().nodes.find(n => n.id === id); return node?.title || id; }
function edgeDescription(edge) { return `${nodeTitle(edge.from)} → ${nodeTitle(edge.to)} · ${edgeKindLabel(edge.kind)}`; }
function selectEdge(edge, reveal = false) {
  const wasHidden = !shownEdges().includes(edge); selection = { type: 'edge', edge };
  if (reveal && wasHidden) {
    const routes = routeEdges(flow(), view.positions, [edge]), subset = { ...flow(), nodes: flow().nodes.filter(n => n.id === edge.from || n.id === edge.to) };
    $('viewport').scrollLeft = 0; $('viewport').scrollTop = 0;
    view.fitGraph(subset, $('viewport').clientWidth, $('viewport').clientHeight, routes, edge); saveView();
  }
  renderNavigation(); renderCanvas(); renderProperties();
}
function hoverVisibleEdge(event, routes = shownRoutes()) {
  const group = event.target.closest('[data-edge-index]');
  if (!group) hoveredEdge = undefined;
  else if (event.target.closest('.edge-label')) hoveredEdge = flow().edges[Number(group.dataset.edgeIndex)];
  else {
    const transform = renderedCanvasTransform($('world').getBoundingClientRect(), $('world').offsetWidth);
    hoveredEdge = pickRouteAtPoint(routes, { x: event.clientX, y: event.clientY }, transform).route?.edge;
  }
  updateEdgeLabels();
}
function updateEdgeLabels() {
  const show = hoveredEdge ?? selection.edge;
  for (const label of $('edges').querySelectorAll('.edge-label[data-edge-index]')) {
    const visible = flow().edges[Number(label.dataset.edgeIndex)] === show;
    label.classList.toggle('label-visible', visible); label.setAttribute('aria-hidden', String(!visible)); label.setAttribute('tabindex', visible ? '0' : '-1');
  }
}
function renderCanvas() {
  $('nodes').replaceChildren(); $('edges').replaceChildren();
  $('world').style.transform = `translate(${view.x}px,${view.y}px) scale(${view.scale})`;
  $('zoom-label').textContent = `${Math.round(view.scale * 100)}%`;
  let current; try { current = flow(); } catch (error) { status(`${flowPath}：${error.message}`, true); return; }
  const defs = svg('defs'), marker = svg('marker', { id: 'arrow', viewBox: '0 0 10 10', refX: 9, refY: 5, markerWidth: 7, markerHeight: 7, orient: 'auto-start-reverse' }); marker.append(svg('path', { d: 'M 0 0 L 10 5 L 0 10 z', fill: '#7994b6' })); defs.append(marker); $('edges').append(defs);
  view.prepare(current);
  const routes = shownRoutes(), labels = svg('g', { id: 'edge-labels' });
  $('edge-count').textContent = `显示 ${routes.length} / 共 ${(current.edges ?? []).length} 条连线`;
  $('graph-mode').textContent = graphMode === 'main' ? '主流程 ▾' : '全部连线 ▾';
  for (const route of routes) {
    const { edge, index, d, label } = route;
    const active = selection.edge === edge, related = selection.node && (edge.from === selection.node.id || edge.to === selection.node.id);
    const group = svg('g', { 'data-edge-index': index, 'data-from': edge.from, 'data-to': edge.to });
    const hit = svg('path', { d, class: 'edge-hit', tabindex: 0, role: 'button', 'aria-label': `${edge.from} 到 ${edge.to}，${edge.kind}` });
    group.addEventListener('pointerdown', event => event.stopPropagation());
    group.addEventListener('click', event => {
      event.stopPropagation(); if (suppressClick) return;
      if (event.target.closest('.edge-label')) { selectEdge(edge); return; }
      const transform = renderedCanvasTransform($('world').getBoundingClientRect(), $('world').offsetWidth);
      const picked = pickRouteAtPoint(routes, { x: event.clientX, y: event.clientY }, transform);
      if (picked.ambiguous.length) status(`此处多条连线交汇（${picked.ambiguous.map(r => r.index + 1).join('、')}），请点击编号标签或左侧“全部连线”选择。`);
      else if (picked.route) selectEdge(picked.route.edge);
    });
    group.addEventListener('keydown', event => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); selectEdge(edge); } });
    group.append(hit, svg('path', { d, class: `edge kind-${edge.kind} ${active ? 'active' : related ? 'related' : ''}`, 'marker-end': 'url(#arrow)' }));
    const caption = `${index + 1} ${edgeKindLabel(edge.kind)}`, width = route.labelWidth;
    const labelGroup = svg('g', { class: `edge-label ${active ? 'active label-visible' : ''}`, 'data-edge-index': index, 'data-from': edge.from, 'data-to': edge.to, role: 'button', tabindex: 0, 'aria-label': `连线标签：${edgeDescription(edge)}` });
    labelGroup.append(svg('rect', { x: label.x - width / 2, y: label.y - 13, width, height: 26, rx: 5 }), svg('text', { x: label.x, y: label.y + 4, 'text-anchor': 'middle' }, caption));
    labelGroup.addEventListener('pointerdown', event => event.stopPropagation());
    labelGroup.addEventListener('click', event => { event.stopPropagation(); if (!suppressClick) selectEdge(edge); });
    labelGroup.addEventListener('keydown', event => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); selectEdge(edge); } });
    labelGroup.addEventListener('pointerenter', () => { hoveredEdge = edge; updateEdgeLabels(); });
    labels.append(labelGroup); $('edges').append(group);
    group.addEventListener('pointerenter', event => hoverVisibleEdge(event, routes));
  }
  $('edges').append(labels); hoveredEdge = undefined; updateEdgeLabels();
  current.nodes.forEach((node, index) => {
    const pos = position(node, index);
    const endpoint = selection.edge && (selection.edge.from === node.id || selection.edge.to === node.id);
    const b = button('', () => { if (!suppressClick) selectNode(node); }, `node ${selection.node === node ? 'selected' : ''} ${endpoint ? 'endpoint' : ''} ${node.gate ? 'gated' : ''}`);
    b.title = `${node.id} · ${node.title}`; b.dataset.nodeId = node.id; b.style.width = `${CARD_WIDTH}px`; b.style.height = `${CARD_HEIGHT}px`;
    b.style.left = `${pos.x}px`; b.style.top = `${pos.y}px`;
    const head = element('span', undefined, 'node-head'); head.append(element('span', node.id)); if (node.id === current.entry) head.append(element('span', '入口', 'entry-mark'));
    b.append(head, element('span', node.title, 'node-title'), element('span', `${node.executor === 'human' ? '人工' : 'agent'}${node.gate ? ' · 门槛' : ''} · ${(node.inputs ?? []).length} 输入 / ${(node.outputs ?? []).length} 输出`, 'node-meta'));
    b.addEventListener('pointerdown', event => {
      if (event.button !== 0) return;
      event.stopPropagation(); selection = { type: 'node', node }; renderNavigation(); renderProperties();
      for (const card of $('nodes').children) card.classList.toggle('selected', card.dataset.nodeId === node.id);
      suppressClick = false; $('viewport').setPointerCapture(event.pointerId);
      drag = { type: 'node', id: node.id, startX: event.clientX, startY: event.clientY, x: pos.x, y: pos.y };
    });
    $('nodes').append(b);
  });
}
function sourceForm(parent, row, node) {
  let draft = describeSource(row.from, flow(), model.files, node.id), rawPreview;
  const body = element('div'); parent.append(body);
  const commit = () => {
    const value = composeSource(draft, flow(), model.files, node.id);
    if (value !== undefined) { edit(flowPath, row, 'from', value); if (rawPreview) rawPreview.textContent = row.from; }
    else status('来源尚未选完整，原有来源保持不变。');
  };
  const renderSource = () => {
    body.replaceChildren();
    field(body, '资料从哪里来', draft.type, type => {
      draft = type === draft.type ? draft : { type };
      if (type === 'custom') draft.raw = row.from;
      commit(); renderSource();
    }, { options: [['start', '任务开始时提供'], ['step', '另一步的输出'], ['resource', '方法中的参考文件'], ['stats', '运行统计'], ['custom', '自定义来源（保留原文）']] });
    if (draft.type === 'start') field(body, '开始时提供的资料名称', draft.key, value => { draft.key = value; commit(); }, { help: '例如 task；留空时不会改写原有来源。' });
    else if (draft.type === 'step') {
      const steps = sourceChoices(flow(), model.files, node.id).steps;
      field(body, '来自哪一步', draft.node, value => { draft.node = value; draft.output = undefined; commit(); renderSource(); }, { options: [['', '请选择步骤'], ...steps.map(n => [n.id, `${n.title || n.id}（${n.id}）`])] });
      const outputs = steps.find(n => n.id === draft.node)?.outputs ?? [];
      field(body, '读取这一步的哪个输出', draft.output, value => { draft.output = value; commit(); }, { options: [['', outputs.length ? '请选择输出' : '此步骤没有声明输出'], ...outputs.filter(o => typeof o.name === 'string').map(o => [o.name, `${o.name} · ${o.path ?? '未声明文件名'}`])] });
    } else if (draft.type === 'resource') {
      const resources = sourceChoices(flow(), model.files, node.id).resources;
      field(body, '选择参考文件', draft.path, value => { draft.path = value; commit(); }, { options: [['', resources.length ? '请选择文件' : '方法中还没有参考文件'], ...resources.map(p => [p, p])] });
    } else if (draft.type === 'stats') body.append(element('p', '由引擎提供本次运行的统计资料。', 'field-help'));
    else field(body, '自定义来源原文', draft.raw, value => { draft.raw = value; commit(); }, { mono: true, help: '无法识别或缺失的引用会原样保留，由检查结构给出结果。' });
    const syntax = advanced(body, '查看来源原文'); rawPreview = element('code', row.from === undefined ? '尚未声明' : row.from); syntax.append(rawPreview);
  };
  renderSource();
}
function adoptModel(next, nodeId) {
  if (next === model) return;
  model = next; selection = { type: 'node', node: flow().nodes.find(n => n.id === nodeId) }; changed(); render();
}
function sourceSummary(row, node) { return bindingSummary(row, 'inputs', flow(), model.files, flowPath, node.id); }
function renderInputSource(place, row, node) {
  place.replaceChildren(); place.dataset.from = String(row.from);
  const reference = fixedReference(model.files, row.from, flowPath, node.id);
  if (reference) field(place, '选择或输入资料', reference.location, value => adoptModel(editFixedReference(model, flowPath, node.id, node.inputs.indexOf(row), value), node.id), { help: '固定引用位置；确认后建立新资源副本，编辑器不会访问此位置。' });
  else place.append(element('p', sourceSummary(row, node), 'source-summary'));
}
function materialChooser(parent, node, onStagedChange = () => {}) {
  const box = element('div', undefined, 'material-chooser'); parent.append(box);
  const wrap = element('label', undefined, 'field'); wrap.append(element('span', '选择或输入资料'));
  const input = element('input'); input.type = 'text'; input.placeholder = '选择前置资料，或输入引用位置后按 Enter'; input.setAttribute('role', 'combobox'); input.setAttribute('aria-autocomplete', 'list'); input.setAttribute('aria-expanded', 'false'); wrap.append(input); box.append(wrap);
  const dropdown = element('div', undefined, 'material-options'), chips = element('div', undefined, 'material-chips'), preview = element('p', undefined, 'field-help'); dropdown.hidden = true;
  const staged = [];
  const materials = () => predecessorMaterials(flow(), model.files, node.id);
  const plannedRows = () => staged.map(item => ({ ...item, from: item.location === undefined ? item.from : (node.inputs ?? []).find(row => row.name === item.name && fixedReference(model.files, row.from, flowPath, node.id)?.location === item.location)?.from ?? `固定引用:${item.location}` }));
  const redraw = () => {
    chips.replaceChildren();
    for (const item of staged) {
      const chip = element('div', undefined, 'material-chip'); chip.append(element('span', item.location ?? `${item.nodeTitle} · ${item.name}`));
      field(chip, '资料名称', item.name, value => { item.name = value; refreshPreview(); }, { mono: true });
      chip.append(button('移除选择', () => { removeStagedChoice(staged, item); redraw(); })); chips.append(chip);
    }
    renderOptions(); refreshPreview();
  };
  const addExternal = () => {
    if (!input.value.trim()) return;
    staged.push({ location: input.value, name: suggestedInputName('reference', `固定引用:${input.value}`, [...(node.inputs ?? []), ...plannedRows()]) }); input.value = ''; redraw();
  };
  const optionElements = new Map(), optionGroups = new Map();
  const placeOptions = (container, children) => {
    children.forEach((child, index) => { if (container.children[index] !== child) container.insertBefore(child, container.children[index] ?? null); });
    for (const child of [...container.children]) if (!children.includes(child)) child.remove();
  };
  const externalChoice = button('将输入内容选为固定引用', addExternal);
  const renderOptions = () => {
    const query = input.value.toLowerCase(), options = materials(), blocks = [];
    for (const [kind, title] of [['output', '前置步骤的输出'], ['input', '沿用前置步骤原资料来源']]) {
      let group = optionGroups.get(kind);
      if (!group) { const container = element('div'), rows = element('div'), empty = element('p', '没有可选资料；也可以直接输入固定引用。', 'field-help'); container.append(element('strong', title, 'material-group-title'), rows); group = { container, rows, empty }; optionGroups.set(kind, group); }
      const matching = options.filter(option => option.kind === kind && `${option.nodeTitle} ${option.name} ${option.from} ${option.usedNames.join(' ')}`.toLowerCase().includes(query)), choices = [];
      for (const option of matching) {
        let entry = optionElements.get(option.key);
        if (!entry) {
          const label = element('label', undefined, 'material-option'), checkbox = element('input'), caption = element('span'), aliases = element('small'), reason = element('small'); checkbox.type = 'checkbox'; label.append(checkbox, caption, aliases, reason);
          entry = { label, checkbox, caption, aliases, reason }; optionElements.set(option.key, entry);
          checkbox.addEventListener('change', () => {
            const currentOption = materials().find(item => item.key === option.key);
            if (!currentOption || currentOption.usedNames.length || currentOption.reason) { renderOptions(); return; }
            if (checkbox.checked) { if (!staged.some(item => item.key === option.key)) staged.push({ ...currentOption, name: suggestedInputName(currentOption.name, currentOption.from, [...(node.inputs ?? []), ...plannedRows()]) }); }
            else { const index = staged.findIndex(item => item.key === option.key); if (index >= 0) staged.splice(index, 1); }
            redraw();
          });
        }
        const used = option.usedNames.length > 0;
        entry.checkbox.disabled = used || Boolean(option.reason); entry.checkbox.checked = used || staged.some(item => item.key === option.key);
        entry.label.classList.toggle('already-used', used); entry.caption.textContent = `${option.nodeTitle} · ${option.name}${option.required === false ? '（可选）' : ''}`;
        entry.aliases.hidden = !used; entry.aliases.textContent = used ? `已使用 · 本步骤：${option.usedNames.join('、')}` : '';
        entry.reason.hidden = !option.reason; entry.reason.textContent = option.reason; choices.push(entry.label);
      }
      placeOptions(group.rows, choices.length ? choices : [group.empty]); blocks.push(group.container);
    }
    if (input.value.trim()) blocks.push(externalChoice);
    placeOptions(dropdown, blocks);
  };
  const refreshPreview = () => {
    const plan = planInputRows(node.inputs ?? [], plannedRows());
    preview.textContent = plan.conflicts.length ? plan.conflicts.map(c => `${c.name}：${c.reason}`).join(' ') : staged.length ? `准备添加 ${plan.additions.length} 项；已存在 ${plan.skipped.length} 项不会重复添加。` : '下拉可多选；直接输入的位置固定写入方法，编辑器不会访问该位置。';
    submit.disabled = !staged.length || Boolean(plan.conflicts.length); onStagedChange(staged.length);
  };
  const submit = button('添加所选资料', safe(() => { const result = addMaterialInputs(model, flowPath, node.id, staged); if (result.model === model) status('所选资料已存在，未重复添加。'); else { adoptModel(result.model, node.id); status(`已添加 ${result.added} 份资料。`); } }));
  const open = () => { dropdown.hidden = false; input.setAttribute('aria-expanded', 'true'); renderOptions(); };
  input.addEventListener('focus', open); input.addEventListener('input', open);
  input.addEventListener('keydown', event => { if (event.key === 'Enter') { event.preventDefault(); addExternal(); } else if (event.key === 'Escape') { event.preventDefault(); dropdown.hidden = true; input.setAttribute('aria-expanded', 'false'); } });
  refreshMaterialOptions = () => { if (pruneUsedStaged(staged, materials())) redraw(); else { renderOptions(); refreshPreview(); } };
  box.append(element('p', '已勾选表示本步骤正在使用；移除请在已选列表逐项操作。', 'field-help'));
  box.append(button('展开资料选项', () => { dropdown.hidden = !dropdown.hidden; input.setAttribute('aria-expanded', String(!dropdown.hidden)); if (!dropdown.hidden) renderOptions(); }), dropdown, chips, preview, submit); refreshPreview();
}
function bindingEditor(parent, node, key, row) {
  const output = key === 'outputs';
  field(parent, output ? '交付物名称' : '资料名称', row.name, value => edit(flowPath, row, 'name', value));
  if (output) field(parent, '文件名或相对路径', row.path, value => edit(flowPath, row, 'path', value), { mono: true, help: '例如 report.md 或 reports/summary.md。' });
  else { const place = element('div'); place.dataset.inputSourceIndex = node.inputs.indexOf(row); parent.append(place); renderInputSource(place, row, node); }
  const extra = advanced(parent, '高级：精确来源、提供要求与最终成果');
  if (!output) sourceForm(extra, row, node);
  booleanField(extra, '必须提供', row.required, value => edit(flowPath, row, 'required', value), '未声明时为必需；声明为否时允许缺少。');
  booleanField(extra, '作为最终成果', row.result, value => edit(flowPath, row, 'result', value), '仅成功终点的必需资料或交付物可以作为最终成果。');
  if (output) field(extra, '大小上限（字节）', row.max_bytes, value => edit(flowPath, row, 'max_bytes', declarationBytes(value)), { help: '留空表示未声明，默认 1048576 字节；原有大整数准确保留。' });
  parent.append(button(output ? '移除此输出' : '移除此输入', () => {
    node[key].splice(node[key].indexOf(row), 1); model.commit(flowPath); bindingState.expanded = undefined; changed(); renderProperties();
  }, 'danger'));
}
function bindingPane(parent, node) {
  if (!bindingState || bindingState.flow !== flowPath || bindingState.node !== node.id) bindingState = { flow: flowPath, node: node.id, tab: 'inputs', query: '', expanded: undefined };
  const pane = element('section', undefined, 'binding-pane'), tabs = element('div', undefined, 'binding-tabs'), tools = element('div', undefined, 'binding-tools'), editor = element('div', undefined, 'binding-editor'), list = element('div', undefined, 'compact-binding-list'); parent.append(pane); tabs.setAttribute('role', 'tablist'); tabs.setAttribute('aria-label', '节点资料');
  const current = () => flow().nodes.find(n => n.id === selection.node?.id) ?? node;
  let editorTitle;
  const redrawEditor = () => {
    editor.replaceChildren(); const actual = current(), row = actual[bindingState.tab]?.[bindingState.expanded]; editor.hidden = !row;
    if (!row) return;
    const header = element('div', undefined, 'binding-editor-head'); editorTitle = element('strong', `编辑：${row.name ?? '未声明名称'}`); header.append(editorTitle, button('收起编辑', () => { bindingState.expanded = undefined; redrawEditor(); redrawList(); })); editor.append(header);
    bindingEditor(editor, actual, bindingState.tab, row);
  };
  const groupElements = new Map(), rowElements = new Map();
  const placeInOrder = (container, children) => {
    children.forEach((child, index) => { if (container.children[index] !== child) container.insertBefore(child, container.children[index] ?? null); });
    for (const child of [...container.children]) if (!children.includes(child)) child.remove();
  };
  const redrawList = () => {
    const actual = current(), groups = projectBindingGroups(actual, bindingState.tab, flow(), model.files, flowPath, bindingState.query), blocks = [];
    for (const group of groups) {
      let block = groupElements.get(group.label);
      if (!block) {
        block = element('div', undefined, 'compact-binding-group'); const head = element('div', undefined, 'compact-group-title'); head.append(element('span', group.label), element('small')); block.append(head, element('div', undefined, 'compact-group-rows')); groupElements.set(group.label, block);
      }
      block.querySelector('.compact-group-title small').textContent = `${group.rows.length} 项`;
      const buttons = [];
      for (const item of group.rows) {
        let row = rowElements.get(item.row);
        if (!row) {
          row = button('', () => { const index = Number(row.dataset.bindingIndex); bindingState.expanded = bindingState.expanded === index ? undefined : index; redrawEditor(); redrawList(); }, 'compact-binding-row');
          const top = element('span', undefined, 'compact-row-name'); top.append(element('strong'), element('small', '可选', 'binding-badge')); row.append(top, element('span', undefined, 'compact-row-summary')); rowElements.set(item.row, row);
        }
        const active = item.index === bindingState.expanded; row.dataset.bindingIndex = item.index; row.classList.toggle('expanded', active); row.setAttribute('aria-expanded', String(active));
        row.querySelector('strong').textContent = item.row.name === undefined ? '未声明名称' : item.row.name;
        row.querySelector('.binding-badge').hidden = item.row.required !== false; row.querySelector('.compact-row-summary').textContent = item.summary; buttons.push(row);
      }
      placeInOrder(block.querySelector('.compact-group-rows'), buttons); blocks.push(block);
    }
    if (!blocks.length) blocks.push(element('p', bindingState.query ? '没有匹配的资料。' : '尚未添加资料。', 'field-help'));
    placeInOrder(list, blocks);
    if (editorTitle && bindingState.expanded !== undefined) editorTitle.textContent = `编辑：${actual[bindingState.tab]?.[bindingState.expanded]?.name ?? '未声明名称'}`;
    inputTab.textContent = `输入 ${actual.inputs?.length ?? 0}`; outputTab.textContent = `输出 ${actual.outputs?.length ?? 0}`;
  };
  const inputTools = element('div'), outputTools = element('div'); tools.append(inputTools, outputTools);
  const add = advanced(inputTools, '＋ 添加资料'), addTitle = add.querySelector('summary');
  materialChooser(add, current(), count => { addTitle.textContent = count ? `＋ 添加资料 · 待添加 ${count} 项` : '＋ 添加资料'; });
  outputTools.append(button('＋ 添加输出', () => {
    const actual = current(); actual.outputs ??= []; let number = 1; while (actual.outputs.some(row => row.name === `output-${number}`)) number++;
    actual.outputs.push({ name: `output-${number}`, path: `output-${number}.md` }); model.commit(flowPath); bindingState.expanded = actual.outputs.length - 1; changed(); renderProperties();
  }));
  const redrawTools = () => { inputTools.hidden = bindingState.tab !== 'inputs'; outputTools.hidden = bindingState.tab !== 'outputs'; };
  const selectTab = key => {
    bindingState.tab = key; bindingState.expanded = undefined; bindingState.query = ''; search.value = '';
    inputTab.setAttribute('aria-selected', String(key === 'inputs')); outputTab.setAttribute('aria-selected', String(key === 'outputs'));
    redrawTools(); redrawEditor(); redrawList();
  };
  const inputTab = button(`输入 ${node.inputs?.length ?? 0}`, () => selectTab('inputs')), outputTab = button(`输出 ${node.outputs?.length ?? 0}`, () => selectTab('outputs'));
  for (const tab of [inputTab, outputTab]) tab.setAttribute('role', 'tab'); tabs.append(inputTab, outputTab);
  inputTab.setAttribute('aria-selected', String(bindingState.tab === 'inputs')); outputTab.setAttribute('aria-selected', String(bindingState.tab === 'outputs'));
  const searchWrap = element('label', undefined, 'field binding-search'); searchWrap.append(element('span', '搜索资料'));
  const search = element('input'); search.type = 'search'; search.placeholder = '名称、来源或文件名'; search.value = bindingState.query; search.addEventListener('input', () => { bindingState.query = search.value; redrawList(); }); searchWrap.append(search);
  pane.append(tabs, searchWrap, editor, list, tools); redrawTools(); redrawEditor(); redrawList(); refreshBindingPane = redrawList;
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
  field(parent, '由谁完成这一步', node.executor, value => edit(flowPath, node, 'executor', value), { options: [['agent', '由 agent 完成'], ['human', '由人完成']] });
  parent.append(button('设为入口', () => edit(flowPath, flow(), 'entry', node.id)));
  bindingPane(parent, node);
  const instructions = advanced(parent, '说明与操作指引'); instructionForm(instructions, node);
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
  refreshBindingPane = undefined; refreshMaterialOptions = undefined;
  const parent = $('property-content'); parent.replaceChildren();
  try {
    $('property-title').textContent = selection.type === 'manifest' ? '方法属性' : selection.type === 'node' ? '节点属性' : selection.type === 'edge' ? '显式边属性' : 'Flow 属性';
    if (selection.type === 'manifest') manifestForm(parent);
    else if (selection.type === 'node') nodeForm(parent, selection.node);
    else if (selection.type === 'edge') {
      const edge = selection.edge;
      const summary = element('p', `从〔${nodeTitle(edge.from)}〕到〔${nodeTitle(edge.to)}〕`, 'edge-summary'); summary.dataset.edgeFrom = edge.from; summary.dataset.edgeTo = edge.to; parent.append(summary);
      const options = flow().nodes.map(node => [String(node.id), `${node.id} · ${node.title}`]);
      field(parent, '从哪一步出发', edge.from, value => { edit(flowPath, edge, 'from', value); renderProperties(); }, { options });
      field(parent, '到哪一步', edge.to, value => { edit(flowPath, edge, 'to', value); renderProperties(); }, { options });
      field(parent, '如何前往', edge.kind, value => edit(flowPath, edge, 'kind', value), { options: ['main', 'back', 'branch', 're_review'].map(v => [v, edgeKindLabel(v)]) });
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
function replaceModel(next) { model = next; flowPath = model.manifest.flows[0]; selection = { type: 'manifest' }; graphMode = 'main'; dirty = false; loadView(); render(); clearFailure(); details(''); $('error-details').open = false; status('草稿已打开。选择节点编辑；保存前会重新检查当前文件。'); }
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
$('add-edge').addEventListener('click', safe(() => {
  const current = flow(), nodes = current.nodes;
  if (nodes.length < 2) throw new Error('先增加至少两个节点，再添加显式边。');
  const from = selection.node ?? nodes[0], to = nodes.find(n => n !== from);
  const previousEdges = current.edges?.slice() ?? [];
  const edge = model.addEdge(flowPath, from.id, to.id), existing = previousEdges.includes(edge);
  if (!existing) changed();
  selectEdge(edge);
  if (existing) status(`该连线已存在，已选中 ${edge.from} → ${edge.to}。`);
}));
$('add-flow').addEventListener('click', safe(() => {
  const id = prompt('新 Flow ID（小写字母、数字和单个连字符）', 'new-flow'); if (id === null) return;
  if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(id)) throw new Error('Flow ID 格式不正确。');
  const path = `flows/${id}.toml`; if (model.files.has(path) || [...model.documents.values()].some(d => d.schema === 'flow/v1' && d.id === id)) throw new Error('Flow 路径或 ID 已存在。');
  const source = newWorkbook(); const definition = source.flow('flows/default.toml'); definition.id = id; source.commit('flows/default.toml'); model.writeText(path, source.text('flows/default.toml')); model.load(path); model.manifest.flows.push(path); model.commit('workbook.toml'); flowPath = path; selection = { type: 'flow' }; changed(); loadView(); render();
}));
function zoom(factor, px = $('viewport').clientWidth / 2, py = $('viewport').clientHeight / 2) { view.zoom(factor, px, py, $('viewport').scrollLeft, $('viewport').scrollTop); renderCanvas(); saveView(); }
$('zoom-in').addEventListener('click', () => zoom(1.15)); $('zoom-out').addEventListener('click', () => zoom(1 / 1.15));
function fitCanvas(arrange = false) {
  const viewport = $('viewport'); viewport.scrollLeft = 0; viewport.scrollTop = 0;
  if (arrange) view.arrange(flow());
  view.fitGraph(flow(), viewport.clientWidth, viewport.clientHeight, shownRoutes(), selection.edge); renderCanvas(); saveView();
}
$('graph-mode').addEventListener('click', safe(() => { graphMode = graphMode === 'main' ? 'all' : 'main'; renderCanvas(); status(graphMode === 'main' ? '主流程视图：其他连线仍可从左侧列表选择。' : '完整视图：显示全部定义连线。'); }));
$('reset-view').addEventListener('click', safe(() => fitCanvas()));
$('arrange').addEventListener('click', safe(() => { fitCanvas(true); status('已整理主流程布局。方法内容未修改。'); }));
$('edges').addEventListener('pointermove', event => { if (!drag && !busy) hoverVisibleEdge(event); });
$('edges').addEventListener('pointerleave', () => { hoveredEdge = undefined; updateEdgeLabels(); });
$('viewport').addEventListener('wheel', event => { event.preventDefault(); const rect = $('viewport').getBoundingClientRect(); zoom(event.deltaY < 0 ? 1.1 : 1 / 1.1, event.clientX - rect.left, event.clientY - rect.top); }, { passive: false });
$('viewport').addEventListener('pointerdown', event => { if (event.button !== 0 || event.target.closest('button')) return; $('viewport').setPointerCapture(event.pointerId); drag = { type: 'pan', startX: event.clientX, startY: event.clientY, x: view.x, y: view.y }; });
$('viewport').addEventListener('pointermove', event => { if (!drag || busy || Math.hypot(event.clientX - drag.startX, event.clientY - drag.startY) < 4) return; suppressClick = true; view.move(drag, event.clientX, event.clientY); renderCanvas(); });
for (const event of ['pointerup', 'pointercancel']) $('viewport').addEventListener(event, () => { if (drag) { drag = null; saveView(); renderCanvas(); if (suppressClick) setTimeout(() => { suppressClick = false; }, 0); } });
const narrowWindow = window.matchMedia('(max-width:760px)');
function setPanel(panel, visible) {
  document.body.classList.toggle(`hide-${panel}`, !visible);
  document.body.classList.toggle(`show-${panel}`, visible);
  $(`toggle-${panel}`).setAttribute('aria-expanded', String(visible));
}
function closePanels() { setPanel('nav', false); setPanel('props', false); $('viewport').focus(); }
for (const [panel, id] of [['nav', 'navigation'], ['props', 'properties']]) {
  $(`toggle-${panel}`).addEventListener('click', () => {
    const visible = getComputedStyle($(id)).display === 'none';
    if (visible && narrowWindow.matches) setPanel(panel === 'nav' ? 'props' : 'nav', false);
    setPanel(panel, visible);
  });
  $(`close-${panel}`).addEventListener('click', () => { setPanel(panel, false); $('viewport').focus(); });
}
function resizePanels() { setPanel('nav', !narrowWindow.matches); setPanel('props', !narrowWindow.matches); }
narrowWindow.addEventListener('change', resizePanels); resizePanels();
document.addEventListener('keydown', event => { if (event.key === 'Escape') { event.preventDefault(); closePanels(); } });
window.addEventListener('beforeunload', event => { if (dirty) { event.preventDefault(); event.returnValue = ''; } });
loadView(); render(); $('check').disabled = true; $('save').disabled = true;
try { const response = await fetch('/api/session'); if (!response.ok) throw new Error('本地服务会话不可用，请确认以 127.0.0.1 地址打开页面。'); token = (await response.json()).token; $('check').disabled = false; $('save').disabled = false; status('新草稿已就绪。增加节点与显式边，或打开目录／固定样例。'); } catch (error) { showError(error); }
