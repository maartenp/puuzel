#!/usr/bin/env bash
set -euo pipefail

# Read current version from Cargo.toml
current=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo "Current version: $current"

read -rp "Increment [M]ajor / [m]inor / [p]atch? " choice

# Parse version into components
IFS='.' read -r MAJOR MINOR PATCH <<< "$current"

case "$choice" in
  M|major)
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
    ;;
  m|minor)
    MINOR=$((MINOR + 1))
    PATCH=0
    ;;
  p|patch)
    PATCH=$((PATCH + 1))
    ;;
  *)
    echo "Invalid choice: $choice"
    exit 1
    ;;
esac

new_version="$MAJOR.$MINOR.$PATCH"
echo "New version: $new_version"

read -rp "Confirm? [y/N] " confirm
if [[ "$confirm" != "y" ]]; then
  echo "Aborted."
  exit 0
fi

# Update version in Cargo.toml (portable sed for macOS and Linux)
if [[ "$(uname)" == "Darwin" ]]; then
  sed -i '' "s/^version = \"$current\"/version = \"$new_version\"/" Cargo.toml
else
  sed -i "s/^version = \"$current\"/version = \"$new_version\"/" Cargo.toml
fi

# Update Cargo.lock by running cargo check
cargo check

# Stage Cargo.toml and Cargo.lock
git add Cargo.toml Cargo.lock

git commit -m "chore: bump version to $new_version"
git tag "v$new_version"
git push && git push --tags

echo "Released v$new_version"

# Verify DIST-04: no git deps
git_deps=$(grep -c 'source = "git+' Cargo.lock || true)
if [[ "$git_deps" -gt 0 ]]; then
  echo "WARNING: Cargo.lock contains $git_deps git-sourced dependencies. DIST-04 requires crates.io only."
fi
