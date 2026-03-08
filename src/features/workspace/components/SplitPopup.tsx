import { FolderOpen } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/Button";
import { Select } from "@/components/ui/Select";
import { Input } from "@/components/ui/Input";
import {
  CUSTOM_PROFILE_ID,
  BROWSER_PROFILE_ID,
  type PaneProfile,
  type SplitDirection,
} from "@/features/workspace/domain";
import { pickDirectory } from "@/lib/pickDirectory";

interface SplitPopupProps {
  direction: SplitDirection;
  profiles: PaneProfile[];
  defaultProfileId: string;
  defaultCwd: string;
  onConfirm: (profileId: string, cwd: string, startupCommand: string | null) => void;
  onCancel: () => void;
}

export function SplitPopup({
  direction,
  profiles,
  defaultProfileId,
  defaultCwd,
  onConfirm,
  onCancel,
}: SplitPopupProps) {
  const resolvedDefaultProfileId = profiles.some((profile) => profile.id === defaultProfileId)
    ? defaultProfileId
    : (profiles.find((profile) => profile.id === "terminal")?.id ?? "terminal");
  const [profileId, setProfileId] = useState(resolvedDefaultProfileId);
  const [cwd, setCwd] = useState(defaultCwd);
  const [customCommand, setCustomCommand] = useState("");
  const isCustomProfileInvalid =
    profileId === CUSTOM_PROFILE_ID && !customCommand.trim();

  const stateRef = useRef({ profileId, cwd, customCommand });
  useEffect(() => {
    stateRef.current = { profileId, cwd, customCommand };
  });

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        onCancel();
      } else if (event.key === "Enter") {
        const { profileId: pid, cwd: dir, customCommand: cmd } = stateRef.current;
        if (pid === CUSTOM_PROFILE_ID && !cmd.trim()) {
          return;
        }

        event.preventDefault();
        onConfirm(
          pid,
          dir,
          pid === CUSTOM_PROFILE_ID ? cmd.trim() || null : null,
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
    if (isCustomProfileInvalid) {
      return;
    }

    onConfirm(
      profileId,
      cwd,
      profileId === CUSTOM_PROFILE_ID ? customCommand.trim() || null : null,
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
      <div className="w-full max-w-xs rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-xl">
        <p className="mb-3 text-sm font-medium">
          Split {dirLabel}
        </p>

        <div className="space-y-3">
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

          {profileId !== BROWSER_PROFILE_ID ? (
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
          ) : null}
        </div>

        <div className="mt-3 flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={onCancel}>
            Cancel
          </Button>
          <Button size="sm" disabled={isCustomProfileInvalid} onClick={handleConfirm}>
            Split
          </Button>
        </div>
      </div>
    </div>
  );
}
