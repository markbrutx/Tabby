import { useCallback, useState } from "react";
import { emit } from "@tauri-apps/api/event";
import { isTauriRuntime } from "@/lib/runtime";

type ConfirmAction =
  | { type: "closePane"; paneId: string }
  | { type: "closeTab"; tabId: string }
  | { type: "quitApp" };

interface MinimalWorkspace {
  tabs: Array<{ id: string; panes: unknown[] }>;
}

function buildConfirmMessage(
  action: ConfirmAction,
  workspace: MinimalWorkspace,
): { title: string; message: string } {
  switch (action.type) {
    case "closePane":
      return {
        title: "Close pane?",
        message: "The terminal session will be terminated.",
      };
    case "closeTab": {
      const tab = workspace.tabs.find((t) => t.id === action.tabId);
      const count = tab ? tab.panes.length : 0;
      return {
        title: "Close workspace?",
        message: `All ${count} session${count !== 1 ? "s" : ""} will be terminated.`,
      };
    }
    case "quitApp": {
      const tabCount = workspace.tabs.length;
      return {
        title: "Quit Tabby?",
        message: `All sessions across ${tabCount} workspace${tabCount !== 1 ? "s" : ""} will be terminated.`,
      };
    }
  }
}

interface UseConfirmActionOpts {
  workspace: MinimalWorkspace;
  closePane: (paneId: string) => Promise<void>;
  closeTab: (tabId: string) => Promise<void>;
}

export function useConfirmAction({
  workspace,
  closePane,
  closeTab,
}: UseConfirmActionOpts) {
  const [action, setAction] = useState<ConfirmAction | null>(null);

  const requestClosePane = useCallback((paneId: string) => {
    setAction({ type: "closePane", paneId });
  }, []);

  const requestCloseTab = useCallback((tabId: string) => {
    setAction({ type: "closeTab", tabId });
  }, []);

  const requestQuitApp = useCallback(() => {
    setAction({ type: "quitApp" });
  }, []);

  const confirm = useCallback(() => {
    if (!action) return;

    switch (action.type) {
      case "closePane":
        void closePane(action.paneId);
        break;
      case "closeTab":
        void closeTab(action.tabId);
        break;
      case "quitApp":
        if (isTauriRuntime()) {
          void emit("quit-confirmed");
        }
        break;
    }

    setAction(null);
  }, [action, closePane, closeTab]);

  const cancel = useCallback(() => {
    setAction(null);
  }, []);

  const message = action ? buildConfirmMessage(action, workspace) : null;

  return {
    action,
    message,
    requestClosePane,
    requestCloseTab,
    requestQuitApp,
    confirm,
    cancel,
  };
}
