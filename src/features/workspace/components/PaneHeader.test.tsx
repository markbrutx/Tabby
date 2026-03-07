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

  describe("browser mode", () => {
    it("renders URL input and nav buttons when isBrowser is true", () => {
      renderHeader({ isBrowser: true, browserUrl: "https://example.com" });
      expect(screen.getByTestId("browser-url-input")).toBeTruthy();
      expect(screen.getByTestId("browser-back-btn")).toBeTruthy();
      expect(screen.getByTestId("browser-forward-btn")).toBeTruthy();
      expect(screen.getByTestId("browser-reload-btn")).toBeTruthy();
      expect(screen.getByTestId("browser-go-btn")).toBeTruthy();
    });

    it("hides profile and cwd in browser mode", () => {
      renderHeader({ isBrowser: true, browserUrl: "https://example.com" });
      expect(screen.queryByTestId("pane-header-profile")).toBeNull();
      expect(screen.queryByTestId("pane-header-cwd")).toBeNull();
    });

    it("uses taller height in browser mode", () => {
      renderHeader({ isBrowser: true, browserUrl: "https://example.com" });
      const header = screen.getByTestId("pane-header");
      expect(header.className).toContain("h-8");
    });

    it("calls onBrowserNavigate when Go is clicked", () => {
      const onBrowserNavigate = vi.fn();
      renderHeader({
        isBrowser: true,
        browserUrl: "https://example.com",
        onBrowserNavigate,
      });

      const input = screen.getByTestId("browser-url-input") as HTMLInputElement;
      fireEvent.change(input, { target: { value: "https://new-url.com" } });
      fireEvent.click(screen.getByTestId("browser-go-btn"));
      expect(onBrowserNavigate).toHaveBeenCalledWith("https://new-url.com");
    });

    it("calls onBrowserNavigate on Enter key", () => {
      const onBrowserNavigate = vi.fn();
      renderHeader({
        isBrowser: true,
        browserUrl: "https://example.com",
        onBrowserNavigate,
      });

      const input = screen.getByTestId("browser-url-input") as HTMLInputElement;
      fireEvent.change(input, { target: { value: "https://enter-url.com" } });
      fireEvent.keyDown(input, { key: "Enter" });
      expect(onBrowserNavigate).toHaveBeenCalledWith("https://enter-url.com");
    });

    it("calls onBrowserReload when reload is clicked", () => {
      const onBrowserReload = vi.fn();
      renderHeader({
        isBrowser: true,
        browserUrl: "https://example.com",
        onBrowserReload,
      });

      fireEvent.click(screen.getByTestId("browser-reload-btn"));
      expect(onBrowserReload).toHaveBeenCalledTimes(1);
    });

    it("still shows close button in browser mode with multiple panes", () => {
      renderHeader({ isBrowser: true, browserUrl: "https://example.com", paneCount: 2 });
      expect(screen.getByTestId("pane-header-close")).toBeTruthy();
    });
  });
});
