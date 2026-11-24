# lightsnip

**Do this now** (from the repo checkout, ~1 minute after deps):

```bash
cargo run --release -- region
```

Or fullscreen:

```bash
cargo run --release -- full
```

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install and “replace default screenshot tool” are separate steps and off by default — see [Optional install](#optional-install).

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

1. **Capture** — `cargo run --release -- region` (primary) and `cargo run --release -- full`.
2. **Annotate** — arrow, rectangle, blur/pixelate, undo/clear (planned).
3. **Output** — PNG path printed to stdout; configurable save dir + clipboard later.
4. **Settings** — small control → window for shortcuts, startup, replace-default, tray, save dir, clipboard preference (planned).
5. **Quiet UI** — neutral surfaces, clear hierarchy, small chrome (no gradient/glow marketing look).

## Prerequisites

Rust toolchain, a Wayland session, and either a working xdg-desktop-portal Screenshot path **or** `grim` + `slurp`.

```bash
git clone https://github.com/kmfd14/lightsnip.git
cd lightsnip
```

## Quick start

Primary path: build and run from the checkout. Do not install a system binary.

1. Region (prints the PNG path):

   ```bash
   cargo run --release -- region
   ```

2. Fullscreen:

   ```bash
   cargo run --release -- full
   ```

3. Optional custom path:

   ```bash
   cargo run --release -- region -o /tmp/snip.png
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

Settings window lands in a later milestone. Planned fields:

Desktop / session:

1. **Shortcuts** — docs + configure what Wayland allows.
2. **Run at startup** — opt-in autostart (off by default).
3. **Replace default screenshot tool** — off by default; only where the DE allows.
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
