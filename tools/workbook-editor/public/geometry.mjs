export const CARD_WIDTH = 240, CARD_HEIGHT = 156;
const edgeKindLabels = { main: '继续', back: '返工', branch: '分支', re_review: '重新检查' };
export const edgeKindLabel = kind => Object.hasOwn(edgeKindLabels, kind) ? edgeKindLabels[kind] : `原类型：${kind}`;
export function mainSkeleton(flow) {
  const nodes = flow.nodes ?? [], ids = new Set(nodes.map(n => n.id)), visited = new Set(), order = [], edges = [];
  if (!ids.has(flow.entry)) return { nodes: order, edges };
  const queue = [flow.entry]; visited.add(flow.entry);
  for (let i = 0; i < queue.length; i++) {
    const id = queue[i]; order.push(id);
    for (const edge of flow.edges ?? []) if (edge.kind === 'main' && edge.from === id && ids.has(edge.to) && !visited.has(edge.to)) { visited.add(edge.to); queue.push(edge.to); edges.push(edge); }
  }
  return { nodes: order, edges };
}
export function visibleEdges(flow, mode = 'main', selectedNode, selectedEdge) {
  if (mode === 'all') return (flow.edges ?? []).slice();
  const skeleton = new Set(mainSkeleton(flow).edges);
  return (flow.edges ?? []).filter(e => skeleton.has(e) || e === selectedEdge || (selectedNode && (e.from === selectedNode || e.to === selectedNode)));
}
export function graphPositions(flow) {
  const skeleton = mainSkeleton(flow), positions = new Map();
  skeleton.nodes.forEach((id, index) => {
    const row = Math.floor(index / 3), col = row % 2 ? 2 - index % 3 : index % 3;
    positions.set(id, { x: col * 320, y: row * 216 });
  });
  const occupied = new Set();
  for (const node of flow.nodes ?? []) if (!positions.has(node.id)) {
    const related = (flow.edges ?? []).find(e => e.to === node.id && positions.has(e.from)) ?? (flow.edges ?? []).find(e => e.from === node.id && positions.has(e.to));
    const anchor = related ? positions.get(related.from === node.id ? related.to : related.from) : { y: 0 };
    let row = Math.round(anchor.y / 216);
    while (occupied.has(row)) row = row > 0 && !occupied.has(row - 1) ? row - 1 : row + 1;
    occupied.add(row); positions.set(node.id, { x: 960, y: row * 216 });
  }
  return positions;
}
function crossesCard(points, pos) {
  const left = pos.x, right = pos.x + CARD_WIDTH, top = pos.y, bottom = pos.y + CARD_HEIGHT;
  return points.slice(1).some((p, i) => {
    const q = points[i];
    return p.x === q.x ? p.x > left && p.x < right && Math.max(p.y, q.y) > top && Math.min(p.y, q.y) < bottom : p.y > top && p.y < bottom && Math.max(p.x, q.x) > left && Math.min(p.x, q.x) < right;
  });
}
function port(pos, side, offset = 0) {
  if (side === 'left') return { x: pos.x, y: pos.y + CARD_HEIGHT / 2 + offset };
  if (side === 'right') return { x: pos.x + CARD_WIDTH, y: pos.y + CARD_HEIGHT / 2 + offset };
  if (side === 'top') return { x: pos.x + CARD_WIDTH / 2 + offset, y: pos.y };
  return { x: pos.x + CARD_WIDTH / 2 + offset, y: pos.y + CARD_HEIGHT };
}
function directPoints(start, end, vertical) {
  if (vertical) { const mid = (start.y + end.y) / 2; return { points: [start, { x: start.x, y: mid }, { x: end.x, y: mid }, end], label: { x: (start.x + end.x) / 2, y: mid } }; }
  const mid = (start.x + end.x) / 2; return { points: [start, { x: mid, y: start.y }, { x: mid, y: end.y }, end], label: { x: mid, y: (start.y + end.y) / 2 } };
}
function clearPath(points, cards) { return [...cards.values()].every(card => !crossesCard(points, card)); }
function orderedCoordinates(values, target) { return [...new Set(values)].sort((a, b) => Math.abs(a - target) - Math.abs(b - target)); }
function corridorCoordinates(cards, start, end) {
  const xs = [start.x, end.x], ys = [start.y, end.y];
  for (const card of cards.values()) { xs.push(card.x, card.x + CARD_WIDTH); ys.push(card.y, card.y + CARD_HEIGHT); }
  const expand = values => {
    const sorted = [...new Set(values)].sort((a, b) => a - b);
    return [...sorted, ...sorted.flatMap(value => [value - 8, value + 8]), ...sorted.slice(1).map((value, i) => (value + sorted[i]) / 2)];
  };
  return { xs: expand(xs), ys: expand(ys) };
}
function portStub(point, side, cards) {
  const horizontal = side === 'left' || side === 'right', positive = side === 'right' || side === 'bottom';
  let distance = 8;
  for (const card of cards.values()) {
    const cross = horizontal ? point.y > card.y && point.y < card.y + CARD_HEIGHT : point.x > card.x && point.x < card.x + CARD_WIDTH;
    const boundary = horizontal ? positive ? card.x : card.x + CARD_WIDTH : positive ? card.y : card.y + CARD_HEIGHT;
    const gap = (boundary - (horizontal ? point.x : point.y)) * (positive ? 1 : -1);
    if (cross && gap >= 0) distance = Math.min(distance, gap / 2);
  }
  return horizontal ? { x: point.x + (positive ? distance : -distance), y: point.y } : { x: point.x, y: point.y + (positive ? distance : -distance) };
}
function corridorExit(point, side, desiredX, lane, coordinates, cards, avoidX) {
  const stub = portStub(point, side, cards), xs = orderedCoordinates(coordinates.xs, desiredX);
  for (const x of xs) {
    if (x === avoidX) continue;
    const points = [point, stub, { x, y: stub.y }, { x, y: lane }]; if (clearPath(points, cards)) return points;
  }
  for (const y of orderedCoordinates(coordinates.ys, stub.y)) {
    if ((y - stub.y) * (lane - stub.y) < 0 || Math.abs(y - stub.y) >= Math.abs(lane - stub.y)) continue;
    for (const outX of orderedCoordinates(coordinates.xs, stub.x)) {
      const prefix = [point, stub, { x: outX, y: stub.y }, { x: outX, y }]; if (!clearPath(prefix, cards)) continue;
      for (const x of xs) { if (x === avoidX) continue; const points = [...prefix, { x, y }, { x, y: lane }]; if (clearPath(points, cards)) return points; }
    }
  }
  return undefined;
}
function withoutExtraBends(points) {
  const result = [];
  for (const point of points) {
    if (result.at(-1)?.x === point.x && result.at(-1)?.y === point.y) continue;
    while (result.length >= 2) {
      const a = result.at(-2), b = result.at(-1), between = b.x >= Math.min(a.x, point.x) && b.x <= Math.max(a.x, point.x) && b.y >= Math.min(a.y, point.y) && b.y <= Math.max(a.y, point.y);
      if (between && ((a.x === b.x && b.x === point.x) || (a.y === b.y && b.y === point.y))) result.pop(); else break;
    }
    result.push(point);
  }
  return result;
}
function clearOutsidePath(plan, start, end, preferredLane, top, bottom, cards) {
  const coordinates = corridorCoordinates(cards, start, end), gap = 20 + (plan.index % 5) * 7;
  const desiredOut = start.x + (plan.forward ? gap : -gap), desiredIn = end.x + (plan.forward ? -gap : gap);
  const otherLane = preferredLane < top ? bottom + 45 + plan.index * 30 : top - 45 - plan.index * 30;
  for (const lane of [preferredLane, otherLane]) {
    const exit = corridorExit(start, plan.fromSide, desiredOut, lane, coordinates, cards);
    const entry = exit && corridorExit(end, plan.toSide, desiredIn, lane, coordinates, cards, exit.at(-1).x);
    if (!exit || !entry) continue;
    const points = withoutExtraBends([...exit, ...entry.slice().reverse()]);
    if (clearPath(points, cards)) return { points, label: { x: (exit.at(-1).x + entry.at(-1).x) / 2, y: lane } };
  }
  throw new Error('当前卡片位置没有可用的外侧走廊；请分开相互遮挡的卡片后重试。');
}
export function routeEdges(flow, positions, shown = flow.edges ?? []) {
  const nodes = flow.nodes ?? [], cards = new Map(nodes.filter(n => positions.has(n.id)).map(n => [n.id, positions.get(n.id)]));
  const skeleton = new Set(mainSkeleton(flow).edges), top = Math.min(0, ...[...cards.values()].map(p => p.y)), bottom = Math.max(CARD_HEIGHT, ...[...cards.values()].map(p => p.y + CARD_HEIGHT));
  const plans = [], incident = new Map(), sideKey = (id, side) => JSON.stringify([id, side]);
  for (const edge of shown) {
    const a = cards.get(edge.from), b = cards.get(edge.to); if (!a || !b) continue;
    const index = (flow.edges ?? []).indexOf(edge), dx = Math.abs(b.x - a.x), dy = Math.abs(b.y - a.y);
    const vertical = dx < CARD_WIDTH && dy >= CARD_HEIGHT ? true : dy < CARD_HEIGHT && dx >= CARD_WIDTH ? false : dy > dx, forward = b.x >= a.x, down = b.y >= a.y;
    let fromSide = vertical ? down ? 'bottom' : 'top' : forward ? 'right' : 'left';
    let toSide = vertical ? down ? 'top' : 'bottom' : forward ? 'left' : 'right';
    const direct = directPoints(port(a, fromSide), port(b, toSide), vertical);
    const outside = !skeleton.has(edge) || [...cards].some(([id, pos]) => id !== edge.from && id !== edge.to && crossesCard(direct.points, pos));
    if (outside) { fromSide = forward ? 'right' : 'left'; toSide = forward ? 'left' : 'right'; }
    const plan = { edge, index, a, b, vertical, forward, outside, fromSide, toSide }; plans.push(plan);
    for (const [id, side, role] of [[edge.from, fromSide, 'from'], [edge.to, toSide, 'to']]) {
      const key = sideKey(id, side); if (!incident.has(key)) incident.set(key, []); incident.get(key).push({ plan, role });
    }
  }
  const offset = (plan, role) => {
    const id = role === 'from' ? plan.edge.from : plan.edge.to, side = role === 'from' ? plan.fromSide : plan.toSide;
    const items = incident.get(sideKey(id, side)), at = items.findIndex(item => item.plan === plan && item.role === role);
    return (at - (items.length - 1) / 2) * Math.min(9, 80 / Math.max(1, items.length - 1));
  };
  let upper = 0, lower = 0;
  return plans.map(plan => {
    const { edge, index, a, b, vertical, forward, outside } = plan;
    const start = port(a, plan.fromSide, offset(plan, 'from')), end = port(b, plan.toSide, offset(plan, 'to'));
    let { points, label } = directPoints(start, end, vertical);
    if (outside || !clearPath(points, cards)) {
      const lane = edge.kind === 'back' || edge.kind === 'main' ? top - 45 - 30 * upper++ : bottom + 45 + 30 * lower++;
      const gap = 20 + (index % 5) * 7, exitX = start.x + (forward ? gap : -gap), enterX = end.x + (forward ? -gap : gap);
      points = [start, { x: exitX, y: start.y }, { x: exitX, y: lane }, { x: enterX, y: lane }, { x: enterX, y: end.y }, end]; label = { x: (exitX + enterX) / 2, y: lane };
      if (!clearPath(points, cards)) ({ points, label } = clearOutsidePath(plan, start, end, lane, top, bottom, cards));
    }
    const labelWidth = Math.max(70, `${index + 1} ${edgeKindLabel(edge.kind)}`.length * 13 + 16);
    return { edge, index, start, end, points, label, labelWidth, d: points.map((p, i) => `${i ? 'L' : 'M'}${p.x},${p.y}`).join(' ') };
  });
}
export function graphBounds(flow, positions, routes = routeEdges(flow, positions), labelEdge) {
  const points = (flow.nodes ?? []).filter(n => positions.has(n.id)).flatMap(n => { const p = positions.get(n.id); return [p, { x: p.x + CARD_WIDTH, y: p.y + CARD_HEIGHT }]; });
  for (const route of routes) { points.push(...route.points); if (route.edge === labelEdge) points.push({ x: route.label.x - route.labelWidth / 2, y: route.label.y - 15 }, { x: route.label.x + route.labelWidth / 2, y: route.label.y + 15 }); }
  return { minX: Math.min(0, ...points.map(p => p.x)), minY: Math.min(0, ...points.map(p => p.y)), maxX: Math.max(CARD_WIDTH, ...points.map(p => p.x)), maxY: Math.max(CARD_HEIGHT, ...points.map(p => p.y)) };
}

function segmentDistance(point, a, b) {
  const dx = b.x - a.x, dy = b.y - a.y, length = dx * dx + dy * dy;
  const fraction = length ? Math.max(0, Math.min(1, ((point.x - a.x) * dx + (point.y - a.y) * dy) / length)) : 0;
  return Math.hypot(point.x - a.x - fraction * dx, point.y - a.y - fraction * dy);
}
export function pickRouteAtPoint(routes, screenPoint, transform) {
  const point = { x: (screenPoint.x - transform.x) / transform.scale, y: (screenPoint.y - transform.y) / transform.scale };
  const distances = routes.map(route => ({ route, distance: Math.min(...route.points.slice(1).map((p, i) => segmentDistance(point, route.points[i], p))) * transform.scale }));
  const nearest = Math.min(...distances.map(item => item.distance));
  const closest = distances.filter(item => Math.abs(item.distance - nearest) <= 1e-6).map(item => item.route);
  return { route: closest.length === 1 ? closest[0] : undefined, ambiguous: closest.length > 1 ? closest : [] };
}

export function renderedCanvasTransform(worldRect, worldWidth) {
  return { x: worldRect.left, y: worldRect.top, scale: worldRect.width / worldWidth };
}
