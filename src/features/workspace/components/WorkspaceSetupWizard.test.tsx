import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { WorkspaceSetupWizard } from "./WorkspaceSetupWizard";
import type { ProfileReadModel, SettingsReadModel } from "@/features/settings/domain/models";

vi.mock("@/lib/pickDirectory", () => ({
  pickDirectory: vi.fn().mockResolvedValue(null),
}));

const profiles: ProfileReadModel[] = [
  { id: "terminal", label: "Terminal", description: "Shell", startupCommandTemplate: null },
  { id: "claude", label: "Claude Code", description: "AI assistant", startupCommandTemplate: "claude" },
  { id: "codex", label: "Codex", description: "OpenAI coding agent", startupCommandTemplate: "codex" },
  { id: "gemini", label: "Gemini CLI", description: "Google Gemini coding agent", startupCommandTemplate: "gemini" },
  { id: "opencode", label: "OpenCode CLI", description: "OpenCode coding agent", startupCommandTemplate: "opencode" },
  { id: "custom", label: "Custom", description: "Run any command", startupCommandTemplate: null },
];

const settings: SettingsReadModel = {
  defaultLayout: "1x1",
  defaultTerminalProfileId: "terminal",
  defaultWorkingDirectory: "/Users/test",
  defaultCustomCommand: "",
  fontSize: 13,
  theme: "system",
  launchFullscreen: true,
  hasCompletedOnboarding: false,
  lastWorkingDirectory: null,
};

describe("WorkspaceSetupWizard", () => {
  it("renders 'Welcome to Tabby' for first launch", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    expect(screen.getByTestId("wizard-title")).toHaveTextContent("Welcome to Tabby");
  });

  it("renders 'New Workspace' for subsequent launches", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={{ ...settings, hasCompletedOnboarding: true }}
        isFirstLaunch={false}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    expect(screen.getByTestId("wizard-title")).toHaveTextContent("New Workspace");
  });

  it("initial group uses settings defaults", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    expect(screen.getByTestId("group-0-profile")).toHaveValue("terminal");
    expect(screen.getByTestId("group-count-0")).toHaveTextContent("1");
  });

  it("falls back to terminal when settings default profile is empty", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={{ ...settings, defaultTerminalProfileId: "" }}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );

    expect(screen.getByTestId("group-0-profile")).toHaveValue("terminal");
  });

  it("add terminal group button adds a new terminal group", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByTestId("add-terminal-group"));
    expect(screen.getByTestId("pane-group-1")).toBeInTheDocument();
  });

  it("add git group button adds a new git group", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByTestId("add-git-group"));
    expect(screen.getByTestId("pane-group-1")).toBeInTheDocument();
    expect(screen.getByTestId("group-1-dir")).toBeInTheDocument();
  });

  it("remove group removes it", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByTestId("add-terminal-group"));
    expect(screen.getByTestId("pane-group-1")).toBeInTheDocument();
    fireEvent.click(screen.getByTestId("group-remove-1"));
    expect(screen.queryByTestId("pane-group-1")).not.toBeInTheDocument();
  });

  it("count increment and decrement work", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    expect(screen.getByTestId("group-count-0")).toHaveTextContent("1");
    fireEvent.click(screen.getByTestId("group-increment-0"));
    expect(screen.getByTestId("group-count-0")).toHaveTextContent("2");
    fireEvent.click(screen.getByTestId("group-decrement-0"));
    expect(screen.getByTestId("group-count-0")).toHaveTextContent("1");
  });

  it("decrement is disabled at count 1", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    expect(screen.getByTestId("group-decrement-0")).toBeDisabled();
  });

  it("cannot exceed 9 panes total", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    // Increment to 9
    for (let i = 0; i < 8; i++) {
      fireEvent.click(screen.getByTestId("group-increment-0"));
    }
    expect(screen.getByTestId("group-count-0")).toHaveTextContent("9");
    expect(screen.getByTestId("group-increment-0")).toBeDisabled();
  });

  it("custom profile shows command input", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    fireEvent.change(screen.getByTestId("group-0-profile"), {
      target: { value: "custom" },
    });
    expect(screen.getByTestId("group-0-command")).toBeInTheDocument();
  });

  it("disables create when custom profile has no command", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByTestId("group-0-profile"), {
      target: { value: "custom" },
    });

    expect(screen.getByTestId("wizard-create")).toBeDisabled();
  });

  it("re-enables create when custom profile command is provided", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByTestId("group-0-profile"), {
      target: { value: "custom" },
    });
    fireEvent.change(screen.getByTestId("group-0-command"), {
      target: { value: "npm run dev" },
    });

    expect(screen.getByTestId("wizard-create")).not.toBeDisabled();
  });

  it("create button calls onComplete with SetupWizardConfig", () => {
    const onComplete = vi.fn();
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={onComplete}
      />,
    );
    fireEvent.click(screen.getByTestId("wizard-create"));
    expect(onComplete).toHaveBeenCalledWith({
      groups: [
        {
          mode: "terminal",
          profileId: "terminal",
          workingDirectory: "/Users/test",
          customCommand: "",
          count: 1,
        },
      ],
      layoutVariantId: null,
    });
  });

  it("git group does not block create button", () => {
    const onComplete = vi.fn();
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={onComplete}
      />,
    );
    fireEvent.click(screen.getByTestId("add-git-group"));
    expect(screen.getByTestId("wizard-create")).not.toBeDisabled();
  });

  it("cancel button calls onCancel", () => {
    const onCancel = vi.fn();
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={false}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
        onCancel={onCancel}
      />,
    );
    fireEvent.click(screen.getByTestId("wizard-cancel"));
    expect(onCancel).toHaveBeenCalled();
  });

  it("cancel button is hidden when onCancel is undefined", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    expect(screen.queryByTestId("wizard-cancel")).not.toBeInTheDocument();
  });

  it("shows live layout preview", () => {
    render(
      <WorkspaceSetupWizard
        profiles={profiles}
        settings={settings}
        isFirstLaunch={true}
        defaultTitle="Workspace 1"
        onComplete={vi.fn()}
      />,
    );
    expect(screen.getByTestId("layout-preview")).toBeInTheDocument();
  });
});
