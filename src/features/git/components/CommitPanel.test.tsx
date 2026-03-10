import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { CommitPanel, type CommitPanelProps } from "./CommitPanel";
import type { CommitInfo, FileStatus } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeStagedFile(path: string): FileStatus {
  return {
    path,
    oldPath: null,
    indexStatus: "modified",
    worktreeStatus: "modified",
  };
}

function makeUnstagedFile(path: string): FileStatus {
  return {
    path,
    oldPath: null,
    indexStatus: "untracked",
    worktreeStatus: "untracked",
  };
}

function makeCommitInfo(overrides?: Partial<CommitInfo>): CommitInfo {
  return {
    hash: "abc123def456",
    shortHash: "abc123d",
    authorName: "Test Author",
    authorEmail: "test@example.com",
    date: "2026-03-10",
    message: "previous commit message",
    parentHashes: [],
    ...overrides,
  };
}

function renderPanel(overrides?: Partial<CommitPanelProps>) {
  const defaults: CommitPanelProps = {
    files: [makeStagedFile("src/main.ts"), makeStagedFile("src/lib.ts")],
    onCommit: vi.fn().mockResolvedValue(undefined),
    onFetchLastCommitInfo: vi.fn().mockResolvedValue(makeCommitInfo()),
    onCommitSuccess: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  };
  return { ...render(<CommitPanel {...defaults} />), props: defaults };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("CommitPanel", () => {
  it("renders commit message textarea with placeholder", () => {
    renderPanel();
    expect(screen.getByTestId("commit-message-input")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Commit message...")).toBeInTheDocument();
  });

  it("shows staged file count", () => {
    renderPanel({ files: [makeStagedFile("a.ts"), makeStagedFile("b.ts"), makeStagedFile("c.ts")] });
    expect(screen.getByTestId("staged-count")).toHaveTextContent("3 files staged");
  });

  it("shows singular form for 1 staged file", () => {
    renderPanel({ files: [makeStagedFile("a.ts")] });
    expect(screen.getByTestId("staged-count")).toHaveTextContent("1 file staged");
  });

  it("shows 0 files staged when no files are staged", () => {
    renderPanel({ files: [makeUnstagedFile("a.ts")] });
    expect(screen.getByTestId("staged-count")).toHaveTextContent("0 files staged");
  });

  it("disables commit button when message is empty", () => {
    renderPanel();
    expect(screen.getByTestId("commit-button")).toBeDisabled();
  });

  it("disables commit button when no staged files even with message", () => {
    renderPanel({ files: [makeUnstagedFile("a.ts")] });
    fireEvent.change(screen.getByTestId("commit-message-input"), {
      target: { value: "some message" },
    });
    expect(screen.getByTestId("commit-button")).toBeDisabled();
  });

  it("enables commit button with message and staged files", () => {
    renderPanel();
    fireEvent.change(screen.getByTestId("commit-message-input"), {
      target: { value: "feat: add feature" },
    });
    expect(screen.getByTestId("commit-button")).toBeEnabled();
  });

  it("calls commit and clears message on success", async () => {
    const onCommit = vi.fn().mockResolvedValue(undefined);
    const onCommitSuccess = vi.fn().mockResolvedValue(undefined);

    renderPanel({ onCommit, onCommitSuccess });

    fireEvent.change(screen.getByTestId("commit-message-input"), {
      target: { value: "feat: new feature" },
    });
    fireEvent.click(screen.getByTestId("commit-button"));

    await waitFor(() => {
      expect(onCommit).toHaveBeenCalledWith("feat: new feature", false);
    });

    await waitFor(() => {
      expect(onCommitSuccess).toHaveBeenCalled();
    });

    await waitFor(() => {
      expect(screen.getByTestId("commit-message-input")).toHaveValue("");
    });
  });

  it("shows error when commit fails", async () => {
    const onCommit = vi.fn().mockRejectedValue(new Error("nothing to commit"));

    renderPanel({ onCommit });

    fireEvent.change(screen.getByTestId("commit-message-input"), {
      target: { value: "test commit" },
    });
    fireEvent.click(screen.getByTestId("commit-button"));

    await waitFor(() => {
      expect(screen.getByTestId("commit-error")).toHaveTextContent("nothing to commit");
    });
  });

  it("amend checkbox populates textarea with last commit message", async () => {
    renderPanel();

    await waitFor(() => {
      expect(screen.getByTestId("commit-author")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("amend-checkbox"));

    await waitFor(() => {
      expect(screen.getByTestId("commit-message-input")).toHaveValue("previous commit message");
    });
  });

  it("shows author info from last commit", async () => {
    renderPanel();

    await waitFor(() => {
      expect(screen.getByTestId("commit-author")).toHaveTextContent("Test Author");
      expect(screen.getByTestId("commit-author")).toHaveTextContent("test@example.com");
    });
  });

  it("commits with Cmd+Enter keyboard shortcut", async () => {
    const onCommit = vi.fn().mockResolvedValue(undefined);

    renderPanel({ onCommit });

    const textarea = screen.getByTestId("commit-message-input");
    fireEvent.change(textarea, { target: { value: "keyboard commit" } });
    fireEvent.keyDown(textarea, { key: "Enter", metaKey: true });

    await waitFor(() => {
      expect(onCommit).toHaveBeenCalledWith("keyboard commit", false);
    });
  });

  it("sends amend flag when amend is checked", async () => {
    const onCommit = vi.fn().mockResolvedValue(undefined);
    const onFetchLastCommitInfo = vi.fn().mockResolvedValue(makeCommitInfo({ message: "amend me" }));
    const onCommitSuccess = vi.fn().mockResolvedValue(undefined);

    renderPanel({ onCommit, onFetchLastCommitInfo, onCommitSuccess });

    await waitFor(() => {
      expect(screen.getByTestId("commit-author")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("amend-checkbox"));

    await waitFor(() => {
      expect(screen.getByTestId("commit-message-input")).toHaveValue("amend me");
    });

    fireEvent.click(screen.getByTestId("commit-button"));

    await waitFor(() => {
      expect(onCommit).toHaveBeenCalledWith("amend me", true);
    });
  });
});
