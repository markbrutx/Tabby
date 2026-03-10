import { describe, expect, it } from "vitest";
import type { PaneRuntimeView } from "@/contracts/tauri-bindings";
import { mapRuntimeFromDto } from "./snapshot-mappers";

function makeDto(overrides?: Partial<PaneRuntimeView>): PaneRuntimeView {
  return {
    paneId: "pane-1",
    runtimeSessionId: "session-abc",
    kind: "terminal",
    status: "running",
    lastError: null,
    browserLocation: null,
    terminalCwd: "/Users/dev/project",
    gitRepoPath: null,
    ...overrides,
  };
}

describe("mapRuntimeFromDto", () => {
  it("maps a full PaneRuntimeView to RuntimeReadModel", () => {
    const dto = makeDto();
    const result = mapRuntimeFromDto(dto);

    expect(result).toEqual({
      paneId: "pane-1",
      runtimeSessionId: "session-abc",
      kind: "terminal",
      status: "running",
      lastError: null,
      browserLocation: null,
      terminalCwd: "/Users/dev/project",
      gitRepoPath: null,
    });
  });

  it("maps a browser runtime with browserLocation", () => {
    const dto = makeDto({
      paneId: "pane-2",
      runtimeSessionId: "session-xyz",
      kind: "browser",
      status: "running",
      browserLocation: "https://example.com",
      terminalCwd: null,
    });

    const result = mapRuntimeFromDto(dto);

    expect(result.kind).toBe("browser");
    expect(result.browserLocation).toBe("https://example.com");
    expect(result.terminalCwd).toBeNull();
  });

  it("handles null lastError", () => {
    const result = mapRuntimeFromDto(makeDto({ lastError: null }));
    expect(result.lastError).toBeNull();
  });

  it("preserves a non-null lastError", () => {
    const dto = makeDto({
      runtimeSessionId: null,
      status: "failed",
      lastError: "PTY spawn failed: command not found",
    });

    const result = mapRuntimeFromDto(dto);

    expect(result.lastError).toBe("PTY spawn failed: command not found");
    expect(result.status).toBe("failed");
  });

  it("handles null runtimeSessionId", () => {
    const dto = makeDto({ runtimeSessionId: null, status: "starting" });
    const result = mapRuntimeFromDto(dto);

    expect(result.runtimeSessionId).toBeNull();
    expect(result.status).toBe("starting");
  });

  it("handles missing browserLocation for terminal runtime", () => {
    const dto = makeDto({
      kind: "terminal",
      browserLocation: null,
      terminalCwd: "/home/user",
    });

    const result = mapRuntimeFromDto(dto);

    expect(result.browserLocation).toBeNull();
    expect(result.kind).toBe("terminal");
  });

  it("result contains only camelCase field names", () => {
    const result = mapRuntimeFromDto(makeDto({ kind: "browser" }));
    const keys = Object.keys(result);

    for (const key of keys) {
      expect(key).not.toContain("_");
    }
  });

  it("does not mutate the original DTO", () => {
    const dto = makeDto({ status: "exited", lastError: "exit code 1" });
    const original = JSON.stringify(dto);
    mapRuntimeFromDto(dto);

    expect(JSON.stringify(dto)).toBe(original);
  });

  // AC1: Exhaustive status × kind field combinations
  describe("all status × kind combinations", () => {
    const statuses = ["starting", "running", "exited", "failed"] as const;
    const kinds = ["terminal", "browser", "git"] as const;

    for (const status of statuses) {
      for (const kind of kinds) {
        it(`maps ${kind} runtime with status ${status}`, () => {
          const dto = makeDto({
            paneId: `pane-${kind}-${status}`,
            kind,
            status,
            browserLocation: kind === "browser" ? "https://example.com" : null,
            terminalCwd: kind === "terminal" ? "/home/user" : null,
            gitRepoPath: kind === "git" ? "/home/user/repo" : null,
            lastError: status === "failed" ? "spawn error" : null,
            runtimeSessionId: status === "starting" ? null : `session-${kind}-${status}`,
          });

          const result = mapRuntimeFromDto(dto);

          expect(result.paneId).toBe(`pane-${kind}-${status}`);
          expect(result.kind).toBe(kind);
          expect(result.status).toBe(status);
          expect(result.browserLocation).toBe(
            kind === "browser" ? "https://example.com" : null,
          );
          expect(result.terminalCwd).toBe(
            kind === "terminal" ? "/home/user" : null,
          );
          expect(result.gitRepoPath).toBe(
            kind === "git" ? "/home/user/repo" : null,
          );
          expect(result.lastError).toBe(
            status === "failed" ? "spawn error" : null,
          );
          expect(result.runtimeSessionId).toBe(
            status === "starting" ? null : `session-${kind}-${status}`,
          );
        });
      }
    }
  });

  describe("nullable field edge cases", () => {
    it("all nullable fields set to null simultaneously", () => {
      const dto = makeDto({
        runtimeSessionId: null,
        lastError: null,
        browserLocation: null,
        terminalCwd: null,
      });

      const result = mapRuntimeFromDto(dto);

      expect(result.runtimeSessionId).toBeNull();
      expect(result.lastError).toBeNull();
      expect(result.browserLocation).toBeNull();
      expect(result.terminalCwd).toBeNull();
    });

    it("all nullable fields set to non-null simultaneously", () => {
      const dto = makeDto({
        runtimeSessionId: "sess-full",
        lastError: "some error",
        browserLocation: "https://example.com",
        terminalCwd: "/tmp",
      });

      const result = mapRuntimeFromDto(dto);

      expect(result.runtimeSessionId).toBe("sess-full");
      expect(result.lastError).toBe("some error");
      expect(result.browserLocation).toBe("https://example.com");
      expect(result.terminalCwd).toBe("/tmp");
    });
  });
});
