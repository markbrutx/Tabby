import { useEffect, useRef } from "react";
import { FitAddon } from "xterm-addon-fit";
import { WebglAddon } from "xterm-addon-webgl";
import { Terminal } from "xterm";
import type { PaneSnapshot } from "@/features/workspace/domain";
import { bridge } from "@/lib/bridge";
import { isTauriRuntime } from "@/lib/runtime";

interface UseTerminalSessionOptions {
  pane: PaneSnapshot;
  fontSize: number;
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
  active,
  visible,
}: UseTerminalSessionOptions) {
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);

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

    terminal.open(container);

    // Defer WebGL addon and initial fit to next frame so the container
    // has its final layout dimensions. Without this, the WebGL renderer
    // crashes on `_renderer.value.dimensions` in Tauri's webview.
    const rafId = requestAnimationFrame(() => {
      if (!terminalRef.current) {
        return;
      }

      try {
        terminal.loadAddon(new WebglAddon());
      } catch {
        // WebGL is optional; xterm falls back to canvas/DOM rendering.
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

    const dataDisposable = terminal.onData((data) => {
      void bridge.writePty(pane.id, data);
    });

    terminalRef.current = terminal;

    const observer = new ResizeObserver(() => {
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
    };
  }, [fontSize, pane.id, pane.sessionId]);

  useEffect(() => {
    const container = containerRef.current;
    if (!visible || !terminalRef.current || !fitAddonRef.current || !container) {
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
