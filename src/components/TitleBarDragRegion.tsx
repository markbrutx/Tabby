import { useCallback } from "react";
import { isTauriRuntime } from "@/lib/runtime";

export function TitleBarDragRegion() {
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;
    if (!isTauriRuntime()) return;
    void import("@tauri-apps/api/window").then(({ getCurrentWindow }) => {
      void getCurrentWindow().startDragging();
    });
  }, []);

  return (
    <div
      className="h-11 w-full shrink-0 select-none border-b border-[var(--color-border)] bg-[var(--color-bg)]"
      data-tauri-drag-region
      onMouseDown={handleMouseDown}
    />
  );
}
