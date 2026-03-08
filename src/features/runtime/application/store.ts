import { create } from "zustand";
import type { PaneRuntimeView } from "@/contracts/tauri-bindings";
import type { BrowserBounds, RuntimeReadModel } from "@/features/runtime/domain/models";
import type { RuntimeClient } from "@/app-shell/clients";
import { mapRuntimeFromDto } from "@/features/runtime/application/snapshot-mappers";

export interface RuntimeState {
  runtimes: Record<string, RuntimeReadModel>;
  initializeListeners: () => Promise<void>;
  loadBootstrap: (runtimes: PaneRuntimeView[]) => void;

  // Terminal actions
  writeTerminalInput: (paneId: string, input: string) => Promise<void>;
  observeTerminalCwd: (paneId: string, workingDirectory: string) => Promise<void>;
  resizeTerminal: (paneId: string, cols: number, rows: number) => Promise<void>;
  initTerminalOutputDispatcher: () => Promise<void>;
  teardownTerminalOutputDispatcher: () => void;

  // Browser actions
  ensureBrowserSurface: (paneId: string, url: string, bounds: BrowserBounds) => Promise<void>;
  setBrowserBounds: (paneId: string, bounds: BrowserBounds) => Promise<void>;
  setBrowserVisible: (paneId: string, visible: boolean) => Promise<void>;
  navigateBrowser: (paneId: string, url: string) => Promise<void>;
}

function toRuntimeMap(dtos: PaneRuntimeView[]): Record<string, RuntimeReadModel> {
  return Object.fromEntries(dtos.map((dto) => [dto.paneId, mapRuntimeFromDto(dto)]));
}

export interface RuntimeStoreDeps {
  runtimeClient: RuntimeClient;
  initTerminalDispatcher: (client: RuntimeClient) => Promise<void>;
  teardownTerminalDispatcher: () => void;
}

export function createRuntimeStore(deps: RuntimeStoreDeps) {
  const { runtimeClient } = deps;
  let runtimeListenersReady: Promise<void> | null = null;

  return create<RuntimeState>((set, get) => ({
    runtimes: {},

    async initializeListeners() {
      if (runtimeListenersReady) {
        await runtimeListenersReady;
        return;
      }

      runtimeListenersReady = runtimeClient
        .listenStatusChanged((dto) => {
          const runtime = mapRuntimeFromDto(dto);
          set((state) => ({
            runtimes: {
              ...state.runtimes,
              [runtime.paneId]: runtime,
            },
          }));
        })
        .then(() => undefined);

      await runtimeListenersReady;
    },

    loadBootstrap(runtimes) {
      set({ runtimes: toRuntimeMap(runtimes) });
      void get().initializeListeners();
    },

    // Terminal actions
    async writeTerminalInput(paneId, input) {
      await runtimeClient.dispatch({
        kind: "writeTerminalInput",
        pane_id: paneId,
        input,
      });
    },

    async observeTerminalCwd(paneId, workingDirectory) {
      await runtimeClient.dispatch({
        kind: "observeTerminalCwd",
        pane_id: paneId,
        working_directory: workingDirectory,
      });
    },

    async resizeTerminal(paneId, cols, rows) {
      await runtimeClient.dispatch({
        kind: "resizeTerminal",
        pane_id: paneId,
        cols,
        rows,
      });
    },

    async initTerminalOutputDispatcher() {
      await deps.initTerminalDispatcher(runtimeClient);
    },

    teardownTerminalOutputDispatcher() {
      deps.teardownTerminalDispatcher();
    },

    // Browser actions
    async ensureBrowserSurface(paneId, url, bounds) {
      await runtimeClient.dispatchBrowserSurface({
        kind: "ensure",
        pane_id: paneId,
        url,
        bounds,
      });
    },

    async setBrowserBounds(paneId, bounds) {
      await runtimeClient.dispatchBrowserSurface({
        kind: "setBounds",
        pane_id: paneId,
        bounds,
      });
    },

    async setBrowserVisible(paneId, visible) {
      await runtimeClient.dispatchBrowserSurface({
        kind: "setVisible",
        pane_id: paneId,
        visible,
      });
    },

    async navigateBrowser(paneId, url) {
      await runtimeClient.dispatch({
        kind: "navigateBrowser",
        pane_id: paneId,
        url,
      });
    },

  }));
}
