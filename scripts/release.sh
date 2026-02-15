#!/usr/bin/env bash
set -euo pipefail

FORCE="${2:-false}"  # Optional: ./release.sh v1.2.3 --force
NEW_VERSION="${1#v}"   # Entferne führendes 'v' falls vorhanden (v0.1.2 -> 0.1.2)

if [[ -z "$NEW_VERSION" ]]; then
  echo "Usage: $0 <new-version> [--force]"
  echo "  --force: Allow rebuild of same/existing version"
  exit 1
fi

# Aktuelle Version (roh und ohne v-Prefix)
OLD_VERSION_RAW=$(grep '^version = ' Cargo.toml | head -n1 | sed 's/version = "\(.*\)"/\1/')
OLD_VERSION=$(echo "$OLD_VERSION_RAW" | sed 's/^v//')

echo "Current: $OLD_VERSION → New: $NEW_VERSION"

# Skip-Check bei --force oder gleicher Version
if [[ "$FORCE" != "--force" && "$(printf '%s\n%s\n' "$OLD_VERSION" "$NEW_VERSION" | sort -V | head -n1)" != "$OLD_VERSION" ]]; then
  echo "❌ New version ($NEW_VERSION) <= current ($OLD_VERSION). Use --force to override."
  exit 0
fi

# Version ersetzen (mit oder ohne v-Prefix)
perl -i -pe "s/version = \"$OLD_VERSION_RAW\"/version = \"$NEW_VERSION\"/" Cargo.toml

git add Cargo.toml
git commit -m "Release v$NEW_VERSION" || echo "No changes to commit"
EXISTING_TAG=$(git tag -l "v$NEW_VERSION")
if [[ -n "$EXISTING_TAG" && "$FORCE" != "--force" ]]; then
  read -p "Tag v$NEW_VERSION exists. Overwrite? [y/N] " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
  fi
fi
git tag -f "v$NEW_VERSION"  # -f für Force-Overwrite
git push origin HEAD:main   # Dein Branch
git push origin --force --tags  # Tags force-push (sicher bei Rebuilds)
echo "✅ Released v$NEW_VERSION"
