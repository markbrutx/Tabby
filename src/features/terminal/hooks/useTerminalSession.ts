import { useEffect, useRef } from "react";
import { FitAddon } from "xterm-addon-fit";
import { WebglAddon } from "xterm-addon-webgl";
import { Terminal } from "xterm";
import { useRuntimeStore } from "@/contexts/stores";
import type { PaneSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import type { ThemeDefinition } from "@/features/theme/domain/models";
import { getTerminalTheme } from "@/features/workspace/theme";
import { registerPtyOutput } from "@/features/terminal/ptyOutputDispatcher";
import { isTauriRuntime } from "@/lib/runtime";

const TERMINAL_FONT_SIZE = 14;

// ---------------------------------------------------------------------------
// Terminal instance stash — preserves scrollback content during pane swaps.
//
// When Effect 2 cleans up (pane.id changed), we stash the terminal instead of
// disposing it.  When Effect 2 sets up at the new position, it reclaims the
// stashed instance and moves its DOM into the new container.
// If nobody reclaims within 200 ms (real close, not swap), we dispose.
// ---------------------------------------------------------------------------

interface StashedTerminal {
  terminal: Terminal;
  fitAddon: FitAddon;
  webglAddon: WebglAddon | null;
}

const terminalStash = new Map<string, StashedTerminal>();

function stashKey(paneId: string, sessionId: string): string {
  return `${paneId}:${sessionId}`;
}

function stashTerminal(key: string, entry: StashedTerminal): void {
  terminalStash.set(key, entry);

  setTimeout(() => {
    const stashed = terminalStash.get(key);
    if (stashed) {
      terminalStash.delete(key);
      if (stashed.webglAddon) {
        try { stashed.webglAddon.dispose(); } catch { /* lost GL ctx */ }
      }
      try { stashed.terminal.dispose(); } catch { /* already gone */ }
    }
  }, 200);
}

function reclaimTerminal(key: string): StashedTerminal | undefined {
  const entry = terminalStash.get(key);
  if (entry) {
    terminalStash.delete(key);
  }
  return entry;
}

// ---------------------------------------------------------------------------

interface UseTerminalSessionOptions {
  pane: PaneSnapshotModel;
  theme: ThemeDefinition;
  active: boolean;
  visible: boolean;
}

function hasContainerSize(container: HTMLElement): boolean {
  return container.offsetWidth > 0 && container.offsetHeight > 0;
}

function safeFit(fitAddon: FitAddon, container: HTMLElement) {
  if (!hasContainerSize(container)) {
    return;
  }

  try {
    fitAddon.fit();
  } catch {
    // FitAddon can throw if the renderer is in a bad state.
  }
}

export function useTerminalSession({
  pane,
  theme,
  active,
  visible,
}: UseTerminalSessionOptions) {
  const writeTerminalInput = useRuntimeStore((s) => s.writeTerminalInput);
  const observeTerminalCwd = useRuntimeStore((s) => s.observeTerminalCwd);
  const resizeTerminal = useRuntimeStore((s) => s.resizeTerminal);
  const initTerminalOutputDispatcher = useRuntimeStore((s) => s.initTerminalOutputDispatcher);
  const teardownTerminalOutputDispatcher = useRuntimeStore((s) => s.teardownTerminalOutputDispatcher);

  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const webglAddonRef = useRef<WebglAddon | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const pendingDataRef = useRef<string[]>([]);
  const initializedRef = useRef(false);
  // Tracks whether the terminal was reclaimed from the stash (swap path).
  const reclaimedRef = useRef(false);

  // Effect 1 — PTY output dispatcher registration
  useEffect(() => {
    if (!pane.sessionId) {
      return;
    }

    void initTerminalOutputDispatcher();

    const unregister = registerPtyOutput(pane.id, pane.sessionId, (chunk) => {
      if (terminalRef.current && initializedRef.current) {
        try {
          terminalRef.current.write(chunk);
        } catch {
          // Terminal disposed or renderer gone.
        }
      } else {
        pendingDataRef.current.push(chunk);
      }
    });

    return () => {
      unregister();
      teardownTerminalOutputDispatcher();
    };
  }, [pane.id, pane.sessionId, initTerminalOutputDispatcher, teardownTerminalOutputDispatcher]);

  // Effect 2 — Terminal instance lifecycle (create / reclaim / stash)
  useEffect(() => {
    const container = containerRef.current;
    if (!container || !pane.sessionId) {
      return;
    }

    const key = stashKey(pane.id, pane.sessionId);
    const reclaimed = reclaimTerminal(key);

    let terminal: Terminal;
    let fitAddon: FitAddon;

    if (reclaimed) {
      // Swap path — reuse the existing terminal (preserves scrollback)
      terminal = reclaimed.terminal;
      fitAddon = reclaimed.fitAddon;
      webglAddonRef.current = reclaimed.webglAddon;
      reclaimedRef.current = true;
    } else {
      // Fresh path — create a new terminal
      terminal = new Terminal({
        allowTransparency: true,
        cursorBlink: true,
        fontFamily:
          '"IBM Plex Mono", "SFMono-Regular", "JetBrains Mono", "Menlo", monospace',
        fontSize: TERMINAL_FONT_SIZE,
        lineHeight: 1.2,
        letterSpacing: 0,
        theme: getTerminalTheme(theme.kind),
      });

      fitAddon = new FitAddon();
      terminal.loadAddon(fitAddon);
      webglAddonRef.current = null;
      reclaimedRef.current = false;
    }

    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;
    initializedRef.current = false;

    const dataDisposable = terminal.onData((data) => {
      void writeTerminalInput(pane.id, data);
    });

    const oscDisposable = terminal.parser.registerOscHandler(7, (data) => {
      try {
        const url = new URL(data);
        const cwd = decodeURIComponent(url.pathname);
        if (cwd) {
          void observeTerminalCwd(pane.id, cwd);
        }
      } catch {
        // Malformed URL - ignore.
      }
      return true;
    });

    const observer = new ResizeObserver(() => {
      if (!initializedRef.current || !terminalRef.current) {
        return;
      }

      safeFit(fitAddon, container);

      if (isTauriRuntime()) {
        void resizeTerminal(pane.id, terminal.cols, terminal.rows);
      }
    });

    observer.observe(container);

    return () => {
      initializedRef.current = false;
      observer.disconnect();
      dataDisposable.dispose();
      oscDisposable.dispose();

      // Stash the terminal for potential swap reuse instead of disposing.
      if (terminalRef.current && fitAddonRef.current) {
        stashTerminal(key, {
          terminal: terminalRef.current,
          fitAddon: fitAddonRef.current,
          webglAddon: webglAddonRef.current,
        });
      }

      terminalRef.current = null;
      fitAddonRef.current = null;
      webglAddonRef.current = null;
      pendingDataRef.current = [];
    };
  }, [pane.id, pane.sessionId, writeTerminalInput, observeTerminalCwd, resizeTerminal, theme]);

  // Effect 3 — Attach terminal to DOM (open or move)
  useEffect(() => {
    const container = containerRef.current;
    const terminal = terminalRef.current;
    const fitAddon = fitAddonRef.current;

    if (!visible || !terminal || !fitAddon || !container) {
      return;
    }

    if (initializedRef.current) {
      if (active) {
        safeFit(fitAddon, container);
        if (isTauriRuntime()) {
          void resizeTerminal(pane.id, terminal.cols, terminal.rows);
        }
      }
      return;
    }

    const rafId = requestAnimationFrame(() => {
      if (!terminalRef.current || initializedRef.current) {
        return;
      }

      // Clear leftover DOM from a previous terminal
      while (container.firstChild) {
        container.removeChild(container.firstChild);
      }

      if (reclaimedRef.current && terminal.element) {
        // Swap path — move the existing DOM element into the new container.
        // This preserves all scrollback content.
        container.appendChild(terminal.element);
      } else {
        // Fresh path — open a brand-new terminal into the container.
        try {
          terminal.open(container);
        } catch {
          return;
        }

        try {
          const webgl = new WebglAddon();
          terminal.loadAddon(webgl);
          webglAddonRef.current = webgl;
        } catch {
          // WebGL is optional.
        }
      }

      safeFit(fitAddon, container);

      // Force xterm.js to repaint all visible rows
      try {
        terminal.refresh(0, terminal.rows - 1);
      } catch {
        // Renderer may not be ready yet.
      }

      if (isTauriRuntime()) {
        void resizeTerminal(pane.id, terminal.cols, terminal.rows);
        pendingDataRef.current = [];
      } else {
        const pending = pendingDataRef.current.splice(0);
        for (const chunk of pending) {
          try {
            terminal.write(chunk);
          } catch {
            break;
          }
        }
      }

      initializedRef.current = true;
    });

    return () => cancelAnimationFrame(rafId);
  }, [pane.id, pane.sessionId, resizeTerminal, visible, active]);

  // Effect 4 — Theme sync
  useEffect(() => {
    if (!terminalRef.current || !initializedRef.current) {
      return;
    }

    try {
      terminalRef.current.options.theme = getTerminalTheme(theme.kind);
    } catch {
      // Renderer may be in a bad state.
    }
  }, [theme]);

  // Effect 5 — Focus tracking
  const prevActiveRef = useRef(false);
  useEffect(() => {
    const wasActive = prevActiveRef.current;
    prevActiveRef.current = active;

    if (!wasActive && active && terminalRef.current && initializedRef.current) {
      if (fitAddonRef.current && containerRef.current) {
        safeFit(fitAddonRef.current, containerRef.current);
      }
      terminalRef.current.focus();
    }
  }, [active]);

  return {
    containerRef,
  };
}
