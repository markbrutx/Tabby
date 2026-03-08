#!/usr/bin/env bash
# check-dto-boundary.sh — Enforce that snake_case DTO field names exist only
# at the transport boundary (clients, application-layer mappers/stores).
#
# Allowed zones (snake_case DTO fields are expected here):
#   - src/contracts/                          (auto-generated bindings)
#   - src/app-shell/clients/                  (transport clients)
#   - src/features/*/application/             (stores, mappers, and their tests)
#
# All other .ts/.tsx files must use camelCase domain models exclusively.
# Test files (*.test.ts, *.test.tsx) are excluded everywhere because they
# construct DTO fixtures for store/mapper/hook assertions.
#
# Usage:  ./scripts/check-dto-boundary.sh
# Exit 0 = clean, Exit 1 = violations found

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC_DIR="$REPO_ROOT/src"

# Snake_case DTO field names to check (from tauri-bindings.ts contracts)
SNAKE_FIELDS=(
  'pane_id'
  'tab_id'
  'runtime_session_id'
  'command_override'
  'font_size'
  'launch_profile_id'
  'working_directory'
  'auto_layout'
  'pane_specs'
  'pane_spec'
  'initial_url'
  'pane_id_a'
  'pane_id_b'
  'profile_id'
  'active_tab_id'
  'pane_slot'
  'runtime_status'
)

# Build a single regex alternation: \b(pane_id|tab_id|...)\b
PATTERN="\b($(IFS='|'; echo "${SNAKE_FIELDS[*]}"))\b"

VIOLATIONS=""

while IFS= read -r file; do
  # Relative path for readability
  rel="${file#"$SRC_DIR/"}"

  # Skip allowed zones
  case "$rel" in
    contracts/*) continue ;;
    app-shell/clients/*) continue ;;
  esac

  # Allow anything inside features/*/application/ (stores, mappers, tests)
  if [[ "$rel" =~ ^features/[^/]+/application/ ]]; then
    continue
  fi

  # Skip test files — they construct DTO fixtures for assertions
  case "$rel" in
    *.test.ts|*.test.tsx) continue ;;
  esac

  # Check for snake_case DTO field access
  if grep -nE "$PATTERN" "$file" > /dev/null 2>&1; then
    matches=$(grep -nE "$PATTERN" "$file")
    VIOLATIONS+="
--- $rel ---
$matches
"
  fi
done < <(find "$SRC_DIR" -type f \( -name '*.ts' -o -name '*.tsx' \) ! -name '*.d.ts')

if [ -n "$VIOLATIONS" ]; then
  echo "❌ DTO boundary violation: snake_case field names found outside allowed zones."
  echo ""
  echo "Allowed zones:"
  echo "  - src/contracts/              (auto-generated bindings)"
  echo "  - src/app-shell/clients/      (transport clients)"
  echo "  - src/features/*/application/ (stores, mappers, tests)"
  echo ""
  echo "Violations:$VIOLATIONS"
  echo ""
  echo "Fix: move DTO access into the application layer or use camelCase domain models."
  exit 1
fi

echo "✅ DTO boundary check passed — no snake_case leaks outside transport boundary."
exit 0
