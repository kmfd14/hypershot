# HyperShot

<p align="center">
  <img src="./assets/logo.svg" alt="HyperShot logo" width="96" height="96">
</p>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![CI](https://github.com/kmfd14/hypershot/actions/workflows/ci.yml/badge.svg)](https://github.com/kmfd14/hypershot/actions/workflows/ci.yml)

**Product:** HyperShot · **Crate / binary:** `hypershot` · **v1.3.0**

**Do this now** (from the repo checkout, graphical session):

```bash
cargo run --release -- region
```

1. Drag-select a region — live size text like `1280 x 720` updates while you drag.
2. Mark up with the bottom flat toolbar. Only the **active** tool is purple. Use **Open in preview** for the floating window.
3. **Save** → pick a file → close. **Enter** → same dialog + copy + toast + close. **Copy** → clipboard toast + close.
4. Cancel the save dialog (or **Exit**) to dismiss.

PNG default folder: `~/Pictures/hypershot/` (change in Settings).

```bash
cargo run --release -- settings
```

No install required. Other screenshot apps stay untouched. Optional install, autostart, tray, and “replace default screenshot tool” are **off by default**.

Screenshot tool with capture + annotation. Region or fullscreen → annotate → save or copy. Built in Rust. Minimal flat annotate toolbar + clean settings.

## Platform support (v1.3)

| Platform | Capture | Annotate / Save / Clipboard | Tray | Native ARM |
| --- | --- | --- | --- | --- |
| Linux | **Works** — full capture (portal / `grim`), then in-app region picker with live `W x H` | **Works** | **Works** (tray icon) | `aarch64-unknown-linux-gnu` via CI `cargo check` |
| macOS | **Best-effort** — `screencapture` (needs Screen Recording) + in-app region picker | **Works** | Not in v1.3 | **Yes** — `aarch64-apple-darwin` |
| Windows 10/11 | **WIP** — use Snipping Tool then `annotate` | **Works** | Not in v1.3 | `aarch64-pc-windows-msvc` via CI `cargo check` |

> [!CAUTION]
> No upload, accounts, or video recording. Global shortcuts and “replace default screenshot tool” depend on your desktop. Headless / cloud VMs usually **compile** but cannot open annotate / settings — use `--no-annotate` or `annotate` on a real display.
>
> Default annotate is an **on-screen overlay** at the capture. Settings → Annotate mode can switch to the floating preview window. Toolbar: light gray/white flat bar, charcoal icons, purple active-only highlight.

### ARM / cross builds

```bash
cargo build --release --target aarch64-apple-darwin

rustup target add aarch64-pc-windows-msvc
cargo build --release --target aarch64-pc-windows-msvc

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
  <img src="https://cdn.simpleicons.org/apple/000000" width="32" height="32" alt="Apple">
  <img src="https://cdn.simpleicons.org/windows/0078D4" width="32" height="32" alt="Windows">
</p>

* **Language / CLI:** Rust, `tokio`, `clap`, `directories`, `serde` / `toml`
* **Capture:** Linux portal + `grim`/`slurp`; macOS `screencapture`; Windows stub (WIP)
* **Annotate / settings / region UI:** egui + eframe; save dialog via `rfd`
* **Toolbar icons:** [Iconoir](https://iconoir.com/) SVGs (vendored under `assets/icons/`, MIT), rasterized with `resvg`
* **Images:** `image` / `imageproc`
* **Clipboard:** `arboard` (on Linux, `wl-clipboard` on PATH helps)
* **Tray:** Linux only (`ksni`)

## Features

1. **Capture** — `cargo run --release -- region` / `full`.
2. **On-screen annotate (default)** — selection handles, `W x H` pill, bottom flat toolbar.
3. **Region resolution** — live size text while selecting.
4. **Preview annotate** — floating HyperShot window with Iconoir tool rail. Ctrl+scroll adjusts thickness.
5. **Save** — file dialog. Cancel keeps the window open.
6. **Enter** — save dialog + copy + toast + close.
7. **Pixelate** — soft mosaic so fine text is hard to read.
8. **Clipboard** — settings default + `--clipboard` / `--no-clipboard`.
9. **Settings** — cards with help tooltips, annotate mode, folder picker, UI + About (`1.3`).
10. **Tray** — Linux: `cargo run --release -- tray`.

## Runtime dependencies

### Linux

Portal first for fullscreen capture; `grim` (+ `slurp` for older region fallback paths) when the portal fails. Install `wl-clipboard` for reliable clipboard.

#### Fedora 44 COSMIC — install now

```bash
sudo dnf install grim slurp wl-clipboard \
  xdg-desktop-portal xdg-desktop-portal-cosmic
```

| Need | Binary / package | Role |
| --- | --- | --- |
| Capture fallback | `grim` | PNG when portal fails |
| Region fallback | `slurp` | Legacy geometry helper |
| Clipboard | `wl-copy` (`wl-clipboard`) | Helps clipboard copy |

| Desktop | Fedora packages |
| --- | --- |
| COSMIC | `xdg-desktop-portal` + `xdg-desktop-portal-cosmic` |
| GNOME | `xdg-desktop-portal` + `xdg-desktop-portal-gnome` |
| KDE Plasma | `xdg-desktop-portal` + `xdg-desktop-portal-kde` |
| Sway / Hyprland | `xdg-desktop-portal` + `xdg-desktop-portal-wlr` |

> [!NOTE]
> Keep `grim` + `slurp` installed on COSMIC / tiling. Re-login after adding a portal backend if capture still fails.

### macOS

1. Grant **Screen Recording** to your terminal (or the `hypershot` binary) in System Settings → Privacy & Security.
2. `cargo run --release -- region` captures the display, then opens the region picker with live size text.

### Windows 10/11

1. Capture is **WIP** in v1.3.
2. Do this instead: Win+Shift+S (or Snipping Tool) → save a PNG →

   ```bash
   cargo run --release -- annotate path\to\snip.png
   ```

## Install

1. Install platform deps above.
2. Clone:

   ```bash
   git clone https://github.com/kmfd14/hypershot.git
   cd HyperShot
   ```

3. Jump to [Quick start](#quick-start).

### Optional install

```bash
cargo build --release
cp target/release/hypershot ~/.local/bin/   # or Windows equivalent on PATH
```

## Quick start

1. Region → annotate → Save dialog:

   ```bash
   cargo run --release -- region
   ```

2. Fullscreen: `cargo run --release -- full`

3. Annotate an existing PNG: `cargo run --release -- annotate ~/Pictures/hypershot/hypershot-123.png`

4. Settings: `cargo run --release -- settings`

5. Flags:

   ```bash
   cargo run --release -- region --no-annotate
   cargo run --release -- region -o /tmp/snip.png
   cargo run --release -- region --no-clipboard
   cargo run --release -- full --save-dir ~/Pictures/Screenshots
   ```

## Hotkeys

**Assign… in Settings is not a live global hotkey.** Bind the **bind command** in your desktop keyboard settings. macOS / Windows: bind your own shortcut to `hypershot region` after optional install.

| Desktop | Where to bind | Sample |
| --- | --- | --- |
| Fedora COSMIC | Settings → Keyboard → Custom shortcuts | `cargo run --release --manifest-path /path/to/HyperShot/Cargo.toml -- region` |
| Hyprland | `hyprland.conf` | `bind = SUPER, Print, exec, cargo run --release --manifest-path /path/to/HyperShot/Cargo.toml -- region` |
| Sway | config | `bindsym $mod+Print exec cargo run --release --manifest-path /path/to/HyperShot/Cargo.toml -- region` |
| GNOME / KDE | Custom shortcut | Same command (does not steal Print Screen by itself) |

Annotate keys:

* **Enter** = save dialog + copy + toast + close
* **Save** = save dialog + close
* **Copy** = clipboard toast + close
* **Exit** = cancel; **Esc** = same when Esc-to-exit is on
* **Ctrl+Z** / **Ctrl+Shift+Z** = undo / redo

## Settings

`cargo run --release -- settings`, or **Settings** on the annotate bar.

Config: `~/.config/hypershot/config.toml` (macOS/Windows use the platform equivalent).

1. **Shortcuts** — label only (not a live system shortcut).
2. **Run at startup** / **Replace default** — Linux desktop helpers; off by default.
3. **Stay in tray** — Linux tray icon; off by default.
4. **User Interface** — annotate mode, toolbar, theme, Esc-to-exit, transparency (toolbar stays opaque).
5. **About** — HyperShot `1.3`.
6. **Save directory** + **Save to clipboard** defaults.

Env override: `HYPERSHOT_SAVE_DIR` (legacy `LIGHTSNIP_SAVE_DIR` still works).

## Documentation

| Guide | Description |
| --- | --- |
| [User guide](./docs/user-guide.md) | Capture → annotate → save / clipboard → settings → hotkeys |

## Contributing

1. Fork the repository.
2. Create a feature branch from `main`.
3. Open a pull request with a clear description of the change.
4. Run `cargo test` (and CI covers cross targets).

Issues: <https://github.com/kmfd14/hypershot/issues>

## License

[MIT](./LICENSE)
