import { createContext, useContext } from "react";
import type { WorkspaceTransport } from "./shared";

export const TransportContext = createContext<WorkspaceTransport | null>(null);

export function useTransport(): WorkspaceTransport {
  const transport = useContext(TransportContext);
  if (!transport) {
    throw new Error("useTransport must be used within a TransportContext.Provider");
  }
  return transport;
}
