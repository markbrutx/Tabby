import { useEffect, useRef } from "react";
import { FitAddon } from "xterm-addon-fit";
import { WebglAddon } from "xterm-addon-webgl";
import { Terminal } from "xterm";
import type { PaneSnapshot } from "@/features/workspace/domain";
import {
  getTerminalTheme,
  type ResolvedTheme,
} from "@/features/workspace/theme";
import { bridge } from "@/lib/bridge";
import {
  registerPtyOutput,
  initDispatcher,
  teardownDispatcher,
} from "@/lib/bridge/ptyOutputDispatcher";
import { isTauriRuntime } from "@/lib/runtime";

interface UseTerminalSessionOptions {
  pane: PaneSnapshot;
  fontSize: number;
  theme: ResolvedTheme;
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
  fontSize,
  theme,
  active,
  visible,
}: UseTerminalSessionOptions) {
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const webglAddonRef = useRef<WebglAddon | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const pendingDataRef = useRef<string[]>([]);
  const initializedRef = useRef(false);

  // --- Effect 1: PTY output listener (via dispatcher) ---
  // Registers with the centralized dispatcher instead of subscribing
  // independently. One global subscription fans out to N pane handlers.
  useEffect(() => {
    void initDispatcher(bridge);

    const unregister = registerPtyOutput(pane.id, pane.sessionId, (chunk) => {
      if (terminalRef.current && initializedRef.current) {
        try {
          terminalRef.current.write(chunk);
        } catch {
          // Terminal disposed or renderer gone — drop the chunk.
        }
      } else {
        pendingDataRef.current.push(chunk);
      }
    });

    return () => {
      unregister();
      teardownDispatcher();
    };
  }, [pane.id, pane.sessionId]);

  // --- Effect 2: Terminal lifecycle ---
  // Creates & disposes Terminal on session/font changes.
  // Does NOT call open() — that is deferred to Effect 3 (visibility).
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
      theme: getTerminalTheme(theme),
    });

    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);

    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;
    webglAddonRef.current = null;
    initializedRef.current = false;

    const dataDisposable = terminal.onData((data) => {
      void bridge.writePty(pane.id, data);
    });

    // OSC 7 handler — tracks CWD changes reported by the shell.
    // macOS zsh emits OSC 7 automatically. Format: "file://host/path"
    const oscDisposable = terminal.parser.registerOscHandler(7, (data) => {
      try {
        const url = new URL(data);
        const cwd = decodeURIComponent(url.pathname);
        if (cwd) {
          void bridge.trackPaneCwd(pane.id, cwd);
        }
      } catch {
        // Malformed URL — ignore.
      }
      return true;
    });

    const observer = new ResizeObserver(() => {
      if (!initializedRef.current || !terminalRef.current) {
        return;
      }

      safeFit(fitAddon, container);

      if (isTauriRuntime()) {
        try {
          void bridge.resizePty({
            paneId: pane.id,
            cols: terminal.cols,
            rows: terminal.rows,
          });
        } catch {
          // Terminal may be disposed during resize.
        }
      }
    });

    observer.observe(container);

    return () => {
      // 1. Mark uninitialized FIRST — blocks all callbacks
      initializedRef.current = false;
      // 2. Stop observing before disposal
      observer.disconnect();
      // 3. Dispose data and OSC listeners
      dataDisposable.dispose();
      oscDisposable.dispose();
      // 4. Dispose WebGL addon explicitly (can throw on lost context)
      if (webglAddonRef.current) {
        try { webglAddonRef.current.dispose(); } catch { /* lost GL context */ }
        webglAddonRef.current = null;
      }
      // 5. Dispose terminal
      try { terminal.dispose(); } catch { /* already disposed */ }
      terminalRef.current = null;
      fitAddonRef.current = null;
      pendingDataRef.current = [];
    };
  }, [fontSize, pane.id, pane.sessionId]);

  // --- Effect 3: Renderer activation (visibility-gated) ---
  // Opens the terminal in the DOM only when the container is visible.
  // Hidden tabs skip this entirely — no open(), no WebGL, no renderer.
  useEffect(() => {
    const container = containerRef.current;
    const terminal = terminalRef.current;
    const fitAddon = fitAddonRef.current;

    if (!visible || !terminal || !fitAddon || !container) {
      return;
    }

    // Already initialized — just fit and resize. Do NOT call terminal.focus()
    // here: when the user mousedowns to start a text selection, onFocus fires
    // which flips `active` → Effect reruns → focus() would steal the mousedown
    // and break selection. xterm.js handles focus internally via mouse events.
    if (initializedRef.current) {
      if (active) {
        safeFit(fitAddon, container);

        if (isTauriRuntime()) {
          void bridge.resizePty({
            paneId: pane.id,
            cols: terminal.cols,
            rows: terminal.rows,
          });
        }
      }

      return;
    }

    // First time visible — activate renderer.
    const rafId = requestAnimationFrame(() => {
      if (!terminalRef.current || initializedRef.current) {
        return;
      }

      try {
        terminal.open(container);
      } catch {
        // open() can fail on detached or zero-size containers.
        return;
      }

      try {
        const webgl = new WebglAddon();
        terminal.loadAddon(webgl);
        webglAddonRef.current = webgl;
      } catch {
        // WebGL is optional; canvas renderer continues to work.
      }

      // Fit BEFORE marking initialized so the resize event uses correct dims.
      safeFit(fitAddon, container);

      if (isTauriRuntime()) {
        // Resize the PTY to actual container size. The shell will reprint
        // its prompt at the correct dimensions via SIGWINCH.
        void bridge.resizePty({
          paneId: pane.id,
          cols: terminal.cols,
          rows: terminal.rows,
        });

        // Drop buffered output — it was rendered at default 80×24 and will
        // garble in the real viewport. The resize above triggers a redraw.
        pendingDataRef.current = [];
      } else {
        // Dev/mock mode: flush buffer so welcome text is visible.
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

    return () => {
      cancelAnimationFrame(rafId);
    };
  }, [pane.id, visible]);

  // --- Effect 4: Theme ---
  useEffect(() => {
    if (!terminalRef.current || !initializedRef.current) {
      return;
    }

    try {
      terminalRef.current.options.theme = getTerminalTheme(theme);
    } catch {
      // Theme update can fail if renderer is in a bad state.
    }
  }, [theme]);

  // --- Effect 5: Focus on keyboard navigation ---
  // When the user switches panes via Cmd+Alt+Arrow / Cmd+[/], the store
  // updates `activePaneId` but xterm doesn't receive `.focus()`. This effect
  // detects the false→true transition and programmatically focuses the terminal.
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
