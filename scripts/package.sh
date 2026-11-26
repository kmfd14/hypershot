#!/usr/bin/env bash
# Build HyperShot installers. Usage: ./scripts/package.sh <deb|rpm|appimage|dmg|all-linux>
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
  cargo build --release
}

pkg_deb() {
  need_cmd cargo-packager cargo-packager
  build_release
  # Config: [package.metadata.packager] in Cargo.toml
  cargo packager --release --formats deb
  echo "Deb artifacts under target/packager/"
}

pkg_rpm() {
  need_cmd cargo-generate-rpm cargo-generate-rpm
  build_release
  cargo generate-rpm
  echo "RPM artifacts under target/generate-rpm/"
}

pkg_appimage() {
  need_cmd cargo-packager cargo-packager
  build_release
  cargo packager --release --formats appimage
  echo "AppImage artifacts under target/packager/"
}

pkg_dmg() {
  if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "DMG requires macOS (or GitHub Actions macos-latest)." >&2
    echo "See docs/packaging.md and .github/workflows/package.yml." >&2
    exit 1
  fi
  need_cmd cargo-packager cargo-packager
  build_release
  cargo packager --release --formats app,dmg
  echo "DMG / .app artifacts under target/packager/"
}

usage() {
  echo "Usage: $0 <deb|rpm|appimage|dmg|all-linux>" >&2
  exit 1
}

case "${1:-}" in
  deb) pkg_deb ;;
  rpm) pkg_rpm ;;
  appimage) pkg_appimage ;;
  dmg) pkg_dmg ;;
  all-linux)
    pkg_deb
    pkg_rpm
    pkg_appimage
    ;;
  *) usage ;;
esac
