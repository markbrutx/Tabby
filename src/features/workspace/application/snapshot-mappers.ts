/**
 * Workspace snapshot mappers.
 *
 * Converts transport DTOs (wire format) to internal workspace read models.
 * This is part of the anti-corruption layer that keeps domain code independent
 * of the generated contract types.
 */

import type {
  PaneSpecDto,
  PaneView,
  SplitNodeDto,
  TabView,
  WorkspaceView,
} from "@/contracts/tauri-bindings";
import type {
  PaneReadModel,
  PaneSpec,
  SplitNode,
  TabReadModel,
  WorkspaceReadModel,
} from "@/features/workspace/domain/models";

export function mapPaneSpecFromDto(dto: PaneSpecDto): PaneSpec {
  if (dto.kind === "browser") {
    return {
      kind: "browser",
      initialUrl: dto.initial_url,
    };
  }

  return {
    kind: "terminal",
    launchProfileId: dto.launch_profile_id,
    workingDirectory: dto.working_directory,
    commandOverride: dto.command_override,
  };
}

export function mapSplitNodeFromDto(dto: SplitNodeDto): SplitNode {
  if (dto.type === "pane") {
    return { type: "pane", paneId: dto.paneId };
  }

  return {
    type: "split",
    direction: dto.direction,
    ratio: dto.ratio,
    first: mapSplitNodeFromDto(dto.first),
    second: mapSplitNodeFromDto(dto.second),
  };
}

export function mapPaneFromDto(dto: PaneView): PaneReadModel {
  return {
    paneId: dto.paneId,
    title: dto.title,
    spec: mapPaneSpecFromDto(dto.spec),
  };
}

export function mapTabFromDto(dto: TabView): TabReadModel {
  return {
    tabId: dto.tabId,
    title: dto.title,
    layout: mapSplitNodeFromDto(dto.layout),
    panes: dto.panes.map(mapPaneFromDto),
    activePaneId: dto.activePaneId,
  };
}

export function mapWorkspaceFromDto(dto: WorkspaceView): WorkspaceReadModel {
  return {
    activeTabId: dto.activeTabId,
    tabs: dto.tabs.map(mapTabFromDto),
  };
}
