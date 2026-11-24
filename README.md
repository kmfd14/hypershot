# lightsnip

**Do this now** (from the repo checkout, Wayland session, ~1 minute after deps):

```bash
cargo run --release -- region
```

Draw arrow or box → **Save** / Enter. Escape cancels. PNG lands in `~/Pictures/lightsnip/` and copies to the clipboard when `wl-copy` is installed.

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install and “replace default screenshot tool” are separate steps and off by default — see [Optional install](#optional-install).

Wayland screenshot tool with capture and annotation. Region or fullscreen → mark up → save or copy. Built in Rust. Quiet, functional desktop UI for the annotate overlay.

Use it when you want a Lightshot / Flameshot-style loop on Linux Wayland (GNOME, KDE, Sway, Hyprland) without leaving your compositor’s permission model.

**Milestone 3 (this tree):** capture → annotate overlay (arrow, box, undo/clear) → save + clipboard. Settings window comes later. Blur/pixelate is Milestone 4.

It provides / will provide:

* Portal-first capture with grim/slurp fallback (**now**)
* Annotate overlay: arrow, rectangle/box, undo/clear (**now**)
* Save to timestamped PNG under a default / configurable dir (**now**)
* Clipboard copy via `wl-copy` (**now**; default on after Save)
* Blur / pixelate tool — later
* Small settings window (shortcuts, startup, tray, paths) — later

> [!CAUTION]
> Linux Wayland only for v1. No Windows, macOS, or X11-first support. No upload, accounts, or video recording. Global shortcuts and “replace default screenshot tool” depend on your desktop — lightsnip documents limits instead of pretending they always work.
>
> The annotate overlay needs a real graphical session (`WAYLAND_DISPLAY` or `DISPLAY`) plus native libs for egui/winit (for example `libxkbcommon`). Headless / cloud VMs usually can **compile** but cannot run the overlay; use `--no-annotate` there, or expect a warning and a raw (unannotated) save.

## Tech stack

* **Language:** Rust, `tokio`, `clap`, `directories`
* **Capture:** `ashpd` (xdg-desktop-portal Screenshot); fallback `grim` + `slurp`
* **Annotate UI:** egui + eframe (fullscreen overlay)
* **Images:** `image` / `imageproc` (draw annotations onto PNG)
* **Clipboard:** `wl-clipboard` (`wl-copy`) when on `PATH`

## Features

1. **Capture** — `cargo run --release -- region` (primary) and `cargo run --release -- full`.
2. **Annotate** — fullscreen overlay after capture, or `cargo run --release -- annotate path.png`. Tools: arrow, box, undo, clear. Enter / Save commits; Escape cancels. Blur later.
3. **Save** — default `~/Pictures/lightsnip/lightsnip-<unix>.png`; override with `-o` / `--output`, `--save-dir`, or `LIGHTSNIP_SAVE_DIR`.
4. **Clipboard** — on by default via `wl-copy` after Save; use `--no-clipboard` to skip; `--clipboard` to affirm.
5. **Settings** — small control → window for shortcuts, startup, replace-default, tray, save dir, clipboard preference (planned).
6. **Quiet UI** — neutral charcoal chrome, small radius, no gradient/glow/pill marketing look.

## Prerequisites

Rust toolchain, a Wayland session, and either a working xdg-desktop-portal Screenshot path **or** `grim` + `slurp`. For clipboard: `wl-clipboard` (`wl-copy`). For annotate: a display plus egui native deps (see caution above).

```bash
git clone https://github.com/kmfd14/lightsnip.git
cd lightsnip
```

## Quick start

Primary path: build and run from the checkout. Do not install a system binary.

1. Region → annotate → Save (prints the PNG path; copies to clipboard when `wl-copy` exists):

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

4. Skip overlay, custom path, or skip clipboard:

   ```bash
   cargo run --release -- region --no-annotate
   cargo run --release -- region -o /tmp/snip.png
   cargo run --release -- region --no-clipboard
   cargo run --release -- full --save-dir ~/Pictures/Screenshots
   ```

5. Bind a compositor hotkey to the same `cargo run …` command, or to a binary only after optional install (see [Hotkeys](#hotkeys)).

## Optional install

Skip this unless you want a `lightsnip` command on your `PATH`. Default workflow stays `cargo run` from the checkout — nothing is copied to `/usr`, and other screenshot apps are unaffected. Replace-default is also off by default (settings milestone).

1. Build:

   ```bash
   cargo build --release
   ```

2. Copy somewhere on your user `PATH` (not required):

   ```bash
   cp target/release/lightsnip ~/.local/bin/
   ```

3. Then you can run `lightsnip region` / `lightsnip full` / `lightsnip annotate …` the same way.

## Hotkeys

lightsnip documents compositor binds for v1. Global grabs are compositor-dependent.

1. Bind a key to `cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` (or `full`), or to `lightsnip region` only if you used optional install.
2. Sample ideas (adjust to your config):

   | Desktop | Idea |
   | --- | --- |
   | Sway / Hyprland | Bind a key to `exec` the `cargo run … region` command (or `lightsnip region` after optional install) |
   | GNOME / KDE | Custom shortcut → same command |

Overlay keys (when annotate is open): Enter = Save, Escape = Cancel, Ctrl+Z = Undo.

## Settings

Settings window lands in a later milestone. Until then, override save location with `--save-dir`, `-o` / `--output`, or `LIGHTSNIP_SAVE_DIR`, and clipboard with `--clipboard` / `--no-clipboard`. Use `--no-annotate` when you need raw capture without the overlay.

Planned settings fields:

Desktop / session:

1. **Shortcuts** — docs + configure what Wayland allows.
2. **Run at startup** — opt-in autostart (off by default).
3. **Replace default screenshot tool** — off by default; only where the DE allows.
4. **Stay in taskbar / tray** — keep tray/status presence when enabled.

Output defaults:

1. **Save directory** — default PNG folder (CLI / env cover this for now).
2. **Save to clipboard** — default on/off preference (CLI flags cover this for now).

## Documentation

| Guide | Description |
| --- | --- |
| [User guide](./docs/user-guide.md) | Capture → annotate → save / clipboard; settings roadmap |

## Contributing

1. Fork the repository.
2. Create a feature branch from `main`.
3. Open a pull request with a clear description of the change.

## License

[MIT](./LICENSE)
