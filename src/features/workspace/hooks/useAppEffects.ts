import { useEffect } from "react";
import { bootstrapCoordinator } from "@/contexts/stores";
import { applyTheme } from "@/features/theme/application/themeApplicator";
import type { AppOrchestration } from "./useAppOrchestration";

export function useAppEffects(orchestration: AppOrchestration): void {
  const { initializeThemes, resolvedTheme, settings } = orchestration;

  useEffect(() => {
    void bootstrapCoordinator.initialize();
  }, []);

  useEffect(() => {
    initializeThemes();
  }, [initializeThemes]);

  useEffect(() => {
    applyTheme(resolvedTheme);
  }, [resolvedTheme]);

  useEffect(() => {
    if (settings?.fontSize) {
      document.documentElement.style.setProperty(
        "--ui-font-size",
        `${settings.fontSize}px`,
      );
    }
  }, [settings?.fontSize]);
}
