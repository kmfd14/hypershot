# lightsnip

**Do this now** (from the repo checkout, Wayland session, ~1 minute after deps):

```bash
cargo run --release -- region
```

Draw arrow, box, or pixelate a private region → **Save** / Enter. Escape clears a draft drag, then cancels. PNG lands in your save directory (default `~/Pictures/lightsnip/`) and copies to the clipboard when that preference is on and `wl-copy` is installed.

Open settings anytime:

```bash
cargo run --release -- settings
```

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install, autostart, tray, and “replace default screenshot tool” are separate and **off by default**.

Wayland screenshot tool with capture and annotation. Region or fullscreen → mark up → save or copy. Built in Rust. Quiet, functional desktop UI for the annotate overlay and settings window.

Use it when you want a Lightshot / Flameshot-style loop on Linux Wayland (GNOME, KDE, Sway, Hyprland) without leaving your compositor’s permission model.

**Milestone 5 (this tree):** capture → annotate → save + clipboard, plus a settings window (shortcuts, startup, replace-default helper, tray, save dir, clipboard default) persisted as TOML.

It provides:

* Portal-first capture with grim/slurp fallback (**now**)
* Annotate overlay: arrow, rectangle/box, pixelate (privacy), undo/clear (**now**)
* Save to timestamped PNG under a default / configurable dir (**now**)
* Clipboard copy via `wl-copy` (**now**; default from settings, on out of the box)
* Settings window + optional StatusNotifierItem tray (**now**)

> [!CAUTION]
> Linux Wayland only for v1. No Windows, macOS, or X11-first support. No upload, accounts, or video recording. Global shortcuts and “replace default screenshot tool” depend on your desktop. lightsnip documents limits and writes opt-in helpers; it does not pretend PrintScreen is always stolen for you.
>
> The annotate / settings windows need a real graphical session (`WAYLAND_DISPLAY` or `DISPLAY`) plus native libs for egui/winit (for example `libxkbcommon`). Headless / cloud VMs usually can **compile** but cannot run those UIs; use `--no-annotate` there, or expect a warning and a raw (unannotated) save.

## Tech stack

* **Language:** Rust, `tokio`, `clap`, `directories`, `serde` / `toml`
* **Capture:** `ashpd` (xdg-desktop-portal Screenshot); fallback `grim` + `slurp`
* **Annotate / settings UI:** egui + eframe (fullscreen overlay + settings window)
* **Images:** `image` / `imageproc` (draw annotations onto PNG)
* **Clipboard:** `wl-clipboard` (`wl-copy`) when on `PATH`
* **Tray:** `ksni` (StatusNotifierItem) when you enable stay-in-tray / run `tray`

## Features

1. **Capture**: `cargo run --release -- region` (primary) and `cargo run --release -- full`.
2. **Annotate**: fullscreen overlay after capture, or `cargo run --release -- annotate path.png`. Tools: arrow, box, pixelate, undo, clear. **Settings** opens the settings popup. Enter / Save commits; Escape clears an in-progress drag, then cancels.
3. **Save**: default from settings (else `~/Pictures/lightsnip/lightsnip-<unix>.png`); override with `-o` / `--output`, `--save-dir`, or `LIGHTSNIP_SAVE_DIR`.
4. **Clipboard**: default from settings (`save_to_clipboard`, on by default); `--clipboard` / `--no-clipboard` override for one run.
5. **Settings**: `cargo run --release -- settings` (or the annotate **Settings** control). Six fields: shortcuts, run at startup, replace-default helper, stay in tray, save directory, save to clipboard.
6. **Tray**: `cargo run --release -- tray` (Region / Full / Settings / Quit). Needs a StatusNotifier watcher; GNOME often needs an AppIndicator extension.
7. **Quiet UI**: neutral charcoal chrome, small radius, no gradient/glow/pill marketing look.

## Prerequisites

Rust toolchain, a Wayland session, and either a working xdg-desktop-portal Screenshot path **or** `grim` + `slurp`. For clipboard: `wl-clipboard` (`wl-copy`). For annotate / settings: a display plus egui native deps (see caution above).

```bash
git clone https://github.com/kmfd14/lightsnip.git
cd lightsnip
```

## Quick start

Primary path: build and run from the checkout. Do not install a system binary.

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

## Optional install

Skip this unless you want a `lightsnip` command on your `PATH`. Default workflow stays `cargo run` from the checkout. Nothing is copied to `/usr`, and other screenshot apps are unaffected. Replace-default and autostart stay off until you enable them in settings.

1. Build:

   ```bash
   cargo build --release
   ```

2. Copy somewhere on your user `PATH` (not required):

   ```bash
   cp target/release/lightsnip ~/.local/bin/
   ```

3. Then you can run `lightsnip region` / `lightsnip full` / `lightsnip settings` / `lightsnip tray` the same way.

## Hotkeys

lightsnip documents compositor binds for v1. Global grabs are compositor-dependent.

1. Bind a key to `cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` (or `full`), or to `lightsnip region` only if you used optional install.
2. Or copy the suggested command from **Settings → Shortcuts** (stored in the TOML config).
3. Sample ideas (adjust to your config):

   | Desktop | Idea |
   | --- | --- |
   | Sway / Hyprland | Bind a key to `exec` the `cargo run … region` command (or `lightsnip region` after optional install) |
   | GNOME / KDE | Custom shortcut → same command |

Overlay keys (when annotate is open): Enter = Save, Escape = clear draft then Cancel (or close Settings first), Ctrl+Z = Undo.

## Settings

Open with `cargo run --release -- settings`, or **Settings** on the annotate bar.

Config file (created on Save): `~/.config/lightsnip/config.toml` (via the `directories` crate).

Desktop / session:

1. **Shortcuts**: editable bind hint; Wayland has no portable global hotkey API. Bind the command in your compositor or DE.
2. **Run at startup**: opt-in XDG autostart (`~/.config/autostart/lightsnip.desktop` → `tray`). Off by default.
3. **Replace default screenshot tool**: opt-in user `.desktop` helper under `~/.local/share/applications/`. Does **not** rebind PrintScreen on GNOME/KDE; assign a custom shortcut yourself. Off by default.
4. **Stay in taskbar / tray**: prefer the StatusNotifierItem tray (`cargo run --release -- tray`). Needs a tray watcher; GNOME often needs an extension. Off by default.

Output defaults:

1. **Save directory**: default PNG folder (CLI / env still override).
2. **Save to clipboard**: default on/off after Save (CLI flags still override).

Precedence for save path: `--output` → `--save-dir` → `LIGHTSNIP_SAVE_DIR` → config → `~/Pictures/lightsnip`.

Precedence for clipboard: `--clipboard` / `--no-clipboard` → config `save_to_clipboard`.

## Documentation

| Guide | Description |
| --- | --- |
| [User guide](./docs/user-guide.md) | Capture → annotate → save / clipboard → settings |

## Contributing

1. Fork the repository.
2. Create a feature branch from `main`.
3. Open a pull request with a clear description of the change.

## License

[MIT](./LICENSE)
