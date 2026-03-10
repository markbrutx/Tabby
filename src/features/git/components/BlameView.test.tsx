import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { BlameView, type BlameViewProps } from "./BlameView";
import type { BlameEntry } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeEntry(overrides?: Partial<BlameEntry>): BlameEntry {
  return {
    hash: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
    author: "Developer",
    date: "2026-03-10T12:00:00Z",
    lineStart: 1,
    lineCount: 3,
    content: "line 1\nline 2\nline 3",
    ...overrides,
  };
}

function defaultEntries(): readonly BlameEntry[] {
  return [
    makeEntry({
      hash: "aaa111a",
      author: "Alice",
      date: "2026-03-10T12:00:00Z",
      lineStart: 1,
      lineCount: 3,
      content: "import { app } from './app';\napp.init();\napp.start();",
    }),
    makeEntry({
      hash: "bbb222b",
      author: "Bob",
      date: "2026-03-08T10:00:00Z",
      lineStart: 4,
      lineCount: 2,
      content: "export default app;\n// end",
    }),
  ];
}

function renderBlame(overrides?: Partial<BlameViewProps>) {
  const defaults: BlameViewProps = {
    filePath: "src/main.ts",
    entries: defaultEntries(),
    onCommitClick: vi.fn(),
    ...overrides,
  };
  return { ...render(<BlameView {...defaults} />), props: defaults };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("BlameView", () => {
  it("renders blame annotations for each block", () => {
    renderBlame();
    expect(screen.getByTestId("blame-view")).toBeDefined();
    expect(screen.getByTestId("blame-hash-aaa111a")).toBeDefined();
    expect(screen.getByTestId("blame-hash-bbb222b")).toBeDefined();
  });

  it("displays file path in header", () => {
    renderBlame({ filePath: "src/utils/helper.ts" });
    expect(screen.getByText("Blame: src/utils/helper.ts")).toBeDefined();
  });

  it("shows author name and commit hash in annotations", () => {
    renderBlame();
    const hashButton = screen.getByTestId("blame-hash-aaa111a");
    expect(hashButton.textContent).toBe("aaa111a");

    // Author name should appear in the first line of the block
    const line1 = screen.getByTestId("blame-line-1");
    expect(line1.textContent).toContain("Alice");
  });

  it("renders all expanded lines with line numbers", () => {
    renderBlame();
    // First block: 3 lines (1-3), second block: 2 lines (4-5)
    expect(screen.getByTestId("blame-line-1")).toBeDefined();
    expect(screen.getByTestId("blame-line-2")).toBeDefined();
    expect(screen.getByTestId("blame-line-3")).toBeDefined();
    expect(screen.getByTestId("blame-line-4")).toBeDefined();
    expect(screen.getByTestId("blame-line-5")).toBeDefined();
  });

  it("displays line content in monospace", () => {
    renderBlame();
    const line1 = screen.getByTestId("blame-line-1");
    expect(line1.textContent).toContain("import { app } from './app';");
  });

  it("calls onCommitClick when clicking a commit hash", () => {
    const { props } = renderBlame();
    const hashButton = screen.getByTestId("blame-hash-aaa111a");
    fireEvent.click(hashButton);
    expect(props.onCommitClick).toHaveBeenCalledWith("aaa111a");
  });

  it("shows empty state when no entries", () => {
    renderBlame({ entries: [] });
    expect(screen.getByTestId("blame-empty")).toBeDefined();
    expect(screen.getByText("No blame data available")).toBeDefined();
  });

  it("applies alternating background colors for different blocks", () => {
    renderBlame();
    const line1 = screen.getByTestId("blame-line-1");
    const line4 = screen.getByTestId("blame-line-4");

    // First block uses bg-[var(--color-bg)], second uses bg-[var(--color-bg-elevated)]
    expect(line1.className).toContain("bg-[var(--color-bg)]");
    expect(line4.className).toContain("bg-[var(--color-bg-elevated)]");
  });

  it("only shows annotation on the first line of each block", () => {
    renderBlame();
    // Line 1 should have the hash button (block start)
    const line1 = screen.getByTestId("blame-line-1");
    const hashInLine1 = line1.querySelector("[data-testid='blame-hash-aaa111a']");
    expect(hashInLine1).not.toBeNull();

    // Line 2 should NOT have a hash button (continuation line)
    const line2 = screen.getByTestId("blame-line-2");
    const hashInLine2 = line2.querySelector("[data-testid]");
    // Only the blame-line-2 testid should exist, not a hash button
    expect(hashInLine2).toBeNull();
  });
});
