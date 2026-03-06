export function isTauriRuntime(): boolean {
  if (typeof window === "undefined") {
    return false;
  }

  return Object.prototype.hasOwnProperty.call(
    window as unknown as Record<string, unknown>,
    "__TAURI_INTERNALS__",
  );
}
