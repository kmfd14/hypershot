# lightsnip — user guide

**Do this now:** run `lightsnip region`, then open the PNG path it prints.

About 1 minute for first capture (Milestone 1). Annotation, clipboard defaults, and the settings window land in later milestones.

Quiet desktop UI planned for annotate/settings: small chrome, neutral surfaces. No decorative gradients or emoji controls.

---

## Capture (available now)

Pick one mode. Region is the default path.

1. **Region** — `lightsnip region` → portal interactive picker, or `slurp` → `grim` if the portal fails.
2. **Fullscreen** — `lightsnip full` → active output via portal, or `grim` fallback.
3. Optional output path: `lightsnip region -o ~/Pictures/snip.png`.
4. The command prints the PNG path on success (default: `/tmp/lightsnip-<unix>.png`).
5. Escape / cancel in the picker aborts with a non-zero exit.

Portal capture is preferred. If the portal path fails, lightsnip uses `grim` + `slurp` when installed.

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

Until then: use the path printed by `lightsnip region` / `lightsnip full`.

---

## Hotkeys (compositor binds)

Global shortcuts are compositor-dependent. lightsnip cannot guarantee grabs on every desktop.

1. In your compositor or DE keyboard settings, bind a key to `lightsnip region`.
2. Optionally bind fullscreen the same way (`lightsnip full`).
3. Sample command to bind: `lightsnip region`.

| Desktop | Idea |
| --- | --- |
| Sway / Hyprland | Bind a key to `exec lightsnip region` |
| GNOME / KDE | Custom shortcut → command `lightsnip region` |

---

## Settings (coming soon)

A later milestone adds a small settings control → settings window (~1 minute to set the basics).

1. **Shortcuts** — see docs; configure only what Wayland allows.
2. **Run at startup** — enable if you want lightsnip available after login.
3. **Replace default screenshot tool** — enable only where the DE allows; otherwise ignore.
4. **Stay in taskbar / tray** — enable to keep a tray/status presence.
5. **Save directory** + **Save to clipboard** — set once; change anytime.

Settings will persist in a TOML config file under your user config directory.

---

## Next

1. Build: `cargo build --release`
2. Run: `lightsnip region`
3. Bind one compositor key to `lightsnip region`
