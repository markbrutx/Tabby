import { FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { CUSTOM_PROFILE_ID, DEFAULT_BROWSER_URL } from "@/features/workspace/domain/models";
import type { ProfileReadModel } from "@/features/settings/domain/models";
import { pickDirectory } from "@/lib/pickDirectory";

export type PaneFieldValues =
  | { mode: "terminal"; profileId: string; workingDirectory: string; customCommand: string }
  | { mode: "browser"; url: string }
  | { mode: "git"; workingDirectory: string };

interface PaneConfiguratorProps {
  values: PaneFieldValues;
  profiles: readonly ProfileReadModel[];
  onChange: (values: PaneFieldValues) => void;
  autoFocus?: boolean;
  testIdPrefix?: string;
  layout?: "stacked" | "inline";
}

export function isFieldValuesValid(values: PaneFieldValues): boolean {
  switch (values.mode) {
    case "terminal":
      return !!values.profileId
        && (values.profileId !== CUSTOM_PROFILE_ID || !!values.customCommand.trim());
    case "browser":
      return !!values.url.trim();
    case "git":
      return true;
  }
}

export function PaneConfigurator({
  values,
  profiles,
  onChange,
  autoFocus = false,
  testIdPrefix = "pane",
  layout = "stacked",
}: PaneConfiguratorProps) {
  async function handlePickDirectory(currentDir: string) {
    const selected = await pickDirectory(currentDir || undefined);
    if (!selected || values.mode === "browser") return;
    onChange({ ...values, workingDirectory: selected });
  }

  const containerClass = layout === "inline" ? "flex flex-1 min-w-0 items-center gap-2" : "space-y-3";

  if (values.mode === "browser") {
    return (
      <div className={containerClass}>
        <Input
          data-testid={`${testIdPrefix}-url`}
          value={values.url}
          onChange={(event) => onChange({ ...values, url: event.target.value })}
          placeholder={DEFAULT_BROWSER_URL}
          className="w-full min-w-0 text-sm"
          autoFocus={autoFocus}
        />
      </div>
    );
  }

  if (values.mode === "git") {
    return (
      <div className={containerClass}>
        <div className="flex w-full flex-1 min-w-0 gap-1.5">
          <Input
            data-testid={`${testIdPrefix}-dir`}
            value={values.workingDirectory}
            onChange={(event) => onChange({ ...values, workingDirectory: event.target.value })}
            placeholder="Working directory"
            className="w-full min-w-0 text-sm"
            autoFocus={autoFocus}
          />
          <Button
            variant="secondary"
            size="sm"
            className="shrink-0 px-2"
            onClick={() => void handlePickDirectory(values.workingDirectory)}
          >
            <FolderOpen size={14} />
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className={containerClass}>
      <Select
        data-testid={`${testIdPrefix}-profile`}
        value={values.profileId}
        onChange={(event) => onChange({ ...values, profileId: event.target.value })}
        className={layout === "inline" ? "w-[130px] shrink-0 text-sm" : "w-full text-sm"}
      >
        {profiles.map((profile) => (
          <option key={profile.id} value={profile.id}>
            {profile.label}
          </option>
        ))}
      </Select>

      {values.profileId === CUSTOM_PROFILE_ID ? (
        <Input
          data-testid={`${testIdPrefix}-command`}
          value={values.customCommand}
          onChange={(event) => onChange({ ...values, customCommand: event.target.value })}
          placeholder="Custom command"
          className="w-full min-w-0 text-sm"
          autoFocus={autoFocus}
        />
      ) : null}

      {layout === "stacked" ? (
        <div className="flex w-full flex-1 min-w-0 gap-1.5">
          <Input
            data-testid={`${testIdPrefix}-dir`}
            value={values.workingDirectory}
            onChange={(event) => onChange({ ...values, workingDirectory: event.target.value })}
            placeholder="Working directory"
            className="w-full min-w-0 text-sm"
          />
          <Button
            variant="secondary"
            size="sm"
            className="shrink-0 px-2"
            onClick={() => void handlePickDirectory(values.workingDirectory)}
          >
            <FolderOpen size={14} />
          </Button>
        </div>
      ) : null}
    </div>
  );
}
