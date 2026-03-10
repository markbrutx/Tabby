import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FileTreePanel, type FileTreePanelProps } from "./FileTreePanel";
import type { FileStatus } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeFile(overrides: Partial<FileStatus> = {}): FileStatus {
  return {
    path: "src/main.ts",
    oldPath: null,
    indexStatus: "untracked",
    worktreeStatus: "modified",
    ...overrides,
  };
}

function renderPanel(overrides: Partial<FileTreePanelProps> = {}) {
  const props: FileTreePanelProps = {
    files: [],
    selectedFile: null,
    onSelectFile: vi.fn(),
    onStageFiles: vi.fn(),
    onUnstageFiles: vi.fn(),
    onDiscardChanges: vi.fn(),
    ...overrides,
  };

  return {
    ...render(<FileTreePanel {...props} />),
    props,
  };
}

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

const STAGED_FILE: FileStatus = makeFile({
  path: "src/staged.ts",
  indexStatus: "modified",
  worktreeStatus: "untracked",
});

const UNSTAGED_FILE: FileStatus = makeFile({
  path: "src/unstaged.ts",
  indexStatus: "untracked",
  worktreeStatus: "modified",
});

const MIXED_FILE: FileStatus = makeFile({
  path: "src/mixed.ts",
  indexStatus: "modified",
  worktreeStatus: "modified",
});

const ADDED_FILE: FileStatus = makeFile({
  path: "src/new-file.ts",
  indexStatus: "added",
  worktreeStatus: "untracked",
});

const DELETED_FILE: FileStatus = makeFile({
  path: "src/old.ts",
  indexStatus: "deleted",
  worktreeStatus: "deleted",
});

const UNTRACKED_FILE: FileStatus = makeFile({
  path: "temp.log",
  indexStatus: "untracked",
  worktreeStatus: "untracked",
});

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("FileTreePanel", () => {
  it("shows empty state when no files", () => {
    renderPanel({ files: [] });

    expect(screen.getByTestId("file-tree-empty")).toBeInTheDocument();
    expect(screen.getByText("No changes in working directory")).toBeInTheDocument();
  });

  it("renders staged and unstaged sections", () => {
    renderPanel({ files: [STAGED_FILE, UNSTAGED_FILE] });

    expect(screen.getByTestId("staged-section")).toBeInTheDocument();
    expect(screen.getByTestId("unstaged-section")).toBeInTheDocument();
    expect(screen.getByText("Staged Changes")).toBeInTheDocument();
    expect(screen.getByText("Changes")).toBeInTheDocument();
  });

  it("renders file entries with correct status badges", () => {
    const stagedOnly: FileStatus = makeFile({
      path: "src/staged-only.ts",
      indexStatus: "modified",
      worktreeStatus: "ignored",
    });
    const unstagedOnly: FileStatus = makeFile({
      path: "src/unstaged-only.ts",
      indexStatus: "untracked",
      worktreeStatus: "modified",
    });
    renderPanel({ files: [stagedOnly, unstagedOnly] });

    const badges = screen.getAllByTestId("status-badge");
    expect(badges).toHaveLength(2);
    expect(badges[0]).toHaveTextContent("M");
    expect(badges[1]).toHaveTextContent("M");
  });

  it("renders added file with A badge", () => {
    renderPanel({ files: [ADDED_FILE] });

    const badges = screen.getAllByTestId("status-badge");
    const addedBadge = badges.find((b) => b.textContent === "A");
    expect(addedBadge).toBeDefined();
  });

  it("renders deleted file with D badge", () => {
    renderPanel({ files: [DELETED_FILE] });

    const badges = screen.getAllByTestId("status-badge");
    const deletedBadge = badges.find((b) => b.textContent === "D");
    expect(deletedBadge).toBeDefined();
  });

  it("renders untracked file with ? badge", () => {
    renderPanel({ files: [UNTRACKED_FILE] });

    const badges = screen.getAllByTestId("status-badge");
    const untrackedBadge = badges.find((b) => b.textContent === "?");
    expect(untrackedBadge).toBeDefined();
  });

  it("calls onSelectFile when file is clicked", () => {
    const { props } = renderPanel({ files: [UNSTAGED_FILE] });

    fireEvent.click(screen.getByTestId("file-select-button"));

    expect(props.onSelectFile).toHaveBeenCalledTimes(1);
    expect(props.onSelectFile).toHaveBeenCalledWith("src/unstaged.ts");
  });

  it("shows stage button on unstaged files", () => {
    renderPanel({ files: [UNSTAGED_FILE] });

    const unstagedSection = screen.getByTestId("unstaged-section");
    const stageButton = within(unstagedSection).getByTestId("stage-button");
    expect(stageButton).toBeInTheDocument();
  });

  it("shows unstage button on staged files", () => {
    renderPanel({ files: [STAGED_FILE] });

    const stagedSection = screen.getByTestId("staged-section");
    const unstageButton = within(stagedSection).getByTestId("unstage-button");
    expect(unstageButton).toBeInTheDocument();
  });

  it("shows discard button on unstaged files", () => {
    renderPanel({ files: [UNSTAGED_FILE] });

    const unstagedSection = screen.getByTestId("unstaged-section");
    const discardButton = within(unstagedSection).getByTestId("discard-button");
    expect(discardButton).toBeInTheDocument();
  });

  it("calls onStageFiles when stage button clicked", () => {
    const { props } = renderPanel({ files: [UNSTAGED_FILE] });

    fireEvent.click(screen.getByTestId("stage-button"));

    expect(props.onStageFiles).toHaveBeenCalledTimes(1);
    expect(props.onStageFiles).toHaveBeenCalledWith(["src/unstaged.ts"]);
  });

  it("calls onUnstageFiles when unstage button clicked", () => {
    const { props } = renderPanel({ files: [STAGED_FILE] });

    fireEvent.click(screen.getByTestId("unstage-button"));

    expect(props.onUnstageFiles).toHaveBeenCalledTimes(1);
    expect(props.onUnstageFiles).toHaveBeenCalledWith(["src/staged.ts"]);
  });

  it("shows confirmation before discarding changes", () => {
    const { props } = renderPanel({ files: [UNSTAGED_FILE] });

    fireEvent.click(screen.getByTestId("discard-button"));

    expect(screen.getByTestId("discard-confirm")).toBeInTheDocument();
    expect(props.onDiscardChanges).not.toHaveBeenCalled();
  });

  it("calls onDiscardChanges after confirmation", () => {
    const { props } = renderPanel({ files: [UNSTAGED_FILE] });

    fireEvent.click(screen.getByTestId("discard-button"));
    fireEvent.click(screen.getByTestId("discard-confirm-yes"));

    expect(props.onDiscardChanges).toHaveBeenCalledTimes(1);
    expect(props.onDiscardChanges).toHaveBeenCalledWith(["src/unstaged.ts"]);
  });

  it("cancels discard when cancel clicked", () => {
    const { props } = renderPanel({ files: [UNSTAGED_FILE] });

    fireEvent.click(screen.getByTestId("discard-button"));
    fireEvent.click(screen.getByTestId("discard-confirm-no"));

    expect(props.onDiscardChanges).not.toHaveBeenCalled();
    expect(screen.queryByTestId("discard-confirm")).not.toBeInTheDocument();
  });

  it("calls onStageFiles with all unstaged paths on Stage All", () => {
    const files = [UNSTAGED_FILE, UNTRACKED_FILE];
    const { props } = renderPanel({ files });

    fireEvent.click(screen.getByTestId("stage-all-button"));

    expect(props.onStageFiles).toHaveBeenCalledTimes(1);
    expect(props.onStageFiles).toHaveBeenCalledWith(["src/unstaged.ts", "temp.log"]);
  });

  it("calls onUnstageFiles with all staged paths on Unstage All", () => {
    const files = [STAGED_FILE, ADDED_FILE];
    const { props } = renderPanel({ files });

    fireEvent.click(screen.getByTestId("unstage-all-button"));

    expect(props.onUnstageFiles).toHaveBeenCalledTimes(1);
    expect(props.onUnstageFiles).toHaveBeenCalledWith(["src/staged.ts", "src/new-file.ts"]);
  });

  it("collapses staged section when header toggled", () => {
    renderPanel({ files: [STAGED_FILE] });

    const stagedSection = screen.getByTestId("staged-section");
    expect(within(stagedSection).getByText("src/staged.ts")).toBeInTheDocument();

    const toggles = screen.getAllByTestId("section-toggle");
    fireEvent.click(toggles[0]);

    expect(within(stagedSection).queryByText("src/staged.ts")).not.toBeInTheDocument();
  });

  it("collapses unstaged section when header toggled", () => {
    renderPanel({ files: [UNSTAGED_FILE] });

    const unstagedSection = screen.getByTestId("unstaged-section");
    expect(within(unstagedSection).getByText("src/unstaged.ts")).toBeInTheDocument();

    const toggles = screen.getAllByTestId("section-toggle");
    fireEvent.click(toggles[1]);

    expect(within(unstagedSection).queryByText("src/unstaged.ts")).not.toBeInTheDocument();
  });

  it("shows file in both sections when it has mixed status", () => {
    renderPanel({ files: [MIXED_FILE] });

    const stagedSection = screen.getByTestId("staged-section");
    const unstagedSection = screen.getByTestId("unstaged-section");

    expect(within(stagedSection).getByText("src/mixed.ts")).toBeInTheDocument();
    expect(within(unstagedSection).getByText("src/mixed.ts")).toBeInTheDocument();
  });

  it("highlights selected file", () => {
    renderPanel({
      files: [UNSTAGED_FILE],
      selectedFile: "src/unstaged.ts",
    });

    const entry = screen.getByTestId("file-entry");
    expect(entry.className).toContain("bg-[var(--color-accent)]");
  });
});
