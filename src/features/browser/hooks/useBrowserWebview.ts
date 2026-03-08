import { useCallback, useEffect, useRef, useState } from "react";
import type { PaneSnapshot } from "@/features/workspace/domain";
import { DEFAULT_BROWSER_URL } from "@/features/workspace/domain";
import type { BrowserBounds } from "@/lib/bridge";
import { useTransport } from "@/lib/bridge/TransportContext";
import { isTauriRuntime } from "@/lib/runtime";

interface UseBrowserWebviewOptions {
  pane: PaneSnapshot;
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
  const bridge = useTransport();
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
        void bridge.setBrowserWebviewVisible(pane.id, false).catch(() => undefined);
      }
    };
  }, [pane.id, isTauri]);

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
      void bridge.createBrowserWebview(
        pane.id,
        normalizeUrl(currentUrlRef.current),
        bounds,
      ).catch(() => {
        createdRef.current = false;
        lastBoundsRef.current = null;
      });
    });

    return () => {
      cancelled = true;
      cancelAnimationFrame(rafId);
    };
  }, [pane.id, visible, isTauri]);

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
        void bridge.setBrowserWebviewBounds(pane.id, bounds).catch(() => undefined);
      });
    });

    observer.observe(container);

    return () => {
      unmounted = true;
      observer.disconnect();
      if (rafId !== null) cancelAnimationFrame(rafId);
    };
  }, [pane.id, isTauri]);

  // Effect 4: Visibility sync
  useEffect(() => {
    if (!isTauri || !createdRef.current) return;
    void bridge.setBrowserWebviewVisible(pane.id, visible).catch(() => undefined);
  }, [pane.id, visible, isTauri]);

  // Effect 5: URL change events from native webview
  useEffect(() => {
    if (!isTauri) return;

    let cancelled = false;
    let unlisten: (() => void) | null = null;

    void bridge.listenToBrowserUrlChanged((event) => {
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
  }, [pane.id, isTauri]);

  const navigate = useCallback(
    (url: string) => {
      const normalized = normalizeUrl(url);
      setCurrentUrl(normalized);

      if (isTauri && createdRef.current) {
        void bridge.navigateBrowser(pane.id, normalized).catch(() => undefined);
      }
    },
    [pane.id, isTauri],
  );

  return { containerRef, currentUrl, navigate };
}
