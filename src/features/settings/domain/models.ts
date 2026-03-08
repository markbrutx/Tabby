/**
 * Internal settings domain models.
 *
 * These types are independent of transport/generated bindings and use
 * camelCase field names throughout. Mappers in the application layer
 * convert between these models and the wire-format DTOs.
 */

export type ThemeMode = "system" | "dawn" | "midnight";

export type LayoutPreset = "1x1" | "1x2" | "2x2" | "2x3" | "3x3";

export interface ProfileReadModel {
  readonly id: string;
  readonly label: string;
  readonly description: string;
  readonly startupCommandTemplate: string | null;
}

export interface ProfileCatalogReadModel {
  readonly terminalProfiles: readonly ProfileReadModel[];
}

export interface SettingsReadModel {
  readonly defaultLayout: LayoutPreset;
  readonly defaultTerminalProfileId: string;
  readonly defaultWorkingDirectory: string;
  readonly defaultCustomCommand: string;
  readonly fontSize: number;
  readonly theme: ThemeMode;
  readonly launchFullscreen: boolean;
  readonly hasCompletedOnboarding: boolean;
  readonly lastWorkingDirectory: string | null;
}
