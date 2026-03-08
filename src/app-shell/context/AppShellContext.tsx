import { createContext, useContext } from "react";
import type { AppShellClients } from "@/app-shell/clients/shared";

export const AppShellContext = createContext<AppShellClients | null>(null);

function useAppShell(): AppShellClients {
  const clients = useContext(AppShellContext);
  if (!clients) {
    throw new Error("AppShellContext is missing");
  }
  return clients;
}

export function useWorkspaceClient() {
  return useAppShell().workspace;
}

export function useSettingsClient() {
  return useAppShell().settings;
}

export function useRuntimeClient() {
  return useAppShell().runtime;
}
