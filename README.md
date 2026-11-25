# HyperShot

<p align="center">
  <img src="./assets/logo.svg" alt="HyperShot logo" width="96" height="96">
</p>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![CI](https://github.com/kmfd14/lightsnip/actions/workflows/ci.yml/badge.svg)](https://github.com/kmfd14/lightsnip/actions/workflows/ci.yml)

**Product name:** HyperShot · **Crate / binary / repo:** `lightsnip`

**Do this now** (from the repo checkout, graphical session, ~1 minute after deps):

```bash
cargo run --release -- region
```

1. Drag-select a region on the dimmed desktop — live size text like `1280 x 720` updates while you drag (Flameshot-style on-screen overlay by default).
2. Mark up in place with the bottom pill toolbar (arrow, box, pixelate, step number). Only the **active** tool is purple-highlighted. Use **Open in preview** for the floating window.
3. **Save** → pick a file in the dialog → close. **Enter** → same dialog + copy to clipboard + toast + close. **Copy** → clipboard toast + close (no dialog).
4. Cancel the save dialog (or **Exit**) to dismiss.

PNG default folder: `~/Pictures/lightsnip/` (overridable in Settings).

```bash
cargo run --release -- settings
```

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install, autostart, tray, and “replace default screenshot tool” are separate and **off by default**.

Screenshot tool with capture + annotation. Region or fullscreen → floating annotate window → save or copy. Built in Rust. Minimal pill annotate toolbar + Apple-inspired settings.

## Platform support (v0.7)

| Platform | Capture | Annotate / Save / Clipboard | Tray | Native ARM |
| --- | --- | --- | --- | --- |
| Linux Wayland | **Works** — full capture (portal / `grim`), then in-app region picker with live `W x H` | **Works** | **Works** (SNI / `ksni`) | `aarch64-unknown-linux-gnu` via CI `cargo check` |
| macOS | **Best-effort** — `screencapture` (needs Screen Recording permission) + in-app region picker | **Works** (`arboard` + `rfd`) | Not in v0.7 | **Yes** — `aarch64-apple-darwin` (CI on `macos-latest`) |
| Windows 10/11 | **WIP** — stub error; use Snipping Tool then `annotate` | **Works** (`arboard` + `rfd`) | Not in v0.7 | `aarch64-pc-windows-msvc` via CI `cargo check` |

> [!CAUTION]
> No upload, accounts, or video recording. Global shortcuts and “replace default screenshot tool” depend on your desktop (Linux XDG helpers only). Headless / cloud VMs usually **compile** but cannot open annotate / settings — use `--no-annotate` or `annotate` on a real display.
>
> Default annotate is an **on-screen overlay** at the capture (selection handles, resolution pill, bottom pill toolbar). Settings → Annotate mode can switch to the floating preview window. Wayland: fullscreen overlay is best-effort after a full capture; compositors may ignore always-on-top. Toolbar chrome: light gray/white pill, charcoal icons, purple active-only highlight.

### ARM / cross builds

```bash
# Apple Silicon (native on M-series)
cargo build --release --target aarch64-apple-darwin

# Windows ARM64 (from an MSVC toolchain host)
rustup target add aarch64-pc-windows-msvc
cargo build --release --target aarch64-pc-windows-msvc

# Linux ARM64
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

CI runs `cargo test` on Linux x86_64, macOS aarch64, and Windows x86_64, plus `cargo check` for the ARM targets above.

## Tech stack

<p align="left">
  <img src="https://cdn.simpleicons.org/rust/000000" width="32" height="32" alt="Rust">
  <img src="https://cdn.simpleicons.org/tokio/000000" width="32" height="32" alt="Tokio">
  <img src="https://cdn.simpleicons.org/toml/9C4121" width="32" height="32" alt="TOML">
  <img src="https://cdn.simpleicons.org/linux/FCC624" width="32" height="32" alt="Linux">
  <img src="https://cdn.simpleicons.org/wayland/ff0000" width="32" height="32" alt="Wayland">
  <img src="https://cdn.simpleicons.org/apple/000000" width="32" height="32" alt="Apple">
  <img src="https://cdn.simpleicons.org/windows/0078D4" width="32" height="32" alt="Windows">
</p>

* **Language / CLI:** Rust, `tokio`, `clap`, `directories`, `serde` / `toml`
* **Capture:** Linux `ashpd` + `grim`/`slurp`; macOS `screencapture`; Windows stub (WIP)
* **Annotate / settings / region UI:** egui + eframe; save dialog via `rfd`
* **Images:** `image` / `imageproc`
* **Clipboard:** `arboard` (Wayland still benefits from `wl-clipboard` on PATH)
* **Tray:** Linux `ksni` only

## Features

1. **Capture** — `cargo run --release -- region` / `full` (see [Platform support](#platform-support-v07)).
2. **On-screen annotate (default)** — dimmed desktop overlay with selection handles, `W x H` pill, bottom pill toolbar; tools draw on the region in place. Button opens the floating preview window.
3. **Region resolution** — live size text (`1280 x 720`) while selecting; hidden when not selecting.
4. **Preview annotate** — floating **HyperShot** window (`annotate path.png`, or Settings → Annotate mode → Preview). Pill icon rail: arrow, box, pixelate, step, color, undo/redo, clear, settings, save, copy, exit. **Only the active tool** is purple-highlighted.
5. **Save** — file dialog (defaults under configured save dir). Cancel keeps the window open.
6. **Enter** — save dialog + copy to clipboard + toast + close (always). Esc respects **Esc to exit**; Enter does not.
7. **Pixelate** — soft classic mosaic (`PIXELATE_BLOCK = 20`, averages the image behind) so fine text is hard to read but the region still looks like the capture; seamless fill (no block borders); undo/redo still work.
8. **Clipboard** — settings default + `--clipboard` / `--no-clipboard`.
9. **Settings** — cards with `?` tooltips, annotate mode, folder picker, UI + About (`0.7`).
10. **Tray** — Linux only: `cargo run --release -- tray`.

## Runtime dependencies

### Linux (Wayland)

Portal first for fullscreen capture; `grim` (+ `slurp` for older region fallback paths) when the portal fails. Install `wl-clipboard` for reliable clipboard. Region mode then opens HyperShot’s in-app picker with live `W x H`.

#### Fedora 44 COSMIC — install now

```bash
sudo dnf install grim slurp wl-clipboard \
  xdg-desktop-portal xdg-desktop-portal-cosmic
```

| Need | Binary / package | Role |
| --- | --- | --- |
| Capture fallback | `grim` | PNG when portal fails |
| Region fallback | `slurp` | Legacy geometry helper (region now uses in-app picker after full capture) |
| Clipboard | `wl-copy` (`wl-clipboard`) | Helps `arboard` on Wayland |

| Desktop | Fedora packages |
| --- | --- |
| COSMIC | `xdg-desktop-portal` + `xdg-desktop-portal-cosmic` |
| GNOME | `xdg-desktop-portal` + `xdg-desktop-portal-gnome` |
| KDE Plasma | `xdg-desktop-portal` + `xdg-desktop-portal-kde` |
| Sway / Hyprland | `xdg-desktop-portal` + `xdg-desktop-portal-wlr` |

> [!NOTE]
> Keep `grim` + `slurp` installed on COSMIC / tiling. Re-login after adding a portal backend if capture still fails.

### macOS

1. Grant **Screen Recording** to your terminal (or the `lightsnip` binary) in System Settings → Privacy & Security.
2. `cargo run --release -- region` captures the display, then opens HyperShot’s region picker with live size text.

### Windows 10/11

1. Capture is **WIP** in v0.7.
2. Do this instead: Win+Shift+S (or Snipping Tool) → save a PNG →

   ```bash
   cargo run --release -- annotate path\to\snip.png
   ```

## Install

1. Install platform deps above (Linux one-liner, or macOS Screen Recording).
2. Clone:

   ```bash
   git clone https://github.com/kmfd14/lightsnip.git
   cd lightsnip
   ```

3. Jump to [Quick start](#quick-start).

### Optional install

```bash
cargo build --release
cp target/release/lightsnip ~/.local/bin/   # or Windows equivalent on PATH
```

The binary stays named `lightsnip`; the product UI shows **HyperShot**.

## Quick start

1. Region → annotate → Save dialog:

   ```bash
   cargo run --release -- region
   ```

2. Fullscreen: `cargo run --release -- full`

3. Annotate an existing PNG: `cargo run --release -- annotate ~/Pictures/lightsnip/lightsnip-123.png`

4. Settings: `cargo run --release -- settings`

5. Flags:

   ```bash
   cargo run --release -- region --no-annotate
   cargo run --release -- region -o /tmp/snip.png
   cargo run --release -- region --no-clipboard
   cargo run --release -- full --save-dir ~/Pictures/Screenshots
   ```

## Hotkeys

**Assign… in Settings is not a live global hotkey.** Bind the **bind command** in your DE / compositor (Linux). macOS / Windows: bind your own shortcut to `lightsnip region` after optional install.

| Desktop | Where to bind | Sample |
| --- | --- | --- |
| Fedora COSMIC | Settings → Input Devices → Keyboard → Custom shortcuts | `cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
| Hyprland | `hyprland.conf` | `bind = SUPER, Print, exec, cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
| Sway | config | `bindsym $mod+Print exec cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
| GNOME / KDE | Custom shortcut | Same command (does not steal PrintScreen by itself) |

Annotate keys:

* **Enter** = save dialog + copy + toast + close
* **Save** = save dialog + close
* **Copy** = write + clipboard toast + close
* **Exit** = cancel; **Esc** = same when Esc-to-exit is on
* **Ctrl+Z** / **Ctrl+Shift+Z** = undo / redo

## Settings

`cargo run --release -- settings`, or **Settings** on the annotate bar.

Config: `~/.config/lightsnip/config.toml` (via `directories`; macOS/Windows use the platform equivalent).

1. **Shortcuts** — label only (not a global grab).
2. **Run at startup** / **Replace default** — Linux XDG `.desktop` helpers; off by default.
3. **Stay in tray** — Linux SNI tray; off by default.
4. **User Interface** — annotate mode (on-screen / preview), toolbar / theme / Esc-to-exit / transparency (toolbar stays opaque).
5. **About** — HyperShot `0.7`.
6. **Save directory** + **Save to clipboard** defaults.

## Documentation

| Guide | Description |
| --- | --- |
| [User guide](./docs/user-guide.md) | Capture → annotate → save / clipboard → settings → hotkeys |

## Contributing

1. Fork the repository.
2. Create a feature branch from `main`.
3. Open a pull request with a clear description of the change.
4. Run `cargo test` (and CI covers cross targets).

Issues: <https://github.com/kmfd14/lightsnip/issues>

## License

[MIT](./LICENSE)
