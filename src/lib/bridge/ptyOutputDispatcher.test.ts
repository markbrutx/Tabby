import { describe, expect, it, vi, beforeEach } from "vitest";
import {
  registerPtyOutput,
  resetDispatcher,
  initDispatcher,
  teardownDispatcher,
} from "./ptyOutputDispatcher";
import type { PtyOutputEvent } from "@/features/workspace/domain";
import type { WorkspaceTransport, UnlistenFn } from "./shared";

let captured: ((event: PtyOutputEvent) => void) | null = null;

function makeMockTransport(): WorkspaceTransport {
  return {
    listenToPtyOutput: vi.fn(async (handler) => {
      captured = handler;
      return (() => { captured = null; }) as UnlistenFn;
    }),
  } as unknown as WorkspaceTransport;
}

function emit(paneId: string, sessionId: string, chunk: string) {
  captured?.({ paneId, sessionId, chunk });
}

beforeEach(() => {
  resetDispatcher();
  captured = null;
});

describe("ptyOutputDispatcher", () => {
  it("dispatches to registered callback", async () => {
    const transport = makeMockTransport();
    await initDispatcher(transport);

    const handler = vi.fn();
    registerPtyOutput("pane-a", "session-1", handler);

    emit("pane-a", "session-1", "hello");

    expect(handler).toHaveBeenCalledWith("hello");
  });

  it("does not dispatch to other pane", async () => {
    const transport = makeMockTransport();
    await initDispatcher(transport);

    const handler = vi.fn();
    registerPtyOutput("pane-a", "session-1", handler);

    emit("pane-b", "session-1", "data");

    expect(handler).not.toHaveBeenCalled();
  });

  it("unregister stops dispatch", async () => {
    const transport = makeMockTransport();
    await initDispatcher(transport);

    const handler = vi.fn();
    const unregister = registerPtyOutput("pane-a", "session-1", handler);

    unregister();
    emit("pane-a", "session-1", "data");

    expect(handler).not.toHaveBeenCalled();
  });

  it("session mismatch prevents dispatch", async () => {
    const transport = makeMockTransport();
    await initDispatcher(transport);

    const handler = vi.fn();
    registerPtyOutput("pane-a", "session-1", handler);

    emit("pane-a", "session-2", "data");

    expect(handler).not.toHaveBeenCalled();
  });

  it("multiple handlers for same pane all receive data", async () => {
    const transport = makeMockTransport();
    await initDispatcher(transport);

    const h1 = vi.fn();
    const h2 = vi.fn();
    registerPtyOutput("pane-a", "session-1", h1);
    registerPtyOutput("pane-a", "session-1", h2);

    emit("pane-a", "session-1", "chunk");

    expect(h1).toHaveBeenCalledWith("chunk");
    expect(h2).toHaveBeenCalledWith("chunk");
  });

  it("pending guard prevents duplicate listeners", async () => {
    let resolveFirst!: (fn: UnlistenFn) => void;
    let resolveSecond!: (fn: UnlistenFn) => void;
    const unlisten1 = vi.fn();
    const unlisten2 = vi.fn();
    let callCount = 0;

    const transport = {
      listenToPtyOutput: vi.fn(() => {
        callCount += 1;
        if (callCount === 1) {
          return new Promise<UnlistenFn>((r) => { resolveFirst = r; });
        }
        return new Promise<UnlistenFn>((r) => { resolveSecond = r; });
      }),
    } as unknown as WorkspaceTransport;

    // Start two concurrent inits (simulates StrictMode mount-unmount-mount)
    const p1 = initDispatcher(transport);
    const p2 = initDispatcher(transport);

    // Only one listenToPtyOutput call should have been made
    expect(transport.listenToPtyOutput).toHaveBeenCalledTimes(1);

    resolveFirst(unlisten1);
    await p1;
    await p2;

    // unlisten1 should be the active listener
    expect(unlisten1).not.toHaveBeenCalled();
  });

  it("cleans up listener if teardown ran during await", async () => {
    let resolveListener!: (fn: UnlistenFn) => void;
    const unlisten = vi.fn();

    const transport = {
      listenToPtyOutput: vi.fn(() => {
        return new Promise<UnlistenFn>((r) => { resolveListener = r; });
      }),
    } as unknown as WorkspaceTransport;

    const p = initDispatcher(transport);

    // Teardown runs while listenToPtyOutput is still pending
    teardownDispatcher();

    // Now the listener resolves
    resolveListener(unlisten);
    await p;

    // The returned listener should be immediately cleaned up
    expect(unlisten).toHaveBeenCalled();
  });
});
