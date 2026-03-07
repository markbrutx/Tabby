import type { PtyOutputEvent } from "@/features/workspace/domain";
import type { UnlistenFn, WorkspaceTransport } from "./shared";

type OutputHandler = (chunk: string) => void;

interface Registration {
  paneId: string;
  sessionId: string;
  handler: OutputHandler;
}

let registrations: Registration[] = [];
let globalUnlisten: UnlistenFn | null = null;
let subscriberCount = 0;
let pending = false;

function dispatch(event: PtyOutputEvent) {
  for (const reg of registrations) {
    if (reg.paneId === event.paneId && reg.sessionId === event.sessionId) {
      reg.handler(event.chunk);
    }
  }
}

export function registerPtyOutput(
  paneId: string,
  sessionId: string,
  handler: OutputHandler,
): UnlistenFn {
  const registration: Registration = { paneId, sessionId, handler };
  registrations = [...registrations, registration];

  return () => {
    registrations = registrations.filter((r) => r !== registration);
  };
}

export async function initDispatcher(transport: WorkspaceTransport): Promise<void> {
  subscriberCount += 1;
  if (globalUnlisten || pending) {
    return;
  }
  pending = true;
  const unlisten = await transport.listenToPtyOutput(dispatch);
  pending = false;

  // Teardown ran while we were awaiting — immediately clean up
  if (subscriberCount <= 0) {
    unlisten();
    return;
  }

  // Another init completed first (shouldn't happen with pending guard, but be safe)
  if (globalUnlisten) {
    unlisten();
    return;
  }

  globalUnlisten = unlisten;
}

export function teardownDispatcher(): void {
  subscriberCount -= 1;
  if (subscriberCount <= 0) {
    globalUnlisten?.();
    globalUnlisten = null;
    subscriberCount = 0;
  }
}

/** Reset all state — for testing only. */
export function resetDispatcher(): void {
  registrations = [];
  globalUnlisten?.();
  globalUnlisten = null;
  subscriberCount = 0;
  pending = false;
}
