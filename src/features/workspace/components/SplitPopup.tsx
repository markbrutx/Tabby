import { FolderOpen } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import type { PaneSpecDto } from "@/contracts/tauri-bindings";
import { CUSTOM_PROFILE_ID, type PaneSpec, type SplitDirection } from "@/features/workspace/domain/models";
import type { ProfileReadModel } from "@/features/settings/domain/models";
import { pickDirectory } from "@/lib/pickDirectory";

interface SplitPopupProps {
  direction: SplitDirection;
  profiles: readonly ProfileReadModel[];
  defaultSpec: PaneSpec;
  onConfirm: (paneSpec: PaneSpecDto) => void;
  onCancel: () => void;
}

export function SplitPopup({
  direction,
  profiles,
  defaultSpec,
  onConfirm,
  onCancel,
}: SplitPopupProps) {
  const initialMode = defaultSpec.kind === "browser" ? "browser" : "terminal";
  const [mode, setMode] = useState<"terminal" | "browser">(initialMode);
  const [profileId, setProfileId] = useState(
    defaultSpec.kind === "terminal" ? defaultSpec.launchProfileId : "terminal",
  );
  const [cwd, setCwd] = useState(
    defaultSpec.kind === "terminal" ? defaultSpec.workingDirectory : "~",
  );
  const [customCommand, setCustomCommand] = useState(
    defaultSpec.kind === "terminal" ? defaultSpec.commandOverride ?? "" : "",
  );
  const [url, setUrl] = useState(
    defaultSpec.kind === "browser" ? defaultSpec.initialUrl : "https://google.com",
  );

  const stateRef = useRef({ mode, profileId, cwd, customCommand, url });
  useEffect(() => {
    stateRef.current = { mode, profileId, cwd, customCommand, url };
  }, [mode, profileId, cwd, customCommand, url]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        onCancel();
      } else if (event.key === "Enter") {
        event.preventDefault();
        const { mode: nextMode, profileId: nextProfileId, cwd: nextCwd, customCommand: nextCommand, url: nextUrl } = stateRef.current;
        if (nextMode === "terminal" && nextProfileId === CUSTOM_PROFILE_ID && !nextCommand.trim()) {
          return;
        }

        onConfirm(
          nextMode === "browser"
            ? { kind: "browser", initial_url: nextUrl.trim() || "https://google.com" }
            : {
                kind: "terminal",
                launch_profile_id: nextProfileId,
                working_directory: nextCwd,
                command_override:
                  nextProfileId === CUSTOM_PROFILE_ID ? nextCommand.trim() || null : null,
              },
        );
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onCancel, onConfirm]);

  async function handlePickDirectory() {
    const selected = await pickDirectory(cwd);
    if (selected) {
      setCwd(selected);
    }
  }

  function handleConfirm() {
    if (mode === "terminal" && profileId === CUSTOM_PROFILE_ID && !customCommand.trim()) {
      return;
    }

    onConfirm(
      mode === "browser"
        ? { kind: "browser", initial_url: url.trim() || "https://google.com" }
        : {
            kind: "terminal",
            launch_profile_id: profileId,
            working_directory: cwd,
            command_override: profileId === CUSTOM_PROFILE_ID ? customCommand.trim() || null : null,
          },
    );
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
      <div className="w-full max-w-sm rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-xl">
        <p className="mb-3 text-sm font-medium">
          Split {dirLabel}
        </p>

        <div className="space-y-3">
          <Select
            value={mode}
            onChange={(event) => setMode(event.target.value as "terminal" | "browser")}
            className="text-sm"
          >
            <option value="terminal">Terminal</option>
            <option value="browser">Browser</option>
          </Select>

          {mode === "browser" ? (
            <Input
              value={url}
              onChange={(event) => setUrl(event.target.value)}
              placeholder="https://google.com"
              className="text-sm"
              autoFocus
            />
          ) : (
            <>
              <Select
                value={profileId}
                onChange={(event) => setProfileId(event.target.value)}
                className="text-sm"
              >
                {profiles.map((profile) => (
                  <option key={profile.id} value={profile.id}>
                    {profile.label}
                  </option>
                ))}
              </Select>

              {profileId === CUSTOM_PROFILE_ID ? (
                <Input
                  value={customCommand}
                  onChange={(event) => setCustomCommand(event.target.value)}
                  placeholder="Custom command"
                  className="text-sm"
                  autoFocus
                />
              ) : null}

              <div className="flex gap-2">
                <Input
                  value={cwd}
                  onChange={(event) => setCwd(event.target.value)}
                  placeholder="Working directory"
                  className="text-sm"
                />
                <Button variant="secondary" size="sm" onClick={() => void handlePickDirectory()}>
                  <FolderOpen size={14} />
                </Button>
              </div>
            </>
          )}
        </div>

        <div className="mt-3 flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            size="sm"
            disabled={mode === "terminal" && profileId === CUSTOM_PROFILE_ID && !customCommand.trim()}
            onClick={handleConfirm}
          >
            Split
          </Button>
        </div>
      </div>
    </div>
  );
}
