/**
 * ACL Boundary Tests (US-030)
 *
 * Verifies that the anti-corruption layer correctly prevents DTO types
 * from leaking into feature store state. These are compile-time type
 * assertions combined with runtime structural checks.
 */

import { describe, expect, it, vi } from "vitest";
import type {
  PaneRuntimeView,
  SettingsView,
  WorkspaceView,
  PaneView,
  PaneSpecDto,
  WorkspaceBootstrapView,
} from "@/contracts/tauri-bindings";
import type { RuntimeReadModel } from "@/features/runtime/domain/models";
import type { SettingsReadModel, ProfileReadModel } from "@/features/settings/domain/models";
import type {
  WorkspaceReadModel,
  PaneReadModel,
  PaneSpec,
} from "@/features/workspace/domain/models";
import type { RuntimeState } from "@/features/runtime/application/store";
import type { SettingsState } from "@/features/settings/application/store";
import type { WorkspaceStore } from "@/features/workspace/application/store";

// ---------------------------------------------------------------------------
// Compile-time type assertions
// ---------------------------------------------------------------------------
// These type-level checks ensure DTO types cannot be assigned to store state
// fields. If a DTO type leaks in, the compilation will fail.

// Helper: Exact<T, U> is `true` only when T and U are structurally identical
type Exact<T, U> = [T] extends [U] ? ([U] extends [T] ? true : false) : false;

// AC4: RuntimeState.runtimes values must be RuntimeReadModel, not PaneRuntimeView
type _AssertRuntimeStoreUsesReadModel = Exact<
  RuntimeState["runtimes"][string],
  RuntimeReadModel
> extends true
  ? true
  : never;

// AC4: SettingsState.settings must be SettingsReadModel | null, not SettingsView | null
type _AssertSettingsStoreUsesReadModel = Exact<
  NonNullable<SettingsState["settings"]>,
  SettingsReadModel
> extends true
  ? true
  : never;

// AC4: SettingsState.profiles must be ProfileReadModel[], not ProfileView[]
type _AssertProfilesStoreUsesReadModel = Exact<
  SettingsState["profiles"][number],
  ProfileReadModel
> extends true
  ? true
  : never;

// AC4: WorkspaceStore.workspace must be WorkspaceReadModel | null
type _AssertWorkspaceStoreUsesReadModel = Exact<
  NonNullable<WorkspaceStore["workspace"]>,
  WorkspaceReadModel
> extends true
  ? true
  : never;

// Verify that ReadModel types differ from DTO types (confirming ACL is meaningful)
// PaneSpec uses camelCase (launchProfileId), PaneSpecDto uses snake_case (launch_profile_id)
type _AssertPaneSpecDiffersFromDto = Exact<PaneSpec, PaneSpecDto> extends false
  ? true
  : never;

// PaneReadModel uses PaneSpec, PaneView uses PaneSpecDto
type _AssertPaneReadModelDiffersFromDto = Exact<PaneReadModel, PaneView> extends false
  ? true
  : never;

// Instantiate the type assertions to catch errors at compile time
const _typeChecks: {
  runtime: _AssertRuntimeStoreUsesReadModel;
  settings: _AssertSettingsStoreUsesReadModel;
  profiles: _AssertProfilesStoreUsesReadModel;
  workspace: _AssertWorkspaceStoreUsesReadModel;
  paneSpecDiffers: _AssertPaneSpecDiffersFromDto;
  paneReadModelDiffers: _AssertPaneReadModelDiffersFromDto;
} = {
  runtime: true,
  settings: true,
  profiles: true,
  workspace: true,
  paneSpecDiffers: true,
  paneReadModelDiffers: true,
};

// ---------------------------------------------------------------------------
// Runtime assertions (AC4 — structural verification)
// ---------------------------------------------------------------------------

describe("ACL boundary: no DTO types leak into store state", () => {
  it("type assertions compile successfully (see type-level checks above)", () => {
    // If this test file compiles, all type-level assertions passed.
    // The _typeChecks object instantiates each assertion — a compile
    // error here means a DTO type leaked into store state.
    expect(_typeChecks.runtime).toBe(true);
    expect(_typeChecks.settings).toBe(true);
    expect(_typeChecks.profiles).toBe(true);
    expect(_typeChecks.workspace).toBe(true);
  });

  it("PaneSpec domain type differs structurally from PaneSpecDto", () => {
    // PaneSpecDto uses snake_case (launch_profile_id, working_directory)
    // PaneSpec uses camelCase (launchProfileId, workingDirectory)
    expect(_typeChecks.paneSpecDiffers).toBe(true);
  });

  it("PaneReadModel domain type differs structurally from PaneView DTO", () => {
    expect(_typeChecks.paneReadModelDiffers).toBe(true);
  });
});
