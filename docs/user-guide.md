# lightsnip — user guide

**Do this now** (from the repo checkout):

```bash
cargo run --release -- region
```

Then open the PNG path it prints. About 1 minute for first capture after deps (Milestone 1).

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install and “replace default screenshot tool” are separate and off by default.

Annotation, clipboard defaults, and the settings window land in later milestones.

Quiet desktop UI planned for annotate/settings: small chrome, neutral surfaces. No decorative gradients or emoji controls.

---

## Capture (available now)

Pick one mode. Region is the default path. Run from the checkout — you do not need a system binary.

1. **Region** — `cargo run --release -- region` → portal interactive picker, or `slurp` → `grim` if the portal fails.
2. **Fullscreen** — `cargo run --release -- full` → active output via portal, or `grim` fallback.
3. Optional output path: `cargo run --release -- region -o ~/Pictures/snip.png`.
4. The command prints the PNG path on success (default: `/tmp/lightsnip-<unix>.png`).
5. Escape / cancel in the picker aborts with a non-zero exit.

Portal capture is preferred. If the portal path fails, lightsnip uses `grim` + `slurp` when installed.

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

## Save and clipboard (coming soon)

1. Open **Settings** → set **Save directory** (timestamped PNG names).
2. Set **Save to clipboard** to your preference (default on/off or always-copy).
3. After commit, check the save folder and/or paste into another app.
4. Clipboard uses the portal when available; otherwise `wl-copy` (`wl-clipboard`).
5. If nothing lands in the clipboard, install `wl-clipboard` or fix portal clipboard permissions, then retry.

Until then: use the path printed by `cargo run --release -- region` / `cargo run --release -- full`.

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

Settings will persist in a TOML config file under your user config directory.

---

## Next

1. From the checkout: `cargo run --release -- region`
2. Open the printed PNG path
3. Bind one compositor key to that same `cargo run …` command (optional install later if you want)
