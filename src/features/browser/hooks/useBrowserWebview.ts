import { useCallback, useEffect, useRef, useState } from "react";
import { useRuntimeClient } from "@/app-shell/context/AppShellContext";
import { DEFAULT_BROWSER_URL, type BrowserBounds } from "@/features/workspace/domain";
import type { PaneSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import { isTauriRuntime } from "@/lib/runtime";

interface UseBrowserWebviewOptions {
  pane: PaneSnapshotModel;
  visible: boolean;
  onUrlChange?: (url: string) => void;
}

interface UseBrowserWebviewResult {
  containerRef: React.RefObject<HTMLDivElement>;
  currentUrl: string;
  navigate: (url: string) => void;
}

export function normalizeUrl(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) return DEFAULT_BROWSER_URL;

  if (!/^https?:\/\//i.test(trimmed)) {
    return `https://${trimmed}`;
  }

  return trimmed;
}

function hasRenderableBounds(bounds: BrowserBounds): boolean {
  return bounds.width > 0 && bounds.height > 0;
}

function boundsFromElement(container: HTMLDivElement): BrowserBounds {
  const rect = container.getBoundingClientRect();
  return {
    x: rect.x,
    y: rect.y,
    width: rect.width,
    height: rect.height,
  };
}

function sameBounds(
  current: BrowserBounds | null,
  next: BrowserBounds,
): boolean {
  return (
    current?.x === next.x &&
    current?.y === next.y &&
    current?.width === next.width &&
    current?.height === next.height
  );
}


export function useBrowserWebview({
  pane,
  visible,
  onUrlChange,
}: UseBrowserWebviewOptions): UseBrowserWebviewResult {
  const runtimeClient = useRuntimeClient();
  const containerRef = useRef<HTMLDivElement>(null);
  const [currentUrl, setCurrentUrl] = useState(pane.url ?? DEFAULT_BROWSER_URL);
  const createdRef = useRef(false);
  const currentUrlRef = useRef(currentUrl);
  const lastBoundsRef = useRef<BrowserBounds | null>(null);
  const isTauri = isTauriRuntime();

  // Stable ref for onUrlChange to avoid re-registering the listener on every render
  const onUrlChangeRef = useRef(onUrlChange);
  onUrlChangeRef.current = onUrlChange;
  currentUrlRef.current = currentUrl;
  useEffect(() => {
    setCurrentUrl(pane.url ?? DEFAULT_BROWSER_URL);
  }, [pane.id, pane.url]);

  // Effect 1: Destroy native webview on unmount / pane switch
  useEffect(() => {
    if (!isTauri) return;

    return () => {
      if (createdRef.current) {
        createdRef.current = false;
        lastBoundsRef.current = null;
        // Browser panes can transiently unmount during layout changes (for example split/remount).
        // Hide instead of closing so the native webview survives those React tree moves.
        void runtimeClient.dispatchBrowserSurface({
          kind: "setVisible",
          pane_id: pane.id,
          visible: false,
        }).catch(() => undefined);
      }
    };
  }, [pane.id, isTauri, runtimeClient]);

  // Effect 2: Lazily create the native webview the first time the pane is shown.
  useEffect(() => {
    if (!isTauri || createdRef.current || !visible) return;

    const container = containerRef.current;
    if (!container) return;

    let cancelled = false;

    const rafId = requestAnimationFrame(() => {
      if (cancelled || createdRef.current) return;

      const bounds = boundsFromElement(container);
      if (!hasRenderableBounds(bounds)) return;

      createdRef.current = true;
      lastBoundsRef.current = bounds;
      void runtimeClient.dispatchBrowserSurface({
        kind: "ensure",
        pane_id: pane.id,
        url: normalizeUrl(currentUrlRef.current),
        bounds,
      }).catch(() => {
        createdRef.current = false;
        lastBoundsRef.current = null;
      });
    });

    return () => {
      cancelled = true;
      cancelAnimationFrame(rafId);
    };
  }, [pane.id, visible, isTauri, runtimeClient]);

  // Effect 3: Resize sync via ResizeObserver
  useEffect(() => {
    if (!isTauri) return;

    const container = containerRef.current;
    if (!container) return;

    let rafId: number | null = null;
    let unmounted = false;

    const observer = new ResizeObserver(() => {
      if (rafId !== null) cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(() => {
        rafId = null;
        if (unmounted || !createdRef.current) return;
        const bounds = boundsFromElement(container);
        if (!hasRenderableBounds(bounds) || sameBounds(lastBoundsRef.current, bounds)) {
          return;
        }

        lastBoundsRef.current = bounds;
        void runtimeClient.dispatchBrowserSurface({
          kind: "setBounds",
          pane_id: pane.id,
          bounds,
        }).catch(() => undefined);
      });
    });

    observer.observe(container);

    return () => {
      unmounted = true;
      observer.disconnect();
      if (rafId !== null) cancelAnimationFrame(rafId);
    };
  }, [pane.id, isTauri, runtimeClient]);

  // Effect 4: Visibility sync
  useEffect(() => {
    if (!isTauri || !createdRef.current) return;
    void runtimeClient.dispatchBrowserSurface({
      kind: "setVisible",
      pane_id: pane.id,
      visible,
    }).catch(() => undefined);
  }, [pane.id, visible, isTauri, runtimeClient]);

  // Effect 5: URL change events from native webview
  useEffect(() => {
    if (!isTauri) return;

    let cancelled = false;
    let unlisten: (() => void) | null = null;

    void runtimeClient.listenBrowserLocationObserved((event) => {
      if (cancelled) return;
      if (event.paneId === pane.id) {
        setCurrentUrl(event.url);
        onUrlChangeRef.current?.(event.url);
      }
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [pane.id, isTauri, runtimeClient]);

  const navigate = useCallback(
    (url: string) => {
      const normalized = normalizeUrl(url);
      setCurrentUrl(normalized);

      if (isTauri && createdRef.current) {
        void runtimeClient.dispatch({
          kind: "navigateBrowser",
          pane_id: pane.id,
          url: normalized,
        }).catch(() => undefined);
      }
    },
    [pane.id, isTauri, runtimeClient],
  );

  return { containerRef, currentUrl, navigate };
}
