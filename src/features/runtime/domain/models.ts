/**
 * Internal runtime domain models.
 *
 * These types are independent of transport/generated bindings and use
 * camelCase field names throughout. Mappers in the application layer
 * convert between these models and the wire-format DTOs.
 */

export type RuntimeKind = "terminal" | "browser";

export type RuntimeStatus = "starting" | "running" | "exited" | "failed";

export interface RuntimeReadModel {
  readonly paneId: string;
  readonly runtimeSessionId: string | null;
  readonly kind: RuntimeKind;
  readonly status: RuntimeStatus;
  readonly lastError: string | null;
  readonly browserLocation: string | null;
  readonly terminalCwd: string | null;
}

// ---------------------------------------------------------------------------
// Browser surface geometry
// ---------------------------------------------------------------------------

export interface BrowserBounds {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}
