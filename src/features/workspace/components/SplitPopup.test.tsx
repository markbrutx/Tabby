import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SplitPopup } from "./SplitPopup";
import type { ProfileReadModel } from "@/features/settings/domain/models";

vi.mock("@/lib/pickDirectory", () => ({
  pickDirectory: vi.fn().mockResolvedValue(null),
}));

const profiles: ProfileReadModel[] = [
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
          launchProfileId: "",
          workingDirectory: "/tmp",
          commandOverride: null,
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
          launchProfileId: "custom",
          workingDirectory: "/tmp",
          commandOverride: null,
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
          launchProfileId: "custom",
          workingDirectory: "/tmp",
          commandOverride: null,
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
          launchProfileId: "custom",
          workingDirectory: "/tmp",
          commandOverride: null,
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
      launchProfileId: "custom",
      workingDirectory: "/tmp",
      commandOverride: "npm run dev",
    });
  });

  it("shows Git option in mode selector", () => {
    render(
      <SplitPopup
        direction="horizontal"
        profiles={profiles}
        defaultSpec={{
          kind: "terminal",
          launchProfileId: "terminal",
          workingDirectory: "/tmp",
          commandOverride: null,
        }}
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    const modeSelect = screen.getAllByRole("combobox")[0];
    const options = Array.from(modeSelect.querySelectorAll("option"));
    const values = options.map((opt) => opt.getAttribute("value"));
    expect(values).toContain("git");
  });

  it("shows working directory input when Git mode is selected", () => {
    render(
      <SplitPopup
        direction="horizontal"
        profiles={profiles}
        defaultSpec={{
          kind: "terminal",
          launchProfileId: "terminal",
          workingDirectory: "/projects/my-repo",
          commandOverride: null,
        }}
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    const modeSelect = screen.getAllByRole("combobox")[0];
    fireEvent.change(modeSelect, { target: { value: "git" } });

    expect(screen.getByPlaceholderText("Working directory")).toBeInTheDocument();
  });

  it("produces correct git PaneSpec on confirm", () => {
    const onConfirm = vi.fn();
    render(
      <SplitPopup
        direction="horizontal"
        profiles={profiles}
        defaultSpec={{
          kind: "terminal",
          launchProfileId: "terminal",
          workingDirectory: "/projects/my-repo",
          commandOverride: null,
        }}
        onConfirm={onConfirm}
        onCancel={vi.fn()}
      />,
    );

    const modeSelect = screen.getAllByRole("combobox")[0];
    fireEvent.change(modeSelect, { target: { value: "git" } });
    fireEvent.click(screen.getByRole("button", { name: "Split" }));

    expect(onConfirm).toHaveBeenCalledWith({
      kind: "git",
      workingDirectory: "/projects/my-repo",
    });
  });

  it("defaults to git mode when defaultSpec is git", () => {
    render(
      <SplitPopup
        direction="horizontal"
        profiles={profiles}
        defaultSpec={{
          kind: "git",
          workingDirectory: "/projects/git-repo",
        }}
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    const modeSelect = screen.getAllByRole("combobox")[0];
    expect(modeSelect).toHaveValue("git");
    expect(screen.getByDisplayValue("/projects/git-repo")).toBeInTheDocument();
  });

  it("cancels without changes from git mode", () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();
    render(
      <SplitPopup
        direction="horizontal"
        profiles={profiles}
        defaultSpec={{
          kind: "git",
          workingDirectory: "/projects/git-repo",
        }}
        onConfirm={onConfirm}
        onCancel={onCancel}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

    expect(onCancel).toHaveBeenCalled();
    expect(onConfirm).not.toHaveBeenCalled();
  });
});
