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
    const rafId = requestAnimationFrame(() => {
      if (!terminalRef.current) {
        return;
      }

      terminal.open(container);

      try {
        terminal.loadAddon(new WebglAddon());
      } catch {
        // WebGL is optional; xterm falls back to canvas/DOM rendering.
      }

      rendererReadyRef.current = true;
      safeFit(fitAddon, container);

      if (isTauriRuntime()) {
        void bridge.resizePty({
          paneId: pane.id,
          cols: terminal.cols,
          rows: terminal.rows,
        });
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
          payload.paneId === pane.id &&
          payload.sessionId === pane.sessionId &&
          terminalRef.current
        ) {
          terminalRef.current.write(payload.chunk);
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
    };
  }, [fontSize, pane.id, pane.sessionId]);

  useEffect(() => {
    if (!terminalRef.current || !rendererReadyRef.current) {
      return;
    }

    terminalRef.current.options.theme = getTerminalTheme(theme);
  }, [theme]);

  useEffect(() => {
    const container = containerRef.current;
    if (
      !visible ||
      !terminalRef.current ||
      !fitAddonRef.current ||
      !rendererReadyRef.current ||
      !container
    ) {
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
