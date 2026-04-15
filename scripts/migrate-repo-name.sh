#!/bin/bash
# scripts/migrate-repo-name.sh
set -e
OLD_NAME="AIOE"
NEW_NAME="HMIR"
OLD_REPO="bhattkunalb/AIOE"
NEW_REPO="bhattkunalb/HMIR"

echo "Applying repository migration: ${OLD_NAME} -> ${NEW_NAME}..."

# Update Rust code, configs, docs (exclude vendored deps)
find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.yml" -o -name "*.md" -o -name "*.sh" \) \
  -not -path "./target/*" -not -path "./.git/*" \
  -exec sed -i "s|${OLD_REPO}|${NEW_REPO}|g" {} +

if [ $? -eq 0 ]; then
    echo "Replaced ${OLD_REPO} with ${NEW_REPO} in text files."
fi

find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.yml" -o -name "*.md" -o -name "*.sh" \) \
  -not -path "./target/*" -not -path "./.git/*" \
  -exec sed -i "s|${OLD_NAME}|${NEW_NAME}|g" {} +

if [ $? -eq 0 ]; then
    echo "Replaced ${OLD_NAME} with ${NEW_NAME} in text files."
fi

echo "✅ Migration complete. Review changes with: git diff"
