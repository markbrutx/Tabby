import { useEffect, useState } from "react";
import { CUSTOM_PROFILE_ID, type PaneProfile, type PaneSnapshot } from "@/features/workspace/domain";
import { pickDirectory } from "@/lib/pickDirectory";
import { PaneHeader } from "@/features/workspace/components/PaneHeader";
import type { ResolvedTheme } from "@/features/workspace/theme";
import { useTerminalSession } from "@/features/workspace/hooks/useTerminalSession";

interface TerminalPaneProps {
  pane: PaneSnapshot;
  profiles: PaneProfile[];
  fontSize: number;
  theme: ResolvedTheme;
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
  theme,
  active,
  visible,
  onFocus,
  onUpdateProfile,
  onUpdateCwd,
  onRestart,
}: TerminalPaneProps) {
  const [profileDraft, setProfileDraft] = useState(pane.profileId);
  const [commandDraft, setCommandDraft] = useState(pane.startupCommand ?? "");
  const [isApplying, setIsApplying] = useState(false);
  const { containerRef } = useTerminalSession({
    pane,
    fontSize,
    theme,
    active,
    visible,
  });

  useEffect(() => {
    setProfileDraft(pane.profileId);
    setCommandDraft(pane.startupCommand ?? "");
  }, [pane.profileId, pane.sessionId, pane.startupCommand]);

  async function applyProfile() {
    setIsApplying(true);
    try {
      await onUpdateProfile(
        pane.id,
        profileDraft,
        profileDraft === CUSTOM_PROFILE_ID ? commandDraft : null,
      );
    } finally {
      setIsApplying(false);
    }
  }

  async function chooseDirectory() {
    const selected = await pickDirectory(pane.cwd);
    if (selected) {
      setIsApplying(true);
      try {
        await onUpdateCwd(pane.id, selected);
      } finally {
        setIsApplying(false);
      }
    }
  }

  function handleSelectProfile(nextProfile: string) {
    setProfileDraft(nextProfile);
    if (nextProfile !== CUSTOM_PROFILE_ID) {
      void onUpdateProfile(pane.id, nextProfile, null);
    }
  }

  return (
    <div
      data-testid={`pane-${pane.id}`}
      data-active={active ? "true" : "false"}
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
        commandDraft={commandDraft}
        isApplying={isApplying}
        onSelectProfile={handleSelectProfile}
        onCommandChange={setCommandDraft}
        onApplyProfile={() => void applyProfile()}
        onChooseDirectory={() => void chooseDirectory()}
        onRestart={() => void onRestart(pane.id)}
      />

      <div className="terminal-shell min-h-0 flex-1" onDoubleClick={() => void onRestart(pane.id)}>
        <div ref={containerRef} className="h-full w-full px-3 py-2" />
      </div>
    </div>
  );
}
