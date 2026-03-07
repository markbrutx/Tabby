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
  if (hasContainerSize(container)) {
    fitAddon.fit();
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
  const rendererReadyRef = useRef(false);
  const pendingWritesRef = useRef<string[]>([]);

  // Activate the renderer: open terminal in DOM, attach WebGL, fit.
  // Called once per terminal lifetime — either from the creation RAF
  // (if the tab is already visible) or from the visibility effect
  // (when a hidden tab becomes visible for the first time).
  function activateRenderer(
    terminal: Terminal,
    fitAddon: FitAddon,
    container: HTMLElement,
  ) {
    if (rendererReadyRef.current) {
      return;
    }

    terminal.open(container);

    try {
      terminal.loadAddon(new WebglAddon());
    } catch {
      // WebGL is optional; xterm falls back to canvas/DOM rendering.
    }

    rendererReadyRef.current = true;

    // Flush any PTY output that arrived while the terminal was hidden.
    for (const chunk of pendingWritesRef.current) {
      terminal.write(chunk);
    }
    pendingWritesRef.current = [];

    safeFit(fitAddon, container);

    if (isTauriRuntime()) {
      void bridge.resizePty({
        paneId: pane.id,
        cols: terminal.cols,
        rows: terminal.rows,
      });
    }
  }

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

    const dataDisposable = terminal.onData((data) => {
      void bridge.writePty(pane.id, data);
    });

    // Defer open + WebGL to next frame so the container has its final
    // layout dimensions.  xterm.js buffers write() calls until open(),
    // so PTY output arriving before this frame is not lost.
    // Only activate if the container is visible (has dimensions);
    // hidden tabs will be activated when they first become visible.
    const rafId = requestAnimationFrame(() => {
      if (!terminalRef.current || rendererReadyRef.current) {
        return;
      }

      if (hasContainerSize(container)) {
        activateRenderer(terminal, fitAddon, container);
      }
    });

    const observer = new ResizeObserver(() => {
      if (!rendererReadyRef.current) {
        return;
      }

      safeFit(fitAddon, container);

      if (isTauriRuntime() && terminalRef.current) {
        void bridge.resizePty({
          paneId: pane.id,
          cols: terminal.cols,
          rows: terminal.rows,
        });
      }
    });

    observer.observe(container);

    let disposeEvent = () => {};
    void bridge
      .listenToPtyOutput((payload) => {
        if (
          payload.paneId !== pane.id ||
          payload.sessionId !== pane.sessionId ||
          !terminalRef.current
        ) {
          return;
        }

        if (rendererReadyRef.current) {
          terminalRef.current.write(payload.chunk);
        } else {
          pendingWritesRef.current.push(payload.chunk);
        }
      })
      .then((unlisten) => {
        disposeEvent = unlisten;
      });

    return () => {
      cancelAnimationFrame(rafId);
      observer.disconnect();
      disposeEvent();
      dataDisposable.dispose();
      terminal.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
      rendererReadyRef.current = false;
      pendingWritesRef.current = [];
    };
  }, [fontSize, pane.id, pane.sessionId]);

  useEffect(() => {
    if (!terminalRef.current || !rendererReadyRef.current) {
      return;
    }

    terminalRef.current.options.theme = getTerminalTheme(theme);
  }, [theme]);

  // When a hidden tab becomes visible for the first time, activate
  // its renderer (deferred from creation because the container had
  // display:none / zero dimensions).  For already-active terminals
  // this just fits and optionally focuses.
  useEffect(() => {
    const container = containerRef.current;
    if (!visible || !terminalRef.current || !fitAddonRef.current || !container) {
      return;
    }

    if (!rendererReadyRef.current) {
      activateRenderer(terminalRef.current, fitAddonRef.current, container);
    }

    if (!rendererReadyRef.current) {
      return;
    }

    safeFit(fitAddonRef.current, container);

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

  return {
    containerRef,
  };
}
