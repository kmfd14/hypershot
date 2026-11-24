# lightsnip

**Do this now:** `cargo build --release && ./target/release/lightsnip region`

Wayland screenshot tool with capture and annotation. Region or fullscreen → (soon) mark up → save or copy. Built in Rust. Quiet, functional desktop UI planned for annotate/settings.

Use it when you want a Lightshot / Flameshot-style loop on Linux Wayland (GNOME, KDE, Sway, Hyprland) without leaving your compositor’s permission model.

**Milestone 1 (this tree):** capture CLI only — portal-first PNG write with grim/slurp fallback.

It provides / will provide:

* Portal-first capture with grim/slurp fallback (**now**)
* Annotate overlay (arrow, box, blur/pixelate, undo) — later
* Save to file and/or clipboard — later
* Small settings window (shortcuts, startup, tray, paths) — later

> [!CAUTION]
> Linux Wayland only for v1. No Windows, macOS, or X11-first support. No upload, accounts, or video recording. Global shortcuts and “replace default screenshot tool” depend on your desktop — lightsnip documents limits instead of pretending they always work.

## Tech stack

* **Language:** Rust, `tokio`, `clap`
* **Capture:** `ashpd` (xdg-desktop-portal Screenshot); fallback `grim` + `slurp`
* **Images (later):** `image` / `imageproc` for annotate
* **UI (later):** egui + eframe (overlay + settings)
* **Clipboard (later):** portal or `wl-clipboard` (`wl-copy`)

## Features

1. **Capture** — `lightsnip region` (primary) and `lightsnip full`.
2. **Annotate** — arrow, rectangle, blur/pixelate, undo/clear (planned).
3. **Output** — PNG path printed to stdout; configurable save dir + clipboard later.
4. **Settings** — small control → window for shortcuts, startup, replace-default, tray, save dir, clipboard preference (planned).
5. **Quiet UI** — neutral surfaces, clear hierarchy, small chrome (no gradient/glow marketing look).

## Install

Prerequisites: Rust toolchain, a Wayland session, and either a working xdg-desktop-portal Screenshot path **or** `grim` + `slurp`.

1. Clone:

   ```bash
   git clone https://github.com/kmfd14/lightsnip.git
   cd lightsnip
   ```

2. Build the release binary:

   ```bash
   cargo build --release
   ```

3. Put the binary on your `PATH` (example):

   ```bash
   cp target/release/lightsnip ~/.local/bin/
   ```

4. Confirm the session can capture (portal **or** grim/slurp installed).

## Quick start

1. Capture a region (prints the PNG path):

   ```bash
   lightsnip region
   ```

2. Or fullscreen:

   ```bash
   lightsnip full
   ```

3. Optional custom path:

   ```bash
   lightsnip region -o /tmp/snip.png
   ```

4. Bind your compositor hotkey to `lightsnip region` (see [Hotkeys](#hotkeys)).

## Hotkeys

lightsnip documents compositor binds for v1. Global grabs are compositor-dependent.

1. Add a compositor bind that runs `lightsnip region` (or `full`).
2. Sample ideas (adjust to your config):

   | Desktop | Idea |
   | --- | --- |
   | Sway / Hyprland | Bind a key to `exec lightsnip region` |
   | GNOME / KDE | Custom shortcut → command `lightsnip region` |

## Settings

Settings window lands in a later milestone. Planned fields:

Desktop / session:

1. **Shortcuts** — docs + configure what Wayland allows.
2. **Run at startup** — opt-in autostart.
3. **Replace default screenshot tool** — only where the DE allows.
4. **Stay in taskbar / tray** — keep tray/status presence when enabled.

Output defaults:

1. **Save directory** — default PNG folder.
2. **Save to clipboard** — default on/off or always-copy preference.

## Documentation

| Guide | Description |
| --- | --- |
| [User guide](./docs/user-guide.md) | Capture now; annotate / save / settings roadmap |

## Contributing

1. Fork the repository.
2. Create a feature branch from `main`.
3. Open a pull request with a clear description of the change.

## License

[MIT](./LICENSE)
