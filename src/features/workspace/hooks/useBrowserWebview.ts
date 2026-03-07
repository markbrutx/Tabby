import { useCallback, useEffect, useRef, useState } from "react";
import type { PaneSnapshot } from "@/features/workspace/domain";
import { DEFAULT_BROWSER_URL } from "@/features/workspace/domain";
import { bridge, type BrowserBounds } from "@/lib/bridge";
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

function normalizeUrl(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) return DEFAULT_BROWSER_URL;

  if (!/^https?:\/\//i.test(trimmed)) {
    return `https://${trimmed}`;
  }

  return trimmed;
}

export function useBrowserWebview({
  pane,
  visible,
  onUrlChange,
}: UseBrowserWebviewOptions): UseBrowserWebviewResult {
  const containerRef = useRef<HTMLDivElement>(null);
  const [currentUrl, setCurrentUrl] = useState(pane.url ?? DEFAULT_BROWSER_URL);
  const createdRef = useRef(false);
  const isTauri = isTauriRuntime();

  // Stable ref for onUrlChange to avoid re-registering the listener on every render
  const onUrlChangeRef = useRef(onUrlChange);
  useEffect(() => { onUrlChangeRef.current = onUrlChange; });

  // Effect 1: Create / Destroy native webview
  useEffect(() => {
    if (!isTauri) return;

    const container = containerRef.current;
    if (!container) return;

    let cancelled = false;

    // Use rAF to ensure layout paint has completed before reading bounds
    const rafId = requestAnimationFrame(() => {
      if (cancelled) return;

      const rect = container.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) return;

      const bounds: BrowserBounds = {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
      };

      createdRef.current = true;
      void bridge.createBrowserWebview(pane.id, normalizeUrl(currentUrl), bounds);
    });

    return () => {
      cancelled = true;
      cancelAnimationFrame(rafId);
      if (createdRef.current) {
        createdRef.current = false;
        void bridge.closeBrowserWebview(pane.id);
      }
    };
    // Only create/destroy on mount/unmount — do NOT depend on currentUrl
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pane.id, isTauri]);

  // Effect 2: Resize sync via ResizeObserver
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
        const rect = container.getBoundingClientRect();
        if (rect.width <= 0 || rect.height <= 0) return;
        void bridge.setBrowserWebviewBounds(pane.id, {
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height,
        });
      });
    });

    observer.observe(container);

    return () => {
      unmounted = true;
      observer.disconnect();
      if (rafId !== null) cancelAnimationFrame(rafId);
    };
  }, [pane.id, isTauri]);

  // Effect 3: Visibility sync
  useEffect(() => {
    if (!isTauri || !createdRef.current) return;
    void bridge.setBrowserWebviewVisible(pane.id, visible);
  }, [pane.id, visible, isTauri]);

  // Effect 4: URL change events from native webview
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
        void bridge.navigateBrowser(pane.id, normalized);
      }
    },
    [pane.id, isTauri],
  );

  return { containerRef, currentUrl, navigate };
}
