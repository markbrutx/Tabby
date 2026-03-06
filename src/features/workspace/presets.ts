import {
  Grid2x2,
  Grid3x3,
  PanelTop,
  Sparkles,
} from "lucide-react";
import type { LayoutPreset, ThemeMode } from "@/features/workspace/domain";

export const LAYOUT_PRESET_CARDS: {
  preset: LayoutPreset;
  title: string;
  description: string;
  icon: typeof PanelTop;
}[] = [
  {
    preset: "1x1",
    title: "Solo",
    description: "Single focused shell",
    icon: PanelTop,
  },
  {
    preset: "1x2",
    title: "Pair",
    description: "Two terminals side by side",
    icon: Grid2x2,
  },
  {
    preset: "2x2",
    title: "Quad",
    description: "Balanced workspace",
    icon: Grid2x2,
  },
  {
    preset: "2x3",
    title: "Research",
    description: "Six panes for agent work",
    icon: Grid3x3,
  },
  {
    preset: "3x3",
    title: "War Room",
    description: "Nine live panes",
    icon: Sparkles,
  },
];

export const THEME_OPTIONS: { value: ThemeMode; label: string }[] = [
  { value: "system", label: "System" },
  { value: "dawn", label: "Dawn" },
  { value: "midnight", label: "Midnight" },
];
