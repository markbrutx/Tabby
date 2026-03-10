import { useEffect, useRef, useState } from "react";
import { Terminal, Globe, GitBranch } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { CUSTOM_PROFILE_ID, DEFAULT_BROWSER_URL, type PaneSpec, type SplitDirection } from "@/features/workspace/domain/models";
import type { ProfileReadModel } from "@/features/settings/domain/models";
import { PaneConfigurator, isFieldValuesValid, type PaneFieldValues } from "./PaneConfigurator";

function defaultSpecToFieldValues(spec: PaneSpec): PaneFieldValues {
  switch (spec.kind) {
    case "browser":
      return { mode: "browser", url: spec.initialUrl };
    case "git":
      return { mode: "git", workingDirectory: spec.workingDirectory };
    case "terminal":
      return {
        mode: "terminal",
        profileId: spec.launchProfileId,
        workingDirectory: spec.workingDirectory,
        customCommand: spec.commandOverride ?? "",
      };
  }
}

function makeFieldValues(mode: PaneFieldValues["mode"], previousCwd: string): PaneFieldValues {
  switch (mode) {
    case "browser":
      return { mode: "browser", url: DEFAULT_BROWSER_URL };
    case "git":
      return { mode: "git", workingDirectory: previousCwd };
    case "terminal":
      return { mode: "terminal", profileId: "terminal", workingDirectory: previousCwd, customCommand: "" };
  }
}

function fieldValuesToPaneSpec(values: PaneFieldValues): PaneSpec {
  switch (values.mode) {
    case "browser":
      return { kind: "browser", initialUrl: values.url.trim() || DEFAULT_BROWSER_URL };
    case "git":
      return { kind: "git", workingDirectory: values.workingDirectory };
    case "terminal":
      return {
        kind: "terminal",
        launchProfileId: values.profileId,
        workingDirectory: values.workingDirectory,
        commandOverride: values.profileId === CUSTOM_PROFILE_ID ? values.customCommand.trim() || null : null,
      };
  }
}

function extractCwd(values: PaneFieldValues): string {
  return values.mode === "browser" ? "~" : values.workingDirectory;
}

interface SplitPopupProps {
  direction: SplitDirection;
  profiles: readonly ProfileReadModel[];
  defaultSpec: PaneSpec;
  onConfirm: (paneSpec: PaneSpec) => void;
  onCancel: () => void;
}

export function SplitPopup({
  direction,
  profiles,
  defaultSpec,
  onConfirm,
  onCancel,
}: SplitPopupProps) {
  const [fieldValues, setFieldValues] = useState<PaneFieldValues>(
    () => defaultSpecToFieldValues(defaultSpec),
  );

  const stateRef = useRef(fieldValues);
  useEffect(() => {
    stateRef.current = fieldValues;
  }, [fieldValues]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        onCancel();
      } else if (event.key === "Enter") {
        event.preventDefault();
        if (!isFieldValuesValid(stateRef.current)) return;
        onConfirm(fieldValuesToPaneSpec(stateRef.current));
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onCancel, onConfirm]);

  function handleModeChange(nextMode: PaneFieldValues["mode"]) {
    const currentCwd = extractCwd(fieldValues);
    setFieldValues(makeFieldValues(nextMode, currentCwd));
  }

  function handleConfirm() {
    if (!isFieldValuesValid(fieldValues)) return;
    onConfirm(fieldValuesToPaneSpec(fieldValues));
  }

  const dirLabel = direction === "horizontal" ? "right" : "below";

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
      onClick={(event) => {
        if (event.target === event.currentTarget) onCancel();
      }}
      role="dialog"
    >
      <div className="w-full max-w-sm rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xl">
        <p className="mb-4 text-sm font-medium">
          Split {dirLabel}
        </p>

        <div className="space-y-4">
          <div className="flex w-full gap-1 rounded-lg bg-[var(--color-surface-hover)] p-1">
            <button
              className={`flex flex-1 items-center justify-center gap-2 rounded-md py-1.5 text-sm font-medium transition ${fieldValues.mode === "terminal"
                  ? "bg-[var(--color-surface)] text-[var(--color-text)] shadow-sm ring-1 ring-black/5"
                  : "text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
                }`}
              onClick={() => handleModeChange("terminal")}
            >
              <Terminal size={14} />
              Terminal
            </button>
            <button
              className={`flex flex-1 items-center justify-center gap-2 rounded-md py-1.5 text-sm font-medium transition ${fieldValues.mode === "browser"
                  ? "bg-[var(--color-surface)] text-[var(--color-text)] shadow-sm ring-1 ring-black/5"
                  : "text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
                }`}
              onClick={() => handleModeChange("browser")}
            >
              <Globe size={14} />
              Browser
            </button>
            <button
              className={`flex flex-1 items-center justify-center gap-2 rounded-md py-1.5 text-sm font-medium transition ${fieldValues.mode === "git"
                  ? "bg-[var(--color-surface)] text-[var(--color-text)] shadow-sm ring-1 ring-black/5"
                  : "text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
                }`}
              onClick={() => handleModeChange("git")}
            >
              <GitBranch size={14} />
              Git
            </button>
          </div>

          <PaneConfigurator
            values={fieldValues}
            profiles={profiles}
            onChange={setFieldValues}
            autoFocus
            testIdPrefix="split"
          />
        </div>

        <div className="mt-3 flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            size="sm"
            disabled={!isFieldValuesValid(fieldValues)}
            onClick={handleConfirm}
          >
            Split
          </Button>
        </div>
      </div>
    </div>
  );
}
