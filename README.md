# lightsnip

<p align="center">
  <img src="./assets/logo.svg" alt="lightsnip logo" width="96" height="96">
</p>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![CI](https://github.com/kmfd14/lightsnip/actions/workflows/ci.yml/badge.svg)](https://github.com/kmfd14/lightsnip/actions/workflows/ci.yml)

**Do this now** (from the repo checkout, graphical session, ~1 minute after deps):

```bash
cargo run --release -- region
```

1. Mark up with icon tools (arrow, box, pixelate, step number).
2. **Save** → pick a file in the dialog → close. **Enter** → same dialog + copy to clipboard + toast + close. **Copy** → clipboard toast + close (no dialog).
3. Cancel the save dialog (or **Exit**) to keep the window open.

PNG default folder: `~/Pictures/lightsnip/` (overridable in Settings).

```bash
cargo run --release -- settings
```

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install, autostart, tray, and “replace default screenshot tool” are separate and **off by default**.

Screenshot tool with capture + annotation. Region or fullscreen → floating annotate window → save or copy. Built in Rust. Flat pastel tool rail + Apple-inspired settings.

## Platform support (v0.5)

| Platform | Capture | Annotate / Save / Clipboard | Tray | Native ARM |
| --- | --- | --- | --- | --- |
| Linux Wayland | **Works** — portal first, `grim`/`slurp` fallback | **Works** | **Works** (SNI / `ksni`) | `aarch64-unknown-linux-gnu` via CI `cargo check` |
| macOS | **Best-effort** — `screencapture` (needs Screen Recording permission) | **Works** (`arboard` + `rfd`) | Not in v0.5 | **Yes** — `aarch64-apple-darwin` (CI on `macos-latest`) |
| Windows 10/11 | **WIP** — stub error; use Snipping Tool then `annotate` | **Works** (`arboard` + `rfd`) | Not in v0.5 | `aarch64-pc-windows-msvc` via CI `cargo check` |

> [!CAUTION]
> No upload, accounts, or video recording. Global shortcuts and “replace default screenshot tool” depend on your desktop (Linux XDG helpers only). Headless / cloud VMs usually **compile** but cannot open annotate / settings — use `--no-annotate` or `annotate` on a real display.
>
> Annotate chrome is flat pastel purple / gray / white (no liquid-glass). Transparency is compositor-dependent; “background blur” frosts the panel only.

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
* **Annotate / settings UI:** egui + eframe; save dialog via `rfd`
* **Images:** `image` / `imageproc`
* **Clipboard:** `arboard` (Wayland still benefits from `wl-clipboard` on PATH)
* **Tray:** Linux `ksni` only

## Features

1. **Capture** — `cargo run --release -- region` / `full` (see [Platform support](#platform-support-v05)).
2. **Annotate** — floating **Lightsnip** window, or `cargo run --release -- annotate path.png`. Flat pastel icon rail (no text label): arrow, box, pixelate, step number, color (default red), undo, redo, clear, settings, **Save**, **Copy**, **Exit**.
3. **Save** — file dialog (defaults under configured save dir). Cancel keeps the window open.
4. **Enter** — save dialog + copy to clipboard + toast + close (always). Esc respects **Esc to exit**; Enter does not.
5. **Pixelate** — strong single-pass mosaic (`PIXELATE_BLOCK = 28` + offset densify) so text is unreadable; undo/redo still work.
6. **Clipboard** — settings default + `--clipboard` / `--no-clipboard`.
7. **Settings** — cards with `?` tooltips, folder picker, UI + About (`0.5`).
8. **Tray** — Linux only: `cargo run --release -- tray`.

## Runtime dependencies

### Linux (Wayland)

Portal first; `grim` + `slurp` fallback. Install `wl-clipboard` for reliable clipboard.

#### Fedora 44 COSMIC — install now

```bash
sudo dnf install grim slurp wl-clipboard \
  xdg-desktop-portal xdg-desktop-portal-cosmic
```

| Need | Binary / package | Role |
| --- | --- | --- |
| Capture fallback | `grim` | PNG when portal fails |
| Region fallback | `slurp` | Region geometry for `grim` |
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
2. `cargo run --release -- region` uses `screencapture -i`.

### Windows 10/11

1. Capture is **WIP** in v0.5.
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
4. **User Interface** — toolbar / theme / Esc-to-exit / transparency (toolbar stays opaque).
5. **About** — Lightsnip `0.5`.
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
