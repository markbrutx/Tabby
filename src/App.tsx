import { RecoveryScreen } from "@/components/RecoveryScreen";
import { TitleBarDragRegion } from "@/components/TitleBarDragRegion";
import { AppLayout } from "@/components/AppLayout";
import { bootstrapCoordinator } from "@/contexts/stores";
import { useAppOrchestration } from "@/features/workspace/hooks/useAppOrchestration";
import { useAppEffects } from "@/features/workspace/hooks/useAppEffects";
import { useAppCallbacks } from "@/features/workspace/hooks/useAppCallbacks";

function App() {
  const orchestration = useAppOrchestration();
  useAppEffects(orchestration);
  const callbacks = useAppCallbacks(orchestration);

  if (orchestration.isHydrating) {
    return (
      <div className="flex h-screen flex-col bg-[var(--color-bg)] text-[var(--color-text)]">
        <TitleBarDragRegion />
        <div className="flex flex-1 items-center justify-center">
          <p className="text-sm text-[var(--color-text-muted)]">Starting...</p>
        </div>
      </div>
    );
  }

  if (!orchestration.workspaceModel || !orchestration.settings) {
    return (
      <div className="flex h-screen flex-col bg-[var(--color-bg)]">
        <TitleBarDragRegion />
        <RecoveryScreen
          title="Workspace unavailable"
          message={orchestration.error ?? "Tabby could not bootstrap the workspace."}
          onRetry={() => void bootstrapCoordinator.initialize()}
        />
      </div>
    );
  }

  return <AppLayout orchestration={orchestration} callbacks={callbacks} />;
}

export default App;
