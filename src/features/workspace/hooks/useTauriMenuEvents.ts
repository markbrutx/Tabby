import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { isTauriRuntime } from "@/lib/runtime";

export function useTauriMenuEvents(
  onOpenSettings: () => void,
  onCloseRequested: () => void,
) {
  useEffect(() => {
    if (!isTauriRuntime()) return;

    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listen("menu-open-settings", () => {
      onOpenSettings();
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
  }, [onOpenSettings]);

  useEffect(() => {
    if (!isTauriRuntime()) return;

    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listen("app-close-requested", () => {
      onCloseRequested();
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
  }, [onCloseRequested]);
}
