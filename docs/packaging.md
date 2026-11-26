# Packaging HyperShot

Build installers for optional system installs. Day-to-day development still uses `cargo run` from a checkout.

| Format | Tool | Host |
| --- | --- | --- |
| `.deb` | `cargo-packager` or `cargo-deb` | Linux |
| `.rpm` | `cargo-generate-rpm` | Linux |
| `.AppImage` | `cargo-packager` | Linux |
| `.dmg` (+ `.app`) | `cargo-packager` | **macOS** / CI `macos-latest` |
| `.msi` | `cargo-packager` (WiX) | **Windows** / CI `windows-latest` |

Linux hosts build deb / rpm / AppImage locally. DMG and MSI need macOS and Windows (or the Package workflow artifacts).

Every staged file under `target/dist/` includes the crate **version** in its filename and ships with `SHA256SUMS` plus a `.sha256` sidecar.

Config lives in:

- `Cargo.toml` → `[package.metadata.packager]` (deb / AppImage / dmg / wix)
- `Cargo.toml` → `[package.metadata.deb]` and `[package.metadata.generate-rpm]`
- `assets/packaging/` — icons + desktop entry
- `scripts/package.sh` / `scripts/stage-and-hash.sh`

## One-time tool installs

```bash
cargo install cargo-packager --locked
cargo install cargo-deb --locked
cargo install cargo-generate-rpm --locked
```

Linux AppImage builds also need FUSE / `libfuse2` on the builder (CI installs these). Windows MSI needs WiX (cargo-packager fetches tooling on the Windows runner).

## Build each format (local)

From the repo root:

```bash
./scripts/package.sh all-linux   # deb + rpm + AppImage → target/dist/ + SHA256SUMS
./scripts/package.sh deb
./scripts/package.sh rpm
./scripts/package.sh appimage
./scripts/package.sh dmg         # macOS only
./scripts/package.sh msi         # Windows only
./scripts/package.sh stage       # re-stage + re-hash existing outputs
```

Equivalent manual steps:

### Deb (Linux)

```bash
cargo build --release --locked
cargo packager --release --formats deb
./scripts/stage-and-hash.sh
```

### RPM (Linux)

```bash
cargo build --release --locked
cargo generate-rpm
./scripts/stage-and-hash.sh
```

### AppImage (Linux)

```bash
cargo build --release --locked
cargo packager --release --formats appimage
./scripts/stage-and-hash.sh
```

### DMG (macOS only)

```bash
cargo build --release --locked
cargo packager --release --formats app,dmg
./scripts/stage-and-hash.sh
```

### MSI (Windows only)

```bash
cargo build --release --locked
cargo packager --release --formats wix
./scripts/stage-and-hash.sh
```

Staged paths look like:

- `target/dist/hypershot_<version>_amd64.deb`
- `target/dist/hypershot-<version>-1.x86_64.rpm`
- `target/dist/hypershot_<version>_x86_64.AppImage`
- `target/dist/HyperShot_<version>_*.dmg` (CI / macOS)
- `target/dist/HyperShot_<version>_*.msi` (CI / Windows)
- `target/dist/SHA256SUMS`
- `target/dist/<file>.sha256`

## Verify checksums

```bash
cd target/dist
sha256sum -c SHA256SUMS          # Linux
# shasum -a 256 -c SHA256SUMS    # macOS
```

Or check one file:

```bash
sha256sum -c hypershot_1.23.0_amd64.deb.sha256
```

## Get DMG / MSI from CI

This Linux machine cannot produce DMG or MSI. Use GitHub Actions:

1. Open **Actions → Package** (`.github/workflows/package.yml`).
2. Run on a PR push, `main`, a `v*` tag, or **Run workflow**.
3. Download artifacts:
   - `hypershot-macos-dmg` — versioned `.dmg` + `SHA256SUMS`
   - `hypershot-windows-msi` — versioned `.msi` + `SHA256SUMS`
   - `hypershot-linux-packages` — `.deb` / `.rpm` / `.AppImage` + checksums

From the CLI (after a workflow run finishes):

```bash
gh run list --workflow=package.yml --branch <branch> --limit 5
gh run download <run-id> -n hypershot-macos-dmg
gh run download <run-id> -n hypershot-windows-msi
```

## CI summary

`.github/workflows/package.yml` jobs:

1. **linux** — deb + rpm + AppImage → stage + hash → upload
2. **macos** — app + dmg → stage + hash → upload
3. **windows** — WiX MSI → stage + hash → upload

## Install tips

- Deb: `sudo apt install ./hypershot_*.deb`
- RPM: `sudo dnf install ./hypershot-*.rpm`
- AppImage: `chmod +x hypershot_*.AppImage && ./hypershot_*.AppImage capture`
- DMG: open the disk image and drag HyperShot into Applications
- MSI: run the installer on Windows 10/11

Runtime deps (portal / `grim` / clipboard tools) are still needed on Linux; see the main README.
