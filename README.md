# lightsnip

**Do this now** (from the repo checkout, ~1 minute after deps):

```bash
cargo run --release -- region
```

Or fullscreen:

```bash
cargo run --release -- full
```

PNG lands in `~/Pictures/lightsnip/` (timestamp name) and copies to the clipboard when `wl-copy` is installed. Path prints to stdout.

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install and “replace default screenshot tool” are separate steps and off by default — see [Optional install](#optional-install).

Wayland screenshot tool with capture and annotation. Region or fullscreen → (soon) mark up → save or copy. Built in Rust. Quiet, functional desktop UI planned for annotate/settings.

Use it when you want a Lightshot / Flameshot-style loop on Linux Wayland (GNOME, KDE, Sway, Hyprland) without leaving your compositor’s permission model.

**Milestone 2 (this tree):** capture + save to a known dir + clipboard via `wl-copy`. Annotate overlay and settings window come later.

It provides / will provide:

* Portal-first capture with grim/slurp fallback (**now**)
* Save to timestamped PNG under a default / configurable dir (**now**)
* Clipboard copy via `wl-copy` (**now**; default on)
* Annotate overlay (arrow, box, blur/pixelate, undo) — later
* Small settings window (shortcuts, startup, tray, paths) — later

> [!CAUTION]
> Linux Wayland only for v1. No Windows, macOS, or X11-first support. No upload, accounts, or video recording. Global shortcuts and “replace default screenshot tool” depend on your desktop — lightsnip documents limits instead of pretending they always work.

## Tech stack

* **Language:** Rust, `tokio`, `clap`, `directories`
* **Capture:** `ashpd` (xdg-desktop-portal Screenshot); fallback `grim` + `slurp`
* **Clipboard:** `wl-clipboard` (`wl-copy`) when on `PATH`
* **Images (later):** `image` / `imageproc` for annotate
* **UI (later):** egui + eframe (overlay + settings)

## Features

1. **Capture** — `cargo run --release -- region` (primary) and `cargo run --release -- full`.
2. **Save** — default `~/Pictures/lightsnip/lightsnip-<unix>.png`; override with `-o` / `--output`, `--save-dir`, or `LIGHTSNIP_SAVE_DIR`.
3. **Clipboard** — on by default via `wl-copy`; use `--no-clipboard` to skip; `--clipboard` to affirm.
4. **Annotate** — arrow, rectangle, blur/pixelate, undo/clear (planned).
5. **Settings** — small control → window for shortcuts, startup, replace-default, tray, save dir, clipboard preference (planned).
6. **Quiet UI** — neutral surfaces, clear hierarchy, small chrome (no gradient/glow marketing look).

## Prerequisites

Rust toolchain, a Wayland session, and either a working xdg-desktop-portal Screenshot path **or** `grim` + `slurp`. For clipboard: `wl-clipboard` (`wl-copy`).

```bash
git clone https://github.com/kmfd14/lightsnip.git
cd lightsnip
```

## Quick start

Primary path: build and run from the checkout. Do not install a system binary.

1. Region (prints the PNG path; copies to clipboard when `wl-copy` exists):

   ```bash
   cargo run --release -- region
   ```

2. Fullscreen:

   ```bash
   cargo run --release -- full
   ```

3. Custom path, or skip clipboard:

   ```bash
   cargo run --release -- region -o /tmp/snip.png
   cargo run --release -- region --no-clipboard
   cargo run --release -- full --save-dir ~/Pictures/Screenshots
   ```

4. Bind a compositor hotkey to the same `cargo run …` command, or to a binary only after optional install (see [Hotkeys](#hotkeys)).

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

3. Then you can run `lightsnip region` / `lightsnip full` the same way.

## Hotkeys

lightsnip documents compositor binds for v1. Global grabs are compositor-dependent.

1. Bind a key to `cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` (or `full`), or to `lightsnip region` only if you used optional install.
2. Sample ideas (adjust to your config):

   | Desktop | Idea |
   | --- | --- |
   | Sway / Hyprland | Bind a key to `exec` the `cargo run … region` command (or `lightsnip region` after optional install) |
   | GNOME / KDE | Custom shortcut → same command |

## Settings

Settings window lands in a later milestone. Until then, override save location with `--save-dir`, `-o` / `--output`, or `LIGHTSNIP_SAVE_DIR`, and clipboard with `--clipboard` / `--no-clipboard`.

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
| [User guide](./docs/user-guide.md) | Capture, save, clipboard now; annotate / settings roadmap |

## Contributing

1. Fork the repository.
2. Create a feature branch from `main`.
3. Open a pull request with a clear description of the change.

## License

[MIT](./LICENSE)
