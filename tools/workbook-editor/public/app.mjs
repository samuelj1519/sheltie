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
  status(`Opened candidate ${path}; check engine field ${fieldPath || 'not provided'}.`, true);
}
function showFailure(result) {
  const issue = describeFailure(result, model.manifest.flows);
  $('error-summary').hidden = false;
  $('error-message').textContent = issue.message;
  $('error-fields').replaceChildren();
  for (const [label, value] of [['Error code', issue.code], ['Rule', issue.rule], ['Field', issue.path], ['Reason', issue.reason]]) {
    if (value) $('error-fields').append(element('dt', label), element('dd', value));
  }
  $('error-next').textContent = `Next: ${issue.next}`;
  $('error-file-note').textContent = issue.fileNote;
  $('error-links').replaceChildren();
  for (const path of issue.candidates) if (model.files.has(path)) $('error-links').append(button(`Open candidate ${path}`, () => openErrorCandidate(path, issue.path)));
  details(JSON.stringify(result, null, 2)); $('error-details').open = false;
  status(`Check failed: ${issue.message}`, true);
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
  status('Draft changed. Check structure, then save a new copy.');
}
function edit(path, target, key, value) { model.edit(path, target, key, value); changed(); renderNavigation(); renderCanvas(); }
function field(parent, label, value, onChange, { type = 'text', help, options, optional = false, mono = false, fieldKey } = {}) {
  if (value !== undefined && !['string', 'number', 'bigint', 'boolean'].includes(typeof value)) throw new Error(`${label} cannot be represented by the form; original bytes are preserved.`);
  const wrap = element('label', undefined, 'field'); wrap.append(element('span', label));
  const input = element(options ? 'select' : type === 'textarea' ? 'textarea' : 'input');
  if (options) {
    for (const [key, text] of options) { const option = element('option', text); option.value = key; input.append(option); }
    const val = value === undefined ? '' : String(value);
    if (![...input.options].some(o => o.value === val)) { const option = element('option', `Original value: ${val}`); option.value = val; input.append(option); }
  } else if (type !== 'textarea') input.type = type;
  input.value = value === undefined ? '' : String(value);
  if (fieldKey) input.dataset.fieldKey = fieldKey;
  if (mono) input.classList.add('mono');
  let acceptedValue = input.value;
  input.addEventListener('change', safe(() => {
    let value = input.value;
    if (optional && value === '') value = undefined;
    else if (type === 'number') { value = Number(value); if (!Number.isSafeInteger(value)) throw new Error(`${label} must be an integer.`); value = BigInt(value); }
    try { onChange(value); acceptedValue = input.value; }
    catch (error) { input.value = acceptedValue; throw error; }
  }));
  wrap.append(input);
  if (help) wrap.append(element('small', help, 'field-help'));
  parent.append(wrap); return input;
}
const boolOptions = [['', 'Not declared (use default)'], ['true', 'Yes'], ['false', 'No']];
let suppressClick = false;
function booleanField(parent, label, value, onChange, help) { field(parent, label, value, v => onChange(declarationBoolean(v)), { options: boolOptions, help }); }
function advanced(parent, label = 'Advanced settings') { const d = element('details'); d.append(element('summary', label)); parent.append(d); return d; }
function section(parent, title, add) { const head = element('div', undefined, 'section-label'); head.append(element('h3', title)); if (add) head.append(button('＋ Add', add)); parent.append(head); }
function flow() { return model.flow(flowPath); }
function layoutKey() { return `sheltie-editor-layout:${model.manifest.id}:${flowPath}`; }
function loadView() { view = CanvasView.load(localStorage, layoutKey()); try { view.prepare(flow()); view.initialCamera(flow(), shownRoutes()); } catch { /* The properties panel explains the original error. */ } }
function saveView() { try { view.save(localStorage, layoutKey()); } catch { status('Layout could not be saved in the browser; the Workbook draft can still be saved.'); } }
function position(node, index) { return view.position(node.id, index); }
function shownEdges() { return visibleEdges(flow(), graphMode, selection.node?.id, selection.edge); }
function shownRoutes() { return routeEdges(flow(), view.positions, shownEdges()); }
function svg(tag, attributes = {}, text) { const e = document.createElementNS(svgNS, tag); for (const [k, v] of Object.entries(attributes)) e.setAttribute(k, String(v)); if (text !== undefined) e.textContent = String(text); return e; }
function renderNavigation() {
  $('method').replaceChildren(button(model.manifest.name || 'Unnamed Workbook', () => { selection = { type: 'manifest' }; renderProperties(); }, `nav-item ${selection.type === 'manifest' ? 'active' : ''}`));
  $('flows').replaceChildren();
  for (const path of model.manifest.flows) {
    const b = button(path, () => { flowPath = path; selection = { type: 'flow' }; loadView(); render(); }, `nav-item ${path === flowPath ? 'active' : ''}`);
    const parsed = model.documents.get(path); b.append(element('small', model.errors.has(path) ? 'Cannot parse · Original bytes preserved' : `Flow ${parsed?.id ?? ''}`)); $('flows').append(b);
  }
  $('node-list').replaceChildren();
  try { for (const node of flow().nodes) $('node-list').append(button(`${node.title ?? node.id}`, () => selectNode(node), `nav-item ${selection.node === node ? 'active' : ''}`)); } catch { /* The properties panel shows the original error for an unrepresentable Flow. */ }
  $('edge-list').replaceChildren();
  try { for (const edge of flow().edges ?? []) $('edge-list').append(button(edgeDescription(edge), () => selectEdge(edge, true), `nav-item edge-list-item ${selection.edge === edge ? 'active' : ''}`)); } catch { /* Preserve original bytes. */ }
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
  $('edge-count').textContent = `Showing ${routes.length} of ${(current.edges ?? []).length} edges`;
  $('graph-mode').textContent = graphMode === 'main' ? 'Main flow ▾' : 'All edges ▾';
  for (const route of routes) {
    const { edge, index, d, label } = route;
    const active = selection.edge === edge, related = selection.node && (edge.from === selection.node.id || edge.to === selection.node.id);
    const group = svg('g', { 'data-edge-index': index, 'data-from': edge.from, 'data-to': edge.to });
    const hit = svg('path', { d, class: 'edge-hit', tabindex: 0, role: 'button', 'aria-label': `${edge.from} to ${edge.to}, ${edge.kind}` });
    group.addEventListener('pointerdown', event => event.stopPropagation());
    group.addEventListener('click', event => {
      event.stopPropagation(); if (suppressClick) return;
      if (event.target.closest('.edge-label')) { selectEdge(edge); return; }
      const transform = renderedCanvasTransform($('world').getBoundingClientRect(), $('world').offsetWidth);
      const picked = pickRouteAtPoint(routes, { x: event.clientX, y: event.clientY }, transform);
      if (picked.ambiguous.length) status(`Multiple edges intersect here (${picked.ambiguous.map(r => r.index + 1).join(', ')}). Select a numbered label or an item in All edges.`);
      else if (picked.route) selectEdge(picked.route.edge);
    });
    group.addEventListener('keydown', event => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); selectEdge(edge); } });
    group.append(hit, svg('path', { d, class: `edge kind-${edge.kind} ${active ? 'active' : related ? 'related' : ''}`, 'marker-end': 'url(#arrow)' }));
    const caption = `${index + 1} ${edgeKindLabel(edge.kind)}`, width = route.labelWidth;
    const labelGroup = svg('g', { class: `edge-label ${active ? 'active label-visible' : ''}`, 'data-edge-index': index, 'data-from': edge.from, 'data-to': edge.to, role: 'button', tabindex: 0, 'aria-label': `Edge label: ${edgeDescription(edge)}` });
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
    const head = element('span', undefined, 'node-head'); head.append(element('span', node.id)); if (node.id === current.entry) head.append(element('span', 'Entry', 'entry-mark'));
    b.append(head, element('span', node.title, 'node-title'), element('span', `${node.executor === 'human' ? 'human' : 'agent'}${node.gate ? ' · Gate' : ''} · ${(node.inputs ?? []).length} inputs / ${(node.outputs ?? []).length} outputs`, 'node-meta'));
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
    else status('The source choice is incomplete; the original source is unchanged.');
  };
  const renderSource = () => {
    body.replaceChildren();
    field(body, 'Where the input comes from', draft.type, type => {
      draft = type === draft.type ? draft : { type };
      if (type === 'custom') draft.raw = row.from;
      commit(); renderSource();
    }, { options: [['start', 'Provided at Work start'], ['step', 'Output from another step'], ['resource', 'Workbook reference file'], ['stats', 'Work statistics'], ['custom', 'Custom source (preserve original)']] });
    if (draft.type === 'start') field(body, 'Input name provided at start', draft.key, value => { draft.key = value; commit(); }, { help: 'For example, task. Leaving it blank preserves the original source.' });
    else if (draft.type === 'step') {
      const steps = sourceChoices(flow(), model.files, node.id).steps;
      field(body, 'Source step', draft.node, value => { draft.node = value; draft.output = undefined; commit(); renderSource(); }, { options: [['', 'Choose a step'], ...steps.map(n => [n.id, `${n.title || n.id}（${n.id}）`])] });
      const outputs = steps.find(n => n.id === draft.node)?.outputs ?? [];
      field(body, 'Which output to read', draft.output, value => { draft.output = value; commit(); }, { options: [['', outputs.length ? 'Choose an output' : 'This step declares no outputs'], ...outputs.filter(o => typeof o.name === 'string').map(o => [o.name, `${o.name} · ${o.path ?? 'filename not declared'}`])] });
    } else if (draft.type === 'resource') {
      const resources = sourceChoices(flow(), model.files, node.id).resources;
      field(body, 'Choose a reference file', draft.path, value => { draft.path = value; commit(); }, { options: [['', resources.length ? 'Choose a file' : 'This Workbook has no reference files yet'], ...resources.map(p => [p, p])] });
    } else if (draft.type === 'stats') body.append(element('p', 'The engine provides statistics for this Work.', 'field-help'));
    else field(body, 'Original custom source', draft.raw, value => { draft.raw = value; commit(); }, { mono: true, help: 'Unknown or missing references are preserved; Check structure determines validity.' });
    const syntax = advanced(body, 'View original source'); rawPreview = element('code', row.from === undefined ? 'Not declared' : row.from); syntax.append(rawPreview);
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
  if (reference) field(place, 'Choose or enter inputs', reference.location, value => adoptModel(editFixedReference(model, flowPath, node.id, node.inputs.indexOf(row), value), node.id), { help: 'Fixed reference location. Confirming creates a new resource copy; the editor does not access the location.' });
  else place.append(element('p', sourceSummary(row, node), 'source-summary'));
}
function materialChooser(parent, node, onStagedChange = () => {}) {
  const box = element('div', undefined, 'material-chooser'); parent.append(box);
  const wrap = element('label', undefined, 'field'); wrap.append(element('span', 'Choose or enter inputs'));
  const input = element('input'); input.type = 'text'; input.placeholder = 'Choose predecessor inputs, or enter a reference location and press Enter'; input.setAttribute('role', 'combobox'); input.setAttribute('aria-autocomplete', 'list'); input.setAttribute('aria-expanded', 'false'); wrap.append(input); box.append(wrap);
  const dropdown = element('div', undefined, 'material-options'), chips = element('div', undefined, 'material-chips'), preview = element('p', undefined, 'field-help'); dropdown.hidden = true;
  const staged = [];
  const materials = () => predecessorMaterials(flow(), model.files, node.id);
  const plannedRows = () => staged.map(item => ({ ...item, from: item.location === undefined ? item.from : (node.inputs ?? []).find(row => row.name === item.name && fixedReference(model.files, row.from, flowPath, node.id)?.location === item.location)?.from ?? `Fixed reference: ${item.location}` }));
  const redraw = () => {
    chips.replaceChildren();
    for (const item of staged) {
      const chip = element('div', undefined, 'material-chip'); chip.append(element('span', item.location ?? `${item.nodeTitle} · ${item.name}`));
      field(chip, 'Input name', item.name, value => { item.name = value; refreshPreview(); }, { mono: true });
      chip.append(button('Remove selection', () => { removeStagedChoice(staged, item); redraw(); })); chips.append(chip);
    }
    renderOptions(); refreshPreview();
  };
  const addExternal = () => {
    if (!input.value.trim()) return;
    staged.push({ location: input.value, name: suggestedInputName('reference', `Fixed reference: ${input.value}`, [...(node.inputs ?? []), ...plannedRows()]) }); input.value = ''; redraw();
  };
  const optionElements = new Map(), optionGroups = new Map();
  const placeOptions = (container, children) => {
    children.forEach((child, index) => { if (container.children[index] !== child) container.insertBefore(child, container.children[index] ?? null); });
    for (const child of [...container.children]) if (!children.includes(child)) child.remove();
  };
  const externalChoice = button('Use entered text as a fixed reference', addExternal);
  const renderOptions = () => {
    const query = input.value.toLowerCase(), options = materials(), blocks = [];
    for (const [kind, title] of [['output', 'Predecessor outputs'], ['input', 'Reuse original predecessor input sources']]) {
      let group = optionGroups.get(kind);
      if (!group) { const container = element('div'), rows = element('div'), empty = element('p', 'No inputs available; you can enter a fixed reference directly.', 'field-help'); container.append(element('strong', title, 'material-group-title'), rows); group = { container, rows, empty }; optionGroups.set(kind, group); }
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
        entry.label.classList.toggle('already-used', used); entry.caption.textContent = `${option.nodeTitle} · ${option.name}${option.required === false ? ' (optional)' : ''}`;
        entry.aliases.hidden = !used; entry.aliases.textContent = used ? `Already used · This step: ${option.usedNames.join(', ')}` : '';
        entry.reason.hidden = !option.reason; entry.reason.textContent = option.reason; choices.push(entry.label);
      }
      placeOptions(group.rows, choices.length ? choices : [group.empty]); blocks.push(group.container);
    }
    if (input.value.trim()) blocks.push(externalChoice);
    placeOptions(dropdown, blocks);
  };
  const refreshPreview = () => {
    const plan = planInputRows(node.inputs ?? [], plannedRows());
    preview.textContent = plan.conflicts.length ? plan.conflicts.map(c => `${c.name}：${c.reason}`).join(' ') : staged.length ? `Ready to add ${plan.additions.length} items; ${plan.skipped.length} existing items will not be duplicated.` : 'Select multiple items or enter a location to store in the Workbook. The editor does not access that location.';
    submit.disabled = !staged.length || Boolean(plan.conflicts.length); onStagedChange(staged.length);
  };
  const submit = button('Add selected inputs', safe(() => { const result = addMaterialInputs(model, flowPath, node.id, staged); if (result.model === model) status('Selected inputs already exist; no duplicates added.'); else { adoptModel(result.model, node.id); status(`Added ${result.added} inputs.`); } }));
  const open = () => { dropdown.hidden = false; input.setAttribute('aria-expanded', 'true'); renderOptions(); };
  input.addEventListener('focus', open); input.addEventListener('input', open);
  input.addEventListener('keydown', event => { if (event.key === 'Enter') { event.preventDefault(); addExternal(); } else if (event.key === 'Escape') { event.preventDefault(); dropdown.hidden = true; input.setAttribute('aria-expanded', 'false'); } });
  refreshMaterialOptions = () => { if (pruneUsedStaged(staged, materials())) redraw(); else { renderOptions(); refreshPreview(); } };
  box.append(element('p', 'A checked item is used by this step. Remove it individually from the selected list.', 'field-help'));
  box.append(button('Expand input choices', () => { dropdown.hidden = !dropdown.hidden; input.setAttribute('aria-expanded', String(!dropdown.hidden)); if (!dropdown.hidden) renderOptions(); }), dropdown, chips, preview, submit); refreshPreview();
}
function bindingEditor(parent, node, key, row) {
  const output = key === 'outputs';
  field(parent, output ? 'Output name' : 'Input name', row.name, value => edit(flowPath, row, 'name', value));
  if (output) field(parent, 'Filename or relative path', row.path, value => edit(flowPath, row, 'path', value), { mono: true, help: 'For example, report.md or reports/summary.md.' });
  else { const place = element('div'); place.dataset.inputSourceIndex = node.inputs.indexOf(row); parent.append(place); renderInputSource(place, row, node); }
  const extra = advanced(parent, 'Advanced: exact source, required declaration, and final result');
  if (!output) sourceForm(extra, row, node);
  booleanField(extra, 'Required', row.required, value => edit(flowPath, row, 'required', value), 'Required when omitted; false allows absence.');
  booleanField(extra, 'Select as final result', row.result, value => edit(flowPath, row, 'result', value), 'Only required inputs or outputs of a successful terminal node may be final results.');
  if (output) field(extra, 'Size limit (bytes)', row.max_bytes, value => edit(flowPath, row, 'max_bytes', declarationBytes(value)), { help: 'Blank means omitted; the default is 1048576 bytes. Existing large integers are preserved exactly.' });
  parent.append(button(output ? 'Remove this output' : 'Remove this input', () => {
    node[key].splice(node[key].indexOf(row), 1); model.commit(flowPath); bindingState.expanded = undefined; changed(); renderProperties();
  }, 'danger'));
}
function bindingPane(parent, node) {
  if (!bindingState || bindingState.flow !== flowPath || bindingState.node !== node.id) bindingState = { flow: flowPath, node: node.id, tab: 'inputs', query: '', expanded: undefined };
  const pane = element('section', undefined, 'binding-pane'), tabs = element('div', undefined, 'binding-tabs'), tools = element('div', undefined, 'binding-tools'), editor = element('div', undefined, 'binding-editor'), list = element('div', undefined, 'compact-binding-list'); parent.append(pane); tabs.setAttribute('role', 'tablist'); tabs.setAttribute('aria-label', 'Node inputs and outputs');
  const current = () => flow().nodes.find(n => n.id === selection.node?.id) ?? node;
  let editorTitle;
  const redrawEditor = () => {
    editor.replaceChildren(); const actual = current(), row = actual[bindingState.tab]?.[bindingState.expanded]; editor.hidden = !row;
    if (!row) return;
    const header = element('div', undefined, 'binding-editor-head'); editorTitle = element('strong', `Edit: ${row.name ?? 'name not declared'}`); header.append(editorTitle, button('Collapse editor', () => { bindingState.expanded = undefined; redrawEditor(); redrawList(); })); editor.append(header);
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
      block.querySelector('.compact-group-title small').textContent = `${group.rows.length} items`;
      const buttons = [];
      for (const item of group.rows) {
        let row = rowElements.get(item.row);
        if (!row) {
          row = button('', () => { const index = Number(row.dataset.bindingIndex); bindingState.expanded = bindingState.expanded === index ? undefined : index; redrawEditor(); redrawList(); }, 'compact-binding-row');
          const top = element('span', undefined, 'compact-row-name'); top.append(element('strong'), element('small', 'Optional', 'binding-badge')); row.append(top, element('span', undefined, 'compact-row-summary')); rowElements.set(item.row, row);
        }
        const active = item.index === bindingState.expanded; row.dataset.bindingIndex = item.index; row.classList.toggle('expanded', active); row.setAttribute('aria-expanded', String(active));
        row.querySelector('strong').textContent = item.row.name === undefined ? 'Name not declared' : item.row.name;
        row.querySelector('.binding-badge').hidden = item.row.required !== false; row.querySelector('.compact-row-summary').textContent = item.summary; buttons.push(row);
      }
      placeInOrder(block.querySelector('.compact-group-rows'), buttons); blocks.push(block);
    }
    if (!blocks.length) blocks.push(element('p', bindingState.query ? 'No matching inputs or outputs.' : 'No inputs or outputs added yet.', 'field-help'));
    placeInOrder(list, blocks);
    if (editorTitle && bindingState.expanded !== undefined) editorTitle.textContent = `Edit: ${actual[bindingState.tab]?.[bindingState.expanded]?.name ?? 'name not declared'}`;
    inputTab.textContent = `Inputs ${actual.inputs?.length ?? 0}`; outputTab.textContent = `Outputs ${actual.outputs?.length ?? 0}`;
  };
  const inputTools = element('div'), outputTools = element('div'); tools.append(inputTools, outputTools);
  const add = advanced(inputTools, '＋ Add inputs'), addTitle = add.querySelector('summary');
  materialChooser(add, current(), count => { addTitle.textContent = count ? `＋ Add inputs · ${count} pending` : '＋ Add inputs'; });
  outputTools.append(button('＋ Add output', () => {
    const actual = current(); actual.outputs ??= []; let number = 1; while (actual.outputs.some(row => row.name === `output-${number}`)) number++;
    actual.outputs.push({ name: `output-${number}`, path: `output-${number}.md` }); model.commit(flowPath); bindingState.expanded = actual.outputs.length - 1; changed(); renderProperties();
  }));
  const redrawTools = () => { inputTools.hidden = bindingState.tab !== 'inputs'; outputTools.hidden = bindingState.tab !== 'outputs'; };
  const selectTab = key => {
    bindingState.tab = key; bindingState.expanded = undefined; bindingState.query = ''; search.value = '';
    inputTab.setAttribute('aria-selected', String(key === 'inputs')); outputTab.setAttribute('aria-selected', String(key === 'outputs'));
    redrawTools(); redrawEditor(); redrawList();
  };
  const inputTab = button(`Inputs ${node.inputs?.length ?? 0}`, () => selectTab('inputs')), outputTab = button(`Outputs ${node.outputs?.length ?? 0}`, () => selectTab('outputs'));
  for (const tab of [inputTab, outputTab]) tab.setAttribute('role', 'tab'); tabs.append(inputTab, outputTab);
  inputTab.setAttribute('aria-selected', String(bindingState.tab === 'inputs')); outputTab.setAttribute('aria-selected', String(bindingState.tab === 'outputs'));
  const searchWrap = element('label', undefined, 'field binding-search'); searchWrap.append(element('span', 'Search inputs and outputs'));
  const search = element('input'); search.type = 'search'; search.placeholder = 'Name, source, or filename'; search.value = bindingState.query; search.addEventListener('input', () => { bindingState.query = search.value; redrawList(); }); searchWrap.append(search);
  pane.append(tabs, searchWrap, editor, list, tools); redrawTools(); redrawEditor(); redrawList(); refreshBindingPane = redrawList;
}
function instructionForm(parent, node) {
  if (!node.instruction || typeof node.instruction !== 'object' || Array.isArray(node.instruction)) throw new Error('instruction cannot be represented; original bytes are preserved.');
  const instruction = node.instruction;
  if ((instruction.file === undefined) === (instruction.text === undefined)) throw new Error('instruction must have exactly one of file or text; preserve original bytes without silently choosing.');
  field(parent, 'Instruction source', instruction.file !== undefined ? 'file' : 'text', value => {
    if (value === 'file') { delete instruction.text; instruction.file = `instructions/${node.id}.md`; }
    else { let text = 'Describe what to read, what to complete, and the output requirements.'; if (instruction.file && model.files.has(instruction.file)) text = model.text(instruction.file); delete instruction.file; instruction.text = text; }
    model.commit(flowPath); changed(); renderProperties();
  }, { options: [['text', 'Inline text'], ['file', 'Instruction file']] });
  if (instruction.file !== undefined) {
    field(parent, 'Instruction file path', instruction.file, value => { edit(flowPath, instruction, 'file', value); renderProperties(); }, { mono: true });
    if (model.files.has(instruction.file)) field(parent, 'Instruction file contents', model.text(instruction.file), value => { model.writeText(instruction.file, value); changed(); if (model.documents.has(instruction.file)) { const id = node.id; try { const refreshed = flow().nodes.find(n => n.id === id); selection = refreshed ? { type: 'node', node: refreshed } : { type: 'flow' }; } catch { selection = { type: 'flow' }; } render(); } }, { type: 'textarea', help: 'Edits only this draft file; other nodes sharing the file also see changes.' });
    else { parent.append(element('p', 'This instruction file does not exist. Create it to edit, or switch to inline text.', 'warn')); parent.append(button('Create instruction file', () => { model.writeText(instruction.file, 'Describe what to read, what to complete, and the output requirements.\n'); changed(); renderProperties(); })); }
  } else field(parent, 'Instruction text', instruction.text, value => edit(flowPath, instruction, 'text', value), { type: 'textarea' });
}
function nodeForm(parent, node) {
  field(parent, 'Node ID', node.id, value => edit(flowPath, node, 'id', value), { mono: true, fieldKey: 'id', help: 'After changing the ID, update entry, edges and input references manually; checks locate remaining references.' });
  field(parent, 'Title', node.title, value => edit(flowPath, node, 'title', value), { fieldKey: 'title' });
  field(parent, 'Executor', node.executor, value => edit(flowPath, node, 'executor', value), { options: [['agent', 'Agent'], ['human', 'Human']] });
  parent.append(button('Set as entry', () => edit(flowPath, flow(), 'entry', node.id)));
  bindingPane(parent, node);
  const instructions = advanced(parent, 'Instructions and guidance'); instructionForm(instructions, node);
  const extra = advanced(parent);
  field(extra, 'Model tier', node.tier, value => edit(flowPath, node, 'tier', value === '' ? undefined : value), { options: [['', 'Not declared (standard)'], ['standard', 'standard'], ['strong', 'strong']], help: 'Human executors must not declare tier; choose Not declared.' });
  booleanField(extra, 'Gate (default: false)', node.gate, value => edit(flowPath, node, 'gate', value));
  field(extra, 'max_visits (default: 1)', node.max_visits, value => edit(flowPath, node, 'max_visits', value), { type: 'number', optional: true });
  field(extra, 'max_retries (default: 1)', node.max_retries, value => edit(flowPath, node, 'max_retries', value), { type: 'number', optional: true });
  if (node.requires !== undefined && (!Array.isArray(node.requires) || node.requires.some(v => typeof v !== 'string'))) throw new Error('requires cannot be represented; original bytes are preserved.');
  field(extra, 'Host resources: one kind:name per line', node.requires?.join('\n'), value => edit(flowPath, node, 'requires', value === undefined ? undefined : value.split('\n').map(v => v.trim()).filter(Boolean)), { type: 'textarea', optional: true, mono: true, help: 'Declare resources in Workbook advanced settings first; the editor does not install them.' });
  parent.append(button('Delete node; retain references for checking', () => { model.deleteNode(flowPath, node); selection = { type: 'flow' }; changed(); render(); }, 'danger'));
}
function manifestForm(parent) {
  for (const [key, label] of [['id', 'Workbook ID'], ['version', 'Version'], ['name', 'Name'], ['description', 'Description']]) field(parent, label, model.manifest[key], value => edit('workbook.toml', model.manifest, key, value), { type: key === 'description' ? 'textarea' : 'text', optional: key === 'description', mono: ['id', 'version'].includes(key) });
  parent.append(element('p', `${model.files.size} files. Unedited Flows, instructions and resources preserve their original bytes.`, 'field-help'));
  const extra = advanced(parent, 'Advanced: host resource declarations');
  const rows = model.manifest.requires;
  if (rows !== undefined && (!Array.isArray(rows) || rows.some(row => !row || typeof row !== 'object' || Array.isArray(row)))) throw new Error('Manifest requires cannot be represented; original bytes are preserved.');
  extra.append(button('＋ Add resource declaration', () => { model.manifest.requires ??= []; model.manifest.requires.push({ kind: 'skill', name: 'resource-name' }); model.commit('workbook.toml'); changed(); renderProperties(); }));
  (rows ?? []).forEach((row, index) => {
    const box = element('div', undefined, 'binding-row'); extra.append(box);
    field(box, 'Resource kind', row.kind, value => edit('workbook.toml', row, 'kind', value), { options: [['skill', 'skill'], ['agent', 'agent'], ['mcp', 'mcp']] });
    for (const key of ['name', 'version', 'digest', 'source']) field(box, key, row[key], value => edit('workbook.toml', row, key, value), { mono: true, optional: key !== 'name' });
    box.append(button('Delete resource declaration', () => { rows.splice(index, 1); model.commit('workbook.toml'); changed(); renderProperties(); }, 'danger'));
  });
}
function renderProperties() {
  refreshBindingPane = undefined; refreshMaterialOptions = undefined;
  const parent = $('property-content'); parent.replaceChildren();
  try {
    $('property-title').textContent = selection.type === 'manifest' ? 'Workbook properties' : selection.type === 'node' ? 'Node properties' : selection.type === 'edge' ? 'Explicit edge properties' : 'Flow properties';
    if (selection.type === 'manifest') manifestForm(parent);
    else if (selection.type === 'node') nodeForm(parent, selection.node);
    else if (selection.type === 'edge') {
      const edge = selection.edge;
      const summary = element('p', `From [${nodeTitle(edge.from)}] to [${nodeTitle(edge.to)}]`, 'edge-summary'); summary.dataset.edgeFrom = edge.from; summary.dataset.edgeTo = edge.to; parent.append(summary);
      const options = flow().nodes.map(node => [String(node.id), `${node.id} · ${node.title}`]);
      field(parent, 'From step', edge.from, value => { edit(flowPath, edge, 'from', value); renderProperties(); }, { options });
      field(parent, 'To step', edge.to, value => { edit(flowPath, edge, 'to', value); renderProperties(); }, { options });
      field(parent, 'Edge kind', edge.kind, value => edit(flowPath, edge, 'kind', value), { options: ['main', 'back', 'branch', 're_review'].map(v => [v, edgeKindLabel(v)]) });
      parent.append(element('p', 'Explicit edges define legal next steps; they do not create input bindings.', 'field-help'));
      parent.append(button('Delete explicit edge', () => { const edges = flow().edges; edges.splice(edges.indexOf(edge), 1); model.commit(flowPath); selection = { type: 'flow' }; changed(); render(); }, 'danger'));
    } else {
      const current = flow();
      field(parent, 'Flow ID', current.id, value => edit(flowPath, current, 'id', value), { mono: true });
      field(parent, 'Entry node', current.entry, value => edit(flowPath, current, 'entry', value), { fieldKey: 'entry', options: current.nodes.map(n => [String(n.id), `${n.id} · ${n.title}`]) });
      parent.append(element('p', 'Select a node to edit instructions, inputs and outputs. Select an edge to edit its declaration.', 'field-help'));
    }
    const raw = advanced(parent, 'View current TOML source');
    const path = selection.type === 'manifest' ? 'workbook.toml' : flowPath;
    const pre = element('pre', model.text(path)); pre.dataset.previewPath = path; raw.append(pre);
  } catch (error) { parent.append(element('p', error.message, 'warn')); if (model.files.has(flowPath)) parent.append(element('pre', new TextDecoder().decode(model.files.get(flowPath)))); }
}
function render() { renderNavigation(); renderCanvas(); renderProperties(); }
function replaceModel(next) { model = next; flowPath = model.manifest.flows[0]; selection = { type: 'manifest' }; graphMode = 'main'; dirty = false; loadView(); render(); clearFailure(); details(''); $('error-details').open = false; status('Draft opened. Select a node to edit; saving checks the current files again.'); }
function confirmReplace() { return !dirty || confirm('This draft has no saved copy. Continuing will discard the draft on this page. Continue?'); }
async function sample(name) {
  if (!confirmReplace()) return;
  const response = await fetch(`/api/sample/${name}`); if (!response.ok) throw new Error('Could not read the sample.');
  const entries = await response.json(); const files = new Map(entries.map(([p, encoded]) => [p, Uint8Array.from(atob(encoded), c => c.charCodeAt(0))])); replaceModel(new WorkbookModel(files));
}
async function operation(kind) {
  if (busy || !token) return;
  const entries = toWire(model.snapshot()); busy = true; for (const id of ['new', 'open', 'check', 'save']) $(id).disabled = true; document.querySelector('.workspace').inert = true;
  status(kind === 'export' ? 'Checking current bytes and generating a complete new copy…' : 'Checking structure in a fresh temporary Home…');
  try {
    const response = await fetch(`/api/${kind}`, { method: 'POST', headers: { 'Content-Type': 'application/json', 'X-Editor-Token': token }, body: JSON.stringify(entries) });
    if (!response.ok) { showFailure(await response.json()); return; }
    clearFailure();
    if (kind === 'check') { const result = await response.json(); details(JSON.stringify(result, null, 2)); $('error-details').open = false; status(`Structure check passed: ${result.result.data.id}@${result.result.data.version}. Saving will verify again.`); }
    else {
      const blob = await response.blob(); const url = URL.createObjectURL(blob), link = element('a'); link.href = url; link.download = `${model.manifest.id}-${model.manifest.version}.zip`; document.body.append(link); link.click(); link.remove(); setTimeout(() => URL.revokeObjectURL(url), 60000); dirty = false; details(''); $('error-details').open = false; status('Complete new copy downloaded. Extract it, then reopen the directory. The original directory was not written.');
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
  if (nodes.length < 2) throw new Error('Add at least two nodes before adding an explicit edge.');
  const from = selection.node ?? nodes[0], to = nodes.find(n => n !== from);
  const previousEdges = current.edges?.slice() ?? [];
  const edge = model.addEdge(flowPath, from.id, to.id), existing = previousEdges.includes(edge);
  if (!existing) changed();
  selectEdge(edge);
  if (existing) status(`This edge already exists; selected ${edge.from} → ${edge.to}.`);
}));
$('add-flow').addEventListener('click', safe(() => {
  const id = prompt('New Flow ID (lowercase letters, digits and single hyphens)', 'new-flow'); if (id === null) return;
  if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(id)) throw new Error('Invalid Flow ID format.');
  const path = `flows/${id}.toml`; if (model.files.has(path) || [...model.documents.values()].some(d => d.schema === 'flow/v1' && d.id === id)) throw new Error('Flow path or ID already exists.');
  const source = newWorkbook(); const definition = source.flow('flows/default.toml'); definition.id = id; source.commit('flows/default.toml'); model.writeText(path, source.text('flows/default.toml')); model.load(path); model.manifest.flows.push(path); model.commit('workbook.toml'); flowPath = path; selection = { type: 'flow' }; changed(); loadView(); render();
}));
function zoom(factor, px = $('viewport').clientWidth / 2, py = $('viewport').clientHeight / 2) { view.zoom(factor, px, py, $('viewport').scrollLeft, $('viewport').scrollTop); renderCanvas(); saveView(); }
$('zoom-in').addEventListener('click', () => zoom(1.15)); $('zoom-out').addEventListener('click', () => zoom(1 / 1.15));
function fitCanvas(arrange = false) {
  const viewport = $('viewport'); viewport.scrollLeft = 0; viewport.scrollTop = 0;
  if (arrange) view.arrange(flow());
  view.fitGraph(flow(), viewport.clientWidth, viewport.clientHeight, shownRoutes(), selection.edge); renderCanvas(); saveView();
}
$('graph-mode').addEventListener('click', safe(() => { graphMode = graphMode === 'main' ? 'all' : 'main'; renderCanvas(); status(graphMode === 'main' ? 'Main-flow view: other edges remain selectable in the left-hand list.' : 'Complete view: showing every declared edge.'); }));
$('reset-view').addEventListener('click', safe(() => fitCanvas()));
$('arrange').addEventListener('click', safe(() => { fitCanvas(true); status('Main-flow layout arranged. Workbook content is unchanged.'); }));
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
try { const response = await fetch('/api/session'); if (!response.ok) throw new Error('Local service session unavailable. Open the page using its 127.0.0.1 address.'); token = (await response.json()).token; $('check').disabled = false; $('save').disabled = false; status('New draft ready. Add nodes and explicit edges, or open a directory or sample.'); } catch (error) { showError(error); }
