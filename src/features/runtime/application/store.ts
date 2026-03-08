import { create } from "zustand";
import type {
  BrowserLocationObservedEvent,
  PaneRuntimeView,
} from "@/features/workspace/domain";
import type { RuntimeClient } from "@/app-shell/clients";

export interface RuntimeState {
  runtimes: Record<string, PaneRuntimeView>;
  initializeListeners: () => Promise<void>;
  loadBootstrap: (runtimes: PaneRuntimeView[]) => void;
}

function toRuntimeMap(runtimes: PaneRuntimeView[]): Record<string, PaneRuntimeView> {
  return Object.fromEntries(runtimes.map((runtime) => [runtime.paneId, runtime]));
}

export function createRuntimeStore(runtimeClient: RuntimeClient) {
  let runtimeListenersReady: Promise<void> | null = null;

  return create<RuntimeState>((set, get) => ({
    runtimes: {},

    async initializeListeners() {
      if (runtimeListenersReady) {
        await runtimeListenersReady;
        return;
      }

      runtimeListenersReady = Promise.all([
        runtimeClient.listenStatusChanged((runtime) => {
          set((state) => ({
            runtimes: {
              ...state.runtimes,
              [runtime.paneId]: runtime,
            },
          }));
        }),
        runtimeClient.listenBrowserLocationObserved(
          (event: BrowserLocationObservedEvent) => {
            set((state) => {
              const current = state.runtimes[event.paneId];
              if (!current) {
                return state;
              }

              return {
                runtimes: {
                  ...state.runtimes,
                  [event.paneId]: {
                    ...current,
                    browserLocation: event.url,
                  },
                },
              };
            });
          },
        ),
      ]).then(() => undefined);

      await runtimeListenersReady;
    },

    loadBootstrap(runtimes) {
      set({ runtimes: toRuntimeMap(runtimes) });
      void get().initializeListeners();
    },
  }));
}
