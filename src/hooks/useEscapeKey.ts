import { useEffect } from "react";

export function useEscapeKey(onEscape: (() => void) | undefined) {
  useEffect(() => {
    if (!onEscape) return;

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        onEscape!();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onEscape]);
}
