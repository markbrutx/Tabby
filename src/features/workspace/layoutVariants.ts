import type { SplitNode } from "@/features/workspace/domain/models";
import {
  leaf,
  hsplit,
  vsplit,
  hsplit3,
  vsplit3,
  treeFromCount,
} from "@/features/workspace/layoutReadModel";

export interface LayoutVariant {
  readonly id: string;
  readonly label: string;
  readonly build: (ids: string[]) => SplitNode;
}

function cols3(ids: string[]): SplitNode {
  return hsplit3(ids[0], ids[1], ids[2]);
}

function rows3(ids: string[]): SplitNode {
  return vsplit3(ids[0], ids[1], ids[2]);
}

function topBottom(topCount: number, ids: string[]): SplitNode {
  const topIds = ids.slice(0, topCount);
  const bottomIds = ids.slice(topCount);
  const top = topIds.length === 1
    ? leaf(topIds[0])
    : topIds.length === 2
      ? hsplit(topIds[0], topIds[1])
      : hsplit3(topIds[0], topIds[1], topIds[2]);
  const bottom = bottomIds.length === 1
    ? leaf(bottomIds[0])
    : bottomIds.length === 2
      ? hsplit(bottomIds[0], bottomIds[1])
      : hsplit3(bottomIds[0], bottomIds[1], bottomIds[2]);
  return { type: "split", direction: "vertical", ratio: 500, first: top, second: bottom };
}

function leftRight(leftCount: number, ids: string[]): SplitNode {
  const leftIds = ids.slice(0, leftCount);
  const rightIds = ids.slice(leftCount);
  const left = leftIds.length === 1
    ? leaf(leftIds[0])
    : { type: "split" as const, direction: "vertical" as const, ratio: 500, first: leaf(leftIds[0]), second: leaf(leftIds[1]) };
  const right = rightIds.length === 1
    ? leaf(rightIds[0])
    : rightIds.length === 2
      ? { type: "split" as const, direction: "vertical" as const, ratio: 500, first: leaf(rightIds[0]), second: leaf(rightIds[1]) }
      : vsplit3(rightIds[0], rightIds[1], rightIds[2]);
  return { type: "split", direction: "horizontal", ratio: leftCount === 1 ? 333 : 500, first: left, second: right };
}

const VARIANTS: Record<number, LayoutVariant[]> = {
  2: [
    { id: "2-cols", label: "Side by side", build: (ids) => hsplit(ids[0], ids[1]) },
    { id: "2-rows", label: "Top / bottom", build: (ids) => vsplit(ids[0], ids[1]) },
  ],
  3: [
    { id: "3-auto", label: "Auto", build: (ids) => treeFromCount(ids) },
    { id: "3-cols", label: "3 columns", build: (ids) => cols3(ids) },
    { id: "3-rows", label: "3 rows", build: (ids) => rows3(ids) },
    { id: "3-1t2b", label: "1 + 2", build: (ids) => topBottom(1, ids) },
    { id: "3-2t1b", label: "2 + 1", build: (ids) => topBottom(2, ids) },
  ],
  4: [
    { id: "4-grid", label: "2\u00d72 grid", build: (ids) => treeFromCount(ids) },
    { id: "4-cols", label: "4 columns", build: (ids) => ({
      type: "split", direction: "horizontal", ratio: 500,
      first: hsplit(ids[0], ids[1]),
      second: hsplit(ids[2], ids[3]),
    }) },
    { id: "4-rows", label: "4 rows", build: (ids) => ({
      type: "split", direction: "vertical", ratio: 500,
      first: vsplit(ids[0], ids[1]),
      second: vsplit(ids[2], ids[3]),
    }) },
    { id: "4-1l3r", label: "1 + 3", build: (ids) => leftRight(1, ids) },
    { id: "4-1t3b", label: "1 + 3 rows", build: (ids) => topBottom(1, ids) },
  ],
  5: [
    { id: "5-auto", label: "3 + 2", build: (ids) => treeFromCount(ids) },
    { id: "5-2t3b", label: "2 + 3", build: (ids) => topBottom(2, ids) },
    { id: "5-1l4r", label: "1 + 4", build: (ids) => leftRight(1, ids) },
  ],
  6: [
    { id: "6-auto", label: "3 + 3", build: (ids) => treeFromCount(ids) },
    { id: "6-2x3", label: "2\u00d73 rows", build: (ids) => ({
      type: "split", direction: "vertical", ratio: 333,
      first: hsplit(ids[0], ids[1]),
      second: { type: "split", direction: "vertical", ratio: 500,
        first: hsplit(ids[2], ids[3]),
        second: hsplit(ids[4], ids[5]),
      },
    }) },
  ],
  7: [
    { id: "7-auto", label: "Auto", build: (ids) => treeFromCount(ids) },
    { id: "7-3t4b", label: "3 + 4", build: (ids) => topBottom(3, ids) },
  ],
  8: [
    { id: "8-auto", label: "Auto", build: (ids) => treeFromCount(ids) },
    { id: "8-3t5b", label: "3 + 5", build: (ids) => ({
      type: "split", direction: "vertical", ratio: 500,
      first: hsplit3(ids[0], ids[1], ids[2]),
      second: {
        type: "split", direction: "horizontal", ratio: 500,
        first: hsplit3(ids[3], ids[4], ids[5]),
        second: hsplit(ids[6], ids[7]),
      },
    }) },
  ],
  9: [
    { id: "9-auto", label: "3\u00d73 grid", build: (ids) => treeFromCount(ids) },
  ],
};

export function getLayoutVariants(paneCount: number): LayoutVariant[] {
  if (paneCount <= 1) return [];
  return VARIANTS[paneCount] ?? [
    { id: `${paneCount}-auto`, label: "Auto", build: (ids) => treeFromCount(ids) },
  ];
}
