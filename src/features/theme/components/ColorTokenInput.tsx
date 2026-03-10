import { useCallback, useEffect, useRef, useState } from "react";
import { RotateCcw } from "lucide-react";

interface ColorTokenInputProps {
  readonly label: string;
  readonly value: string;
  readonly baseValue: string;
  readonly onChange: (value: string) => void;
}

function normalizeHex(hex: string): string {
  const cleaned = hex.replace("#", "");
  if (cleaned.length === 3) {
    return `#${cleaned[0]}${cleaned[0]}${cleaned[1]}${cleaned[1]}${cleaned[2]}${cleaned[2]}`;
  }
  return `#${cleaned.slice(0, 6)}`;
}

function isHexColor(value: string): boolean {
  return /^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/.test(value);
}

const DEBOUNCE_MS = 60;

export function ColorTokenInput({
  label,
  value,
  baseValue,
  onChange,
}: ColorTokenInputProps) {
  const [textValue, setTextValue] = useState(value);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isModified = value !== baseValue;

  useEffect(() => {
    setTextValue(value);
  }, [value]);

  const debouncedOnChange = useCallback(
    (newValue: string) => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
      debounceRef.current = setTimeout(() => {
        onChange(newValue);
      }, DEBOUNCE_MS);
    },
    [onChange],
  );

  useEffect(() => {
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  function handleColorPickerChange(
    event: React.ChangeEvent<HTMLInputElement>,
  ) {
    const newColor = event.target.value;
    setTextValue(newColor);
    debouncedOnChange(newColor);
  }

  function handleTextChange(event: React.ChangeEvent<HTMLInputElement>) {
    const newText = event.target.value;
    setTextValue(newText);

    if (isHexColor(newText) || newText.startsWith("rgb") || newText.startsWith("hsl")) {
      debouncedOnChange(newText);
    }
  }

  function handleTextBlur() {
    if (textValue !== value && (isHexColor(textValue) || textValue.startsWith("rgb") || textValue.startsWith("hsl"))) {
      onChange(textValue);
    } else if (!isHexColor(textValue) && !textValue.startsWith("rgb") && !textValue.startsWith("hsl")) {
      setTextValue(value);
    }
  }

  function handleReset() {
    setTextValue(baseValue);
    onChange(baseValue);
  }

  const swatchColor = isHexColor(value) ? value : baseValue;
  const pickerValue = isHexColor(value) ? normalizeHex(value) : "#000000";

  return (
    <div className="flex items-center gap-2 py-1">
      <span className="w-32 shrink-0 text-xs text-[var(--color-text-soft)]">
        {label}
      </span>

      <div
        className="h-4 w-4 shrink-0 rounded-full border border-[var(--color-border-strong)]"
        style={{ backgroundColor: swatchColor }}
      />

      <label className="relative shrink-0 cursor-pointer">
        <input
          type="color"
          value={pickerValue}
          onChange={handleColorPickerChange}
          className="absolute inset-0 h-6 w-6 cursor-pointer opacity-0"
        />
        <div className="flex h-6 w-6 items-center justify-center rounded border border-[var(--color-border)] bg-[var(--color-surface-overlay)] text-[10px] text-[var(--color-text-muted)]">
          ...
        </div>
      </label>

      <input
        type="text"
        value={textValue}
        onChange={handleTextChange}
        onBlur={handleTextBlur}
        spellCheck={false}
        className="h-6 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-surface-overlay)] px-2 font-mono text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-accent-strong)]"
      />

      <button
        type="button"
        onClick={handleReset}
        disabled={!isModified}
        className="shrink-0 rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)] disabled:opacity-30 disabled:hover:bg-transparent"
        title="Reset to base value"
      >
        <RotateCcw size={12} />
      </button>
    </div>
  );
}
