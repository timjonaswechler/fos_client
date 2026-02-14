#!/usr/bin/env bash
set -euo pipefail

FORCE="${2:-false}"  # Optional: ./release.sh v1.2.3 --force
NEW_VERSION="$1"

if [[ -z "$NEW_VERSION" ]]; then
  echo "Usage: $0 <new-version> [--force]"
  echo "  --force: Allow rebuild of same/existing version"
  exit 1
fi

# Aktuelle Version
OLD_VERSION=$(grep '^version = ' Cargo.toml | head -n1 | sed 's/version = "\(.*\)"/\1/')

echo "Current: $OLD_VERSION → New: $NEW_VERSION"

# Skip-Check bei --force oder gleicher Version
if [[ "$FORCE" != "--force" && "$(printf '%s\n%s\n' "$OLD_VERSION" "$NEW_VERSION" | sort -V | head -n1)" != "$OLD_VERSION" ]]; then
  echo "❌ New version ($NEW_VERSION) <= current ($OLD_VERSION). Use --force to override."
  exit 0
fi

# Version ersetzen
perl -i -pe "s/version = \"$OLD_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml

git add Cargo.toml
git commit -m "Release v$NEW_VERSION" || echo "No changes to commit"
EXISTING_TAG=$(git tag -l "v$NEW_VERSION")
if [[ -n "$EXISTING_TAG" && "$FORCE" != "--force" ]]; then
  echo "Tag v$NEW_VERSION exists. Use --force to overwrite."
  exit 1
fi
git tag -f "v$NEW_VERSION"  # -f für Force-Overwrite
git push origin HEAD:main   # Dein Branch
git push origin --force --tags  # Tags force-push (sicher bei Rebuilds)
echo "✅ Released v$NEW_VERSION"
