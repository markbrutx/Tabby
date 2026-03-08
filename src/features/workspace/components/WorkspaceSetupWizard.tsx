import { Plus } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/Button";
import { useEscapeKey } from "@/hooks/useEscapeKey";
import { CUSTOM_PROFILE_ID } from "@/features/workspace/domain/models";
import type { ProfileReadModel, SettingsReadModel } from "@/features/settings/domain/models";
import type { PaneGroupConfig, SetupWizardConfig } from "@/features/workspace/store/types";
import { LayoutPreview } from "./LayoutPreview";
import { PaneGroupRow } from "./PaneGroupRow";

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
  settings: SettingsReadModel,
  profiles: ProfileReadModel[],
): PaneGroupConfig {
  return {
    mode: "terminal",
    profileId: resolveDefaultProfileId(settings, profiles),
    workingDirectory: settings.defaultWorkingDirectory ?? "",
    customCommand: settings.defaultCustomCommand ?? "",
    url: "https://google.com",
    count: 1,
  };
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
    makeDefaultGroup(settings, profiles),
  ]);

  useEscapeKey(onCancel);

  const totalPanes = groups.reduce((sum, group) => sum + group.count, 0);
  const hasInvalidGroup = groups.some((group) =>
    group.mode === "terminal"
      ? !group.profileId || (
        group.profileId === CUSTOM_PROFILE_ID && !(group.customCommand?.trim())
      )
      : !(group.url?.trim())
  );

  function handleUpdateGroup(index: number, update: Partial<PaneGroupConfig>) {
    setGroups((prev) =>
      prev.map((group, groupIndex) =>
        groupIndex === index ? { ...group, ...update } : group,
      ),
    );
  }

  function handleRemoveGroup(index: number) {
    setGroups((prev) => prev.filter((_, groupIndex) => groupIndex !== index));
  }

  function handleAddGroup() {
    setGroups((prev) => [...prev, makeDefaultGroup(settings, profiles)]);
  }

  function handleSubmit() {
    if (hasInvalidGroup || totalPanes === 0) {
      return;
    }
    onComplete({ groups });
  }

  return (
    <div className="flex h-screen items-center justify-center bg-[var(--color-bg)] p-8">
      <div className="w-full max-w-4xl rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-8 shadow-2xl">
        <div className="text-center">
          <h1
            data-testid="wizard-title"
            className="text-2xl font-bold text-[var(--color-text)]"
          >
            {isFirstLaunch ? "Welcome to Tabby" : "New Workspace"}
          </h1>
          <p className="mt-2 text-sm text-[var(--color-text-muted)]">
            Compose explicit terminal and browser pane groups. Layout is derived from pane count.
          </p>
        </div>

        <div className="mt-6 flex gap-8">
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
                onChange={(update) => handleUpdateGroup(index, update)}
                onRemove={() => handleRemoveGroup(index)}
              />
            ))}

            {totalPanes < MAX_PANES ? (
              <button
                data-testid="add-group"
                className="flex w-full items-center justify-center gap-2 rounded-xl border border-dashed border-[var(--color-border)] p-3 text-sm text-[var(--color-text-muted)] transition hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]"
                onClick={handleAddGroup}
              >
                <Plus size={14} />
                Add group
              </button>
            ) : null}
          </div>

          <div className="w-[280px] shrink-0">
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
