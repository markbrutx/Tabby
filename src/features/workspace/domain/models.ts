/**
 * Internal workspace domain models.
 *
 * These types are independent of transport/generated bindings and use
 * camelCase field names throughout. Mappers in the application layer
 * convert between these models and the wire-format DTOs.
 */

// ---------------------------------------------------------------------------
// Layout primitives
// ---------------------------------------------------------------------------

export type SplitDirection = "horizontal" | "vertical";

export type SplitNode =
  | { type: "pane"; paneId: string }
  | {
      type: "split";
      direction: SplitDirection;
      ratio: number;
      first: SplitNode;
      second: SplitNode;
    };

// ---------------------------------------------------------------------------
// Pane specification (what a pane _should_ run)
// ---------------------------------------------------------------------------

export interface TerminalPaneSpec {
  readonly kind: "terminal";
  readonly launchProfileId: string;
  readonly workingDirectory: string;
  readonly commandOverride: string | null;
}

export interface BrowserPaneSpec {
  readonly kind: "browser";
  readonly initialUrl: string;
}

export type PaneSpec = TerminalPaneSpec | BrowserPaneSpec;

// ---------------------------------------------------------------------------
// Read models (projections consumed by UI)
// ---------------------------------------------------------------------------

export interface PaneReadModel {
  readonly paneId: string;
  readonly title: string;
  readonly spec: PaneSpec;
}

export interface TabReadModel {
  readonly tabId: string;
  readonly title: string;
  readonly layout: SplitNode;
  readonly panes: readonly PaneReadModel[];
  readonly activePaneId: string;
}

export interface WorkspaceReadModel {
  readonly activeTabId: string;
  readonly tabs: readonly TabReadModel[];
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

export const CUSTOM_PROFILE_ID = "custom" as const;
export const BROWSER_PROFILE_ID = "browser" as const;
export const DEFAULT_BROWSER_URL = "https://google.com" as const;
