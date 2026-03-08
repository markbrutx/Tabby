import type { RuntimeClient, UnlistenFn } from "@/app-shell/clients";
import type { TerminalOutputEvent } from "@/contracts/tauri-bindings";

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

function dispatch(event: TerminalOutputEvent) {
  for (const registration of registrations) {
    if (
      registration.paneId === event.paneId &&
      registration.sessionId === event.runtimeSessionId
    ) {
      registration.handler(event.chunk);
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
    registrations = registrations.filter((candidate) => candidate !== registration);
  };
}

export async function initDispatcher(runtimeClient: RuntimeClient): Promise<void> {
  subscriberCount += 1;
  if (globalUnlisten || pending) {
    return;
  }

  pending = true;
  const unlisten = await runtimeClient.listenTerminalOutput(dispatch);
  pending = false;

  if (subscriberCount <= 0) {
    unlisten();
    return;
  }

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

export function resetDispatcher(): void {
  registrations = [];
  globalUnlisten?.();
  globalUnlisten = null;
  subscriberCount = 0;
  pending = false;
}
