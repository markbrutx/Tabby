import { useState, useCallback, useEffect, useRef } from "react";
import { LINE_HEIGHT_PX, OVERSCAN_COUNT } from "./diffTypes";

export function useVirtualScroll(totalRows: number, containerRef: React.RefObject<HTMLDivElement | null>) {
  const [scrollTop, setScrollTop] = useState(0);
  const [containerHeight, setContainerHeight] = useState(0);

  const handleScroll = useCallback(() => {
    const el = containerRef.current;
    if (el) {
      setScrollTop(el.scrollTop);
    }
  }, [containerRef]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setContainerHeight(entry.contentRect.height);
      }
    });
    observer.observe(el);
    setContainerHeight(el.clientHeight);

    return () => observer.disconnect();
  }, [containerRef]);

  const totalHeight = totalRows * LINE_HEIGHT_PX;
  const startIdx = Math.max(0, Math.floor(scrollTop / LINE_HEIGHT_PX) - OVERSCAN_COUNT);
  const visibleCount = Math.ceil(containerHeight / LINE_HEIGHT_PX) + 2 * OVERSCAN_COUNT;
  const endIdx = Math.min(totalRows, startIdx + visibleCount);

  return { handleScroll, totalHeight, startIdx, endIdx };
}

export function useSyncScroll(
  leftRef: React.RefObject<HTMLDivElement | null>,
  rightRef: React.RefObject<HTMLDivElement | null>,
) {
  const scrollingRef = useRef<"left" | "right" | null>(null);

  const syncScroll = useCallback(
    (source: "left" | "right") => {
      if (scrollingRef.current !== null && scrollingRef.current !== source) return;

      scrollingRef.current = source;

      const sourceEl = source === "left" ? leftRef.current : rightRef.current;
      const targetEl = source === "left" ? rightRef.current : leftRef.current;

      if (sourceEl && targetEl) {
        targetEl.scrollTop = sourceEl.scrollTop;
      }

      requestAnimationFrame(() => {
        scrollingRef.current = null;
      });
    },
    [leftRef, rightRef],
  );

  const onLeftScroll = useCallback(() => syncScroll("left"), [syncScroll]);
  const onRightScroll = useCallback(() => syncScroll("right"), [syncScroll]);

  return { onLeftScroll, onRightScroll };
}
