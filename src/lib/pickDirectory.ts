import { open } from "@tauri-apps/plugin-dialog";

export async function pickDirectory(defaultPath?: string): Promise<string | null> {
  try {
    const selection = await open({
      directory: true,
      multiple: false,
      defaultPath: defaultPath || undefined,
    });
    return typeof selection === "string" ? selection : null;
  } catch {
    return null;
  }
}
