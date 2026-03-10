import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { PaneHeader } from "./PaneHeader";

describe("Open Git View action", () => {
  it("creates a Git pane with the terminal's cwd when clicked", () => {
    const onOpenGitView = vi.fn();
    const cwd = "/Users/mark/my-project";

    render(
      <PaneHeader
        profileLabel="zsh"
        cwd={cwd}
        isActive={true}
        paneCount={1}
        onClose={vi.fn()}
        onRestart={vi.fn()}
        onOpenGitView={onOpenGitView}
      />,
    );

    const gitButton = screen.getByTestId("pane-header-open-git");
    expect(gitButton).toBeInTheDocument();
    expect(gitButton).toHaveAttribute("title", "Open Git View");

    fireEvent.click(gitButton);
    expect(onOpenGitView).toHaveBeenCalledTimes(1);
  });

  it("is not shown when terminal has no working directory", () => {
    render(
      <PaneHeader
        profileLabel="zsh"
        cwd=""
        isActive={true}
        paneCount={1}
        onClose={vi.fn()}
        onRestart={vi.fn()}
        onOpenGitView={vi.fn()}
      />,
    );

    expect(screen.queryByTestId("pane-header-open-git")).toBeNull();
  });

  it("is not shown when onOpenGitView callback is not provided", () => {
    render(
      <PaneHeader
        profileLabel="zsh"
        cwd="/some/path"
        isActive={true}
        paneCount={1}
        onClose={vi.fn()}
        onRestart={vi.fn()}
      />,
    );

    expect(screen.queryByTestId("pane-header-open-git")).toBeNull();
  });

  it("stops event propagation to avoid triggering pane focus", () => {
    const onOpenGitView = vi.fn();
    const onOuterClick = vi.fn();

    render(
      <div onClick={onOuterClick}>
        <PaneHeader
          profileLabel="zsh"
          cwd="/some/path"
          isActive={true}
          paneCount={1}
          onClose={vi.fn()}
        onRestart={vi.fn()}
          onOpenGitView={onOpenGitView}
        />
      </div>,
    );

    fireEvent.click(screen.getByTestId("pane-header-open-git"));
    expect(onOpenGitView).toHaveBeenCalledTimes(1);
    expect(onOuterClick).not.toHaveBeenCalled();
  });
});
