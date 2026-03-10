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
}: PaneConfiguratorProps) {
  async function handlePickDirectory(currentDir: string) {
    const selected = await pickDirectory(currentDir || undefined);
    if (!selected || values.mode === "browser") return;
    onChange({ ...values, workingDirectory: selected });
  }

  if (values.mode === "browser") {
    return (
      <Input
        data-testid={`${testIdPrefix}-url`}
        value={values.url}
        onChange={(event) => onChange({ ...values, url: event.target.value })}
        placeholder={DEFAULT_BROWSER_URL}
        className="text-sm"
        autoFocus={autoFocus}
      />
    );
  }

  if (values.mode === "git") {
    return (
      <div className="flex gap-2">
        <Input
          data-testid={`${testIdPrefix}-dir`}
          value={values.workingDirectory}
          onChange={(event) => onChange({ ...values, workingDirectory: event.target.value })}
          placeholder="Working directory"
          className="text-sm"
          autoFocus={autoFocus}
        />
        <Button
          variant="secondary"
          size="sm"
          className="shrink-0"
          onClick={() => void handlePickDirectory(values.workingDirectory)}
        >
          <FolderOpen size={14} />
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <Select
        data-testid={`${testIdPrefix}-profile`}
        value={values.profileId}
        onChange={(event) => onChange({ ...values, profileId: event.target.value })}
        className="text-sm"
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
          className="text-sm"
          autoFocus={autoFocus}
        />
      ) : null}

      <div className="flex gap-2">
        <Input
          data-testid={`${testIdPrefix}-dir`}
          value={values.workingDirectory}
          onChange={(event) => onChange({ ...values, workingDirectory: event.target.value })}
          placeholder="Working directory"
          className="text-sm"
        />
        <Button
          variant="secondary"
          size="sm"
          className="shrink-0"
          onClick={() => void handlePickDirectory(values.workingDirectory)}
        >
          <FolderOpen size={14} />
        </Button>
      </div>
    </div>
  );
}
