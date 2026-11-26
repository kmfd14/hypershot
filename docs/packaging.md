# Packaging HyperShot

Build installers for optional system installs. Day-to-day development still uses `cargo run` from a checkout.

| Format | Tool | Host |
| --- | --- | --- |
| `.deb` | `cargo-packager` or `cargo-deb` | Linux |
| `.rpm` | `cargo-generate-rpm` | Linux |
| `.AppImage` | `cargo-packager` | Linux |
| `.dmg` (+ `.app`) | `cargo-packager` | **macOS** (or GitHub Actions `macos-latest`) |

This Linux machine cannot produce a DMG. Use the [Package](../.github/workflows/package.yml) workflow on a macOS runner, or build on a Mac.

Config lives in:

- `Cargo.toml` → `[package.metadata.packager]` (deb / AppImage / dmg)
- `Cargo.toml` → `[package.metadata.deb]` and `[package.metadata.generate-rpm]`
- `assets/packaging/` — icons + desktop entry

## One-time tool installs

```bash
cargo install cargo-packager --locked
cargo install cargo-deb --locked
cargo install cargo-generate-rpm --locked
```

Linux AppImage builds also need FUSE / `libfuse2` on the builder (CI installs these).

## Build each format

From the repo root, after installing the tools above:

### 1. Deb (Linux)

```bash
cargo build --release
cargo packager --release --formats deb
# or: cargo deb
```

Artifacts: `target/packager/*.deb` (packager) or `target/debian/*.deb` (cargo-deb).

### 2. RPM (Linux)

```bash
cargo build --release
cargo generate-rpm
```

Artifact: `target/generate-rpm/*.rpm`.

### 3. AppImage (Linux)

```bash
cargo build --release
cargo packager --release --formats appimage
```

Artifact: `target/packager/*.AppImage`. Mark executable if needed: `chmod +x …AppImage`.

### 4. DMG (macOS only)

```bash
cargo build --release
cargo packager --release --formats app,dmg
```

Artifacts: `target/packager/*.app` and `target/packager/*.dmg`.

On Linux, trigger CI instead:

1. Push a tag `v*` or run **Actions → Package → Run workflow**.
2. Download the `hypershot-macos-dmg` artifact.

## Helper script

```bash
./scripts/package.sh deb        # Linux
./scripts/package.sh rpm        # Linux
./scripts/package.sh appimage   # Linux
./scripts/package.sh all-linux  # deb + rpm + AppImage
./scripts/package.sh dmg        # macOS only; errors on Linux
```

## CI

`.github/workflows/package.yml` runs on:

- `workflow_dispatch`
- push tags matching `v*`
- push to `main` (artifacts only; does not create a GitHub Release unless tagged)

Jobs:

1. **linux** — build release binary, then deb + rpm + AppImage; upload artifacts
2. **macos** — build release binary, then app + dmg; upload artifacts

Windows installers are out of scope for this packaging pass.

## Install tips

- Deb: `sudo apt install ./hypershot_*.deb`
- RPM: `sudo dnf install ./hypershot-*.rpm`
- AppImage: `chmod +x HyperShot*.AppImage && ./HyperShot*.AppImage capture`
- DMG: open the disk image and drag HyperShot into Applications

Runtime deps (portal / `grim` / clipboard tools) are still needed on Linux; see the main README.
