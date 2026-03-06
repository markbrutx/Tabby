import { Fragment } from "react";
import {
  Panel,
  PanelGroup,
  PanelResizeHandle,
} from "react-resizable-panels";
import type { PaneProfile, TabSnapshot } from "@/features/workspace/domain";
import { createGridDefinition } from "@/features/workspace/layouts";
import { TerminalPane } from "./TerminalPane";

interface PaneGridProps {
  tab: TabSnapshot;
  profiles: PaneProfile[];
  fontSize: number;
  visible: boolean;
  onFocus: (tabId: string, paneId: string) => Promise<void>;
  onUpdateProfile: (
    paneId: string,
    profileId: string,
    startupCommand?: string | null,
  ) => Promise<void>;
  onUpdateCwd: (paneId: string, cwd: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
}

function chunk<T>(items: T[], size: number) {
  const rows: T[][] = [];

  for (let index = 0; index < items.length; index += size) {
    rows.push(items.slice(index, index + size));
  }

  return rows;
}

export function PaneGrid({
  tab,
  profiles,
  fontSize,
  visible,
  onFocus,
  onUpdateProfile,
  onUpdateCwd,
  onRestart,
}: PaneGridProps) {
  const definition = createGridDefinition(tab.preset);
  const rows = chunk(tab.panes, definition.columns);

  return (
    <PanelGroup direction="vertical" className="h-full gap-2">
      {rows.map((row, rowIndex) => (
        <Fragment key={`${tab.id}-${rowIndex}`}>
          <Panel defaultSize={100 / rows.length}>
            <PanelGroup direction="horizontal" className="h-full gap-2">
              {row.map((pane, columnIndex) => (
                <Fragment key={pane.id}>
                  <Panel defaultSize={100 / row.length}>
                    <TerminalPane
                      pane={pane}
                      profiles={profiles}
                      fontSize={fontSize}
                      active={tab.activePaneId === pane.id}
                      visible={visible}
                      onFocus={(paneId) => onFocus(tab.id, paneId)}
                      onUpdateProfile={onUpdateProfile}
                      onUpdateCwd={onUpdateCwd}
                      onRestart={onRestart}
                    />
                  </Panel>
                  {columnIndex < row.length - 1 ? (
                    <PanelResizeHandle className="resize-handle w-2 rounded-full" />
                  ) : null}
                </Fragment>
              ))}
            </PanelGroup>
          </Panel>
          {rowIndex < rows.length - 1 ? (
            <PanelResizeHandle className="resize-handle h-2 rounded-full" />
          ) : null}
        </Fragment>
      ))}
    </PanelGroup>
  );
}
