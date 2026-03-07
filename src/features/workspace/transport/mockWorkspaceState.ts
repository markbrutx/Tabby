import {
  BROWSER_PROFILE_ID,
  CUSTOM_PROFILE_ID,
  type LayoutPreset,
  type PaneKind,
  type PaneLifecycleEvent,
  type PaneProfile,
  type PaneSnapshot,
  type PtyOutputEvent,
  type TabSnapshot,
  type WorkspaceSettings,
  type WorkspaceSnapshot,
} from "@/features/workspace/domain";
import {
  treeFromPreset,
  treeFromCount,
  paneCountForPreset,
} from "@/features/workspace/splitTree";

export const MOCK_DEFAULT_SETTINGS: WorkspaceSettings = {
  defaultLayout: "1x1",
  defaultProfileId: "",
  defaultWorkingDirectory: "",
  defaultCustomCommand: "",
  fontSize: 13,
  theme: "midnight",
  launchFullscreen: false,
  hasCompletedOnboarding: false,
  lastWorkingDirectory: null,
};

let idCounter = 0;
export function nextId(prefix: string): string {
  idCounter += 1;
  return `${prefix}-${idCounter}`;
}

export const BUILT_IN_PROFILES: PaneProfile[] = [
  {
    id: "terminal",
    label: "Terminal",
    description: "Standard shell session \u2014 your system shell (zsh, bash)",
    startupCommand: null,
  },
  {
    id: "claude",
    label: "Claude Code",
    description: "Anthropic AI coding assistant \u2014 launches \u2018claude\u2019 CLI",
    startupCommand: "claude",
  },
  {
    id: "codex",
    label: "Codex",
    description: "OpenAI Codex agent \u2014 launches \u2018codex\u2019 CLI",
    startupCommand: "codex",
  },
  {
    id: CUSTOM_PROFILE_ID,
    label: "Custom",
    description: "Run any command of your choice",
    startupCommand: null,
  },
  {
    id: BROWSER_PROFILE_ID,
    label: "Browser",
    description: "Launch Google Chrome with a specific profile and URL",
    startupCommand: null,
  },
];

export function resolveProfile(
  profileId: string,
  startupCommand: string | null,
): { id: string; label: string; startupCommand: string | null } {
  const profile = BUILT_IN_PROFILES.find((p) => p.id === profileId);
  if (!profile) {
    return { id: profileId, label: profileId, startupCommand };
  }

  return {
    id: profile.id,
    label: profile.label,
    startupCommand:
      profile.id === CUSTOM_PROFILE_ID ? startupCommand : profile.startupCommand,
  };
}

export function createPane(
  cwd: string,
  profileId: string,
  startupCommand: string | null,
  index: number,
  url?: string | null,
): PaneSnapshot {
  const resolved = resolveProfile(profileId, startupCommand);
  const isBrowser = profileId === BROWSER_PROFILE_ID;
  const paneKind: PaneKind = isBrowser ? "browser" : "terminal";

  return {
    id: nextId("pane"),
    sessionId: isBrowser ? nextId("browser") : nextId("session"),
    title: `Pane ${index + 1}`,
    cwd,
    profileId: resolved.id,
    profileLabel: resolved.label,
    startupCommand: resolved.startupCommand,
    status: "running",
    paneKind,
    url: isBrowser ? (url ?? null) : null,
  };
}

export interface PaneSlotInput {
  cwd: string;
  profileId: string;
  startupCommand: string | null;
  url?: string | null;
}

export function createTab(
  preset: LayoutPreset,
  slots: PaneSlotInput[],
  tabIndex: number,
  useCountLayout: boolean,
): TabSnapshot {
  const panes: PaneSnapshot[] = slots.map((slot, i) =>
    createPane(slot.cwd, slot.profileId, slot.startupCommand, i, slot.url),
  );

  const paneIds = panes.map((p) => p.id);
  const layout = useCountLayout
    ? treeFromCount(paneIds)
    : treeFromPreset(preset, paneIds);

  return {
    id: nextId("tab"),
    title: `Workspace ${tabIndex}`,
    layout,
    panes,
    activePaneId: panes[0]?.id ?? "",
  };
}

export interface MockState {
  tabs: TabSnapshot[];
  activeTabId: string;
  settings: WorkspaceSettings;
  nextTabIndex: number;
  outputListeners: Array<(payload: PtyOutputEvent) => void>;
  lifecycleListeners: Array<(payload: PaneLifecycleEvent) => void>;
}

export function snapshot(state: MockState): WorkspaceSnapshot {
  return {
    activeTabId: state.activeTabId,
    tabs: state.tabs,
  };
}

export function findTabIndex(state: MockState, tabId: string): number {
  const index = state.tabs.findIndex((t) => t.id === tabId);
  if (index === -1) {
    throw new Error(`Tab not found: ${tabId}`);
  }
  return index;
}

export function findPane(
  state: MockState,
  paneId: string,
): { tabIndex: number; paneIndex: number } {
  for (let ti = 0; ti < state.tabs.length; ti++) {
    const pi = state.tabs[ti].panes.findIndex((p) => p.id === paneId);
    if (pi !== -1) {
      return { tabIndex: ti, paneIndex: pi };
    }
  }
  throw new Error(`Pane not found: ${paneId}`);
}

export function emitMockOutput(state: MockState, paneId: string, sessionId: string, text: string) {
  const payload: PtyOutputEvent = { paneId, sessionId, chunk: text };
  for (const listener of state.outputListeners) {
    listener(payload);
  }
}

export function uniformSlots(
  preset: LayoutPreset,
  cwd: string,
  profileId: string,
  startupCommand: string | null,
): PaneSlotInput[] {
  const count = paneCountForPreset(preset);
  return Array.from({ length: count }, () => ({
    cwd,
    profileId,
    startupCommand,
  }));
}

export function createMockState(): MockState {
  return {
    tabs: [],
    activeTabId: "",
    settings: { ...MOCK_DEFAULT_SETTINGS },
    nextTabIndex: 1,
    outputListeners: [],
    lifecycleListeners: [],
  };
}

export function addTab(
  state: MockState,
  preset: LayoutPreset,
  slots: PaneSlotInput[],
  useCountLayout = false,
): WorkspaceSnapshot {
  const tab = createTab(preset, slots, state.nextTabIndex, useCountLayout);
  state.tabs = [...state.tabs, tab];
  state.activeTabId = tab.id;
  state.nextTabIndex += 1;

  for (const pane of tab.panes) {
    if (pane.paneKind === "browser") continue;
    setTimeout(() => {
      emitMockOutput(
        state,
        pane.id,
        pane.sessionId,
        `\x1b[36m${pane.profileLabel}\x1b[0m session started in \x1b[33m${pane.cwd}\x1b[0m\r\n\r\n` +
        `\x1b[90m[mock] Type here \u2014 keystrokes are echoed locally.\x1b[0m\r\n` +
        `\x1b[32m\u279c\x1b[0m  `,
      );
    }, 80);
  }

  return snapshot(state);
}
