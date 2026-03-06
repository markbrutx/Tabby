import { useEffect, useState } from "react";
import type { PaneProfile, PaneSnapshot } from "@/features/workspace/domain";
import { pickDirectory } from "@/lib/pickDirectory";
import { PaneControls } from "@/features/workspace/components/PaneControls";
import { PaneHeader } from "@/features/workspace/components/PaneHeader";
import { useTerminalSession } from "@/features/workspace/hooks/useTerminalSession";

interface TerminalPaneProps {
  pane: PaneSnapshot;
  profiles: PaneProfile[];
  fontSize: number;
  active: boolean;
  visible: boolean;
  onFocus: (paneId: string) => Promise<void>;
  onUpdateProfile: (
    paneId: string,
    profileId: string,
    startupCommand?: string | null,
  ) => Promise<void>;
  onUpdateCwd: (paneId: string, cwd: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
}

export function TerminalPane({
  pane,
  profiles,
  fontSize,
  active,
  visible,
  onFocus,
  onUpdateProfile,
  onUpdateCwd,
  onRestart,
}: TerminalPaneProps) {
  const [cwdDraft, setCwdDraft] = useState(pane.cwd);
  const [profileDraft, setProfileDraft] = useState(pane.profileId);
  const [commandDraft, setCommandDraft] = useState(pane.startupCommand ?? "");
  const [isApplying, setIsApplying] = useState(false);
  const { containerRef } = useTerminalSession({
    pane,
    fontSize,
    active,
    visible,
  });

  useEffect(() => {
    setCwdDraft(pane.cwd);
    setProfileDraft(pane.profileId);
    setCommandDraft(pane.startupCommand ?? "");
  }, [pane.cwd, pane.profileId, pane.sessionId, pane.startupCommand]);

  async function applyProfile() {
    setIsApplying(true);
    await onUpdateProfile(
      pane.id,
      profileDraft,
      profileDraft === "custom" ? commandDraft : null,
    );
    setIsApplying(false);
  }

  async function applyCwd() {
    setIsApplying(true);
    await onUpdateCwd(pane.id, cwdDraft);
    setIsApplying(false);
  }

  async function chooseDirectory() {
    const selected = await pickDirectory(cwdDraft || pane.cwd);
    if (selected) {
      setCwdDraft(selected);
    }
  }

  function handleSelectProfile(nextProfile: string) {
    setProfileDraft(nextProfile);
    if (nextProfile !== "custom") {
      void onUpdateProfile(pane.id, nextProfile, null);
    }
  }

  return (
    <div
      data-testid={`pane-${pane.id}`}
      className={`surface-panel flex h-full min-h-[220px] flex-col overflow-hidden rounded-[24px] ${
        active ? "border-[var(--color-accent-strong)]" : ""
      }`}
      onMouseDown={() => void onFocus(pane.id)}
    >
      <PaneHeader
        pane={pane}
        profiles={profiles}
        active={active}
        profileDraft={profileDraft}
        isApplying={isApplying}
        onSelectProfile={handleSelectProfile}
        onChooseDirectory={() => void chooseDirectory()}
        onRestart={() => void onRestart(pane.id)}
      />

      <PaneControls
        paneId={pane.id}
        active={active}
        profileDraft={profileDraft}
        cwdDraft={cwdDraft}
        commandDraft={commandDraft}
        isApplying={isApplying}
        onCwdChange={setCwdDraft}
        onCommandChange={setCommandDraft}
        onApplyCwd={() => void applyCwd()}
        onApplyProfile={() => void applyProfile()}
      />

      <div className="terminal-shell min-h-0 flex-1" onDoubleClick={() => void onRestart(pane.id)}>
        <div ref={containerRef} className="h-full w-full px-3 py-2" />
      </div>
    </div>
  );
}
