import { getBrowserTransport } from "@/lib/bridge/browserTransport";
import { createTauriTransport } from "@/lib/bridge/tauriTransport";
import { asErrorMessage, type WorkspaceTransport } from "@/lib/bridge/shared";

const tauriTransport = createTauriTransport();

function resolveTransport(): WorkspaceTransport {
  return getBrowserTransport() ?? tauriTransport;
}

export const bridge: WorkspaceTransport = {
  bootstrapWorkspace: () => resolveTransport().bootstrapWorkspace(),
  createTab: (request) => resolveTransport().createTab(request),
  closeTab: (tabId) => resolveTransport().closeTab(tabId),
  setActiveTab: (tabId) => resolveTransport().setActiveTab(tabId),
  focusPane: (tabId, paneId) => resolveTransport().focusPane(tabId, paneId),
  updatePaneProfile: (request) => resolveTransport().updatePaneProfile(request),
  updatePaneCwd: (request) => resolveTransport().updatePaneCwd(request),
  restartPane: (paneId) => resolveTransport().restartPane(paneId),
  writePty: (paneId, data) => resolveTransport().writePty(paneId, data),
  resizePty: (request) => resolveTransport().resizePty(request),
  getAppSettings: () => resolveTransport().getAppSettings(),
  updateAppSettings: (settings) => resolveTransport().updateAppSettings(settings),
  listenToPtyOutput: (handler) => resolveTransport().listenToPtyOutput(handler),
};

export { asErrorMessage };
export type { UnlistenFn, WorkspaceTransport } from "@/lib/bridge/shared";
