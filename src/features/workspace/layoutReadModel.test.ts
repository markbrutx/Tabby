import { describe, expect, it } from "vitest";
import type { SplitNode } from "@/features/workspace/domain/models";
import {
  collectPaneIds,
  treeFromCount,
  computePaneRects,
  findAdjacentPane,
  findNextPane,
  findPreviousPane,
} from "@/features/workspace/layoutReadModel";

const singlePane: SplitNode = { type: "pane", paneId: "p1" };

const twoPane: SplitNode = {
  type: "split",
  direction: "horizontal",
  ratio: 500,
  first: { type: "pane", paneId: "p1" },
  second: { type: "pane", paneId: "p2" },
};

describe("layoutReadModel", () => {
  describe("collectPaneIds", () => {
    it("returns single pane id", () => {
      expect(collectPaneIds(singlePane)).toEqual(["p1"]);
    });

    it("returns all pane ids from a split", () => {
      expect(collectPaneIds(twoPane)).toEqual(["p1", "p2"]);
    });
  });

  describe("treeFromCount", () => {
    function ids(n: number): string[] {
      return Array.from({ length: n }, (_, i) => `p${i + 1}`);
    }

    it.each([1, 2, 3, 4, 5, 6, 7, 8, 9])("creates correct tree for count=%i", (count) => {
      const paneIds = ids(count);
      const tree = treeFromCount(paneIds);
      const collected = collectPaneIds(tree);
      expect(collected).toHaveLength(count);
      expect(collected).toEqual(paneIds);
    });

    it("throws for count 0", () => {
      expect(() => treeFromCount([])).toThrow();
    });

    it("throws for count > 9", () => {
      expect(() => treeFromCount(ids(10))).toThrow();
    });
  });

  describe("computePaneRects", () => {
    it("returns single rect for 1 pane", () => {
      const tree = treeFromCount(["p1"]);
      const rects = computePaneRects(tree);
      expect(rects.size).toBe(1);
      expect(rects.get("p1")).toEqual({ x: 0, y: 0, w: 1, h: 1 });
    });

    it("returns two rects for 2 panes", () => {
      const tree = treeFromCount(["p1", "p2"]);
      const rects = computePaneRects(tree);
      expect(rects.size).toBe(2);
      expect(rects.get("p1")).toEqual({ x: 0, y: 0, w: 0.5, h: 1 });
      expect(rects.get("p2")).toEqual({ x: 0.5, y: 0, w: 0.5, h: 1 });
    });

    it("returns four quadrant rects for 4 panes", () => {
      const tree = treeFromCount(["p1", "p2", "p3", "p4"]);
      const rects = computePaneRects(tree);
      expect(rects.size).toBe(4);
      expect(rects.get("p1")).toEqual({ x: 0, y: 0, w: 0.5, h: 0.5 });
      expect(rects.get("p2")).toEqual({ x: 0.5, y: 0, w: 0.5, h: 0.5 });
      expect(rects.get("p3")).toEqual({ x: 0, y: 0.5, w: 0.5, h: 0.5 });
      expect(rects.get("p4")).toEqual({ x: 0.5, y: 0.5, w: 0.5, h: 0.5 });
    });
  });

  describe("findNextPane / findPreviousPane", () => {
    it("cycles forward in DFS order", () => {
      expect(findNextPane(twoPane, "p1")).toBe("p2");
      expect(findNextPane(twoPane, "p2")).toBe("p1");
    });

    it("cycles backward in DFS order", () => {
      expect(findPreviousPane(twoPane, "p2")).toBe("p1");
      expect(findPreviousPane(twoPane, "p1")).toBe("p2");
    });

    it("returns first pane for unknown current pane", () => {
      expect(findNextPane(twoPane, "unknown")).toBe("p1");
      expect(findPreviousPane(twoPane, "unknown")).toBe("p1");
    });
  });

  describe("findAdjacentPane", () => {
    it("finds pane to the right in horizontal split", () => {
      expect(findAdjacentPane(twoPane, "p1", "right")).toBe("p2");
    });

    it("finds pane to the left in horizontal split", () => {
      expect(findAdjacentPane(twoPane, "p2", "left")).toBe("p1");
    });

    it("returns null when no adjacent pane in direction", () => {
      expect(findAdjacentPane(twoPane, "p1", "left")).toBeNull();
    });

    it("returns null for unknown pane", () => {
      expect(findAdjacentPane(twoPane, "unknown", "right")).toBeNull();
    });

    it("navigates vertical splits", () => {
      const vertical: SplitNode = {
        type: "split",
        direction: "vertical",
        ratio: 500,
        first: { type: "pane", paneId: "top" },
        second: { type: "pane", paneId: "bottom" },
      };
      expect(findAdjacentPane(vertical, "top", "down")).toBe("bottom");
      expect(findAdjacentPane(vertical, "bottom", "up")).toBe("top");
    });
  });

  describe("no mutation functions exist", () => {
    it("module exports only read-only helpers", async () => {
      const mod = await import("@/features/workspace/layoutReadModel");
      const exportedKeys = Object.keys(mod);
      const mutationNames = ["splitPane", "closePane", "swapPaneSlots"];
      for (const name of mutationNames) {
        expect(exportedKeys).not.toContain(name);
      }
    });
  });
});
