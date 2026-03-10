import { GitBranch, Globe, Terminal } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/Button";
import { useEscapeKey } from "@/hooks/useEscapeKey";
import { DEFAULT_BROWSER_URL } from "@/features/workspace/domain/models";
import type { ProfileReadModel, SettingsReadModel } from "@/features/settings/domain/models";
import type { PaneGroupConfig, SetupWizardConfig } from "@/features/workspace/store/types";
import { isFieldValuesValid } from "./PaneConfigurator";
import { LayoutPreview } from "./LayoutPreview";
import { PaneGroupRow, groupToFieldValues } from "./PaneGroupRow";

const MAX_PANES = 9;

function resolveDefaultProfileId(
  settings: SettingsReadModel,
  profiles: ProfileReadModel[],
): string {
  const configured = settings.defaultTerminalProfileId?.trim();
  if (configured && profiles.some((profile) => profile.id === configured)) {
    return configured;
  }

  return profiles.find((profile) => profile.id === "terminal")?.id
    ?? profiles[0]?.id
    ?? "terminal";
}

function makeDefaultGroup(
  mode: PaneGroupConfig["mode"],
  settings: SettingsReadModel,
  profiles: ProfileReadModel[],
): PaneGroupConfig {
  switch (mode) {
    case "terminal":
      return {
        mode: "terminal",
        profileId: resolveDefaultProfileId(settings, profiles),
        workingDirectory: settings.defaultWorkingDirectory ?? "",
        customCommand: settings.defaultCustomCommand ?? "",
        count: 1,
      };
    case "browser":
      return { mode: "browser", url: DEFAULT_BROWSER_URL, count: 1 };
    case "git":
      return { mode: "git", workingDirectory: settings.defaultWorkingDirectory ?? "", count: 1 };
  }
}

interface WorkspaceSetupWizardProps {
  profiles: ProfileReadModel[];
  settings: SettingsReadModel;
  isFirstLaunch: boolean;
  onComplete: (config: SetupWizardConfig) => void;
  onCancel?: () => void;
}

export function WorkspaceSetupWizard({
  profiles,
  settings,
  isFirstLaunch,
  onComplete,
  onCancel,
}: WorkspaceSetupWizardProps) {
  const [groups, setGroups] = useState<PaneGroupConfig[]>([
    makeDefaultGroup("terminal", settings, profiles),
  ]);

  useEscapeKey(onCancel);

  const totalPanes = groups.reduce((sum, group) => sum + group.count, 0);
  const hasInvalidGroup = groups.some(
    (group) => !isFieldValuesValid(groupToFieldValues(group)),
  );

  function handleUpdateGroup(index: number, updated: PaneGroupConfig) {
    setGroups((prev) =>
      prev.map((group, groupIndex) =>
        groupIndex === index ? updated : group,
      ),
    );
  }

  function handleRemoveGroup(index: number) {
    setGroups((prev) => prev.filter((_, groupIndex) => groupIndex !== index));
  }

  function handleAddGroup(mode: PaneGroupConfig["mode"]) {
    setGroups((prev) => [...prev, makeDefaultGroup(mode, settings, profiles)]);
  }

  function handleSubmit() {
    if (hasInvalidGroup || totalPanes === 0) {
      return;
    }
    onComplete({ groups });
  }

  const canAddMore = totalPanes < MAX_PANES;

  return (
    <div className="flex h-screen items-center justify-center bg-[var(--color-bg)] p-8">
      <div className="w-full max-w-5xl rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-8 shadow-2xl">
        <div className="text-center">
          <h1
            data-testid="wizard-title"
            className="text-2xl font-bold text-[var(--color-text)]"
          >
            {isFirstLaunch ? "Welcome to Tabby" : "New Workspace"}
          </h1>
          <p className="mt-2 text-sm text-[var(--color-text-muted)]">
            Configure your panes and layout will be derived automatically.
          </p>
        </div>

        <div className="mt-8 flex gap-10">
          <div className="flex-1 space-y-3">
            <h3 className="text-xs font-medium text-[var(--color-text-muted)]">
              Groups
            </h3>
            {groups.map((group, index) => (
              <PaneGroupRow
                key={index}
                index={index}
                group={group}
                profiles={profiles}
                maxCount={MAX_PANES - totalPanes + group.count}
                canRemove={groups.length > 1}
                onChange={(updated) => handleUpdateGroup(index, updated)}
                onRemove={() => handleRemoveGroup(index)}
              />
            ))}

            {canAddMore ? (
              <div className="mt-2 flex gap-2.5">
                <button
                  data-testid="add-terminal-group"
                  className="flex flex-1 items-center justify-center gap-2 rounded-xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface-overlay)] p-2.5 text-sm text-[var(--color-text-muted)] transition hover:border-[var(--color-accent)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-accent)]"
                  onClick={() => handleAddGroup("terminal")}
                >
                  <Terminal size={14} />
                  Terminal
                </button>
                <button
                  data-testid="add-browser-group"
                  className="flex flex-1 items-center justify-center gap-2 rounded-xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface-overlay)] p-2.5 text-sm text-[var(--color-text-muted)] transition hover:border-[var(--color-accent)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-accent)]"
                  onClick={() => handleAddGroup("browser")}
                >
                  <Globe size={14} />
                  Browser
                </button>
                <button
                  data-testid="add-git-group"
                  className="flex flex-1 items-center justify-center gap-2 rounded-xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface-overlay)] p-2.5 text-sm text-[var(--color-text-muted)] transition hover:border-[var(--color-accent)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-accent)]"
                  onClick={() => handleAddGroup("git")}
                >
                  <GitBranch size={14} />
                  Git
                </button>
              </div>
            ) : null}
          </div>

          <div className="w-[300px] shrink-0">
            <LayoutPreview groups={groups} />
          </div>
        </div>

        <div className="mt-6 flex items-center justify-between border-t border-[var(--color-border)] pt-4">
          <span className="text-xs text-[var(--color-text-muted)]">
            {totalPanes} of {MAX_PANES} panes
          </span>
          <div className="flex items-center gap-3">
            {onCancel ? (
              <Button
                data-testid="wizard-cancel"
                variant="ghost"
                onClick={onCancel}
              >
                Cancel
              </Button>
            ) : null}
            <Button
              data-testid="wizard-create"
              disabled={totalPanes === 0 || hasInvalidGroup}
              onClick={handleSubmit}
            >
              Create Workspace
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
