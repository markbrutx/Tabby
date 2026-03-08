/**
 * Runtime snapshot mappers.
 *
 * Converts transport DTOs (wire format) to internal runtime read models.
 * This is part of the anti-corruption layer that keeps domain code independent
 * of the generated contract types.
 */

import type { PaneRuntimeView } from "@/contracts/tauri-bindings";
import type { RuntimeReadModel } from "@/features/runtime/domain/models";

export function mapRuntimeFromDto(dto: PaneRuntimeView): RuntimeReadModel {
  return {
    paneId: dto.paneId,
    runtimeSessionId: dto.runtimeSessionId,
    kind: dto.kind,
    status: dto.status,
    lastError: dto.lastError,
    browserLocation: dto.browserLocation,
    terminalCwd: dto.terminalCwd,
  };
}
