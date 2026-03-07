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
  const containerRef = useRef<HTMLDivElement | null>(null);
  const pendingDataRef = useRef<string[]>([]);
  const initializedRef = useRef(false);

  // --- Effect 1: PTY output listener ---
  // Always active. Writes to terminal if initialized, buffers otherwise.
  useEffect(() => {
    let disposeEvent = () => {};

    void bridge
      .listenToPtyOutput((payload) => {
        if (
          payload.paneId !== pane.id ||
          payload.sessionId !== pane.sessionId
        ) {
          return;
        }

        if (terminalRef.current && initializedRef.current) {
          try {
            terminalRef.current.write(payload.chunk);
          } catch {
            // Terminal disposed or renderer gone — drop the chunk.
          }
        } else {
          pendingDataRef.current.push(payload.chunk);
        }
      })
      .then((unlisten) => {
        disposeEvent = unlisten;
      });

    return () => {
      disposeEvent();
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
    initializedRef.current = false;

    const dataDisposable = terminal.onData((data) => {
      void bridge.writePty(pane.id, data);
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
      observer.disconnect();
      dataDisposable.dispose();
      terminal.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
      initializedRef.current = false;
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

    // Already initialized — just fit and focus.
    if (initializedRef.current) {
      safeFit(fitAddon, container);

      if (active) {
        try {
          terminal.focus();
        } catch {
          // Focus can throw on disposed terminal.
        }

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
        terminal.loadAddon(new WebglAddon());
      } catch {
        // WebGL is optional; canvas renderer continues to work.
      }

      initializedRef.current = true;

      // Flush buffered PTY output.
      const pending = pendingDataRef.current.splice(0);
      for (const chunk of pending) {
        try {
          terminal.write(chunk);
        } catch {
          break;
        }
      }

      safeFit(fitAddon, container);

      if (isTauriRuntime()) {
        void bridge.resizePty({
          paneId: pane.id,
          cols: terminal.cols,
          rows: terminal.rows,
        });
      }
    });

    return () => {
      cancelAnimationFrame(rafId);
    };
  }, [active, pane.id, visible]);

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

  return {
    containerRef,
  };
}
