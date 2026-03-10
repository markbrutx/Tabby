import { DEFAULT_BROWSER_URL } from "@/features/workspace/domain/models";
import type { ProfileReadModel, SettingsReadModel } from "@/features/settings/domain/models";
import type { PaneGroupConfig } from "@/features/workspace/store/types";

export function resolveDefaultProfileId(
  settings: SettingsReadModel,
  profiles: readonly ProfileReadModel[],
): string {
  const configured = settings.defaultTerminalProfileId?.trim();
  if (configured && profiles.some((profile) => profile.id === configured)) {
    return configured;
  }

  return profiles.find((profile) => profile.id === "terminal")?.id
    ?? profiles[0]?.id
    ?? "terminal";
}

export function makeDefaultGroup(
  mode: PaneGroupConfig["mode"],
  settings: SettingsReadModel,
  profiles: readonly ProfileReadModel[],
): PaneGroupConfig {
  switch (mode) {
    case "terminal":
      return {
        mode: "terminal",
        profileId: resolveDefaultProfileId(settings, profiles),
        workingDirectory: settings.defaultWorkingDirectory || settings.lastWorkingDirectory || "",
        customCommand: settings.defaultCustomCommand ?? "",
        count: 1,
      };
    case "browser":
      return { mode: "browser", url: DEFAULT_BROWSER_URL, count: 1 };
    case "git":
      return { mode: "git", workingDirectory: settings.defaultWorkingDirectory ?? "", count: 1 };
  }
}
