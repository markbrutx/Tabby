import type { SplitNode } from "@/features/workspace/domain/models";

function leaf(id: string): SplitNode {
  return { type: "pane", paneId: id };
}

function hsplit(a: string, b: string): SplitNode {
  return { type: "split", direction: "horizontal", ratio: 500, first: leaf(a), second: leaf(b) };
}

function hsplit3(a: string, b: string, c: string): SplitNode {
  return {
    type: "split", direction: "horizontal", ratio: 333,
    first: leaf(a),
    second: { type: "split", direction: "horizontal", ratio: 500, first: leaf(b), second: leaf(c) },
  };
}

function hsplit4(a: string, b: string, c: string, d: string): SplitNode {
  return {
    type: "split", direction: "horizontal", ratio: 500,
    first: hsplit(a, b),
    second: hsplit(c, d),
  };
}

export function treeFromCount(paneIds: string[]): SplitNode {
  const n = paneIds.length;
  if (n < 1 || n > 9) {
    throw new Error(`treeFromCount supports 1–9 panes, got ${n}`);
  }

  switch (n) {
    case 1: return leaf(paneIds[0]);
    case 2: return hsplit(paneIds[0], paneIds[1]);
    case 3: return {
      type: "split", direction: "horizontal", ratio: 333,
      first: leaf(paneIds[0]),
      second: hsplit(paneIds[1], paneIds[2]),
    };
    case 4: return {
      type: "split", direction: "vertical", ratio: 500,
      first: hsplit(paneIds[0], paneIds[1]),
      second: hsplit(paneIds[2], paneIds[3]),
    };
    case 5: return {
      type: "split", direction: "vertical", ratio: 500,
      first: hsplit3(paneIds[0], paneIds[1], paneIds[2]),
      second: hsplit(paneIds[3], paneIds[4]),
    };
    case 6: return {
      type: "split", direction: "vertical", ratio: 500,
      first: hsplit3(paneIds[0], paneIds[1], paneIds[2]),
      second: hsplit3(paneIds[3], paneIds[4], paneIds[5]),
    };
    case 7: return {
      type: "split", direction: "vertical", ratio: 500,
      first: hsplit4(paneIds[0], paneIds[1], paneIds[2], paneIds[3]),
      second: hsplit3(paneIds[4], paneIds[5], paneIds[6]),
    };
    case 8: return {
      type: "split", direction: "vertical", ratio: 500,
      first: hsplit4(paneIds[0], paneIds[1], paneIds[2], paneIds[3]),
      second: hsplit4(paneIds[4], paneIds[5], paneIds[6], paneIds[7]),
    };
    case 9: return {
      type: "split", direction: "vertical", ratio: 333,
      first: hsplit3(paneIds[0], paneIds[1], paneIds[2]),
      second: {
        type: "split", direction: "vertical", ratio: 500,
        first: hsplit3(paneIds[3], paneIds[4], paneIds[5]),
        second: hsplit3(paneIds[6], paneIds[7], paneIds[8]),
      },
    };
    default: throw new Error(`treeFromCount supports 1–9 panes, got ${n}`);
  }
}

export function collectPaneIds(root: SplitNode): string[] {
  if (root.type === "pane") {
    return [root.paneId];
  }
  return [...collectPaneIds(root.first), ...collectPaneIds(root.second)];
}

interface PaneRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export function computePaneRects(root: SplitNode): Map<string, PaneRect> {
  const rects = new Map<string, PaneRect>();
  buildRects(root, { x: 0, y: 0, w: 1, h: 1 }, rects);
  return rects;
}

type NavigationDirection = "up" | "down" | "left" | "right";

type Rect = PaneRect;

function buildRects(
  node: SplitNode,
  rect: Rect,
  out: Map<string, Rect>,
): void {
  if (node.type === "pane") {
    out.set(node.paneId, rect);
    return;
  }

  const ratio = node.ratio / 1000;

  if (node.direction === "horizontal") {
    const w1 = rect.w * ratio;
    buildRects(node.first, { ...rect, w: w1 }, out);
    buildRects(node.second, { x: rect.x + w1, y: rect.y, w: rect.w - w1, h: rect.h }, out);
  } else {
    const h1 = rect.h * ratio;
    buildRects(node.first, { ...rect, h: h1 }, out);
    buildRects(node.second, { x: rect.x, y: rect.y + h1, w: rect.w, h: rect.h - h1 }, out);
  }
}

export function findAdjacentPane(
  root: SplitNode,
  currentPaneId: string,
  direction: NavigationDirection,
): string | null {
  const rects = new Map<string, Rect>();
  buildRects(root, { x: 0, y: 0, w: 1, h: 1 }, rects);

  const current = rects.get(currentPaneId);
  if (!current) return null;

  const cx = current.x + current.w / 2;
  const cy = current.y + current.h / 2;

  let bestId: string | null = null;
  let bestDist = Infinity;

  for (const [id, rect] of rects) {
    if (id === currentPaneId) continue;

    const ox = rect.x + rect.w / 2;
    const oy = rect.y + rect.h / 2;

    const isCandidate =
      (direction === "left" && ox < cx) ||
      (direction === "right" && ox > cx) ||
      (direction === "up" && oy < cy) ||
      (direction === "down" && oy > cy);

    if (!isCandidate) continue;

    const dist = Math.abs(ox - cx) + Math.abs(oy - cy);
    if (dist < bestDist) {
      bestDist = dist;
      bestId = id;
    }
  }

  return bestId;
}

export function findNextPane(root: SplitNode, currentPaneId: string): string | null {
  const ids = collectPaneIds(root);
  const index = ids.indexOf(currentPaneId);
  if (index === -1) return ids[0] ?? null;
  return ids[(index + 1) % ids.length] ?? null;
}

export function findPreviousPane(root: SplitNode, currentPaneId: string): string | null {
  const ids = collectPaneIds(root);
  const index = ids.indexOf(currentPaneId);
  if (index === -1) return ids[0] ?? null;
  return ids[(index - 1 + ids.length) % ids.length] ?? null;
}
