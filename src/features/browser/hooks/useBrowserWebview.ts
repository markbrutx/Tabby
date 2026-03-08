import { useCallback, useEffect, useRef, useState } from "react";
import { useRuntimeStore } from "@/contexts/stores";
import { DEFAULT_BROWSER_URL } from "@/features/workspace/domain/models";
import type { BrowserBounds } from "@/features/runtime/domain/models";
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
  const ensureBrowserSurface = useRuntimeStore((s) => s.ensureBrowserSurface);
  const setBrowserBounds = useRuntimeStore((s) => s.setBrowserBounds);
  const setBrowserVisible = useRuntimeStore((s) => s.setBrowserVisible);
  const navigateBrowser = useRuntimeStore((s) => s.navigateBrowser);
  const runtimeBrowserLocation = useRuntimeStore(
    (s) => s.runtimes[pane.id]?.browserLocation ?? null,
  );

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
        void setBrowserVisible(pane.id, false).catch(() => undefined);
      }
    };
  }, [pane.id, isTauri, setBrowserVisible]);

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
      void ensureBrowserSurface(
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
  }, [pane.id, visible, isTauri, ensureBrowserSurface]);

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
        void setBrowserBounds(pane.id, bounds).catch(() => undefined);
      });
    });

    observer.observe(container);

    return () => {
      unmounted = true;
      observer.disconnect();
      if (rafId !== null) cancelAnimationFrame(rafId);
    };
  }, [pane.id, isTauri, setBrowserBounds]);

  // Effect 4: Visibility sync
  useEffect(() => {
    if (!isTauri || !createdRef.current) return;
    void setBrowserVisible(pane.id, visible).catch(() => undefined);
  }, [pane.id, visible, isTauri, setBrowserVisible]);

  // Effect 5: Sync browser location from unified runtime store
  useEffect(() => {
    if (!isTauri || !runtimeBrowserLocation) return;
    if (runtimeBrowserLocation === currentUrlRef.current) return;

    setCurrentUrl(runtimeBrowserLocation);
    onUrlChangeRef.current?.(runtimeBrowserLocation);
  }, [isTauri, runtimeBrowserLocation]);

  const navigate = useCallback(
    (url: string) => {
      const normalized = normalizeUrl(url);
      setCurrentUrl(normalized);

      if (isTauri && createdRef.current) {
        void navigateBrowser(pane.id, normalized).catch(() => undefined);
      }
    },
    [pane.id, isTauri, navigateBrowser],
  );

  return { containerRef, currentUrl, navigate };
}
