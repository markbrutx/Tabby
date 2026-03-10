import { useCallback, useEffect, useRef, useState } from "react";
import type { WorkspaceReadModel } from "@/features/workspace/domain/models";
import type { WizardTab } from "@/features/workspace/store/types";

function makeWizardTab(): WizardTab {
  return {
    id: `__wizard_${Date.now()}__`,
    title: "New Workspace",
  };
}

export interface WizardState {
  readonly wizardTab: WizardTab | null;
  readonly openSetupWizard: () => void;
  readonly closeSetupWizard: () => void;
}

export function useWizardState(workspace: WorkspaceReadModel | null): WizardState {
  const [wizardTab, setWizardTab] = useState<WizardTab | null>(null);
  const hasAutoOpened = useRef(false);

  // Auto-open wizard when workspace has no tabs (initial load or after closing all tabs)
  useEffect(() => {
    if (!workspace) return;

    if (workspace.tabs.length === 0 && !wizardTab) {
      setWizardTab(makeWizardTab());
      hasAutoOpened.current = true;
    } else if (workspace.tabs.length > 0 && wizardTab) {
      // Auto-close wizard when tabs appear (e.g., after creating a tab)
      setWizardTab(null);
    }
  }, [workspace?.tabs.length]); // eslint-disable-line react-hooks/exhaustive-deps

  const openSetupWizard = useCallback(() => {
    setWizardTab(makeWizardTab());
  }, []);

  const closeSetupWizard = useCallback(() => {
    if (workspace && workspace.tabs.length === 0) {
      return; // Can't close wizard if there are no tabs
    }
    setWizardTab(null);
  }, [workspace]);

  return { wizardTab, openSetupWizard, closeSetupWizard };
}
