import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { HistoryPanel, type HistoryPanelProps } from "./HistoryPanel";
import type { CommitInfo } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeCommit(overrides?: Partial<CommitInfo>): CommitInfo {
  return {
    hash: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
    shortHash: "a1b2c3d",
    authorName: "Developer",
    authorEmail: "dev@example.com",
    date: "2026-03-10T12:00:00Z",
    message: "feat: add git client transport",
    parentHashes: ["b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3"],
    ...overrides,
  };
}

function defaultCommits(): readonly CommitInfo[] {
  return [
    makeCommit({
      hash: "aaa111aaa111aaa111aaa111aaa111aaa111aaa1",
      shortHash: "aaa111a",
      message: "feat: first commit",
      date: "2026-03-10T12:00:00Z",
    }),
    makeCommit({
      hash: "bbb222bbb222bbb222bbb222bbb222bbb222bbb2",
      shortHash: "bbb222b",
      authorName: "Contributor",
      message: "fix: resolve bug",
      date: "2026-03-09T10:00:00Z",
    }),
    makeCommit({
      hash: "ccc333ccc333ccc333ccc333ccc333ccc333ccc3",
      shortHash: "ccc333c",
      message: "refactor: clean up utils",
      date: "2026-03-08T08:00:00Z",
    }),
  ];
}

function renderPanel(overrides?: Partial<HistoryPanelProps>) {
  const defaults: HistoryPanelProps = {
    commits: defaultCommits(),
    loading: false,
    hasMore: true,
    selectedCommitHash: null,
    headCommitHash: "aaa111aaa111aaa111aaa111aaa111aaa111aaa1",
    commitDiffContent: null,
    onSelectCommit: vi.fn(),
    onLoadMore: vi.fn(),
    ...overrides,
  };
  return { ...render(<HistoryPanel {...defaults} />), props: defaults };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("HistoryPanel", () => {
  it("renders all commits in the list", () => {
    renderPanel();
    expect(screen.getByTestId("commit-row-aaa111a")).toBeDefined();
    expect(screen.getByTestId("commit-row-bbb222b")).toBeDefined();
    expect(screen.getByTestId("commit-row-ccc333c")).toBeDefined();
  });

  it("displays commit short hash, message, and author", () => {
    renderPanel();
    const row = screen.getByTestId("commit-row-aaa111a");
    expect(row.textContent).toContain("aaa111a");
    expect(row.textContent).toContain("feat: first commit");
    expect(row.textContent).toContain("Developer");
  });

  it("shows HEAD indicator on the head commit", () => {
    renderPanel();
    const headRow = screen.getByTestId("commit-row-aaa111a");
    const headIndicator = headRow.querySelector("[data-testid='head-indicator']");
    expect(headIndicator).not.toBeNull();
    expect(headIndicator?.textContent).toBe("HEAD");

    // Other commits should not have HEAD indicator
    const otherRow = screen.getByTestId("commit-row-bbb222b");
    const otherIndicator = otherRow.querySelector("[data-testid='head-indicator']");
    expect(otherIndicator).toBeNull();
  });

  it("calls onSelectCommit when a commit row is clicked", () => {
    const { props } = renderPanel();
    const row = screen.getByTestId("commit-row-bbb222b");
    fireEvent.click(row);
    expect(props.onSelectCommit).toHaveBeenCalledWith(
      "bbb222bbb222bbb222bbb222bbb222bbb222bbb2",
    );
  });

  it("highlights the selected commit", () => {
    renderPanel({
      selectedCommitHash: "bbb222bbb222bbb222bbb222bbb222bbb222bbb2",
    });
    const row = screen.getByTestId("commit-row-bbb222b");
    expect(row.className).toContain("accent");
  });

  it("shows empty state when no commits", () => {
    renderPanel({ commits: [], hasMore: false });
    expect(screen.getByTestId("history-empty")).toBeDefined();
    expect(screen.getByText("No commits yet")).toBeDefined();
  });

  it("shows loading indicator when loading", () => {
    renderPanel({ loading: true });
    expect(screen.getByTestId("history-loading")).toBeDefined();
  });

  it("triggers onLoadMore when scrolled near bottom", () => {
    const onLoadMore = vi.fn();
    renderPanel({ onLoadMore, hasMore: true });
    const scrollContainer = screen.getByTestId("history-commit-list");

    // Simulate scroll to bottom
    Object.defineProperty(scrollContainer, "scrollHeight", { value: 1000 });
    Object.defineProperty(scrollContainer, "clientHeight", { value: 400 });
    Object.defineProperty(scrollContainer, "scrollTop", { value: 550 });
    fireEvent.scroll(scrollContainer);

    expect(onLoadMore).toHaveBeenCalled();
  });

  it("does not trigger onLoadMore when hasMore is false", () => {
    const onLoadMore = vi.fn();
    renderPanel({ onLoadMore, hasMore: false });
    const scrollContainer = screen.getByTestId("history-commit-list");

    Object.defineProperty(scrollContainer, "scrollHeight", { value: 1000 });
    Object.defineProperty(scrollContainer, "clientHeight", { value: 400 });
    Object.defineProperty(scrollContainer, "scrollTop", { value: 550 });
    fireEvent.scroll(scrollContainer);

    expect(onLoadMore).not.toHaveBeenCalled();
  });

  it("shows diff summary when a commit is selected and diff is available", () => {
    renderPanel({
      selectedCommitHash: "aaa111aaa111aaa111aaa111aaa111aaa111aaa1",
      commitDiffContent: {
        filePath: "src/main.ts",
        oldPath: null,
        hunks: [
          {
            oldStart: 1,
            oldCount: 2,
            newStart: 1,
            newCount: 3,
            header: "@@ -1,2 +1,3 @@",
            lines: [],
          },
        ],
        isBinary: false,
        fileModeChange: null,
      },
    });
    const summary = screen.getByTestId("history-diff-summary");
    expect(summary.textContent).toContain("src/main.ts");
    expect(summary.textContent).toContain("1 hunk");
  });

  it("shows 'End of history' when all commits are loaded", () => {
    renderPanel({ hasMore: false });
    expect(screen.getByText("End of history")).toBeDefined();
  });
});
