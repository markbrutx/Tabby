/**
 * Settings snapshot mappers.
 *
 * Converts transport DTOs (wire format) to internal settings read models.
 * This is part of the anti-corruption layer that keeps domain code independent
 * of the generated contract types.
 */

import type {
  ProfileCatalogView,
  ProfileView,
  SettingsView,
} from "@/contracts/tauri-bindings";
import type {
  ProfileCatalogReadModel,
  ProfileReadModel,
  SettingsReadModel,
} from "@/features/settings/domain/models";

export function mapProfileFromDto(dto: ProfileView): ProfileReadModel {
  return {
    id: dto.id,
    label: dto.label,
    description: dto.description,
    startupCommandTemplate: dto.startupCommandTemplate,
  };
}

export function mapProfileCatalogFromDto(
  dto: ProfileCatalogView,
): ProfileCatalogReadModel {
  return {
    terminalProfiles: dto.terminalProfiles.map(mapProfileFromDto),
  };
}

export function mapSettingsFromDto(dto: SettingsView): SettingsReadModel {
  return {
    defaultLayout: dto.defaultLayout,
    defaultTerminalProfileId: dto.defaultTerminalProfileId,
    defaultWorkingDirectory: dto.defaultWorkingDirectory,
    defaultCustomCommand: dto.defaultCustomCommand,
    fontSize: dto.fontSize,
    theme: dto.theme,
    launchFullscreen: dto.launchFullscreen,
    hasCompletedOnboarding: dto.hasCompletedOnboarding,
    lastWorkingDirectory: dto.lastWorkingDirectory,
  };
}
