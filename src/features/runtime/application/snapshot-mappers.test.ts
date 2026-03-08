import { describe, expect, it } from "vitest";
import type { PaneRuntimeView } from "@/contracts/tauri-bindings";
import { mapRuntimeFromDto } from "./snapshot-mappers";

describe("mapRuntimeFromDto", () => {
  it("maps a full PaneRuntimeView to RuntimeReadModel", () => {
    const dto: PaneRuntimeView = {
      paneId: "pane-1",
      runtimeSessionId: "session-abc",
      kind: "terminal",
      status: "running",
      lastError: null,
      browserLocation: null,
      terminalCwd: "/Users/dev/project",
    };

    const result = mapRuntimeFromDto(dto);

    expect(result).toEqual({
      paneId: "pane-1",
      runtimeSessionId: "session-abc",
      kind: "terminal",
      status: "running",
      lastError: null,
      browserLocation: null,
      terminalCwd: "/Users/dev/project",
    });
  });

  it("maps a browser runtime with browserLocation", () => {
    const dto: PaneRuntimeView = {
      paneId: "pane-2",
      runtimeSessionId: "session-xyz",
      kind: "browser",
      status: "running",
      lastError: null,
      browserLocation: "https://example.com",
      terminalCwd: null,
    };

    const result = mapRuntimeFromDto(dto);

    expect(result.kind).toBe("browser");
    expect(result.browserLocation).toBe("https://example.com");
    expect(result.terminalCwd).toBeNull();
  });

  it("handles null lastError", () => {
    const dto: PaneRuntimeView = {
      paneId: "pane-3",
      runtimeSessionId: "session-123",
      kind: "terminal",
      status: "running",
      lastError: null,
      browserLocation: null,
      terminalCwd: null,
    };

    const result = mapRuntimeFromDto(dto);

    expect(result.lastError).toBeNull();
  });

  it("preserves a non-null lastError", () => {
    const dto: PaneRuntimeView = {
      paneId: "pane-4",
      runtimeSessionId: null,
      kind: "terminal",
      status: "failed",
      lastError: "PTY spawn failed: command not found",
      browserLocation: null,
      terminalCwd: null,
    };

    const result = mapRuntimeFromDto(dto);

    expect(result.lastError).toBe("PTY spawn failed: command not found");
    expect(result.status).toBe("failed");
  });

  it("handles null runtimeSessionId", () => {
    const dto: PaneRuntimeView = {
      paneId: "pane-5",
      runtimeSessionId: null,
      kind: "terminal",
      status: "starting",
      lastError: null,
      browserLocation: null,
      terminalCwd: null,
    };

    const result = mapRuntimeFromDto(dto);

    expect(result.runtimeSessionId).toBeNull();
    expect(result.status).toBe("starting");
  });

  it("handles missing browserLocation for terminal runtime", () => {
    const dto: PaneRuntimeView = {
      paneId: "pane-6",
      runtimeSessionId: "session-term",
      kind: "terminal",
      status: "running",
      lastError: null,
      browserLocation: null,
      terminalCwd: "/home/user",
    };

    const result = mapRuntimeFromDto(dto);

    expect(result.browserLocation).toBeNull();
    expect(result.kind).toBe("terminal");
  });

  it("result contains only camelCase field names", () => {
    const dto: PaneRuntimeView = {
      paneId: "pane-7",
      runtimeSessionId: "session-cc",
      kind: "browser",
      status: "running",
      lastError: null,
      browserLocation: "https://test.com",
      terminalCwd: null,
    };

    const result = mapRuntimeFromDto(dto);
    const keys = Object.keys(result);

    for (const key of keys) {
      expect(key).not.toContain("_");
    }
  });

  it("does not mutate the original DTO", () => {
    const dto: PaneRuntimeView = {
      paneId: "pane-8",
      runtimeSessionId: "session-imm",
      kind: "terminal",
      status: "exited",
      lastError: "exit code 1",
      browserLocation: null,
      terminalCwd: "/tmp",
    };

    const original = JSON.stringify(dto);
    mapRuntimeFromDto(dto);

    expect(JSON.stringify(dto)).toBe(original);
  });
});
