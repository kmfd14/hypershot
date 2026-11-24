# lightsnip — user guide

**Do this now** (from the repo checkout):

```bash
cargo run --release -- region
```

Then open the printed PNG path (or paste — clipboard is on by default when `wl-copy` is installed). About 1 minute for first capture after deps (Milestone 2).

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install and “replace default screenshot tool” are separate and off by default.

Annotation and the settings window land in later milestones.

Quiet desktop UI planned for annotate/settings: small chrome, neutral surfaces. No decorative gradients or emoji controls.

---

## Capture (available now)

Pick one mode. Region is the default path. Run from the checkout — you do not need a system binary.

1. **Region** — `cargo run --release -- region` → portal interactive picker, or `slurp` → `grim` if the portal fails.
2. **Fullscreen** — `cargo run --release -- full` → active output via portal, or `grim` fallback.
3. Escape / cancel in the picker aborts with a non-zero exit.

Portal capture is preferred. If the portal path fails, lightsnip uses `grim` + `slurp` when installed.

---

## Save and clipboard (available now)

After capture, lightsnip writes a PNG and (by default) copies it to the Wayland clipboard.

1. **Default save** — `~/Pictures/lightsnip/lightsnip-<unix>.png` (directory is created if needed).
2. **Custom file** — `cargo run --release -- region -o ~/Pictures/snip.png`.
3. **Custom directory** — `cargo run --release -- full --save-dir ~/Pictures/Screenshots` (timestamped name inside that dir).
4. **Env override** — set `LIGHTSNIP_SAVE_DIR` to change the default directory without flags.
5. **Clipboard** — on by default via `wl-copy`. Skip with `--no-clipboard`. Affirm with `--clipboard`.

If clipboard is skipped with a warning, install `wl-clipboard` (`wl-copy`) and retry. Save still succeeds either way; the PNG path is always printed.

Settings UI for save dir / clipboard defaults comes later — flags and env cover it for now.

---

## Optional install

Skip unless you want `lightsnip` on your `PATH`. Default stays `cargo run` from the checkout.

1. `cargo build --release`
2. Optionally: `cp target/release/lightsnip ~/.local/bin/` (user PATH only — not `/usr`)
3. Then `lightsnip region` / `lightsnip full` work the same way

---

## Annotate (coming soon)

After capture, a later milestone opens a fullscreen overlay.

1. Choose a tool: arrow, rectangle/box, or blur/pixelate.
2. Draw on the image.
3. Undo or clear if you need a clean canvas.
4. Enter or Save commits the annotated PNG.
5. Escape cancels without saving that capture.

Blur/pixelate is for privacy redaction before share or copy.

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

---

## Settings (coming soon)

A later milestone adds a small settings control → settings window (~1 minute to set the basics).

1. **Shortcuts** — see docs; configure only what Wayland allows.
2. **Run at startup** — opt-in; off by default.
3. **Replace default screenshot tool** — off by default; enable only where the DE allows.
4. **Stay in taskbar / tray** — enable to keep a tray/status presence.
5. **Save directory** + **Save to clipboard** — set once; change anytime.

Until then: `--save-dir` / `-o` / `LIGHTSNIP_SAVE_DIR` and `--clipboard` / `--no-clipboard`.

Settings will persist in a TOML config file under your user config directory.

---

## Next

1. From the checkout: `cargo run --release -- region`
2. Paste into another app, or open the printed PNG path
3. Bind one compositor key to that same `cargo run …` command (optional install later if you want)
