import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type {
  BootstrapSnapshot,
  NewTabRequest,
  PtyOutputEvent,
  PtyResizeRequest,
  UpdatePaneCwdRequest,
  UpdatePaneProfileRequest,
  WorkspaceSettings,
  WorkspaceSnapshot,
} from "@/features/workspace/domain";
import { isTauriRuntime } from "./runtime";

type UnlistenFn = () => void;

interface BrowserBridge {
  bootstrapWorkspace: () => Promise<BootstrapSnapshot>;
  createTab: (request: NewTabRequest) => Promise<WorkspaceSnapshot>;
  closeTab: (tabId: string) => Promise<WorkspaceSnapshot>;
  setActiveTab: (tabId: string) => Promise<WorkspaceSnapshot>;
  focusPane: (tabId: string, paneId: string) => Promise<WorkspaceSnapshot>;
  updatePaneProfile: (
    request: UpdatePaneProfileRequest,
  ) => Promise<WorkspaceSnapshot>;
  updatePaneCwd: (request: UpdatePaneCwdRequest) => Promise<WorkspaceSnapshot>;
  restartPane: (paneId: string) => Promise<WorkspaceSnapshot>;
  writePty: (paneId: string, data: string) => Promise<void>;
  resizePty: (request: PtyResizeRequest) => Promise<void>;
  getAppSettings: () => Promise<WorkspaceSettings>;
  updateAppSettings: (settings: WorkspaceSettings) => Promise<WorkspaceSettings>;
  listenToPtyOutput: (
    handler: (payload: PtyOutputEvent) => void,
  ) => Promise<UnlistenFn>;
}

declare global {
  interface Window {
    __TABBY_MOCK__?: BrowserBridge;
  }
}

function getMockBridge(): BrowserBridge | null {
  if (typeof window === "undefined") {
    return null;
  }

  return window.__TABBY_MOCK__ ?? null;
}

function ensureTauri() {
  if (!isTauriRuntime()) {
    throw new Error("Live terminals are available only inside the Tauri shell.");
  }
}

async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  ensureTauri();
  return invoke<T>(command, args);
}

export function asErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  return "Unknown error";
}

export const bridge = {
  bootstrapWorkspace() {
    const mock = getMockBridge();
    if (mock) {
      return mock.bootstrapWorkspace();
    }

    return invokeCommand<BootstrapSnapshot>("bootstrap_workspace");
  },

  createTab(request: NewTabRequest) {
    const mock = getMockBridge();
    if (mock) {
      return mock.createTab(request);
    }

    return invokeCommand<WorkspaceSnapshot>("create_tab", { request });
  },

  closeTab(tabId: string) {
    const mock = getMockBridge();
    if (mock) {
      return mock.closeTab(tabId);
    }

    return invokeCommand<WorkspaceSnapshot>("close_tab", { tabId });
  },

  setActiveTab(tabId: string) {
    const mock = getMockBridge();
    if (mock) {
      return mock.setActiveTab(tabId);
    }

    return invokeCommand<WorkspaceSnapshot>("set_active_tab", { tabId });
  },

  focusPane(tabId: string, paneId: string) {
    const mock = getMockBridge();
    if (mock) {
      return mock.focusPane(tabId, paneId);
    }

    return invokeCommand<WorkspaceSnapshot>("focus_pane", { tabId, paneId });
  },

  updatePaneProfile(request: UpdatePaneProfileRequest) {
    const mock = getMockBridge();
    if (mock) {
      return mock.updatePaneProfile(request);
    }

    return invokeCommand<WorkspaceSnapshot>("update_pane_profile", { request });
  },

  updatePaneCwd(request: UpdatePaneCwdRequest) {
    const mock = getMockBridge();
    if (mock) {
      return mock.updatePaneCwd(request);
    }

    return invokeCommand<WorkspaceSnapshot>("update_pane_cwd", { request });
  },

  restartPane(paneId: string) {
    const mock = getMockBridge();
    if (mock) {
      return mock.restartPane(paneId);
    }

    return invokeCommand<WorkspaceSnapshot>("restart_pane", { paneId });
  },

  writePty(paneId: string, data: string) {
    const mock = getMockBridge();
    if (mock) {
      return mock.writePty(paneId, data);
    }

    return invokeCommand<void>("write_pty", { paneId, data });
  },

  resizePty(request: PtyResizeRequest) {
    const mock = getMockBridge();
    if (mock) {
      return mock.resizePty(request);
    }

    return invokeCommand<void>("resize_pty", { request });
  },

  getAppSettings() {
    const mock = getMockBridge();
    if (mock) {
      return mock.getAppSettings();
    }

    return invokeCommand<WorkspaceSettings>("get_app_settings");
  },

  updateAppSettings(settings: WorkspaceSettings) {
    const mock = getMockBridge();
    if (mock) {
      return mock.updateAppSettings(settings);
    }

    return invokeCommand<WorkspaceSettings>("update_app_settings", { settings });
  },

  async listenToPtyOutput(
    handler: (payload: PtyOutputEvent) => void,
  ): Promise<UnlistenFn> {
    const mock = getMockBridge();
    if (mock) {
      return mock.listenToPtyOutput(handler);
    }

    if (!isTauriRuntime()) {
      return () => undefined;
    }

    return listen<PtyOutputEvent>("pty-output", (event) => {
      handler(event.payload);
    });
  },
};
