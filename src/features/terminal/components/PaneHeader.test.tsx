import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { PaneHeader } from "./PaneHeader";

function renderHeader(overrides: Partial<Parameters<typeof PaneHeader>[0]> = {}) {
  const onClose = vi.fn();
  const onRestart = vi.fn();
  const onOpenGitView = vi.fn();

  render(
    <PaneHeader
      profileLabel="Terminal"
      cwd="/Users/mark/projects/tabby"
      isActive={false}
      paneCount={2}
      onClose={onClose}
      onRestart={onRestart}
      onOpenGitView={onOpenGitView}
      {...overrides}
    />,
  );

  return { onClose, onRestart, onOpenGitView };
}

describe("PaneHeader", () => {
  it("renders profile label and shortened cwd", () => {
    renderHeader();
    expect(screen.getByTestId("pane-header-profile").textContent).toBe("Terminal");
    expect(screen.getByTestId("pane-header-cwd").textContent).toBeTruthy();
  });

  it("close button fires onClose", () => {
    const { onClose } = renderHeader();
    fireEvent.click(screen.getByTestId("pane-header-close"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("active state has accent border styling", () => {
    renderHeader({ isActive: true });
    const header = screen.getByTestId("pane-header");
    expect(header.className).toContain("border-[var(--color-accent)]");
  });

  it("inactive state has normal border styling", () => {
    renderHeader({ isActive: false });
    const header = screen.getByTestId("pane-header");
    expect(header.className).toContain("border-[var(--color-border)]");
  });

  it("close button hidden when paneCount is 1", () => {
    renderHeader({ paneCount: 1 });
    expect(screen.queryByTestId("pane-header-close")).toBeNull();
  });

  it("has draggable attribute when draggable is true", () => {
    renderHeader({ draggable: true });
    const header = screen.getByTestId("pane-header");
    expect(header.getAttribute("draggable")).toBe("true");
  });

  it("does not show drag over ring (drop target moved to pane wrapper)", () => {
    renderHeader();
    const header = screen.getByTestId("pane-header");
    expect(header.className).not.toContain("ring-2");
  });

  it("shows Open Git View button when onOpenGitView and cwd are provided", () => {
    renderHeader();
    expect(screen.getByTestId("pane-header-open-git")).toBeInTheDocument();
  });

  it("Open Git View button fires onOpenGitView", () => {
    const { onOpenGitView } = renderHeader();
    fireEvent.click(screen.getByTestId("pane-header-open-git"));
    expect(onOpenGitView).toHaveBeenCalledTimes(1);
  });

  it("hides Open Git View button when cwd is empty", () => {
    renderHeader({ cwd: "" });
    expect(screen.queryByTestId("pane-header-open-git")).toBeNull();
  });

  it("hides Open Git View button when onOpenGitView is not provided", () => {
    renderHeader({ onOpenGitView: undefined });
    expect(screen.queryByTestId("pane-header-open-git")).toBeNull();
  });

});
