import { parse, stringify } from './toml.mjs';
import { validateMap } from './files.mjs';
const encode = new TextEncoder();
const decode = new TextDecoder('utf-8', { fatal: true });
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);
export class WorkbookModel {
  constructor(files) {
    validateMap(files);
    this.files = new Map([...files].map(([p, bytes]) => [p, bytes.slice()]));
    this.documents = new Map();
    this.errors = new Map();
    this.manifest = this.load('workbook.toml');
    for (const key of ['schema', 'id', 'version', 'name', 'description']) if (this.manifest[key] !== undefined && typeof this.manifest[key] !== 'string') throw new Error(`workbook.toml field ${key} cannot be represented; original bytes are preserved.`);
    if (!Array.isArray(this.manifest.flows) || this.manifest.flows.some(p => typeof p !== 'string')) throw new Error('workbook.toml flows must be a path array. The original file is unchanged.');
    for (const path of this.manifest.flows) {
      try { this.load(path); } catch (error) { this.errors.set(path, error.message); }
    }
  }
  load(path) {
    if (!this.files.has(path)) throw new Error(`File does not exist: ${path}`);
    const parsed = parse(decode.decode(this.files.get(path)), { integersAsBigInt: true });
    if (!object(parsed)) throw new Error(`Not a TOML object: ${path}`);
    this.documents.set(path, parsed);
    return parsed;
  }
  document(path) {
    if (this.errors.has(path)) throw new Error(`${path}: ${this.errors.get(path)}; original bytes are preserved.`);
    if (!this.documents.has(path)) throw new Error(`Cannot edit: ${path}`);
    return this.documents.get(path);
  }
  flow(path) {
    const flow = this.document(path);
    if (!Array.isArray(flow.nodes) || flow.nodes.some(n => !object(n)) || (flow.edges !== undefined && (!Array.isArray(flow.edges) || flow.edges.some(e => !object(e))))) throw new Error(`${path}: nodes/edges cannot be represented; original bytes are preserved.`);
    const requireType = (target, fields, type) => {
      for (const key of fields) if (target[key] !== undefined && typeof target[key] !== type) throw new Error(`${path}: ${key} cannot be represented; original bytes are preserved.`);
    };
    requireType(flow, ['schema', 'id', 'entry'], 'string');
    for (const node of flow.nodes) {
      requireType(node, ['id', 'title', 'executor', 'tier'], 'string');
      requireType(node, ['gate'], 'boolean'); requireType(node, ['max_visits', 'max_retries'], 'bigint');
      if (!object(node.instruction) || (node.instruction.file === undefined) === (node.instruction.text === undefined)) throw new Error(`${path}: instruction cannot be represented; original bytes are preserved.`);
      requireType(node.instruction, ['file', 'text'], 'string');
      if (node.requires !== undefined && (!Array.isArray(node.requires) || node.requires.some(v => typeof v !== 'string'))) throw new Error(`${path}: requires cannot be represented; original bytes are preserved.`);
      for (const key of ['inputs', 'outputs']) {
        if (node[key] !== undefined && (!Array.isArray(node[key]) || node[key].some(row => !object(row)))) throw new Error(`${path}: ${key} cannot be represented; original bytes are preserved.`);
        for (const row of node[key] ?? []) {
          requireType(row, ['name', key === 'inputs' ? 'from' : 'path'], 'string');
          requireType(row, ['required', 'result'], 'boolean'); if (key === 'outputs') requireType(row, ['max_bytes'], 'bigint');
        }
      }
    }
    for (const edge of flow.edges ?? []) requireType(edge, ['from', 'to', 'kind'], 'string');
    return flow;
  }
  commit(path) {
    this.files.set(path, encode.encode(stringify(this.document(path), { numbersAsFloat: true })));
  }
  edit(path, target, key, value) {
    if (path !== 'workbook.toml') {
      const flow = this.flow(path);
      if (flow.edges?.includes(target) && (key === 'from' || key === 'to')) {
        const from = key === 'from' ? value : target.from;
        const to = key === 'to' ? value : target.to;
        if (from === to) throw new Error('An explicit edge cannot connect a node to itself (self-loop); original endpoints are preserved.');
        if (flow.edges.some(edge => edge !== target && edge.from === from && edge.to === to)) throw new Error('An explicit edge already connects these endpoints; duplicates are refused and original endpoints are preserved.');
      }
    } else this.document(path);
    if (value === undefined) delete target[key]; else target[key] = value;
    this.commit(path);
  }
  addNode(path) {
    const flow = this.flow(path);
    let number = 1;
    while (flow.nodes.some(n => n.id === `step-${number}`)) number++;
    const node = { id: `step-${number}`, title: `Step ${number}`, executor: 'agent', instruction: { text: 'Describe what to read, what to complete, and the output requirements.' }, inputs: [], outputs: [] };
    flow.nodes.push(node);
    if (!flow.entry && flow.nodes.length === 1) flow.entry = node.id;
    this.commit(path);
    return node;
  }
  deleteNode(path, node) {
    const flow = this.flow(path);
    flow.nodes.splice(flow.nodes.indexOf(node), 1);
    this.commit(path);
  }
  addEdge(path, from, to, kind = 'main') {
    const flow = this.flow(path);
    const existing = flow.edges?.find(edge => edge.from === from && edge.to === to);
    if (existing) return existing;
    if (from === to) throw new Error('An explicit edge cannot connect a node to itself (self-loop).');
    flow.edges ??= [];
    const edge = { from, to, kind };
    flow.edges.push(edge);
    this.commit(path);
    return edge;
  }
  text(path) {
    if (!this.files.has(path)) throw new Error(`File does not exist: ${path}`);
    return decode.decode(this.files.get(path));
  }
  writeText(path, text) {
    const candidate = new Map(this.files);
    candidate.set(path, encode.encode(text));
    validateMap(candidate);
    if (this.documents.has(path)) {
      const parsed = new WorkbookModel(candidate);
      this.files = parsed.files; this.documents = parsed.documents; this.errors = parsed.errors; this.manifest = parsed.manifest;
    } else this.files = candidate;
  }
  snapshot() { validateMap(this.files); return new Map([...this.files].map(([p, bytes]) => [p, bytes.slice()])); }
}
export function newWorkbook() {
  return new WorkbookModel(new Map([
    ['workbook.toml', encode.encode('schema = "workbook/v1"\nid = "my-workbook"\nversion = "1.0.0"\nname = "My Workbook"\nflows = ["flows/default.toml"]\n')],
    ['flows/default.toml', encode.encode('schema = "flow/v1"\nid = "default"\nentry = "step-1"\n[[nodes]]\nid = "step-1"\ntitle = "First step"\nexecutor = "agent"\ninstruction = { text = "Describe what to read, what to complete, and the output requirements." }\n')],
  ]));
}
