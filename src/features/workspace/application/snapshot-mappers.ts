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

  if (dto.kind === "git") {
    return {
      kind: "git",
      workingDirectory: dto.working_directory,
    };
  }

  return {
    kind: "terminal",
    launchProfileId: dto.launch_profile_id,
    workingDirectory: dto.working_directory,
    commandOverride: dto.command_override,
  };
}

export function mapPaneSpecToDto(spec: PaneSpec): PaneSpecDto {
  if (spec.kind === "browser") {
    return {
      kind: "browser",
      initial_url: spec.initialUrl,
    };
  }

  if (spec.kind === "git") {
    return {
      kind: "git",
      working_directory: spec.workingDirectory,
    };
  }

  return {
    kind: "terminal",
    launch_profile_id: spec.launchProfileId,
    working_directory: spec.workingDirectory,
    command_override: spec.commandOverride,
  };
}

export function mapSplitNodeToDto(node: SplitNode): SplitNodeDto {
  if (node.type === "pane") {
    return { type: "pane", paneId: node.paneId };
  }

  return {
    type: "split",
    direction: node.direction,
    ratio: node.ratio,
    first: mapSplitNodeToDto(node.first),
    second: mapSplitNodeToDto(node.second),
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
