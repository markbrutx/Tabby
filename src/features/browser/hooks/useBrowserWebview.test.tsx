import { act, render } from "@testing-library/react";
import { useEffect, useState } from "react";
import { afterEach, beforeEach, describe, expect, it, type Mock, vi } from "vitest";
import { normalizeUrl, useBrowserWebview } from "./useBrowserWebview";
import type { PaneSnapshot } from "@/features/workspace/domain";
import type { WorkspaceTransport } from "@/lib/bridge/shared";

describe("normalizeUrl", () => {
  it("returns default URL for empty input", () => {
    expect(normalizeUrl("")).toMatch(/^https?:\/\//);
    expect(normalizeUrl("   ")).toMatch(/^https?:\/\//);
  });

  it("prepends https:// when no protocol", () => {
    expect(normalizeUrl("example.com")).toBe("https://example.com");
    expect(normalizeUrl("docs.rs/tauri")).toBe("https://docs.rs/tauri");
  });

  it("preserves existing http:// protocol", () => {
    expect(normalizeUrl("http://localhost:3000")).toBe("http://localhost:3000");
  });

  it("preserves existing https:// protocol", () => {
    expect(normalizeUrl("https://example.com")).toBe("https://example.com");
  });

  it("trims whitespace", () => {
    expect(normalizeUrl("  https://example.com  ")).toBe("https://example.com");
  });
});

// ---------------------------------------------------------------------------
// useBrowserWebview — visibility tests
// ---------------------------------------------------------------------------

// Polyfill ResizeObserver for jsdom
global.ResizeObserver = class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver;

// Mock isTauriRuntime to return true so visibility effects run
vi.mock("@/lib/runtime", () => ({
  isTauriRuntime: () => true,
}));

const mockBridge = {
  setBrowserWebviewVisible: vi.fn(() => Promise.resolve()),
  createBrowserWebview: vi.fn(() => Promise.resolve()),
  closeBrowserWebview: vi.fn(() => Promise.resolve()),
  navigateBrowser: vi.fn(() => Promise.resolve()),
  listenToBrowserUrlChanged: vi.fn(() => Promise.resolve(() => {})),
  setBrowserWebviewBounds: vi.fn(() => Promise.resolve()),
} as unknown as WorkspaceTransport;

vi.mock("@/lib/bridge/TransportContext", () => ({
  useTransport: () => mockBridge,
}));

function makePaneSnapshot(id = "pane-1"): PaneSnapshot {
  return {
    id,
    sessionId: "session-1",
    profileId: "terminal",
    profileLabel: "Terminal",
    title: "Browser",
    cwd: "/tmp",
    startupCommand: null,
    status: "running",
    paneKind: "browser",
    url: "https://example.com",
  };
}

// Wrapper component that renders a sized div so containerRef attaches to DOM
// and Effect 1 can fire (setting createdRef = true).
// Mirrors BrowserPane: computes effectiveVisible = visible && !modalOpen
function HookHarness({
  pane,
  visible,
  modalOpen,
  onSetModal,
}: {
  pane: PaneSnapshot;
  visible: boolean;
  modalOpen: boolean;
  onSetModal?: (setter: (v: boolean) => void) => void;
}) {
  const [modal, setModal] = useState(modalOpen);
  const effectiveVisible = visible && !modal;
  const { containerRef } = useBrowserWebview({ pane, visible: effectiveVisible });

  // Expose setModal once on mount (setModal is referentially stable)
  useEffect(() => { onSetModal?.(setModal); }, [onSetModal, setModal]);

  return <div ref={containerRef} style={{ width: 800, height: 600 }} />;
}

describe("useBrowserWebview — visibility with modal overlay", () => {
  const mockSetVisible = mockBridge.setBrowserWebviewVisible as Mock;
  const mockCreateWebview = mockBridge.createBrowserWebview as Mock;

  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    vi.spyOn(HTMLDivElement.prototype, "getBoundingClientRect").mockReturnValue({
      x: 0, y: 0, width: 800, height: 600,
      top: 0, left: 0, right: 800, bottom: 600,
      toJSON: () => {},
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it("hides native webview when modal opens", async () => {
    const pane = makePaneSnapshot();
    let toggle: (v: boolean) => void = () => {};

    render(
      <HookHarness
        pane={pane}
        visible={true}
        modalOpen={false}
        onSetModal={(s) => { toggle = s; }}
      />,
    );

    // Flush rAF so Effect 1 sets createdRef=true, then re-run effects
    await act(async () => { vi.advanceTimersByTime(16); });

    // After creation, Effect 3 fires with visible=true
    // Open modal → effectiveVisible becomes false → triggers new Effect 3
    act(() => { toggle(true); });
    expect(mockSetVisible).toHaveBeenLastCalledWith(pane.id, false);
  });

  it("restores visibility when modal closes", async () => {
    const pane = makePaneSnapshot();
    let toggle: (v: boolean) => void = () => {};

    render(
      <HookHarness
        pane={pane}
        visible={true}
        modalOpen={true}
        onSetModal={(s) => { toggle = s; }}
      />,
    );

    await act(async () => { vi.advanceTimersByTime(16); });
    mockCreateWebview.mockClear();

    // Close modal → effectiveVisible becomes true
    act(() => { toggle(false); });
    await act(async () => { vi.advanceTimersByTime(16); });

    expect(mockCreateWebview).toHaveBeenCalledTimes(1);
    expect(mockSetVisible).not.toHaveBeenCalled();
  });

  it("hides then shows on modal toggle cycle", async () => {
    const pane = makePaneSnapshot();
    let toggle: (v: boolean) => void = () => {};

    render(
      <HookHarness
        pane={pane}
        visible={true}
        modalOpen={false}
        onSetModal={(s) => { toggle = s; }}
      />,
    );

    await act(async () => { vi.advanceTimersByTime(16); });

    // Open modal → hidden
    act(() => { toggle(true); });
    expect(mockSetVisible).toHaveBeenLastCalledWith(pane.id, false);

    // Close modal → visible again
    act(() => { toggle(false); });
    expect(mockSetVisible).toHaveBeenLastCalledWith(pane.id, true);
  });

  it("toggling modal on non-visible pane does not call bridge", async () => {
    const pane = makePaneSnapshot();
    let toggle: (v: boolean) => void = () => {};

    render(
      <HookHarness
        pane={pane}
        visible={false}
        modalOpen={false}
        onSetModal={(s) => { toggle = s; }}
      />,
    );

    await act(async () => { vi.advanceTimersByTime(16); });
    mockSetVisible.mockClear();

    // Toggle modal — effectiveVisible stays false either way
    act(() => { toggle(true); });

    // No call since visible=false && !modalOpen was already false
    // and visible=false && modalOpen=true is still false — no dep change for hook
    expect(mockSetVisible).not.toHaveBeenCalled();
  });

  it("creates the native webview when a previously hidden pane becomes visible", async () => {
    const pane = makePaneSnapshot();
    const { rerender } = render(
      <HookHarness
        pane={pane}
        visible={false}
        modalOpen={false}
      />,
    );

    await act(async () => { vi.advanceTimersByTime(16); });
    expect(mockCreateWebview).not.toHaveBeenCalled();

    rerender(
      <HookHarness
        pane={pane}
        visible={true}
        modalOpen={false}
      />,
    );

    await act(async () => { vi.advanceTimersByTime(16); });
    expect(mockCreateWebview).toHaveBeenCalledTimes(1);
    expect(mockCreateWebview).toHaveBeenCalledWith(
      pane.id,
      "https://example.com",
      expect.objectContaining({ width: 800, height: 600 }),
    );
  });

  it("does not recreate the native webview after the first successful mount", async () => {
    const pane = makePaneSnapshot();
    const { rerender } = render(
      <HookHarness
        pane={pane}
        visible={true}
        modalOpen={false}
      />,
    );

    await act(async () => { vi.advanceTimersByTime(16); });
    expect(mockCreateWebview).toHaveBeenCalledTimes(1);

    rerender(
      <HookHarness
        pane={pane}
        visible={false}
        modalOpen={false}
      />,
    );
    rerender(
      <HookHarness
        pane={pane}
        visible={true}
        modalOpen={false}
      />,
    );

    await act(async () => { vi.advanceTimersByTime(16); });
    expect(mockCreateWebview).toHaveBeenCalledTimes(1);
  });
});
