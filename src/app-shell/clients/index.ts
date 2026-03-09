import { createTauriShellClients } from "./shared";

export const shellClients = createTauriShellClients();

export type {
  RuntimeClient,
  SettingsClient,
  UnlistenFn,
  WorkspaceClient,
} from "./shared";

export { asErrorMessage } from "./shared";
