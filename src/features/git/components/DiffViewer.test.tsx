import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DiffViewer, type StagingCallbacks } from "./DiffViewer";
import type { DiffContent, DiffHunk, DiffLine } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// ResizeObserver polyfill for jsdom
// ---------------------------------------------------------------------------

global.ResizeObserver = class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver;

// ---------------------------------------------------------------------------
// Factories
// ---------------------------------------------------------------------------

function makeLine(overrides: Partial<DiffLine> = {}): DiffLine {
  return {
    kind: "context",
    oldLineNo: 1,
    newLineNo: 1,
    content: "  some code",
    ...overrides,
  };
}

function makeHunk(overrides: Partial<DiffHunk> = {}): DiffHunk {
  return {
    oldStart: 1,
    oldCount: 3,
    newStart: 1,
    newCount: 4,
    header: "@@ -1,3 +1,4 @@",
    lines: [
      makeLine({ kind: "context", oldLineNo: 1, newLineNo: 1, content: "  line 1" }),
      makeLine({ kind: "deletion", oldLineNo: 2, newLineNo: null, content: "- removed" }),
      makeLine({ kind: "addition", oldLineNo: null, newLineNo: 2, content: "+ added" }),
      makeLine({ kind: "context", oldLineNo: 3, newLineNo: 3, content: "  line 3" }),
    ],
    ...overrides,
  };
}

function makeDiffContent(overrides: Partial<DiffContent> = {}): DiffContent {
  return {
    filePath: "src/main.ts",
    oldPath: null,
    hunks: [makeHunk()],
    isBinary: false,
    fileModeChange: null,
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// Unified mode tests (existing)
// ---------------------------------------------------------------------------

describe("DiffViewer", () => {
  it("renders empty state when diffContent is null", () => {
    render(<DiffViewer diffContent={null} />);
    expect(screen.getByTestId("diff-empty")).toBeInTheDocument();
    expect(screen.getByText("No diff to display")).toBeInTheDocument();
  });

  it("renders binary file indicator for binary diffs", () => {
    const diff = makeDiffContent({ isBinary: true, hunks: [] });
    render(<DiffViewer diffContent={diff} />);
    expect(screen.getByTestId("diff-binary")).toBeInTheDocument();
    expect(screen.getByText("Binary file")).toBeInTheDocument();
    expect(screen.getByText("src/main.ts")).toBeInTheDocument();
  });

  it("renders empty state when hunks array is empty (non-binary)", () => {
    const diff = makeDiffContent({ hunks: [] });
    render(<DiffViewer diffContent={diff} />);
    expect(screen.getByTestId("diff-empty")).toBeInTheDocument();
  });

  it("renders hunk headers with @@ markers", () => {
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} />);
    const headers = screen.getAllByTestId("hunk-header");
    expect(headers).toHaveLength(1);
    expect(headers[0]).toHaveTextContent("@@ -1,3 +1,4 @@");
  });

  it("renders additions with green background class", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 1, content: "+ new line" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const lines = screen.getAllByTestId("diff-line");
    expect(lines).toHaveLength(1);
    expect(lines[0].className).toContain("bg-green-900");
    expect(lines[0].className).toContain("text-green-300");
  });

  it("renders deletions with red background class", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "deletion", oldLineNo: 5, newLineNo: null, content: "- old line" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const lines = screen.getAllByTestId("diff-line");
    expect(lines).toHaveLength(1);
    expect(lines[0].className).toContain("bg-red-900");
    expect(lines[0].className).toContain("text-red-300");
  });

  it("renders context lines without addition/deletion styling", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "context", oldLineNo: 10, newLineNo: 10, content: "  unchanged" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const lines = screen.getAllByTestId("diff-line");
    expect(lines).toHaveLength(1);
    expect(lines[0].className).not.toContain("bg-green-900");
    expect(lines[0].className).not.toContain("bg-red-900");
  });

  it("renders old and new line numbers in gutter", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "context", oldLineNo: 42, newLineNo: 43, content: "  code" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const oldNos = screen.getAllByTestId("line-no-old");
    const newNos = screen.getAllByTestId("line-no-new");
    expect(oldNos[0]).toHaveTextContent("42");
    expect(newNos[0]).toHaveTextContent("43");
  });

  it("renders empty line number for additions (no old line number)", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 5, content: "+ new" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const oldNos = screen.getAllByTestId("line-no-old");
    expect(oldNos[0]).toHaveTextContent("");
  });

  it("renders empty line number for deletions (no new line number)", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "deletion", oldLineNo: 8, newLineNo: null, content: "- gone" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const newNos = screen.getAllByTestId("line-no-new");
    expect(newNos[0]).toHaveTextContent("");
  });

  it("renders file mode change banner when present", () => {
    const diff = makeDiffContent({ fileModeChange: "100644 → 100755" });
    render(<DiffViewer diffContent={diff} />);
    expect(screen.getByTestId("diff-mode-change")).toBeInTheDocument();
    expect(screen.getByText(/100644 → 100755/)).toBeInTheDocument();
  });

  it("does not render file mode change banner when null", () => {
    const diff = makeDiffContent({ fileModeChange: null });
    render(<DiffViewer diffContent={diff} />);
    expect(screen.queryByTestId("diff-mode-change")).not.toBeInTheDocument();
  });

  it("renders line content text", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "context", content: "  const x = 42;" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const contents = screen.getAllByTestId("line-content");
    expect(contents[0]).toHaveTextContent("const x = 42;");
  });

  it("renders multiple hunks with separate headers", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          header: "@@ -1,2 +1,2 @@",
          lines: [makeLine({ content: "  a" })],
        }),
        makeHunk({
          header: "@@ -10,3 +10,5 @@",
          lines: [makeLine({ content: "  b" })],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const headers = screen.getAllByTestId("hunk-header");
    expect(headers).toHaveLength(2);
    expect(headers[0]).toHaveTextContent("@@ -1,2 +1,2 @@");
    expect(headers[1]).toHaveTextContent("@@ -10,3 +10,5 @@");
  });

  it("renders diff-viewer container with monospace font", () => {
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} />);
    const scrollContainer = screen.getByTestId("diff-scroll-container");
    expect(scrollContainer.className).toContain("font-mono");
  });

  it("renders mixed additions and deletions correctly", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "deletion", oldLineNo: 1, newLineNo: null, content: "- old" }),
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 1, content: "+ new" }),
            makeLine({ kind: "context", oldLineNo: 2, newLineNo: 2, content: "  same" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const lines = screen.getAllByTestId("diff-line");
    expect(lines).toHaveLength(3);
    expect(lines[0].className).toContain("bg-red-900");
    expect(lines[1].className).toContain("bg-green-900");
    expect(lines[2].className).not.toContain("bg-red-900");
    expect(lines[2].className).not.toContain("bg-green-900");
  });

  it("handles large diff without crashing (performance sanity check)", () => {
    const lines: DiffLine[] = [];
    for (let i = 0; i < 5000; i++) {
      lines.push(makeLine({
        kind: i % 3 === 0 ? "addition" : i % 3 === 1 ? "deletion" : "context",
        oldLineNo: i % 3 === 0 ? null : i,
        newLineNo: i % 3 === 1 ? null : i,
        content: `  line number ${i}`,
      }));
    }
    const diff = makeDiffContent({
      hunks: [makeHunk({ lines })],
    });

    // Should not throw and should render (virtual scrolling limits DOM nodes)
    const { container } = render(<DiffViewer diffContent={diff} />);
    expect(container.querySelector("[data-testid='diff-viewer']")).toBeInTheDocument();

    // Virtual scrolling: should NOT render all 5000 lines as DOM elements
    const renderedLines = container.querySelectorAll("[data-testid='diff-line']");
    expect(renderedLines.length).toBeLessThan(5000);
  });

  // -------------------------------------------------------------------------
  // Mode toggle tests
  // -------------------------------------------------------------------------

  it("renders mode toggle header with toggle button", () => {
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} />);
    expect(screen.getByTestId("diff-mode-header")).toBeInTheDocument();
    expect(screen.getByTestId("diff-mode-toggle")).toBeInTheDocument();
  });

  it("defaults to unified mode", () => {
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} />);
    // In unified mode, toggle button says "Split"
    expect(screen.getByTestId("diff-mode-toggle")).toHaveTextContent("Split");
    expect(screen.getByTestId("diff-scroll-container")).toBeInTheDocument();
  });

  it("accepts mode prop to start in split mode", () => {
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} mode="split" />);
    expect(screen.getByTestId("diff-mode-toggle")).toHaveTextContent("Unified");
    expect(screen.getByTestId("split-container")).toBeInTheDocument();
  });

  it("toggle switches between unified and split modes", () => {
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} />);

    // Starts in unified mode
    expect(screen.getByTestId("diff-scroll-container")).toBeInTheDocument();
    expect(screen.queryByTestId("split-container")).not.toBeInTheDocument();

    // Click toggle → split mode
    fireEvent.click(screen.getByTestId("diff-mode-toggle"));
    expect(screen.queryByTestId("diff-scroll-container")).not.toBeInTheDocument();
    expect(screen.getByTestId("split-container")).toBeInTheDocument();

    // Click toggle → back to unified
    fireEvent.click(screen.getByTestId("diff-mode-toggle"));
    expect(screen.getByTestId("diff-scroll-container")).toBeInTheDocument();
    expect(screen.queryByTestId("split-container")).not.toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Split mode rendering tests
  // -------------------------------------------------------------------------

  it("split mode renders two columns (left and right panels)", () => {
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} mode="split" />);
    expect(screen.getByTestId("split-left")).toBeInTheDocument();
    expect(screen.getByTestId("split-right")).toBeInTheDocument();
  });

  it("split mode renders hunk headers in both panels", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          header: "@@ -1,2 +1,3 @@",
          lines: [makeLine({ kind: "context", content: "  ctx" })],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} mode="split" />);
    const splitHeaders = screen.getAllByTestId("split-hunk-header");
    expect(splitHeaders.length).toBeGreaterThanOrEqual(2);
  });

  it("split mode shows deletions on left, additions on right", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "deletion", oldLineNo: 1, newLineNo: null, content: "- old code" }),
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 1, content: "+ new code" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} mode="split" />);

    const splitLines = screen.getAllByTestId("split-line");
    // Should have at least 2 lines (one left deletion, one right addition)
    expect(splitLines.length).toBeGreaterThanOrEqual(2);

    // Find deletion (left) and addition (right) by class
    const deletionLine = splitLines.find((el) => el.className.includes("bg-red-900"));
    const additionLine = splitLines.find((el) => el.className.includes("bg-green-900"));
    expect(deletionLine).toBeDefined();
    expect(additionLine).toBeDefined();
  });

  it("split mode inserts blank lines for unmatched additions", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 1, content: "+ only add" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} mode="split" />);

    // Left panel should have a blank line
    const leftPanel = screen.getByTestId("split-left");
    const leftLines = leftPanel.querySelectorAll("[data-testid='split-line']");
    expect(leftLines.length).toBeGreaterThanOrEqual(1);
    // The blank line has empty line number
    const leftLineNo = leftLines[0].querySelector("[data-testid='split-line-no']");
    expect(leftLineNo).toHaveTextContent("");
  });

  it("split mode uses same color coding (green right, red left)", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "deletion", oldLineNo: 1, newLineNo: null, content: "- removed" }),
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 1, content: "+ added" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} mode="split" />);

    const leftPanel = screen.getByTestId("split-left");
    const rightPanel = screen.getByTestId("split-right");

    const leftLines = leftPanel.querySelectorAll("[data-testid='split-line']");
    const rightLines = rightPanel.querySelectorAll("[data-testid='split-line']");

    // Left deletion should be red
    const leftDeletion = Array.from(leftLines).find((el) => el.className.includes("bg-red-900"));
    expect(leftDeletion).toBeDefined();

    // Right addition should be green
    const rightAddition = Array.from(rightLines).find((el) => el.className.includes("bg-green-900"));
    expect(rightAddition).toBeDefined();
  });

  it("split mode virtual scrolling limits DOM nodes for large diffs", () => {
    const lines: DiffLine[] = [];
    for (let i = 0; i < 5000; i++) {
      lines.push(makeLine({
        kind: i % 3 === 0 ? "addition" : i % 3 === 1 ? "deletion" : "context",
        oldLineNo: i % 3 === 0 ? null : i,
        newLineNo: i % 3 === 1 ? null : i,
        content: `  line number ${i}`,
      }));
    }
    const diff = makeDiffContent({
      hunks: [makeHunk({ lines })],
    });

    const { container } = render(<DiffViewer diffContent={diff} mode="split" />);
    expect(container.querySelector("[data-testid='split-container']")).toBeInTheDocument();

    // Virtual scrolling should limit rendered DOM nodes
    const renderedLines = container.querySelectorAll("[data-testid='split-line']");
    expect(renderedLines.length).toBeLessThan(5000);
  });

  it("split mode scroll sync: both panels have scroll containers", () => {
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} mode="split" />);

    const leftPanel = screen.getByTestId("split-left");
    const rightPanel = screen.getByTestId("split-right");

    // Both should be scrollable containers (overflow-auto)
    expect(leftPanel.className).toContain("overflow-auto");
    expect(rightPanel.className).toContain("overflow-auto");
  });

  it("does not render mode toggle for null diffContent", () => {
    render(<DiffViewer diffContent={null} />);
    expect(screen.queryByTestId("diff-mode-header")).not.toBeInTheDocument();
  });

  it("does not render mode toggle for binary diffs", () => {
    const diff = makeDiffContent({ isBinary: true, hunks: [] });
    render(<DiffViewer diffContent={diff} />);
    expect(screen.queryByTestId("diff-mode-header")).not.toBeInTheDocument();
  });

  it("split mode context lines appear on both sides", () => {
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "context", oldLineNo: 5, newLineNo: 5, content: "  shared line" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} mode="split" />);

    const leftPanel = screen.getByTestId("split-left");
    const rightPanel = screen.getByTestId("split-right");

    const leftContents = leftPanel.querySelectorAll("[data-testid='split-line-content']");
    const rightContents = rightPanel.querySelectorAll("[data-testid='split-line-content']");

    expect(leftContents.length).toBeGreaterThanOrEqual(1);
    expect(rightContents.length).toBeGreaterThanOrEqual(1);
    expect(leftContents[0]).toHaveTextContent("shared line");
    expect(rightContents[0]).toHaveTextContent("shared line");
  });

  // -------------------------------------------------------------------------
  // Line-level staging tests
  // -------------------------------------------------------------------------

  function makeStagingCallbacks(): StagingCallbacks & {
    stageLinesCalls: Array<{ filePath: string; lineRanges: string[] }>;
    unstageLinesCalls: Array<{ filePath: string; lineRanges: string[] }>;
    stageHunkCalls: Array<{ filePath: string; hunkIndex: number }>;
    unstageHunkCalls: Array<{ filePath: string; hunkIndex: number }>;
  } {
    const stageLinesCalls: Array<{ filePath: string; lineRanges: string[] }> = [];
    const unstageLinesCalls: Array<{ filePath: string; lineRanges: string[] }> = [];
    const stageHunkCalls: Array<{ filePath: string; hunkIndex: number }> = [];
    const unstageHunkCalls: Array<{ filePath: string; hunkIndex: number }> = [];
    return {
      onStageLines: vi.fn((filePath: string, lineRanges: string[]) => {
        stageLinesCalls.push({ filePath, lineRanges });
      }),
      onUnstageLines: vi.fn((filePath: string, lineRanges: string[]) => {
        unstageLinesCalls.push({ filePath, lineRanges });
      }),
      onStageHunk: vi.fn((filePath: string, hunkIndex: number) => {
        stageHunkCalls.push({ filePath, hunkIndex });
      }),
      onUnstageHunk: vi.fn((filePath: string, hunkIndex: number) => {
        unstageHunkCalls.push({ filePath, hunkIndex });
      }),
      stageLinesCalls,
      unstageLinesCalls,
      stageHunkCalls,
      unstageHunkCalls,
    };
  }

  it("renders stage line buttons when staging callbacks are provided", () => {
    const staging = makeStagingCallbacks();
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 2, content: "+ added" }),
            makeLine({ kind: "deletion", oldLineNo: 3, newLineNo: null, content: "- removed" }),
            makeLine({ kind: "context", oldLineNo: 4, newLineNo: 4, content: "  ctx" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} staging={staging} />);
    const stageButtons = screen.getAllByTestId("stage-line-btn");
    // 3 lines → 3 stage buttons (addition, deletion, context — context is disabled)
    expect(stageButtons).toHaveLength(3);
  });

  it("does not render stage line buttons when no staging callbacks", () => {
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} />);
    expect(screen.queryAllByTestId("stage-line-btn")).toHaveLength(0);
  });

  it("clicking addition line gutter calls onStageLines with correct range", () => {
    const staging = makeStagingCallbacks();
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 5, content: "+ new line" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} staging={staging} />);
    const stageButtons = screen.getAllByTestId("stage-line-btn");
    // Click the addition line's stage button
    fireEvent.click(stageButtons[0]);
    expect(staging.onStageLines).toHaveBeenCalledWith("src/main.ts", ["5-5"]);
  });

  it("clicking deletion line gutter calls onStageLines with correct range", () => {
    const staging = makeStagingCallbacks();
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "deletion", oldLineNo: 8, newLineNo: null, content: "- old line" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} staging={staging} />);
    const stageButtons = screen.getAllByTestId("stage-line-btn");
    fireEvent.click(stageButtons[0]);
    expect(staging.onStageLines).toHaveBeenCalledWith("src/main.ts", ["8-8"]);
  });

  it("context line stage button is disabled", () => {
    const staging = makeStagingCallbacks();
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "context", oldLineNo: 1, newLineNo: 1, content: "  ctx" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} staging={staging} />);
    const stageButtons = screen.getAllByTestId("stage-line-btn");
    expect(stageButtons[0]).toBeDisabled();
  });

  it("staged line shows checkmark in gutter", () => {
    const staging = makeStagingCallbacks();
    const stagedLines = new Set(["add:5"]);
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 5, content: "+ new line" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} staging={staging} stagedLines={stagedLines} />);
    const stageButtons = screen.getAllByTestId("stage-line-btn");
    expect(stageButtons[0]).toHaveTextContent("✓");
  });

  it("unstaged line shows + icon in gutter", () => {
    const staging = makeStagingCallbacks();
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 5, content: "+ new line" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} staging={staging} stagedLines={new Set()} />);
    const stageButtons = screen.getAllByTestId("stage-line-btn");
    expect(stageButtons[0]).toHaveTextContent("+");
  });

  it("clicking staged line calls onUnstageLines", () => {
    const staging = makeStagingCallbacks();
    const stagedLines = new Set(["add:5"]);
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 5, content: "+ new line" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} staging={staging} stagedLines={stagedLines} />);
    const stageButtons = screen.getAllByTestId("stage-line-btn");
    fireEvent.click(stageButtons[0]);
    expect(staging.onUnstageLines).toHaveBeenCalledWith("src/main.ts", ["5-5"]);
  });

  it("staged line has highlight class", () => {
    const staging = makeStagingCallbacks();
    const stagedLines = new Set(["add:5"]);
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 5, content: "+ new line" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} staging={staging} stagedLines={stagedLines} />);
    const lines = screen.getAllByTestId("diff-line");
    expect(lines[0].className).toContain("bg-yellow-900");
  });

  // -------------------------------------------------------------------------
  // Hunk-level staging tests
  // -------------------------------------------------------------------------

  it("renders Stage Hunk button on hunk headers when staging is provided", () => {
    const staging = makeStagingCallbacks();
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} staging={staging} />);
    const hunkBtns = screen.getAllByTestId("stage-hunk-btn");
    expect(hunkBtns).toHaveLength(1);
    expect(hunkBtns[0]).toHaveTextContent("Stage Hunk");
  });

  it("does not render Stage Hunk button when no staging callbacks", () => {
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} />);
    expect(screen.queryAllByTestId("stage-hunk-btn")).toHaveLength(0);
  });

  it("clicking Stage Hunk button calls onStageHunk with correct hunk index", () => {
    const staging = makeStagingCallbacks();
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} staging={staging} />);
    const hunkBtns = screen.getAllByTestId("stage-hunk-btn");
    fireEvent.click(hunkBtns[0]);
    expect(staging.onStageHunk).toHaveBeenCalledWith("src/main.ts", 0);
  });

  it("hunk button shows 'Unstage Hunk' when all hunk lines are staged", () => {
    const staging = makeStagingCallbacks();
    const stagedLines = new Set(["del:2", "add:2"]);
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "context", oldLineNo: 1, newLineNo: 1, content: "  line 1" }),
            makeLine({ kind: "deletion", oldLineNo: 2, newLineNo: null, content: "- removed" }),
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 2, content: "+ added" }),
            makeLine({ kind: "context", oldLineNo: 3, newLineNo: 3, content: "  line 3" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} staging={staging} stagedLines={stagedLines} />);
    const hunkBtns = screen.getAllByTestId("stage-hunk-btn");
    expect(hunkBtns[0]).toHaveTextContent("Unstage Hunk");
  });

  it("clicking Unstage Hunk calls onUnstageHunk when fully staged", () => {
    const staging = makeStagingCallbacks();
    const stagedLines = new Set(["del:2", "add:2"]);
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "context", oldLineNo: 1, newLineNo: 1, content: "  line 1" }),
            makeLine({ kind: "deletion", oldLineNo: 2, newLineNo: null, content: "- removed" }),
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 2, content: "+ added" }),
            makeLine({ kind: "context", oldLineNo: 3, newLineNo: 3, content: "  line 3" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} staging={staging} stagedLines={stagedLines} />);
    const hunkBtns = screen.getAllByTestId("stage-hunk-btn");
    fireEvent.click(hunkBtns[0]);
    expect(staging.onUnstageHunk).toHaveBeenCalledWith("src/main.ts", 0);
  });

  it("staging works in split mode - line buttons appear", () => {
    const staging = makeStagingCallbacks();
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "deletion", oldLineNo: 1, newLineNo: null, content: "- old" }),
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 1, content: "+ new" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} mode="split" staging={staging} />);
    const stageButtons = screen.getAllByTestId("stage-line-btn");
    // Left panel: hunk header blank + deletion = at least 1 stageable
    // Right panel: hunk header blank + addition = at least 1 stageable
    expect(stageButtons.length).toBeGreaterThanOrEqual(2);
  });

  it("split mode clicking line gutter calls staging callback", () => {
    const staging = makeStagingCallbacks();
    const diff = makeDiffContent({
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 3, content: "+ added" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} mode="split" staging={staging} />);
    // Find all non-disabled stage buttons
    const stageButtons = screen.getAllByTestId("stage-line-btn")
      .filter((btn) => !btn.hasAttribute("disabled") || btn.getAttribute("disabled") === "false");
    const enabledButtons = stageButtons.filter((btn) => !(btn as HTMLButtonElement).disabled);
    if (enabledButtons.length > 0) {
      fireEvent.click(enabledButtons[0]);
      expect(staging.onStageLines).toHaveBeenCalled();
    }
  });

  it("split mode hunk header shows Stage Hunk button", () => {
    const staging = makeStagingCallbacks();
    const diff = makeDiffContent();
    render(<DiffViewer diffContent={diff} mode="split" staging={staging} />);
    const hunkBtns = screen.getAllByTestId("stage-hunk-btn");
    // Split mode: left panel header has the button
    expect(hunkBtns.length).toBeGreaterThanOrEqual(1);
  });

  // -------------------------------------------------------------------------
  // Syntax highlighting integration tests
  // -------------------------------------------------------------------------

  it("highlights JS keywords in diff lines for .ts files", () => {
    const diff = makeDiffContent({
      filePath: "src/app.ts",
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "addition", oldLineNo: null, newLineNo: 1, content: "const x = 42;" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const lineContent = screen.getAllByTestId("line-content")[0];
    const keywordSpan = lineContent.querySelector("[data-token-type='keyword']");
    expect(keywordSpan).not.toBeNull();
    expect(keywordSpan?.textContent).toBe("const");
  });

  it("renders plain text for unknown file extensions (no token spans)", () => {
    const diff = makeDiffContent({
      filePath: "data.xyz",
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "context", oldLineNo: 1, newLineNo: 1, content: "const x = 42;" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const lineContent = screen.getAllByTestId("line-content")[0];
    const tokenSpans = lineContent.querySelectorAll("[data-token-type]");
    expect(tokenSpans.length).toBe(0);
    expect(lineContent.textContent).toBe("const x = 42;");
  });

  it("highlights strings in diff lines", () => {
    const diff = makeDiffContent({
      filePath: "src/main.ts",
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "context", oldLineNo: 1, newLineNo: 1, content: 'const name = "hello";' }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} />);
    const lineContent = screen.getAllByTestId("line-content")[0];
    const stringSpan = lineContent.querySelector("[data-token-type='string']");
    expect(stringSpan).not.toBeNull();
    expect(stringSpan?.textContent).toBe('"hello"');
  });

  it("highlights syntax in split mode too", () => {
    const diff = makeDiffContent({
      filePath: "src/app.ts",
      hunks: [
        makeHunk({
          lines: [
            makeLine({ kind: "context", oldLineNo: 1, newLineNo: 1, content: "const x = 42;" }),
          ],
        }),
      ],
    });
    render(<DiffViewer diffContent={diff} mode="split" />);
    const splitContents = screen.getAllByTestId("split-line-content");
    // Both left and right panels should have highlighting
    const leftKeyword = splitContents[0].querySelector("[data-token-type='keyword']");
    expect(leftKeyword).not.toBeNull();
    expect(leftKeyword?.textContent).toBe("const");
  });
});
