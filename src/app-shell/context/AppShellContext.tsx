import { createContext } from "react";
import type { AppShellClients } from "@/app-shell/clients/shared";

export const AppShellContext = createContext<AppShellClients | null>(null);
