import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { OnboardingWizard } from "./OnboardingWizard";
import type { WorkspaceSettings, PaneProfile } from "@/features/workspace/domain";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

const defaultSettings: WorkspaceSettings = {
  defaultLayout: "2x2",
  defaultProfileId: "terminal",
  defaultWorkingDirectory: "/Users/test/projects",
  defaultCustomCommand: "",
  fontSize: 13,
  theme: "system",
  launchFullscreen: true,
  hasCompletedOnboarding: false,
};

const profiles: PaneProfile[] = [
  { id: "terminal", label: "Terminal", description: "Pure login shell", startupCommand: null },
  { id: "claude", label: "Claude Code", description: "Open Claude Code", startupCommand: "claude" },
  { id: "codex", label: "Codex", description: "Open Codex", startupCommand: "codex" },
  { id: "custom", label: "Custom", description: "Run an arbitrary command", startupCommand: null },
];

function renderWizard(onComplete = vi.fn().mockResolvedValue(undefined)) {
  render(
    <OnboardingWizard
      initialSettings={defaultSettings}
      profiles={profiles}
      onComplete={onComplete}
    />,
  );
  return { onComplete };
}

describe("OnboardingWizard", () => {
  it("renders step 1 (layout) by default", () => {
    renderWizard();
    expect(screen.getByTestId("onboarding-wizard")).toBeInTheDocument();
    expect(screen.getByTestId("onboarding-step-layout")).toBeInTheDocument();
    expect(screen.getByText("Pick your deck")).toBeInTheDocument();
  });

  it("navigates forward and backward between steps", () => {
    renderWizard();

    fireEvent.click(screen.getByTestId("onboarding-next"));
    expect(screen.getByTestId("onboarding-step-shell")).toBeInTheDocument();
    expect(screen.getByText("Choose your shell")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("onboarding-next"));
    expect(screen.getByTestId("onboarding-step-personalize")).toBeInTheDocument();
    expect(screen.getByText("Make it yours")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("onboarding-back"));
    expect(screen.getByTestId("onboarding-step-shell")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("onboarding-back"));
    expect(screen.getByTestId("onboarding-step-layout")).toBeInTheDocument();
  });

  it("back button is disabled on step 1", () => {
    renderWizard();
    expect(screen.getByTestId("onboarding-back")).toBeDisabled();
  });

  it("shows finish button only on last step", () => {
    renderWizard();

    expect(screen.queryByTestId("onboarding-finish")).not.toBeInTheDocument();
    expect(screen.getByTestId("onboarding-next")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("onboarding-next"));
    fireEvent.click(screen.getByTestId("onboarding-next"));

    expect(screen.getByTestId("onboarding-finish")).toBeInTheDocument();
    expect(screen.queryByTestId("onboarding-next")).not.toBeInTheDocument();
  });

  it("selects a layout preset", () => {
    renderWizard();

    const soloButton = screen.getByTestId("onboarding-layout-1x1");
    fireEvent.click(soloButton);

    expect(soloButton.className).toContain("accent");
  });

  it("updates profile selection on step 2", () => {
    renderWizard();
    fireEvent.click(screen.getByTestId("onboarding-next"));

    const profileSelect = screen.getByTestId("onboarding-profile");
    fireEvent.change(profileSelect, { target: { value: "claude" } });

    expect((profileSelect as HTMLSelectElement).value).toBe("claude");
  });

  it("shows custom command input when custom profile is selected", () => {
    renderWizard();
    fireEvent.click(screen.getByTestId("onboarding-next"));

    expect(screen.queryByTestId("onboarding-custom-command")).not.toBeInTheDocument();

    const profileSelect = screen.getByTestId("onboarding-profile");
    fireEvent.change(profileSelect, { target: { value: "custom" } });

    expect(screen.getByTestId("onboarding-custom-command")).toBeInTheDocument();
  });

  it("updates theme on step 3", () => {
    renderWizard();
    fireEvent.click(screen.getByTestId("onboarding-next"));
    fireEvent.click(screen.getByTestId("onboarding-next"));

    const themeSelect = screen.getByTestId("onboarding-theme");
    fireEvent.change(themeSelect, { target: { value: "midnight" } });

    expect((themeSelect as HTMLSelectElement).value).toBe("midnight");
  });

  it("calls onComplete with updated settings including hasCompletedOnboarding on finish", async () => {
    const { onComplete } = renderWizard();

    fireEvent.click(screen.getByTestId("onboarding-layout-1x1"));

    fireEvent.click(screen.getByTestId("onboarding-next"));
    const profileSelect = screen.getByTestId("onboarding-profile");
    fireEvent.change(profileSelect, { target: { value: "claude" } });

    fireEvent.click(screen.getByTestId("onboarding-next"));
    const themeSelect = screen.getByTestId("onboarding-theme");
    fireEvent.change(themeSelect, { target: { value: "midnight" } });

    fireEvent.click(screen.getByTestId("onboarding-finish"));

    await waitFor(() => {
      expect(onComplete).toHaveBeenCalledTimes(1);
    });

    const calledWith = onComplete.mock.calls[0][0];
    expect(calledWith.hasCompletedOnboarding).toBe(true);
    expect(calledWith.defaultLayout).toBe("1x1");
    expect(calledWith.defaultProfileId).toBe("claude");
    expect(calledWith.theme).toBe("midnight");
  });

  it("updates working directory input on step 2", () => {
    renderWizard();
    fireEvent.click(screen.getByTestId("onboarding-next"));

    const cwdInput = screen.getByTestId("onboarding-working-directory");
    fireEvent.change(cwdInput, { target: { value: "/new/path" } });

    expect((cwdInput as HTMLInputElement).value).toBe("/new/path");
  });

  it("updates font size on step 3", () => {
    renderWizard();
    fireEvent.click(screen.getByTestId("onboarding-next"));
    fireEvent.click(screen.getByTestId("onboarding-next"));

    const fontSlider = screen.getByTestId("onboarding-font-size");
    fireEvent.change(fontSlider, { target: { value: "18" } });

    expect(screen.getByText("18px")).toBeInTheDocument();
  });
});
