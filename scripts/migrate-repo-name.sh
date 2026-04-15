#!/usr/bin/env bash
# scripts/migrate-repo-name.sh
# Safe migration script: AIOE → HMIR repo references
# Usage: ./scripts/migrate-repo-name.sh --dry-run  # Review changes first

set -euo pipefail

# Configuration
OLD_NAME="AIOE"
NEW_NAME="HMIR"
OLD_REPO="bhattkunalb/AIOE"
NEW_REPO="bhattkunalb/HMIR"
DRY_RUN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run)
      DRY_RUN=true
      echo "🔍 Dry-run mode: showing changes without applying"
      shift
      ;;
    -h|--help)
      echo "Usage: $0 [--dry-run]"
      echo "  --dry-run  Show changes without modifying files"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Directories to exclude
EXCLUDE_DIRS=("./target" "./.git" "./.github/actions" "./vendor" "./node_modules")

# Build find exclude arguments
EXCLUDE_ARGS=()
for dir in "${EXCLUDE_DIRS[@]}"; do
  EXCLUDE_ARGS+=(-not -path "${dir}/*")
done

# File patterns to update
FILE_PATTERNS=("*.rs" "*.toml" "*.yml" "*.yaml" "*.md" "*.sh" "*.ps1" "*.txt" "*.json")

echo "🔄 Migrating repo references: ${OLD_REPO} → ${NEW_REPO}"
echo "🔄 Migrating name references: ${OLD_NAME} → ${NEW_NAME}"

# Function to apply sed with dry-run support
apply_migration() {
  local pattern=$1
  local replacement=$2
  local description=$3

  echo "📝 ${description}..."
  
  if $DRY_RUN; then
    # Show what would change
    find . -type f \( \( -name "${FILE_PATTERNS[0]}" \) \) "${EXCLUDE_ARGS[@]}" \
      -exec grep -l "$pattern" {} + 2>/dev/null | head -20 | while read -r file; do
      echo "  Would update: $file"
      grep -n "$pattern" "$file" | head -3 | sed 's/^/    /'
    done
  else
    # Apply changes
    find . -type f \( \( -name "${FILE_PATTERNS[0]}" \) \) "${EXCLUDE_ARGS[@]}" \
      -exec sed -i "s|${pattern}|${replacement}|g" {} + 2>/dev/null || true
  fi
}

# Apply migrations
apply_migration "${OLD_REPO}" "${NEW_REPO}" "Updating GitHub repo URLs"
apply_migration "${OLD_NAME}" "${NEW_NAME}" "Updating project name references"

# Special handling for Cargo.toml repository field
if ! $DRY_RUN; then
  find . -name "Cargo.toml" -not -path "./target/*" -not -path "./.git/*" \
    -exec sed -i "s|repository = \"https://github\.com/${OLD_REPO}\"|repository = \"https://github.com/${NEW_REPO}\"|g" {} + 2>/dev/null || true
fi

echo ""
if $DRY_RUN; then
  echo "✅ Dry-run complete. Review changes above."
  echo "💡 To apply changes, run: $0"
else
  echo "✅ Migration complete!"
  echo "💡 Review changes with: git diff"
  echo "💡 Commit with: git add -A && git commit -m 'fix: migrate repo references from AIOE to HMIR'"
fi
