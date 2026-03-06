import { FolderOpen, RotateCcw } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import { FitAddon } from "xterm-addon-fit";
import { WebglAddon } from "xterm-addon-webgl";
import { Terminal } from "xterm";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import type { PaneProfile, PaneSnapshot } from "@/features/workspace/domain";
import { bridge } from "@/lib/bridge";
import { isTauriRuntime } from "@/lib/runtime";

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
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [cwdDraft, setCwdDraft] = useState(pane.cwd);
  const [profileDraft, setProfileDraft] = useState(pane.profileId);
  const [commandDraft, setCommandDraft] = useState(pane.startupCommand ?? "");
  const [isApplying, setIsApplying] = useState(false);

  useEffect(() => {
    setCwdDraft(pane.cwd);
    setProfileDraft(pane.profileId);
    setCommandDraft(pane.startupCommand ?? "");
  }, [pane.cwd, pane.profileId, pane.sessionId, pane.startupCommand]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return;
    }

    const terminal = new Terminal({
      allowTransparency: true,
      cursorBlink: true,
      fontFamily:
        '"IBM Plex Mono", "SFMono-Regular", "JetBrains Mono", "Menlo", monospace',
      fontSize,
      lineHeight: 1.2,
      letterSpacing: 0,
      theme: {
        background: "#05070b",
        foreground: "#f6efe9",
        cursor: "#f597b8",
        selectionBackground: "rgba(245,151,184,0.25)",
        black: "#090b10",
        red: "#ff8da1",
        green: "#7de4b8",
        yellow: "#f4bf75",
        blue: "#91a9ff",
        magenta: "#f597b8",
        cyan: "#72d5ff",
        white: "#f6efe9",
        brightBlack: "#6c7482",
        brightRed: "#ffadb9",
        brightGreen: "#98f0cb",
        brightYellow: "#ffd099",
        brightBlue: "#b6c4ff",
        brightMagenta: "#ffc0d4",
        brightCyan: "#90defd",
        brightWhite: "#ffffff",
      },
    });

    const fitAddon = new FitAddon();
    fitAddonRef.current = fitAddon;
    terminal.loadAddon(fitAddon);

    try {
      terminal.loadAddon(new WebglAddon());
    } catch {
      // WebGL is a progressive enhancement; xterm falls back to canvas/DOM.
    }

    terminal.open(container);
    fitAddon.fit();

    if (!isTauriRuntime()) {
      terminal.writeln("");
      terminal.writeln("Tabby live PTY sessions require `bun run tauri dev`.");
      terminal.writeln("The rest of the interface remains available for styling.");
    }

    const dataDisposable = terminal.onData((data) => {
      if (!isTauriRuntime()) {
        return;
      }

      void bridge.writePty(pane.id, data);
    });

    terminalRef.current = terminal;

    const observer = new ResizeObserver(() => {
      fitAddon.fit();
      if (!isTauriRuntime()) {
        return;
      }

      void bridge.resizePty({
        paneId: pane.id,
        cols: terminal.cols,
        rows: terminal.rows,
      });
    });

    observer.observe(container);

    let disposeEvent = () => {};
    void bridge.listenToPtyOutput((payload) => {
      if (
        payload.paneId === pane.id &&
        payload.sessionId === pane.sessionId &&
        terminalRef.current
      ) {
        terminalRef.current.write(payload.chunk);
      }
    }).then((unlisten) => {
      disposeEvent = unlisten;
    });

    return () => {
      observer.disconnect();
      disposeEvent();
      dataDisposable.dispose();
      terminal.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
    };
  }, [fontSize, onFocus, pane.id, pane.sessionId]);

  useEffect(() => {
    if (!visible || !terminalRef.current || !fitAddonRef.current) {
      return;
    }

    fitAddonRef.current.fit();

    if (active) {
      terminalRef.current.focus();
      if (isTauriRuntime()) {
        void bridge.resizePty({
          paneId: pane.id,
          cols: terminalRef.current.cols,
          rows: terminalRef.current.rows,
        });
      }
    }
  }, [active, pane.id, visible]);

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
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: cwdDraft || pane.cwd,
    });

    if (typeof selected === "string") {
      setCwdDraft(selected);
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
      <div className="border-b border-[var(--color-border)] bg-black/15 px-4 py-3">
        <div className="flex items-start gap-3">
          <div
            className={`mt-1.5 h-2.5 w-2.5 rounded-full ${
              active ? "bg-[var(--color-success)]" : "bg-white/20"
            }`}
          />
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <p className="truncate text-sm font-medium">{pane.title}</p>
              <span
                data-testid={`profile-badge-${pane.id}`}
                className="rounded-full bg-white/6 px-2 py-0.5 text-[10px] uppercase tracking-[0.18em] text-[var(--color-text-muted)]"
              >
                {pane.profileLabel}
              </span>
            </div>
            <p className="mt-1 truncate text-xs text-[var(--color-text-soft)]">
              {pane.cwd}
            </p>
          </div>
          <div className="flex min-w-[160px] shrink-0 items-center gap-2">
            <Select
              data-testid={`profile-select-${pane.id}`}
              className="h-8 text-xs"
              value={profileDraft}
              onChange={(event) => {
                const nextProfile = event.target.value;
                setProfileDraft(nextProfile);
                if (nextProfile !== "custom") {
                  void onUpdateProfile(pane.id, nextProfile, null);
                }
              }}
            >
              {profiles.map((profile) => (
                <option key={profile.id} value={profile.id}>
                  {profile.label}
                </option>
              ))}
            </Select>
            <Button variant="secondary" size="sm" onClick={() => void chooseDirectory()}>
              <FolderOpen size={14} />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => void onRestart(pane.id)}
              disabled={isApplying}
            >
              <RotateCcw size={14} />
            </Button>
          </div>
        </div>

        {active ? (
          <div className="mt-3 grid gap-2 lg:grid-cols-[minmax(0,1fr)_auto]">
            <div className="flex gap-2">
              <Input
                data-testid={`cwd-input-${pane.id}`}
                value={cwdDraft}
                onChange={(event) => setCwdDraft(event.target.value)}
                placeholder="Working directory"
                className="h-9 text-xs"
              />
              <Button
                variant="secondary"
                size="sm"
                onClick={() => void applyCwd()}
                disabled={isApplying}
              >
                Apply cwd
              </Button>
            </div>

            {profileDraft === "custom" ? (
              <div className="flex gap-2">
                <Input
                  data-testid={`command-input-${pane.id}`}
                  value={commandDraft}
                  onChange={(event) => setCommandDraft(event.target.value)}
                  placeholder="Custom command"
                  className="h-9 text-xs"
                />
                <Button
                  size="sm"
                  onClick={() => void applyProfile()}
                  disabled={isApplying || !commandDraft.trim()}
                >
                  Launch
                </Button>
              </div>
            ) : (
              <div className="rounded-xl border border-[var(--color-border)] bg-white/4 px-3 py-2 text-xs text-[var(--color-text-muted)]">
                Built-in profiles relaunch instantly when selected.
              </div>
            )}
          </div>
        ) : null}
      </div>

      <div className="terminal-shell min-h-0 flex-1" onDoubleClick={() => void onRestart(pane.id)}>
        <div ref={containerRef} className="h-full w-full px-3 py-2" />
      </div>
    </div>
  );
}
