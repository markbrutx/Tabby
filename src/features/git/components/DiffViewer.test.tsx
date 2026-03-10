import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { DiffViewer } from "./DiffViewer";
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
// Tests
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
});
