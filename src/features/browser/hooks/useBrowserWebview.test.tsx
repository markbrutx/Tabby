import { act, render } from "@testing-library/react";
import { useEffect, useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { RuntimeClient } from "@/app-shell/clients";
import type { BrowserSurfaceCommandDto } from "@/contracts/tauri-bindings";
import { normalizeUrl, useBrowserWebview } from "./useBrowserWebview";
import type { PaneSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import { createRuntimeStore } from "@/features/runtime/application/store";

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

const browserSurfaceCommands: BrowserSurfaceCommandDto[] = [];

const mockBridge = {
  dispatch: vi.fn(() => Promise.resolve()),
  dispatchBrowserSurface: vi.fn(async (command: BrowserSurfaceCommandDto) => {
    browserSurfaceCommands.push(command);
  }),
  listenStatusChanged: vi.fn(() => Promise.resolve(() => {})),
  listenTerminalOutput: vi.fn(() => Promise.resolve(() => {})),
} satisfies RuntimeClient;

// Create a real store backed by the mock client and wire it into the module mock
const mockStore = createRuntimeStore({
  runtimeClient: mockBridge,
  initTerminalDispatcher: vi.fn().mockResolvedValue(undefined),
  teardownTerminalDispatcher: vi.fn(),
});

vi.mock("@/contexts/stores", () => ({
  useRuntimeStore: (selector: (state: unknown) => unknown) => mockStore(selector),
}));

function makePaneSnapshot(id = "pane-1"): PaneSnapshotModel {
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
    spec: {
      kind: "browser",
      initialUrl: "https://example.com",
    },
    runtime: {
      paneId: id,
      runtimeSessionId: "session-1",
      kind: "browser",
      status: "running",
      lastError: null,
      browserLocation: "https://example.com",
      terminalCwd: null,
    },
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
  pane: PaneSnapshotModel;
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

function renderHarness(props: {
  pane: PaneSnapshotModel;
  visible: boolean;
  modalOpen: boolean;
  onSetModal?: (setter: (v: boolean) => void) => void;
}) {
  return render(<HookHarness {...props} />);
}

describe("useBrowserWebview — visibility with modal overlay", () => {
  const mockDispatchBrowserSurface = mockBridge.dispatchBrowserSurface;

  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    browserSurfaceCommands.length = 0;
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

    renderHarness({
      pane,
      visible: true,
      modalOpen: false,
      onSetModal: (s) => { toggle = s; },
    });

    // Flush rAF so Effect 1 sets createdRef=true, then re-run effects
    await act(async () => { vi.advanceTimersByTime(16); });

    // After creation, Effect 3 fires with visible=true
    // Open modal → effectiveVisible becomes false → triggers new Effect 3
    act(() => { toggle(true); });
    expect(mockDispatchBrowserSurface).toHaveBeenLastCalledWith({
      kind: "setVisible",
      pane_id: pane.id,
      visible: false,
    });
  });

  it("restores visibility when modal closes", async () => {
    const pane = makePaneSnapshot();
    let toggle: (v: boolean) => void = () => {};

    renderHarness({
      pane,
      visible: true,
      modalOpen: true,
      onSetModal: (s) => { toggle = s; },
    });

    await act(async () => { vi.advanceTimersByTime(16); });
    mockDispatchBrowserSurface.mockClear();

    // Close modal → effectiveVisible becomes true
    act(() => { toggle(false); });
    await act(async () => { vi.advanceTimersByTime(16); });

    expect(mockDispatchBrowserSurface).toHaveBeenCalledWith({
      kind: "ensure",
      pane_id: pane.id,
      url: "https://example.com",
      bounds: expect.objectContaining({ width: 800, height: 600 }),
    });
  });

  it("hides then shows on modal toggle cycle", async () => {
    const pane = makePaneSnapshot();
    let toggle: (v: boolean) => void = () => {};

    renderHarness({
      pane,
      visible: true,
      modalOpen: false,
      onSetModal: (s) => { toggle = s; },
    });

    await act(async () => { vi.advanceTimersByTime(16); });

    // Open modal → hidden
    act(() => { toggle(true); });
    expect(mockDispatchBrowserSurface).toHaveBeenLastCalledWith({
      kind: "setVisible",
      pane_id: pane.id,
      visible: false,
    });

    // Close modal → visible again
    act(() => { toggle(false); });
    expect(mockDispatchBrowserSurface).toHaveBeenLastCalledWith({
      kind: "setVisible",
      pane_id: pane.id,
      visible: true,
    });
  });

  it("toggling modal on non-visible pane does not call bridge", async () => {
    const pane = makePaneSnapshot();
    let toggle: (v: boolean) => void = () => {};

    renderHarness({
      pane,
      visible: false,
      modalOpen: false,
      onSetModal: (s) => { toggle = s; },
    });

    await act(async () => { vi.advanceTimersByTime(16); });
    mockDispatchBrowserSurface.mockClear();

    // Toggle modal — effectiveVisible stays false either way
    act(() => { toggle(true); });

    // No call since visible=false && !modalOpen was already false
    // and visible=false && modalOpen=true is still false — no dep change for hook
    expect(mockDispatchBrowserSurface).not.toHaveBeenCalled();
  });

  it("creates the native webview when a previously hidden pane becomes visible", async () => {
    const pane = makePaneSnapshot();
    const { rerender } = renderHarness({
      pane,
      visible: false,
      modalOpen: false,
    });

    await act(async () => { vi.advanceTimersByTime(16); });
    expect(mockDispatchBrowserSurface).not.toHaveBeenCalled();

    rerender(
      <HookHarness
        pane={pane}
        visible={true}
        modalOpen={false}
      />,
    );

    await act(async () => { vi.advanceTimersByTime(16); });
    expect(mockDispatchBrowserSurface).toHaveBeenCalledWith({
      kind: "ensure",
      pane_id: pane.id,
      url: "https://example.com",
      bounds: expect.objectContaining({ width: 800, height: 600 }),
    });
  });

  it("does not recreate the native webview after the first successful mount", async () => {
    const pane = makePaneSnapshot();
    const { rerender } = renderHarness({
      pane,
      visible: true,
      modalOpen: false,
    });

    await act(async () => { vi.advanceTimersByTime(16); });
    expect(mockDispatchBrowserSurface).toHaveBeenCalledWith({
      kind: "ensure",
      pane_id: pane.id,
      url: "https://example.com",
      bounds: expect.objectContaining({ width: 800, height: 600 }),
    });

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
    expect(
      browserSurfaceCommands.filter((command) => command.kind === "ensure"),
    ).toHaveLength(1);
  });

  it("hides instead of closing the native webview during unmount cleanup", async () => {
    const pane = makePaneSnapshot();
    const { unmount } = renderHarness({
      pane,
      visible: true,
      modalOpen: false,
    });

    await act(async () => { vi.advanceTimersByTime(16); });
    expect(mockDispatchBrowserSurface).toHaveBeenCalledWith({
      kind: "ensure",
      pane_id: pane.id,
      url: "https://example.com",
      bounds: expect.objectContaining({ width: 800, height: 600 }),
    });

    mockDispatchBrowserSurface.mockClear();

    unmount();

    expect(mockDispatchBrowserSurface).toHaveBeenCalledWith({
      kind: "setVisible",
      pane_id: pane.id,
      visible: false,
    });
  });
});
