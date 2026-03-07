import React from "react";
import ReactDOM from "react-dom/client";
import "xterm/css/xterm.css";
import "./styles.css";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { RecoveryScreen } from "@/components/RecoveryScreen";
import { TransportContext } from "@/lib/bridge/TransportContext";
import { bridge } from "@/lib/bridge";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary
      fallback={(error, reset) => (
        <RecoveryScreen
          title="Something went wrong"
          message={error.message || "An unexpected error occurred. Click retry to recover."}
          onRetry={reset}
        />
      )}
    >
      <TransportContext.Provider value={bridge}>
        <App />
      </TransportContext.Provider>
    </ErrorBoundary>
  </React.StrictMode>,
);
