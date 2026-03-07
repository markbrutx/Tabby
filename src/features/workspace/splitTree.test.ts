import { describe, expect, it } from "vitest";
import type { SplitNode } from "@/features/workspace/domain";
import {
  splitPane,
  closePane,
  collectPaneIds,
  treeFromCount,
  computePaneRects,
  findAdjacentPane,
  findNextPane,
  findPreviousPane,
} from "@/features/workspace/splitTree";

const singlePane: SplitNode = { type: "pane", paneId: "p1" };

const twoPane: SplitNode = {
  type: "split",
  direction: "horizontal",
  ratio: 500,
  first: { type: "pane", paneId: "p1" },
  second: { type: "pane", paneId: "p2" },
};

describe("splitTree", () => {
  describe("collectPaneIds", () => {
    it("returns single pane id", () => {
      expect(collectPaneIds(singlePane)).toEqual(["p1"]);
    });

    it("returns all pane ids from a split", () => {
      expect(collectPaneIds(twoPane)).toEqual(["p1", "p2"]);
    });
  });

  describe("splitPane", () => {
    it("replaces a leaf with a branch", () => {
      const result = splitPane(singlePane, "p1", "horizontal", "p2");
      expect(result).not.toBeNull();
      expect(collectPaneIds(result!)).toEqual(["p1", "p2"]);
    });

    it("returns null for unknown pane", () => {
      expect(splitPane(singlePane, "unknown", "horizontal", "p2")).toBeNull();
    });

    it("splits a nested pane correctly", () => {
      const result = splitPane(twoPane, "p2", "vertical", "p3");
      expect(result).not.toBeNull();
      expect(collectPaneIds(result!)).toEqual(["p1", "p2", "p3"]);
    });
  });

  describe("closePane", () => {
    it("returns null for last pane (tree empty)", () => {
      expect(closePane(singlePane, "p1")).toBeNull();
    });

    it("returns undefined for unknown pane", () => {
      expect(closePane(singlePane, "unknown")).toBeUndefined();
    });

    it("collapses parent when closing one of two panes", () => {
      const result = closePane(twoPane, "p1");
      expect(result).not.toBeNull();
      expect(result).not.toBeUndefined();
      expect(collectPaneIds(result!)).toEqual(["p2"]);
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
});
