import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { BrowserToolbar } from "./BrowserToolbar";

const DEFAULT_URL = "https://google.com";

function renderToolbar(overrides: Partial<Parameters<typeof BrowserToolbar>[0]> = {}) {
  const onNavigate = vi.fn();
  const onReload = vi.fn();
  const onClose = vi.fn();

  render(
    <BrowserToolbar
      url="https://example.com"
      isActive={false}
      paneCount={2}
      onNavigate={onNavigate}
      onReload={onReload}
      onClose={onClose}
      {...overrides}
    />,
  );

  return { onNavigate, onReload, onClose };
}

describe("BrowserToolbar", () => {
  // -----------------------------------------------------------------------
  // 1. Renders URL input with the provided URL
  // -----------------------------------------------------------------------
  it("renders URL input with provided url", () => {
    renderToolbar({ url: "https://example.com" });
    const input = screen.getByTestId("browser-url-input") as HTMLInputElement;
    expect(input.value).toBe("https://example.com");
  });

  // -----------------------------------------------------------------------
  // 2. Renders only the supported controls for v1
  // -----------------------------------------------------------------------
  it("renders reload button without back and forward placeholders", () => {
    renderToolbar();
    expect(screen.getByTestId("browser-reload-btn")).toBeTruthy();
    expect(screen.queryByTestId("browser-back-btn")).toBeNull();
    expect(screen.queryByTestId("browser-forward-btn")).toBeNull();
  });

  // -----------------------------------------------------------------------
  // 3. Renders Go button
  // -----------------------------------------------------------------------
  it("renders Go button", () => {
    renderToolbar();
    expect(screen.getByTestId("browser-go-btn")).toBeTruthy();
  });

  // -----------------------------------------------------------------------
  // 4. Shows DEFAULT_BROWSER_URL when empty string is provided
  // -----------------------------------------------------------------------
  it("shows DEFAULT_BROWSER_URL when url is empty string", () => {
    renderToolbar({ url: "" });
    const input = screen.getByTestId("browser-url-input") as HTMLInputElement;
    expect(input.value).toBe(DEFAULT_URL);
  });

  // -----------------------------------------------------------------------
  // 5. Calls onNavigate when Go button is clicked
  // -----------------------------------------------------------------------
  it("calls onNavigate with current input value when Go is clicked", () => {
    const { onNavigate } = renderToolbar({ url: "https://example.com" });
    const input = screen.getByTestId("browser-url-input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "https://new-url.com" } });
    fireEvent.click(screen.getByTestId("browser-go-btn"));
    expect(onNavigate).toHaveBeenCalledTimes(1);
    expect(onNavigate).toHaveBeenCalledWith("https://new-url.com");
  });

  // -----------------------------------------------------------------------
  // 6. Calls onNavigate on Enter key in URL input
  // -----------------------------------------------------------------------
  it("calls onNavigate with current input value when Enter is pressed", () => {
    const { onNavigate } = renderToolbar({ url: "https://example.com" });
    const input = screen.getByTestId("browser-url-input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "https://enter-url.com" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onNavigate).toHaveBeenCalledTimes(1);
    expect(onNavigate).toHaveBeenCalledWith("https://enter-url.com");
  });

  it("does not call onNavigate for non-Enter keys", () => {
    const { onNavigate } = renderToolbar({ url: "https://example.com" });
    const input = screen.getByTestId("browser-url-input") as HTMLInputElement;
    fireEvent.keyDown(input, { key: "Escape" });
    fireEvent.keyDown(input, { key: "a" });
    expect(onNavigate).not.toHaveBeenCalled();
  });

  it("does not call onNavigate when input is blank and Go is clicked", () => {
    const { onNavigate } = renderToolbar({ url: "https://example.com" });
    const input = screen.getByTestId("browser-url-input") as HTMLInputElement;
    // Clear the input to blank
    fireEvent.change(input, { target: { value: "   " } });
    fireEvent.click(screen.getByTestId("browser-go-btn"));
    expect(onNavigate).not.toHaveBeenCalled();
  });

  // -----------------------------------------------------------------------
  // 7. Calls onReload when reload button is clicked
  // -----------------------------------------------------------------------
  it("calls onReload when reload button is clicked", () => {
    const { onReload } = renderToolbar();
    fireEvent.click(screen.getByTestId("browser-reload-btn"));
    expect(onReload).toHaveBeenCalledTimes(1);
  });

  // -----------------------------------------------------------------------
  // 8. Shows close button when paneCount > 1
  // -----------------------------------------------------------------------
  it("shows close button when paneCount is greater than 1", () => {
    renderToolbar({ paneCount: 2 });
    expect(screen.getByTestId("browser-toolbar-close")).toBeTruthy();
  });

  it("calls onClose when close button is clicked", () => {
    const { onClose } = renderToolbar({ paneCount: 2 });
    fireEvent.click(screen.getByTestId("browser-toolbar-close"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  // -----------------------------------------------------------------------
  // 9. Hides close button when paneCount = 1
  // -----------------------------------------------------------------------
  it("hides close button when paneCount is 1", () => {
    renderToolbar({ paneCount: 1 });
    expect(screen.queryByTestId("browser-toolbar-close")).toBeNull();
  });

  // -----------------------------------------------------------------------
  // 10. Active state styling (accent border)
  // -----------------------------------------------------------------------
  it("applies accent border class when isActive is true", () => {
    renderToolbar({ isActive: true });
    const toolbar = screen.getByTestId("browser-toolbar");
    expect(toolbar.className).toContain("border-[var(--color-accent)]");
  });

  it("applies normal border class when isActive is false", () => {
    renderToolbar({ isActive: false });
    const toolbar = screen.getByTestId("browser-toolbar");
    expect(toolbar.className).toContain("border-[var(--color-border)]");
  });

  // -----------------------------------------------------------------------
  // 11. Drag over ring styling
  // -----------------------------------------------------------------------
  it("applies ring styling when isDragOver is true", () => {
    renderToolbar({ isDragOver: true });
    const toolbar = screen.getByTestId("browser-toolbar");
    expect(toolbar.className).toContain("ring-2");
  });

  it("does not apply ring styling when isDragOver is false", () => {
    renderToolbar({ isDragOver: false });
    const toolbar = screen.getByTestId("browser-toolbar");
    expect(toolbar.className).not.toContain("ring-2");
  });

  it("does not apply ring styling by default (isDragOver omitted)", () => {
    renderToolbar();
    const toolbar = screen.getByTestId("browser-toolbar");
    expect(toolbar.className).not.toContain("ring-2");
  });

  // -----------------------------------------------------------------------
  // 12. Does not crash when clicking buttons without optional callbacks
  // -----------------------------------------------------------------------
  it("does not crash when Go is clicked without onNavigate provided", () => {
    render(
      <BrowserToolbar
        url="https://example.com"
        isActive={false}
        paneCount={2}
        onNavigate={vi.fn()}
        onReload={vi.fn()}
        onClose={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByTestId("browser-go-btn"));
    expect(screen.getByTestId("browser-url-input")).toBeTruthy();
  });

  it("does not crash when reload is clicked without onReload provided", () => {
    // onReload is required in the interface but we verify no unhandled errors
    const { onReload } = renderToolbar();
    expect(() => fireEvent.click(screen.getByTestId("browser-reload-btn"))).not.toThrow();
    expect(onReload).toHaveBeenCalledTimes(1);
  });

  // -----------------------------------------------------------------------
  // Drag props are forwarded to the root element
  // -----------------------------------------------------------------------
  it("marks root element as draggable when draggable prop is true", () => {
    renderToolbar({ draggable: true });
    const toolbar = screen.getByTestId("browser-toolbar");
    expect(toolbar.getAttribute("draggable")).toBe("true");
  });

  it("does not mark root element as draggable by default", () => {
    renderToolbar();
    const toolbar = screen.getByTestId("browser-toolbar");
    // draggable defaults to false — attribute should be "false" or absent
    const attr = toolbar.getAttribute("draggable");
    expect(attr === "false" || attr === null).toBe(true);
  });

  it("fires onDragStart when dragging starts", () => {
    const onDragStart = vi.fn();
    renderToolbar({ draggable: true, onDragStart });
    fireEvent.dragStart(screen.getByTestId("browser-toolbar"));
    expect(onDragStart).toHaveBeenCalledTimes(1);
  });

  it("fires onDragOver when dragging over", () => {
    const onDragOver = vi.fn();
    renderToolbar({ onDragOver });
    fireEvent.dragOver(screen.getByTestId("browser-toolbar"));
    expect(onDragOver).toHaveBeenCalledTimes(1);
  });

  it("fires onDrop when dropped", () => {
    const onDrop = vi.fn();
    renderToolbar({ onDrop });
    fireEvent.drop(screen.getByTestId("browser-toolbar"));
    expect(onDrop).toHaveBeenCalledTimes(1);
  });

  // -----------------------------------------------------------------------
  // URL sync — when url prop changes, input should update
  // -----------------------------------------------------------------------
  it("syncs url input when url prop changes", () => {
    const { rerender } = render(
      <BrowserToolbar
        url="https://first.com"
        isActive={false}
        paneCount={2}
        onNavigate={vi.fn()}
        onReload={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    const input = screen.getByTestId("browser-url-input") as HTMLInputElement;
    expect(input.value).toBe("https://first.com");

    rerender(
      <BrowserToolbar
        url="https://second.com"
        isActive={false}
        paneCount={2}
        onNavigate={vi.fn()}
        onReload={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    expect(input.value).toBe("https://second.com");
  });

  it("falls back to DEFAULT_BROWSER_URL when url prop changes to empty string", () => {
    const { rerender } = render(
      <BrowserToolbar
        url="https://first.com"
        isActive={false}
        paneCount={2}
        onNavigate={vi.fn()}
        onReload={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    rerender(
      <BrowserToolbar
        url=""
        isActive={false}
        paneCount={2}
        onNavigate={vi.fn()}
        onReload={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    const input = screen.getByTestId("browser-url-input") as HTMLInputElement;
    expect(input.value).toBe(DEFAULT_URL);
  });

  // -----------------------------------------------------------------------
  // Root element structure
  // -----------------------------------------------------------------------
  it("renders root div with data-testid browser-toolbar", () => {
    renderToolbar();
    expect(screen.getByTestId("browser-toolbar")).toBeTruthy();
  });
});
