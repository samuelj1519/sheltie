import { graphPositions, graphBounds } from './geometry.mjs';
export class CanvasView {
  constructor() {
    this.x = 45; this.y = 45; this.scale = 1;
    this.positions = new Map(); this.cached = false;
  }
  static load(storage, key) {
    const view = new CanvasView();
    try {
      const stored = JSON.parse(storage.getItem(key) ?? 'null');
      if (stored && Number.isFinite(stored.x) && Number.isFinite(stored.y) && stored.scale >= .01 && stored.scale <= 2.5 && Array.isArray(stored.positions)) {
        view.cached = true; view.x = stored.x; view.y = stored.y; view.scale = stored.scale;
        view.positions = new Map(stored.positions.filter(e => Array.isArray(e) && typeof e[0] === 'string' && Number.isFinite(e[1]?.x) && Number.isFinite(e[1]?.y)));
      }
    } catch { /* A corrupt layout cache never changes Workbook bytes. */ }
    return view;
  }
  save(storage, key) { storage.setItem(key, JSON.stringify({ x: this.x, y: this.y, scale: this.scale, positions: [...this.positions] })); }
  position(id, index) {
    if (!this.positions.has(id)) this.positions.set(id, { x: (index % 3) * 280, y: Math.floor(index / 3) * 185 });
    return this.positions.get(id);
  }
  prepare(flow) {
    for (const [id, pos] of graphPositions(flow)) if (!this.positions.has(id)) this.positions.set(id, pos);
  }
  arrange(flow) { this.positions = graphPositions(flow); }
  initialCamera(flow, routes, labelEdge) {
    if (this.cached) return;
    const { minX, minY } = graphBounds(flow, this.positions, routes, labelEdge);
    this.x = 40 - minX; this.y = 40 - minY;
  }
  fitGraph(flow, width, height, routes, labelEdge) {
    this.prepare(flow);
    const { minX, minY, maxX, maxY } = graphBounds(flow, this.positions, routes, labelEdge);
    this.scale = Math.max(.01, Math.min(1, (width - 80) / (maxX - minX), (height - 80) / (maxY - minY)));
    this.x = 40 - minX * this.scale; this.y = 40 - minY * this.scale;
  }
  move(drag, pointerX, pointerY) {
    const dx = pointerX - drag.startX, dy = pointerY - drag.startY;
    if (drag.type === 'pan') { this.x = drag.x + dx; this.y = drag.y + dy; }
    else if (drag.type === 'node') this.positions.set(drag.id, { x: drag.x + dx / this.scale, y: drag.y + dy / this.scale });
    else throw new Error('Unknown canvas drag type.');
  }
  zoom(factor, px, py, scrollLeft = 0, scrollTop = 0) {
    px += scrollLeft; py += scrollTop;
    const scale = Math.max(.1, Math.min(2.5, this.scale * factor));
    this.x = px - (px - this.x) * scale / this.scale;
    this.y = py - (py - this.y) * scale / this.scale;
    this.scale = scale;
  }
  fit(ids, width, height) {
    const bounds = ids.map((id, i) => this.position(id, i));
    const minX = Math.min(0, ...bounds.map(p => p.x)), minY = Math.min(0, ...bounds.map(p => p.y));
    const maxX = Math.max(240, ...bounds.map(p => p.x + 220)), maxY = Math.max(150, ...bounds.map(p => p.y + 130));
    this.scale = Math.max(.25, Math.min(1, (width - 80) / (maxX - minX), (height - 80) / (maxY - minY)));
    this.x = 40 - minX * this.scale; this.y = 40 - minY * this.scale;
  }
}
