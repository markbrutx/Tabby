import { create } from "zustand";

interface CollapseState {
  /** tabId → Set of collapsed paneIds */
  collapsed: Record<string, ReadonlySet<string>>;

  /** Toggle collapse. Returns true if pane is now collapsed. */
  toggleCollapse: (tabId: string, paneId: string, allPaneIds: string[]) => boolean;

  /** Check if a specific pane is collapsed */
  isCollapsed: (tabId: string, paneId: string) => boolean;

  /** Get the full collapsed set for a tab */
  getCollapsedSet: (tabId: string) => ReadonlySet<string>;

  /** Remove all collapse state for a tab */
  cleanupTab: (tabId: string) => void;

  /** Remove a single pane from collapse tracking */
  cleanupPane: (tabId: string, paneId: string) => void;

  /** Expand a specific pane (e.g. after splitting it) */
  expandPane: (tabId: string, paneId: string) => void;
}

const EMPTY_SET: ReadonlySet<string> = new Set();

export const useCollapseStore = create<CollapseState>((set, get) => ({
  collapsed: {},

  toggleCollapse(tabId, paneId, allPaneIds) {
    const state = get();
    const current = state.collapsed[tabId] ?? EMPTY_SET;

    if (current.has(paneId)) {
      // Expand
      const next = new Set(current);
      next.delete(paneId);
      set({ collapsed: { ...state.collapsed, [tabId]: next } });
      return false;
    }

    // Collapse — guard: at least one pane must remain expanded
    const expandedCount = allPaneIds.filter((id) => !current.has(id)).length;
    if (expandedCount <= 1) return false;

    const next = new Set(current);
    next.add(paneId);
    set({ collapsed: { ...state.collapsed, [tabId]: next } });
    return true;
  },

  isCollapsed(tabId, paneId) {
    const current = get().collapsed[tabId];
    return current ? current.has(paneId) : false;
  },

  getCollapsedSet(tabId) {
    return get().collapsed[tabId] ?? EMPTY_SET;
  },

  cleanupTab(tabId) {
    const { [tabId]: _, ...rest } = get().collapsed;
    set({ collapsed: rest });
  },

  cleanupPane(tabId, paneId) {
    const state = get();
    const current = state.collapsed[tabId];
    if (!current || !current.has(paneId)) return;
    const next = new Set(current);
    next.delete(paneId);
    set({ collapsed: { ...state.collapsed, [tabId]: next } });
  },

  expandPane(tabId, paneId) {
    const state = get();
    const current = state.collapsed[tabId];
    if (!current || !current.has(paneId)) return;
    const next = new Set(current);
    next.delete(paneId);
    set({ collapsed: { ...state.collapsed, [tabId]: next } });
  },
}));
