import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SplitPopup } from "./SplitPopup";
import type { PaneProfile } from "@/features/workspace/domain";

vi.mock("@/lib/pickDirectory", () => ({
  pickDirectory: vi.fn().mockResolvedValue(null),
}));

const profiles: PaneProfile[] = [
  { id: "terminal", label: "Terminal", description: "Shell", startupCommand: null },
  { id: "custom", label: "Custom", description: "Run any command", startupCommand: null },
];

describe("SplitPopup", () => {
  it("falls back to terminal when default profile id is empty", () => {
    render(
      <SplitPopup
        direction="horizontal"
        profiles={profiles}
        defaultProfileId=""
        defaultCwd="/tmp"
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    expect(screen.getByRole("combobox")).toHaveValue("terminal");
  });

  it("disables split for custom profile without a command", () => {
    render(
      <SplitPopup
        direction="horizontal"
        profiles={profiles}
        defaultProfileId="custom"
        defaultCwd="/tmp"
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    expect(screen.getByRole("button", { name: "Split" })).toBeDisabled();
  });

  it("blocks Enter submit for custom profile without a command", () => {
    const onConfirm = vi.fn();
    render(
      <SplitPopup
        direction="horizontal"
        profiles={profiles}
        defaultProfileId="custom"
        defaultCwd="/tmp"
        onConfirm={onConfirm}
        onCancel={vi.fn()}
      />,
    );

    fireEvent.keyDown(window, { key: "Enter" });

    expect(onConfirm).not.toHaveBeenCalled();
  });

  it("enables split after entering a custom command", () => {
    const onConfirm = vi.fn();
    render(
      <SplitPopup
        direction="horizontal"
        profiles={profiles}
        defaultProfileId="custom"
        defaultCwd="/tmp"
        onConfirm={onConfirm}
        onCancel={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByPlaceholderText("Custom command"), {
      target: { value: "npm run dev" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Split" }));

    expect(onConfirm).toHaveBeenCalledWith("custom", "/tmp", "npm run dev");
  });
});
