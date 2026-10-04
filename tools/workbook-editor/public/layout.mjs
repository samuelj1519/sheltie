export class CanvasView {
  constructor() {
    this.x = 45; this.y = 45; this.scale = 1;
    this.positions = new Map();
  }
  static load(storage, key) {
    const view = new CanvasView();
    try {
      const stored = JSON.parse(storage.getItem(key) ?? 'null');
      if (stored && Number.isFinite(stored.x) && Number.isFinite(stored.y) && stored.scale >= .25 && stored.scale <= 2.5 && Array.isArray(stored.positions)) {
        view.x = stored.x; view.y = stored.y; view.scale = stored.scale;
        view.positions = new Map(stored.positions.filter(e => Array.isArray(e) && typeof e[0] === 'string' && Number.isFinite(e[1]?.x) && Number.isFinite(e[1]?.y)));
      }
    } catch { /* 布局缓存损坏不修改 Workbook 字节。 */ }
    return view;
  }
  save(storage, key) { storage.setItem(key, JSON.stringify({ x: this.x, y: this.y, scale: this.scale, positions: [...this.positions] })); }
  position(id, index) {
    if (!this.positions.has(id)) this.positions.set(id, { x: (index % 3) * 280, y: Math.floor(index / 3) * 185 });
    return this.positions.get(id);
  }
  move(drag, pointerX, pointerY) {
    const dx = pointerX - drag.startX, dy = pointerY - drag.startY;
    if (drag.type === 'pan') { this.x = drag.x + dx; this.y = drag.y + dy; }
    else if (drag.type === 'node') this.positions.set(drag.id, { x: drag.x + dx / this.scale, y: drag.y + dy / this.scale });
    else throw new Error('未知画布拖动类型。');
  }
  zoom(factor, px, py) {
    const scale = Math.max(.25, Math.min(2.5, this.scale * factor));
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
