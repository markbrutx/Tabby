import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SplitPopup } from "./SplitPopup";
import type { PaneProfile } from "@/features/workspace/domain";

vi.mock("@/lib/pickDirectory", () => ({
  pickDirectory: vi.fn().mockResolvedValue(null),
}));

const profiles: PaneProfile[] = [
  { id: "terminal", label: "Terminal", description: "Shell", startupCommandTemplate: null },
  { id: "custom", label: "Custom", description: "Run any command", startupCommandTemplate: null },
];

describe("SplitPopup", () => {
  it("falls back to terminal when default profile id is empty", () => {
    render(
      <SplitPopup
        direction="horizontal"
        profiles={profiles}
        defaultSpec={{
          kind: "terminal",
          launch_profile_id: "",
          working_directory: "/tmp",
          command_override: null,
        }}
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    expect(screen.getAllByRole("combobox")[1]).toHaveValue("terminal");
  });

  it("disables split for custom profile without a command", () => {
    render(
      <SplitPopup
        direction="horizontal"
        profiles={profiles}
        defaultSpec={{
          kind: "terminal",
          launch_profile_id: "custom",
          working_directory: "/tmp",
          command_override: null,
        }}
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
        defaultSpec={{
          kind: "terminal",
          launch_profile_id: "custom",
          working_directory: "/tmp",
          command_override: null,
        }}
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
        defaultSpec={{
          kind: "terminal",
          launch_profile_id: "custom",
          working_directory: "/tmp",
          command_override: null,
        }}
        onConfirm={onConfirm}
        onCancel={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByPlaceholderText("Custom command"), {
      target: { value: "npm run dev" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Split" }));

    expect(onConfirm).toHaveBeenCalledWith({
      kind: "terminal",
      launch_profile_id: "custom",
      working_directory: "/tmp",
      command_override: "npm run dev",
    });
  });
});
