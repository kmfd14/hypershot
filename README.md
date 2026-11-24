# lightsnip

<p align="center">
  <img src="./assets/logo.svg" alt="lightsnip logo" width="96" height="96">
</p>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

**Do this now** (from the repo checkout, Wayland session, ~1 minute after deps):

```bash
cargo run --release -- region
```

Draw arrow, box, or pixelate a private region → **Save** / Enter, or **Copy** (always clipboard). Escape clears a draft drag, then cancels. PNG lands in your save directory (default `~/Pictures/lightsnip/`).

Open settings anytime:

```bash
cargo run --release -- settings
```

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install, autostart, tray, and “replace default screenshot tool” are separate and **off by default**.

Wayland screenshot tool with capture and annotation. Region or fullscreen capture → floating annotate window → save or copy. Built in Rust. Left liquid-glass tool strip + Apple-inspired settings.

Use it when you want a Lightshot / Flameshot-style loop on Linux Wayland (COSMIC, GNOME, KDE, Sway, Hyprland) without leaving your compositor’s permission model.

It provides:

* Portal-first capture with grim/slurp fallback
* Annotate overlay: arrow, rectangle/box, pixelate (privacy), undo/clear
* Save to timestamped PNG under a default / configurable dir
* Clipboard copy via `wl-copy` (default from settings, on out of the box)
* Settings window + optional StatusNotifierItem tray

> [!CAUTION]
> Linux Wayland only for v1. No Windows, macOS, or X11-first support. No upload, accounts, or video recording. Global shortcuts and “replace default screenshot tool” depend on your desktop. lightsnip documents limits and writes opt-in helpers; it does not pretend PrintScreen is always stolen for you.
>
> The annotate / settings windows need a real graphical session (`WAYLAND_DISPLAY` or `DISPLAY`) plus native libs for egui/winit (for example `libxkbcommon`). Headless / cloud VMs usually can **compile** but cannot run those UIs; use `--no-annotate` there, or expect a warning and a raw (unannotated) save. Annotate stay-on-top is best-effort on Wayland.

## Tech stack

<p align="left">
  <img src="https://cdn.simpleicons.org/rust/000000" width="32" height="32" alt="Rust">
  <img src="https://cdn.simpleicons.org/tokio/000000" width="32" height="32" alt="Tokio">
  <img src="https://cdn.simpleicons.org/toml/9C4121" width="32" height="32" alt="TOML">
  <img src="https://cdn.simpleicons.org/linux/FCC624" width="32" height="32" alt="Linux">
  <img src="https://cdn.simpleicons.org/wayland/ff0000" width="32" height="32" alt="Wayland">
</p>

* **Language / CLI:** Rust, `tokio`, `clap`, `directories`, `serde` / `toml`
* **Capture:** `ashpd` (xdg-desktop-portal Screenshot); fallback `grim` + `slurp`
* **Annotate / settings UI:** egui + eframe (floating annotate window + settings)
* **Images:** `image` / `imageproc` (draw annotations onto PNG)
* **Clipboard:** `wl-clipboard` (`wl-copy`) when on `PATH`
* **Tray:** `ksni` (StatusNotifierItem) when you enable stay-in-tray / run `tray`

## Features

1. **Capture** — `cargo run --release -- region` (primary) and `cargo run --release -- full`.
2. **Annotate** — floating window after capture (not fullscreen), or `cargo run --release -- annotate path.png`. Left glass strip: arrow, box, pixelate, undo, clear, settings, **Save**, **Copy**. Enter / Save commits; Copy writes + clipboard + dismisses; Escape clears a draft, then cancels.
3. **Save** — default from settings (else `~/Pictures/lightsnip/lightsnip-<unix>.png`); override with `-o` / `--output`, `--save-dir`, or `LIGHTSNIP_SAVE_DIR`.
4. **Clipboard** — default from settings (`save_to_clipboard`, on by default); `--clipboard` / `--no-clipboard` override for one run.
5. **Settings** — `cargo run --release -- settings` (or the annotate **Settings** control). Six fields: shortcuts, run at startup, replace-default helper, stay in tray, save directory, save to clipboard.
6. **Tray** — `cargo run --release -- tray` (Region / Full / Settings / Quit). Needs a StatusNotifier watcher; GNOME often needs an AppIndicator extension.
7. **Floating UI** — annotate is a bordered window sized to the image; vertical liquid-glass tool strip on the left (sketch-matched). Settings uses light grouped cards. No purple / glow marketing chrome.

<!-- VISUAL SUGGESTION [SCREENSHOT]:
Recommended file (does not exist yet): `assets/screenshots/annotate-overlay.png`

Add a screenshot of the floating annotate window on a Wayland desktop.
Show the left glass tool strip, image canvas, Save / Copy, and Settings.
Purpose: Prove the capture → annotate loop at a glance.
-->

## Runtime dependencies

lightsnip prefers **xdg-desktop-portal** Screenshot. If the portal fails, it falls back to `grim` (+ `slurp` for region). Clipboard needs `wl-copy`.

### Fedora 44 COSMIC — install now

```bash
sudo dnf install grim slurp wl-clipboard \
  xdg-desktop-portal xdg-desktop-portal-cosmic
```

Then from the checkout: `cargo run --release -- region`.

### Runtime tools checklist

| Need | Binary / package | Role |
| --- | --- | --- |
| Capture fallback | `grim` | Writes the PNG when the portal fails |
| Region fallback | `slurp` | Interactive region geometry for `grim` |
| Clipboard | `wl-copy` from `wl-clipboard` | Copy PNG after Save |

Also required: Rust toolchain, a Wayland session, and a display for annotate / settings (see caution above).

### Portal backends (by desktop)

Install the shared portal plus the backend for your session. Full desktop installs often already include these.

| Desktop | Fedora packages |
| --- | --- |
| COSMIC | `xdg-desktop-portal` + `xdg-desktop-portal-cosmic` |
| GNOME | `xdg-desktop-portal` + `xdg-desktop-portal-gnome` |
| KDE Plasma | `xdg-desktop-portal` + `xdg-desktop-portal-kde` |
| Sway / Hyprland (wlroots) | `xdg-desktop-portal` + `xdg-desktop-portal-wlr` |

> [!NOTE]
> On COSMIC and some tiling setups the portal Screenshot path can fail or be incomplete. Keep `grim` + `slurp` installed so the fallback works. Re-login after installing a portal backend if capture still fails.

Other distros: same binary names (`grim`, `slurp`, `wl-clipboard`); package managers differ (`apt`, `pacman`, …).

## Install

Primary path: clone and `cargo run`. Do not install a system binary unless you want one.

1. Install [Runtime dependencies](#runtime-dependencies) for your desktop (Fedora COSMIC one-liner above).
2. Clone:

   ```bash
   git clone https://github.com/kmfd14/lightsnip.git
   cd lightsnip
   ```

3. Jump to [Quick start](#quick-start).

### Optional install

Skip unless you want a `lightsnip` command on your `PATH`. Default workflow stays `cargo run` from the checkout. Nothing is copied to `/usr`, and other screenshot apps are unaffected. Replace-default and autostart stay off until you enable them in settings.

1. Build:

   ```bash
   cargo build --release
   ```

2. Copy somewhere on your user `PATH` (not required):

   ```bash
   cp target/release/lightsnip ~/.local/bin/
   ```

3. Then you can run `lightsnip region` / `lightsnip full` / `lightsnip settings` / `lightsnip tray` the same way.

## Quick start

1. Region → annotate → Save (prints the PNG path; copies to clipboard when preferred and `wl-copy` exists):

   ```bash
   cargo run --release -- region
   ```

2. Fullscreen:

   ```bash
   cargo run --release -- full
   ```

3. Annotate an existing PNG:

   ```bash
   cargo run --release -- annotate ~/Pictures/lightsnip/lightsnip-123.png
   ```

4. Open settings (change save dir / clipboard default without editing files by hand):

   ```bash
   cargo run --release -- settings
   ```

5. Skip overlay, custom path, or skip clipboard:

   ```bash
   cargo run --release -- region --no-annotate
   cargo run --release -- region -o /tmp/snip.png
   cargo run --release -- region --no-clipboard
   cargo run --release -- full --save-dir ~/Pictures/Screenshots
   ```

6. Bind a compositor hotkey to the same `cargo run …` command, or to a binary only after optional install (see [Hotkeys](#hotkeys)).

## Hotkeys

lightsnip documents compositor binds for v1. Global grabs are compositor-dependent.

1. Bind a key to `cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` (or `full`), or to `lightsnip region` only if you used optional install.
2. Or copy the suggested command from **Settings → Shortcuts** (stored in the TOML config).
3. Sample binds (replace the path, or use `lightsnip region` after optional install):

   | Desktop | Sample |
   | --- | --- |
   | Hyprland | `bind = SUPER, Print, exec, cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
   | Sway | `bindsym $mod+Print exec cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
   | GNOME | Settings → Keyboard → Custom Shortcut → command above (does not steal PrintScreen by itself) |
   | KDE | System Settings → Shortcuts → Custom → same command |

4. If “replace default screenshot tool” is unavailable on your DE, keep the custom shortcut and leave the DE default alone.

Annotate keys: Enter = Save, Escape = clear draft then Cancel (or close Settings first), Ctrl+Z = Undo. **Copy** always copies to the clipboard after writing the PNG.

**Stay on top:** the annotate window requests always-on-top via winit. Many Wayland compositors ignore or gate this (no portable layer-shell guarantee in egui); X11 is more reliable. If the window sinks, raise it from the task switcher.

## Settings

Open with `cargo run --release -- settings`, or **Settings** on the annotate bar.

Config file (created on Save): `~/.config/lightsnip/config.toml` (via the `directories` crate).

Desktop / session:

1. **Shortcuts** — editable bind hint; Wayland has no portable global hotkey API. Bind the command in your compositor or DE.
2. **Run at startup** — opt-in XDG autostart (`~/.config/autostart/lightsnip.desktop` → `tray`). Off by default.
3. **Replace default screenshot tool** — opt-in user `.desktop` helper under `~/.local/share/applications/`. Does **not** rebind PrintScreen on GNOME/KDE; assign a custom shortcut yourself. Off by default.
4. **Stay in taskbar / tray** — prefer the StatusNotifierItem tray (`cargo run --release -- tray`). Needs a tray watcher; GNOME often needs an extension. Off by default.

Output defaults:

1. **Save directory** — default PNG folder (CLI / env still override).
2. **Save to clipboard** — default on/off after Save (CLI flags still override).

Precedence for save path: `--output` → `--save-dir` → `LIGHTSNIP_SAVE_DIR` → config → `~/Pictures/lightsnip`.

Precedence for clipboard: `--clipboard` / `--no-clipboard` → config `save_to_clipboard`.

## Documentation

| Guide | Description |
| --- | --- |
| [User guide](./docs/user-guide.md) | Capture → annotate → save / clipboard → settings → hotkeys |

## Contributing

1. Fork the repository.
2. Create a feature branch from `main`.
3. Open a pull request with a clear description of the change.

Issues: <https://github.com/kmfd14/lightsnip/issues>

## License

[MIT](./LICENSE)
