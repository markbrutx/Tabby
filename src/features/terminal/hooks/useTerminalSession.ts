import { useEffect, useRef } from "react";
import { FitAddon } from "xterm-addon-fit";
import { WebglAddon } from "xterm-addon-webgl";
import { Terminal } from "xterm";
import { useRuntimeClient } from "@/app-shell/context/AppShellContext";
import type { PaneSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import { getTerminalTheme, type ResolvedTheme } from "@/features/workspace/theme";
import { initDispatcher, registerPtyOutput, teardownDispatcher } from "@/features/terminal/ptyOutputDispatcher";
import { isTauriRuntime } from "@/lib/runtime";

interface UseTerminalSessionOptions {
  pane: PaneSnapshotModel;
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
  const runtimeClient = useRuntimeClient();
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const webglAddonRef = useRef<WebglAddon | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const pendingDataRef = useRef<string[]>([]);
  const initializedRef = useRef(false);

  useEffect(() => {
    if (!pane.sessionId) {
      return;
    }

    void initDispatcher(runtimeClient);

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
      teardownDispatcher();
    };
  }, [pane.id, pane.sessionId, runtimeClient]);

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
      void runtimeClient.dispatch({
        kind: "writeTerminalInput",
        pane_id: pane.id,
        input: data,
      });
    });

    const oscDisposable = terminal.parser.registerOscHandler(7, (data) => {
      try {
        const url = new URL(data);
        const cwd = decodeURIComponent(url.pathname);
        if (cwd) {
          void runtimeClient.dispatch({
            kind: "observeTerminalCwd",
            pane_id: pane.id,
            working_directory: cwd,
          });
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
        void runtimeClient.dispatch({
          kind: "resizeTerminal",
          pane_id: pane.id,
          cols: terminal.cols,
          rows: terminal.rows,
        });
      }
    });

    observer.observe(container);

    return () => {
      initializedRef.current = false;
      observer.disconnect();
      dataDisposable.dispose();
      oscDisposable.dispose();
      if (webglAddonRef.current) {
        try {
          webglAddonRef.current.dispose();
        } catch {
          // lost GL context
        }
        webglAddonRef.current = null;
      }
      try {
        terminal.dispose();
      } catch {
        // already disposed
      }
      terminalRef.current = null;
      fitAddonRef.current = null;
      pendingDataRef.current = [];
    };
  }, [fontSize, pane.id, pane.sessionId, runtimeClient, theme]);

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
          void runtimeClient.dispatch({
            kind: "resizeTerminal",
            pane_id: pane.id,
            cols: terminal.cols,
            rows: terminal.rows,
          });
        }
      }
      return;
    }

    const rafId = requestAnimationFrame(() => {
      if (!terminalRef.current || initializedRef.current) {
        return;
      }

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

      safeFit(fitAddon, container);

      if (isTauriRuntime()) {
        void runtimeClient.dispatch({
          kind: "resizeTerminal",
          pane_id: pane.id,
          cols: terminal.cols,
          rows: terminal.rows,
        });
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
  }, [pane.id, runtimeClient, visible, active]);

  useEffect(() => {
    if (!terminalRef.current || !initializedRef.current) {
      return;
    }

    try {
      terminalRef.current.options.theme = getTerminalTheme(theme);
    } catch {
      // Renderer may be in a bad state.
    }
  }, [theme]);

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
