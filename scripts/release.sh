#!/usr/bin/env bash
set -euo pipefail

NEW_VERSION="$1"

# Aktuelle Version aus Cargo.toml holen
OLD_VERSION=$(grep '^version = ' Cargo.toml | head -n1 | sed 's/version = "\(.*\)"/\1/')

# SemVer-Vergleich: wenn NEW <= OLD -> abbrechen
if [ "$(printf '%s\n%s\n' "$OLD_VERSION" "$NEW_VERSION" | sort -V | head -n1)" != "$OLD_VERSION" ]; then
  echo "Neue Version ($NEW_VERSION) ist nicht größer als aktuelle ($OLD_VERSION). Nichts zu tun."
  exit 0
fi

# Version in Cargo.toml ersetzen (Cross-Platform: macOS + Linux)
perl -i -pe "s/version = \"$OLD_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml

git add Cargo.toml
git commit -m "bump version to $NEW_VERSION"
git tag "v$NEW_VERSION"
git push
git push --tags
