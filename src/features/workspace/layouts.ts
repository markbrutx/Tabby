import type { GridDefinition, LayoutPreset } from "./domain";

const GRID_MAP: Record<LayoutPreset, Omit<GridDefinition, "preset">> = {
  "1x1": {
    rows: 1,
    columns: 1,
    paneCount: 1,
  },
  "1x2": {
    rows: 1,
    columns: 2,
    paneCount: 2,
  },
  "2x2": {
    rows: 2,
    columns: 2,
    paneCount: 4,
  },
  "2x3": {
    rows: 2,
    columns: 3,
    paneCount: 6,
  },
  "3x3": {
    rows: 3,
    columns: 3,
    paneCount: 9,
  },
};

export function createGridDefinition(preset: LayoutPreset): GridDefinition {
  return {
    preset,
    ...GRID_MAP[preset],
  };
}
