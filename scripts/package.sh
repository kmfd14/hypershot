#!/usr/bin/env bash
# Build HyperShot installers into target/packager (and rpm dir), then stage + hash.
# Usage: ./scripts/package.sh <deb|rpm|appimage|dmg|msi|all-linux|stage>
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing tool: $1" >&2
    echo "Install with: cargo install $2 --locked" >&2
    exit 1
  fi
}

build_release() {
  cargo build --release --locked
}

stage_hash() {
  "$ROOT/scripts/stage-and-hash.sh"
}

pkg_deb() {
  need_cmd cargo-packager cargo-packager
  build_release
  cargo packager --release --formats deb
  stage_hash
  echo "Deb staged under target/dist/"
}

pkg_rpm() {
  need_cmd cargo-generate-rpm cargo-generate-rpm
  build_release
  cargo generate-rpm
  stage_hash
  echo "RPM staged under target/dist/"
}

pkg_appimage() {
  need_cmd cargo-packager cargo-packager
  build_release
  cargo packager --release --formats appimage
  stage_hash
  echo "AppImage staged under target/dist/"
}

pkg_dmg() {
  if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "DMG requires macOS (or GitHub Actions macos-latest)." >&2
    echo "See docs/packaging.md — Actions → Package → download hypershot-macos-dmg." >&2
    exit 1
  fi
  need_cmd cargo-packager cargo-packager
  build_release
  cargo packager --release --formats app,dmg
  stage_hash
  echo "DMG staged under target/dist/"
}

pkg_msi() {
  case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*|Windows_NT) ;;
    *)
      echo "MSI requires Windows (or GitHub Actions windows-latest)." >&2
      echo "See docs/packaging.md — Actions → Package → download hypershot-windows-msi." >&2
      exit 1
      ;;
  esac
  need_cmd cargo-packager cargo-packager
  build_release
  cargo packager --release --formats wix
  stage_hash
  echo "MSI staged under target/dist/"
}

usage() {
  echo "Usage: $0 <deb|rpm|appimage|dmg|msi|all-linux|stage>" >&2
  exit 1
}

case "${1:-}" in
  deb) pkg_deb ;;
  rpm) pkg_rpm ;;
  appimage) pkg_appimage ;;
  dmg) pkg_dmg ;;
  msi) pkg_msi ;;
  stage) stage_hash ;;
  all-linux)
    need_cmd cargo-packager cargo-packager
    need_cmd cargo-generate-rpm cargo-generate-rpm
    build_release
    cargo packager --release --formats deb,appimage
    cargo generate-rpm
    stage_hash
    echo "Linux packages staged under target/dist/"
    ;;
  *) usage ;;
esac
