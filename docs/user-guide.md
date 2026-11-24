# lightsnip — user guide

**Do this now** (from the repo checkout, on a Wayland desktop):

```bash
cargo run --release -- region
```

1. Select a region.
2. Mark up with Arrow or Box.
3. Press **Save** or Enter (Escape cancels and discards).

PNG path prints to stdout. Clipboard is on by default when `wl-copy` is installed. About 1–2 minutes for first capture after deps (Milestone 3).

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install and “replace default screenshot tool” are separate and off by default.

Quiet annotate chrome: charcoal bar, small buttons, no decorative gradients or emoji controls. Blur/pixelate and the settings window land later.

---

## Capture (available now)

Pick one mode. Region is the default path. Run from the checkout — you do not need a system binary.

1. **Region** — `cargo run --release -- region` → portal interactive picker, or `slurp` → `grim` if the portal fails.
2. **Fullscreen** — `cargo run --release -- full` → active output via portal, or `grim` fallback.
3. Escape / cancel in the picker aborts with a non-zero exit.

Portal capture is preferred. If the portal path fails, lightsnip uses `grim` + `slurp` when installed.

---

## Annotate (available now)

After capture, a fullscreen overlay opens on the PNG. You can also annotate a file you already have.

1. Tools: **Arrow**, **Box** (rectangle).
2. Drag on the image to draw. **Undo** (or Ctrl+Z) / **Clear** as needed.
3. **Save** or Enter writes the annotated PNG, then clipboard (unless `--no-clipboard`).
4. **Cancel** or Escape exits without keeping the capture (region/full) or without changing the file (`annotate`).
5. Re-open a PNG: `cargo run --release -- annotate ~/Pictures/lightsnip/….png`.
6. Skip the overlay: `cargo run --release -- region --no-annotate`.

Blur / pixelate for privacy redaction is Milestone 4.

**Display note:** annotate needs `WAYLAND_DISPLAY` or `DISPLAY` and egui native libs (for example `libxkbcommon`). Headless / cloud VMs can `cargo build` but usually cannot open the overlay. On failure after capture, lightsnip keeps the raw PNG and continues to clipboard/save.

---

## Save and clipboard (available now)

After **Save** (or after capture when using `--no-annotate` / when the overlay cannot open), lightsnip writes a PNG and (by default) copies it to the Wayland clipboard.

1. **Default save** — `~/Pictures/lightsnip/lightsnip-<unix>.png` (directory is created if needed).
2. **Custom file** — `cargo run --release -- region -o ~/Pictures/snip.png`.
3. **Custom directory** — `cargo run --release -- full --save-dir ~/Pictures/Screenshots` (timestamped name inside that dir).
4. **Env override** — set `LIGHTSNIP_SAVE_DIR` to change the default directory without flags.
5. **Clipboard** — on by default via `wl-copy`. Skip with `--no-clipboard`. Affirm with `--clipboard`.

If clipboard is skipped with a warning, install `wl-clipboard` (`wl-copy`) and retry. Save still succeeds either way; the PNG path is always printed after a successful Save.

Settings UI for save dir / clipboard defaults comes later — flags and env cover it for now.

---

## Optional install

Skip unless you want `lightsnip` on your `PATH`. Default stays `cargo run` from the checkout.

1. `cargo build --release`
2. Optionally: `cp target/release/lightsnip ~/.local/bin/` (user PATH only — not `/usr`)
3. Then `lightsnip region` / `lightsnip full` / `lightsnip annotate …` work the same way

---

## Hotkeys (compositor binds)

Global shortcuts are compositor-dependent. lightsnip cannot guarantee grabs on every desktop.

1. Bind a key to `cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` (or `full`).
2. Or bind `lightsnip region` / `lightsnip full` only after optional install.
3. Sample primary command: `cargo run --release -- region`.

| Desktop | Idea |
| --- | --- |
| Sway / Hyprland | Bind a key to `exec` the `cargo run … region` command |
| GNOME / KDE | Custom shortcut → same command |

Inside the annotate overlay: Enter = Save, Escape = Cancel, Ctrl+Z = Undo.

---

## Settings (coming soon)

A later milestone adds a small settings control → settings window (~1 minute to set the basics).

1. **Shortcuts** — see docs; configure only what Wayland allows.
2. **Run at startup** — opt-in; off by default.
3. **Replace default screenshot tool** — off by default; enable only where the DE allows.
4. **Stay in taskbar / tray** — enable to keep a tray/status presence.
5. **Save directory** + **Save to clipboard** — set once; change anytime.

Until then: `--save-dir` / `-o` / `LIGHTSNIP_SAVE_DIR`, `--clipboard` / `--no-clipboard`, and `--no-annotate`.

Settings will persist in a TOML config file under your user config directory.

---

## Next

1. From the checkout on a Wayland desktop: `cargo run --release -- region`
2. Draw → Save → paste into another app (or open the printed PNG path)
3. Bind one compositor key to that same `cargo run …` command (optional install later if you want)
