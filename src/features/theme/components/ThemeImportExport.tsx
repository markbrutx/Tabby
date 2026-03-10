import { useCallback, useRef } from "react";
import { Download, Upload } from "lucide-react";
import { Button } from "@/components/ui/Button";
import type { ThemeState } from "../application/themeStore";
import type { ThemeDefinition } from "../domain/models";

export function exportThemeToFile(
  themeStore: ThemeState,
  themeId: string,
): void {
  const json = themeStore.exportTheme(themeId);
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `theme-${themeId}.json`;
  anchor.click();
  URL.revokeObjectURL(url);
}

interface ImportThemeButtonProps {
  readonly importTheme: (json: string) => ThemeDefinition;
  readonly onImported: (theme: ThemeDefinition) => void;
  readonly onError?: (error: string) => void;
}

export function ImportThemeButton({
  importTheme,
  onImported,
  onError,
}: ImportThemeButtonProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleClick = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleFileChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = (loadEvent) => {
        const text = loadEvent.target?.result;
        if (typeof text !== "string") return;

        try {
          const imported = importTheme(text);
          onImported(imported);
        } catch (err) {
          const message =
            err instanceof Error ? err.message : "Failed to import theme";
          onError?.(message);
        }
      };
      reader.readAsText(file);

      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    },
    [importTheme, onImported, onError],
  );

  return (
    <>
      <input
        ref={fileInputRef}
        type="file"
        accept=".json"
        onChange={handleFileChange}
        className="hidden"
      />
      <Button variant="secondary" size="sm" onClick={handleClick}>
        <Upload size={14} className="mr-1" />
        Import
      </Button>
    </>
  );
}

interface ExportThemeButtonProps {
  readonly themeId: string;
  readonly store: ThemeState;
}

export function ExportThemeButton({ themeId, store }: ExportThemeButtonProps) {
  const handleExport = useCallback(() => {
    try {
      exportThemeToFile(store, themeId);
    } catch {
      // Theme not found or export failed silently
    }
  }, [store, themeId]);

  return (
    <Button variant="secondary" size="sm" onClick={handleExport}>
      <Download size={14} className="mr-1" />
      Export
    </Button>
  );
}
