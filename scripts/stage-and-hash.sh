#!/usr/bin/env bash
# Copy package outputs into target/dist (current crate version in each name),
# then write SHA256SUMS (+ per-file .sha256 sidecars).
#
# Usage: ./scripts/stage-and-hash.sh
# Optional env: DIST_DIR (default: target/dist)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)"
if [[ -z "$VERSION" ]]; then
  echo "Could not read version from Cargo.toml" >&2
  exit 1
fi

DIST_DIR="${DIST_DIR:-target/dist}"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

sha256_file() {
  local f="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$f" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$f" | awk '{print $1}'
  else
    echo "Need sha256sum or shasum" >&2
    exit 1
  fi
}

stage_one() {
  local src="$1"
  [[ -f "$src" ]] || return 0
  local base dest
  base="$(basename "$src")"
  case "$base" in
    SHA256SUMS|*.sha256) return 0 ;;
  esac
  if [[ "$base" != *"$VERSION"* ]]; then
    echo "skip (version $VERSION not in name): $base" >&2
    return 0
  fi
  dest="$DIST_DIR/$base"
  cp -f "$src" "$dest"
  echo "staged $base"
}

shopt -s nullglob
for f in \
  target/packager/*.deb \
  target/packager/*.AppImage \
  target/packager/*.dmg \
  target/packager/*.msi \
  target/packager/*.exe \
  target/generate-rpm/*.rpm \
  target/debian/*.deb
do
  stage_one "$f"
done
shopt -u nullglob

shopt -s nullglob
artifacts=( "$DIST_DIR"/* )
shopt -u nullglob

if [[ ${#artifacts[@]} -eq 0 ]]; then
  echo "No versioned artifacts found for $VERSION. Build packages first." >&2
  exit 1
fi

: > "$DIST_DIR/SHA256SUMS"
for f in "$DIST_DIR"/*; do
  base="$(basename "$f")"
  case "$base" in
    SHA256SUMS|*.sha256) continue ;;
  esac
  [[ -f "$f" ]] || continue
  digest="$(sha256_file "$f")"
  printf '%s  %s\n' "$digest" "$base" >> "$DIST_DIR/SHA256SUMS"
  printf '%s  %s\n' "$digest" "$base" > "$DIST_DIR/${base}.sha256"
done

echo "Wrote $DIST_DIR/SHA256SUMS (version $VERSION)"
echo "Verify: cd $DIST_DIR && sha256sum -c SHA256SUMS"
echo "   or: cd $DIST_DIR && shasum -a 256 -c SHA256SUMS"
