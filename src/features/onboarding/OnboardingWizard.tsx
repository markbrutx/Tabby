import { useState } from "react";
import {
  ArrowLeft,
  ArrowRight,
  Check,
  FolderOpen,
} from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import type {
  LayoutPreset,
  PaneProfile,
  ThemeMode,
  WorkspaceSettings,
} from "@/features/workspace/domain";
import { createGridDefinition } from "@/features/workspace/layouts";
import { LAYOUT_PRESET_CARDS, THEME_OPTIONS } from "@/features/workspace/presets";
import { pickDirectory } from "@/lib/pickDirectory";

const STEPS = ["Pick your deck", "Choose your shell", "Make it yours"] as const;

type OnboardingDraft = Omit<WorkspaceSettings, "hasCompletedOnboarding">;

interface OnboardingWizardProps {
  initialSettings: WorkspaceSettings;
  profiles: PaneProfile[];
  onComplete: (settings: WorkspaceSettings) => Promise<void>;
}

export function OnboardingWizard({
  initialSettings,
  profiles,
  onComplete,
}: OnboardingWizardProps) {
  const [step, setStep] = useState(0);
  const [isFinishing, setIsFinishing] = useState(false);
  const { hasCompletedOnboarding: _, ...initialDraft } = initialSettings;
  const [draft, setDraft] = useState<OnboardingDraft>(initialDraft);

  function applyPatch(patch: Partial<OnboardingDraft>) {
    setDraft((current) => ({ ...current, ...patch }));
  }

  async function handlePickDirectory() {
    const selected = await pickDirectory(draft.defaultWorkingDirectory);
    if (selected) {
      applyPatch({ defaultWorkingDirectory: selected });
    }
  }

  async function handleFinish() {
    setIsFinishing(true);
    try {
      await onComplete({ ...draft, hasCompletedOnboarding: true });
    } finally {
      setIsFinishing(false);
    }
  }

  function handleNext() {
    if (step < STEPS.length - 1) {
      setStep(step + 1);
    }
  }

  function handleBack() {
    if (step > 0) {
      setStep(step - 1);
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center p-8 text-[var(--color-text)]">
      <div
        data-testid="onboarding-wizard"
        className="surface-panel w-full max-w-2xl rounded-[32px] p-8"
      >
        <p className="text-xs uppercase tracking-[0.35em] text-[var(--color-text-muted)]">
          Welcome to Tabby
        </p>
        <h1 className="mt-3 text-3xl font-semibold">
          {STEPS[step]}
        </h1>

        <div className="mt-2 flex gap-2">
          {STEPS.map((label) => (
            <div
              key={label}
              className={`h-1 flex-1 rounded-full transition-colors ${
                STEPS.indexOf(label) <= step
                  ? "bg-[var(--color-accent-strong)]"
                  : "bg-[var(--color-border)]"
              }`}
            />
          ))}
        </div>

        <div className="mt-6">
          {step === 0 && (
            <StepLayout
              draft={draft}
              onSelect={(preset) => applyPatch({ defaultLayout: preset })}
            />
          )}
          {step === 1 && (
            <StepShell
              draft={draft}
              profiles={profiles}
              onUpdate={applyPatch}
              onPickDirectory={() => void handlePickDirectory()}
            />
          )}
          {step === 2 && (
            <StepPersonalize draft={draft} onUpdate={applyPatch} />
          )}
        </div>

        <div className="mt-8 flex items-center justify-between">
          <Button
            data-testid="onboarding-back"
            variant="ghost"
            onClick={handleBack}
            disabled={step === 0}
          >
            <ArrowLeft size={16} />
            Back
          </Button>

          <div className="flex items-center gap-2 text-xs text-[var(--color-text-muted)]">
            Step {step + 1} of {STEPS.length}
          </div>

          {step < STEPS.length - 1 ? (
            <Button data-testid="onboarding-next" onClick={handleNext}>
              Next
              <ArrowRight size={16} />
            </Button>
          ) : (
            <Button
              data-testid="onboarding-finish"
              disabled={isFinishing}
              onClick={() => void handleFinish()}
            >
              <Check size={16} />
              Finish setup
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

function StepLayout({
  draft,
  onSelect,
}: {
  draft: OnboardingDraft;
  onSelect: (preset: LayoutPreset) => void;
}) {
  return (
    <div className="space-y-2" data-testid="onboarding-step-layout">
      <p className="mb-4 text-sm text-[var(--color-text-soft)]">
        Choose the default grid layout for new workspaces. You can always change this later.
      </p>
      {LAYOUT_PRESET_CARDS.map((card) => {
        const Icon = card.icon;
        const isSelected = draft.defaultLayout === card.preset;
        const paneCount = createGridDefinition(card.preset).paneCount;
        return (
          <button
            key={card.preset}
            data-testid={`onboarding-layout-${card.preset}`}
            className={`surface-muted w-full rounded-2xl p-4 text-start transition ${
              isSelected
                ? "border-[var(--color-accent-strong)] bg-[var(--color-accent-soft)]"
                : "hover:bg-white/6"
            }`}
            onClick={() => onSelect(card.preset)}
          >
            <div className="flex items-center gap-3">
              <div
                className={`flex h-9 w-9 items-center justify-center rounded-xl ${
                  isSelected
                    ? "bg-[var(--color-accent-strong)] text-white"
                    : "bg-[var(--color-accent-soft)] text-[var(--color-accent)]"
                }`}
              >
                <Icon size={18} />
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <p className="font-medium">{card.title}</p>
                  <span className="rounded-full bg-white/8 px-2 py-0.5 text-[10px] uppercase tracking-[0.2em] text-[var(--color-text-muted)]">
                    {card.preset}
                  </span>
                </div>
                <p className="mt-0.5 text-sm text-[var(--color-text-soft)]">
                  {card.description} — {paneCount} {paneCount === 1 ? "pane" : "panes"}
                </p>
              </div>
              {isSelected && (
                <Check size={18} className="text-[var(--color-accent-strong)]" />
              )}
            </div>
          </button>
        );
      })}
    </div>
  );
}

function StepShell({
  draft,
  profiles,
  onUpdate,
  onPickDirectory,
}: {
  draft: OnboardingDraft;
  profiles: PaneProfile[];
  onUpdate: (patch: Partial<OnboardingDraft>) => void;
  onPickDirectory: () => void;
}) {
  return (
    <div className="space-y-5" data-testid="onboarding-step-shell">
      <p className="text-sm text-[var(--color-text-soft)]">
        Select the default profile and working directory for new panes.
      </p>

      <label className="block">
        <span className="mb-2 block text-sm text-[var(--color-text-soft)]">Default profile</span>
        <Select
          data-testid="onboarding-profile"
          value={draft.defaultProfileId}
          onChange={(event) => onUpdate({ defaultProfileId: event.target.value })}
        >
          {profiles.map((profile) => (
            <option key={profile.id} value={profile.id}>
              {profile.label}
            </option>
          ))}
        </Select>
      </label>

      {draft.defaultProfileId === "custom" && (
        <label className="block">
          <span className="mb-2 block text-sm text-[var(--color-text-soft)]">Startup command</span>
          <Input
            data-testid="onboarding-custom-command"
            value={draft.defaultCustomCommand}
            onChange={(event) => onUpdate({ defaultCustomCommand: event.target.value })}
            placeholder="npm run dev"
          />
        </label>
      )}

      <div className="block">
        <span className="mb-2 block text-sm text-[var(--color-text-soft)]">Working directory</span>
        <div className="flex gap-2">
          <Input
            data-testid="onboarding-working-directory"
            value={draft.defaultWorkingDirectory}
            onChange={(event) => onUpdate({ defaultWorkingDirectory: event.target.value })}
            placeholder="~/projects"
          />
          <Button variant="secondary" onClick={onPickDirectory}>
            <FolderOpen size={16} />
          </Button>
        </div>
      </div>
    </div>
  );
}

function StepPersonalize({
  draft,
  onUpdate,
}: {
  draft: OnboardingDraft;
  onUpdate: (patch: Partial<OnboardingDraft>) => void;
}) {
  return (
    <div className="space-y-5" data-testid="onboarding-step-personalize">
      <p className="text-sm text-[var(--color-text-soft)]">
        Fine-tune your terminal experience.
      </p>

      <label className="block">
        <span className="mb-2 block text-sm text-[var(--color-text-soft)]">Theme</span>
        <Select
          data-testid="onboarding-theme"
          value={draft.theme}
          onChange={(event) => onUpdate({ theme: event.target.value as ThemeMode })}
        >
          {THEME_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </Select>
      </label>

      <label className="block">
        <span className="mb-2 block text-sm text-[var(--color-text-soft)]">
          Terminal font size
        </span>
        <Input
          data-testid="onboarding-font-size"
          type="range"
          min={11}
          max={20}
          step={1}
          value={draft.fontSize}
          onChange={(event) => onUpdate({ fontSize: Number(event.target.value) })}
        />
        <span className="mt-2 block text-xs text-[var(--color-text-muted)]">
          {draft.fontSize}px
        </span>
      </label>

      <label className="flex items-center justify-between rounded-2xl border border-[var(--color-border)] bg-white/3 px-4 py-3">
        <div>
          <span className="block text-sm font-medium">Launch fullscreen</span>
          <span className="block text-xs text-[var(--color-text-muted)]">
            Start Tabby in fullscreen mode by default.
          </span>
        </div>
        <input
          data-testid="onboarding-fullscreen"
          type="checkbox"
          checked={draft.launchFullscreen}
          onChange={(event) => onUpdate({ launchFullscreen: event.target.checked })}
          className="h-5 w-5 accent-[var(--color-accent-strong)]"
        />
      </label>
    </div>
  );
}
