# lightsnip: user guide

**Do this now** (from the repo checkout, on a Wayland desktop):

```bash
cargo run --release -- region
```

1. Select a region.
2. Mark up with Arrow or Box, or drag **Pixelate** over private text.
3. Press **Save** or Enter (Escape clears a draft drag, then cancels and discards).

PNG path prints to stdout. Clipboard follows your settings default (on out of the box) when `wl-copy` is installed. About 1–2 minutes for first capture after deps.

Open settings (~1 minute):

```bash
cargo run --release -- settings
```

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install, autostart, tray, and “replace default screenshot tool” are separate and off by default.

Quiet charcoal chrome on annotate and settings. No decorative gradients or emoji controls.

---

## Capture (available now)

Pick one mode. Region is the default path. Run from the checkout. You do not need a system binary.

1. **Region**: `cargo run --release -- region` → portal interactive picker, or `slurp` → `grim` if the portal fails.
2. **Fullscreen**: `cargo run --release -- full` → active output via portal, or `grim` fallback.
3. Escape / cancel in the picker aborts with a non-zero exit.

Portal capture is preferred. If the portal path fails, lightsnip uses `grim` + `slurp` when installed.

---

## Annotate (available now)

After capture, a fullscreen overlay opens on the PNG. You can also annotate a file you already have.

1. Tools: **Arrow**, **Box** (rectangle), **Pixelate** (privacy mosaic).
2. Drag on the image to draw or redact. **Undo** (or Ctrl+Z) / **Clear** as needed.
3. **Settings** opens the settings popup (same six fields as `lightsnip settings`).
4. **Save** or Enter writes the annotated PNG, then clipboard (unless disabled).
5. **Cancel** exits without keeping the capture (region/full) or without changing the file (`annotate`). Escape first drops an in-progress drag (or closes Settings); Escape again cancels.
6. Re-open a PNG: `cargo run --release -- annotate ~/Pictures/lightsnip/….png`.
7. Skip the overlay: `cargo run --release -- region --no-annotate`.

**Display note:** annotate / settings need `WAYLAND_DISPLAY` or `DISPLAY` and egui native libs (for example `libxkbcommon`). Headless / cloud VMs can `cargo build` but usually cannot open those windows. On annotate failure after capture, lightsnip keeps the raw PNG and continues to clipboard/save.

---

## Save and clipboard (available now)

After **Save** (or after capture when using `--no-annotate` / when the overlay cannot open), lightsnip writes a PNG and (by default) copies it to the Wayland clipboard.

1. **Default save**: settings save directory, else `~/Pictures/lightsnip/lightsnip-<unix>.png` (directory is created if needed).
2. **Custom file**: `cargo run --release -- region -o ~/Pictures/snip.png`.
3. **Custom directory**: `cargo run --release -- full --save-dir ~/Pictures/Screenshots` (timestamped name inside that dir).
4. **Env override**: set `LIGHTSNIP_SAVE_DIR` to change the default directory without flags (beats config).
5. **Clipboard**: default from settings. Skip with `--no-clipboard`. Affirm with `--clipboard`.

If clipboard is skipped with a warning, install `wl-clipboard` (`wl-copy`) and retry. Save still succeeds either way; the PNG path is always printed after a successful Save.

---

## Settings (available now)

```bash
cargo run --release -- settings
```

Or click **Settings** on the annotate bar. Persist with **Save** inside the window. Config path: `~/.config/lightsnip/config.toml`.

1. **Shortcuts**: edit the suggested bind command. Wayland has no portable global hotkey API; bind that command in your compositor or DE.
2. **Run at startup**: opt-in XDG autostart → launches `tray`. Off by default.
3. **Replace default screenshot tool**: writes a user `.desktop` helper only. Does **not** steal PrintScreen on GNOME/KDE; assign a custom shortcut yourself. Off by default.
4. **Stay in taskbar / tray**: use StatusNotifierItem via `cargo run --release -- tray` (Region / Full / Settings / Quit). Needs a tray watcher; GNOME often needs an AppIndicator extension. Off by default.
5. **Save directory**: default PNG folder.
6. **Save to clipboard**: default on/off after Save.

Change save dir + clipboard default here without editing files by hand. CLI flags still win for a single run.

---

## Optional install

Skip unless you want `lightsnip` on your `PATH`. Default stays `cargo run` from the checkout.

1. `cargo build --release`
2. Optionally: `cp target/release/lightsnip ~/.local/bin/` (user PATH only, not `/usr`)
3. Then `lightsnip region` / `lightsnip full` / `lightsnip settings` / `lightsnip tray` work the same way

---

## Hotkeys (compositor binds)

Global shortcuts are compositor-dependent. lightsnip cannot guarantee grabs on every desktop.

1. Bind a key to `cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` (or `full`).
2. Or bind `lightsnip region` / `lightsnip full` only after optional install.
3. Or copy the command from Settings → Shortcuts.
4. Sample primary command: `cargo run --release -- region`.

| Desktop | Idea |
| --- | --- |
| Sway / Hyprland | Bind a key to `exec` the `cargo run … region` command |
| GNOME / KDE | Custom shortcut → same command |

Inside the annotate overlay: Enter = Save, Escape = clear draft / close Settings then Cancel, Ctrl+Z = Undo.

---

## Next

1. From the checkout on a Wayland desktop: `cargo run --release -- region`
2. Draw → Save → paste into another app (or open the printed PNG path)
3. `cargo run --release -- settings` → set save dir + clipboard once
4. Bind one compositor key to that same `cargo run …` command (optional install later if you want)
