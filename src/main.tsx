import React from "react";
import ReactDOM from "react-dom/client";
import "xterm/css/xterm.css";
import "./styles.css";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { RecoveryScreen } from "@/components/RecoveryScreen";
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
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);
