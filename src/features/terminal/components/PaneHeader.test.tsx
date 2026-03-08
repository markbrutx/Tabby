import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { PaneHeader } from "./PaneHeader";

function renderHeader(overrides: Partial<Parameters<typeof PaneHeader>[0]> = {}) {
  const onClose = vi.fn();

  render(
    <PaneHeader
      profileLabel="Terminal"
      cwd="/Users/mark/projects/tabby"
      isActive={false}
      paneCount={2}
      onClose={onClose}
      {...overrides}
    />,
  );

  return { onClose };
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

  it("shows drag over ring when isDragOver is true", () => {
    renderHeader({ isDragOver: true });
    const header = screen.getByTestId("pane-header");
    expect(header.className).toContain("ring-2");
  });

});
