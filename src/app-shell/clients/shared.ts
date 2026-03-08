import { listen } from "@tauri-apps/api/event";
import { commands, type Result } from "@/contracts/tauri-bindings";
import type {
  BrowserSurfaceCommandDto,
  PaneRuntimeView,
  RuntimeCommandDto,
  SettingsCommandDto,
  SettingsProjectionUpdatedEvent,
  SettingsView as WorkspaceSettings,
  TerminalOutputEvent,
  WorkspaceBootstrapView,
  WorkspaceCommandDto,
  WorkspaceView,
} from "@/contracts/tauri-bindings";

export type UnlistenFn = () => void;

export interface WorkspaceClient {
  bootstrap: () => Promise<WorkspaceBootstrapView>;
  dispatch: (command: WorkspaceCommandDto) => Promise<WorkspaceView>;
  listenProjectionUpdated: (
    handler: (workspace: WorkspaceView) => void,
  ) => Promise<UnlistenFn>;
}

export interface SettingsClient {
  dispatch: (command: SettingsCommandDto) => Promise<WorkspaceSettings>;
  listenProjectionUpdated: (
    handler: (payload: SettingsProjectionUpdatedEvent) => void,
  ) => Promise<UnlistenFn>;
}

export interface RuntimeClient {
  dispatch: (command: RuntimeCommandDto) => Promise<void>;
  dispatchBrowserSurface: (command: BrowserSurfaceCommandDto) => Promise<void>;
  listenStatusChanged: (
    handler: (runtime: PaneRuntimeView) => void,
  ) => Promise<UnlistenFn>;
  listenTerminalOutput: (
    handler: (payload: TerminalOutputEvent) => void,
  ) => Promise<UnlistenFn>;
}

export interface AppShellClients {
  workspace: WorkspaceClient;
  settings: SettingsClient;
  runtime: RuntimeClient;
}

export const WORKSPACE_PROJECTION_UPDATED_EVENT = "workspace_projection_updated";
export const SETTINGS_PROJECTION_UPDATED_EVENT = "settings_projection_updated";
export const RUNTIME_STATUS_CHANGED_EVENT = "runtime_status_changed";
export const TERMINAL_OUTPUT_RECEIVED_EVENT = "terminal_output_received";

export function unwrapResult<T>(result: Result<T, unknown>): T {
  if (result.status === "ok") {
    return result.data;
  }

  throw new Error(asErrorMessage(result.error));
}

export function asErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  if (error && typeof error === "object") {
    const values = Object.values(error as Record<string, unknown>);
    const message = values.find((value) => typeof value === "string");
    if (typeof message === "string") {
      return message;
    }
  }

  return "Unknown error";
}

export function createTauriShellClients(): AppShellClients {
  return {
    workspace: {
      async bootstrap() {
        return unwrapResult(await commands.bootstrapShell());
      },
      async dispatch(command) {
        return unwrapResult(await commands.dispatchWorkspaceCommand(command));
      },
      async listenProjectionUpdated(handler) {
        return listen<{ workspace: WorkspaceView }>(
          WORKSPACE_PROJECTION_UPDATED_EVENT,
          (event) => handler(event.payload.workspace),
        );
      },
    },
    settings: {
      async dispatch(command) {
        return unwrapResult(await commands.dispatchSettingsCommand(command));
      },
      async listenProjectionUpdated(handler) {
        return listen<SettingsProjectionUpdatedEvent>(
          SETTINGS_PROJECTION_UPDATED_EVENT,
          (event) => handler(event.payload),
        );
      },
    },
    runtime: {
      async dispatch(command) {
        unwrapResult(await commands.dispatchRuntimeCommand(command));
      },
      async dispatchBrowserSurface(command) {
        unwrapResult(await commands.dispatchBrowserSurfaceCommand(command));
      },
      async listenStatusChanged(handler) {
        return listen<{ runtime: PaneRuntimeView }>(
          RUNTIME_STATUS_CHANGED_EVENT,
          (event) => handler(event.payload.runtime),
        );
      },
      async listenTerminalOutput(handler) {
        return listen<TerminalOutputEvent>(
          TERMINAL_OUTPUT_RECEIVED_EVENT,
          (event) => handler(event.payload),
        );
      },
    },
  };
}
