import { create } from "zustand";
import { shellClients } from "@/app-shell/clients";
import type {
  BrowserLocationObservedEvent,
  PaneRuntimeView,
} from "@/features/workspace/domain";

interface RuntimeState {
  runtimes: Record<string, PaneRuntimeView>;
  initializeListeners: () => Promise<void>;
  loadBootstrap: (runtimes: PaneRuntimeView[]) => void;
}

let runtimeListenersReady: Promise<void> | null = null;

function toRuntimeMap(runtimes: PaneRuntimeView[]): Record<string, PaneRuntimeView> {
  return Object.fromEntries(runtimes.map((runtime) => [runtime.paneId, runtime]));
}

export const useRuntimeStore = create<RuntimeState>((set, get) => ({
  runtimes: {},

  async initializeListeners() {
    if (runtimeListenersReady) {
      await runtimeListenersReady;
      return;
    }

    runtimeListenersReady = Promise.all([
      shellClients.runtime.listenStatusChanged((runtime) => {
        set((state) => ({
          runtimes: {
            ...state.runtimes,
            [runtime.paneId]: runtime,
          },
        }));
      }),
      shellClients.runtime.listenBrowserLocationObserved(
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
