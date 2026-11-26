# HyperShot: user guide

**Product:** HyperShot · **Crate / binary:** `hypershot` · **v1.14.0**

**Do this now** (from the repo checkout, graphical desktop):

```bash
cargo run --release -- region
```

1. Drag-select a region — live size text like `1280 x 720` updates while you drag (Esc cancels). Default is **on-screen** overlay.
2. Mark up with the bottom flat toolbar (Arrow, Highlighter, Line, Box, Pixelate, Step, Text, Magnify, color, thickness, Undo, Redo, Clear, Settings, Open in preview, Save, Copy, Exit). Only the **active** tool is purple. Ctrl+scroll adjusts stroke thickness.
3. **Save** → file dialog → close. **Enter** → file dialog + copy + close. **Copy** → close immediately, then a **Copied to Clipboard!** popup. **Open in preview** opens the floating HyperShot window. Exit dismisses.

PNG path prints to stdout. About 1–2 minutes for first capture after deps.

Open settings (~1 minute):

```bash
cargo run --release -- settings
```

No install. Other screenshot apps stay untouched. Optional install, autostart, tray, and “replace default screenshot tool” are off by default.

Default annotate is an on-screen overlay (selection handles, resolution pill, bottom flat toolbar). Settings can switch to a floating preview window. Purple / gray / white — not neon / glow.

---

## Dependencies

Linux: portal first for fullscreen capture; `grim` / `slurp` fallback; `wl-clipboard` helps clipboard.
macOS: `screencapture`. Windows capture: WIP (annotate still works). See README platform matrix.

### Fedora 44 COSMIC — install now

```bash
sudo dnf install grim slurp wl-clipboard \
  xdg-desktop-portal xdg-desktop-portal-cosmic
```

Then: `cargo run --release -- region`.

### Runtime tools

| Need | Binary / package | Role |
| --- | --- | --- |
| Capture fallback | `grim` | PNG when portal fails |
| Region fallback | `slurp` | Legacy geometry helper |
| Clipboard | `wl-copy` (`wl-clipboard`) | Copy after Save |

### Portal backends

| Desktop | Fedora packages |
| --- | --- |
| COSMIC | `xdg-desktop-portal` + `xdg-desktop-portal-cosmic` |
| GNOME | `xdg-desktop-portal` + `xdg-desktop-portal-gnome` |
| KDE Plasma | `xdg-desktop-portal` + `xdg-desktop-portal-kde` |
| Sway / Hyprland | `xdg-desktop-portal` + `xdg-desktop-portal-wlr` |

COSMIC and tiling sessions often need `grim` + `slurp` even when the portal package is installed. Re-login after adding a portal backend if capture still fails. Full checklist: [README → Runtime dependencies](../README.md#runtime-dependencies).

---

## Capture

Run from the checkout. You do not need a system binary.

1. **Screenshot** — `cargo run --release -- capture` → respects Settings → Capture. With **Ask Screenshot Type** on (default), shows a blurred chooser: left = Region Select, right = Full Screen (Esc cancels). Off → **Default Mode** (Region or Full Screen).
2. **Region** — `cargo run --release -- region` → skips the chooser; on-screen overlay drag-select with live `W x H` (Esc cancels).
3. **Full Screen** — `cargo run --release -- full` → skips the chooser; opens the floating preview window.
4. Settings → **Annotate Mode** still chooses on-screen vs preview for Region. Full Screen always opens preview.

If you see `grim/slurp fallback unavailable`, install the missing tools from [Dependencies](#dependencies) and retry.

---

## Annotate

### On-screen (default)

After capture, HyperShot stays on a fullscreen overlay: selection rect with handles, resolution pill (`W x H`), and a bottom flat toolbar. Tools draw on the selected region in place.

1. Drag to select. Resize with the corner/edge dots. Hover the selection border to move it. Live size text updates while selecting.
2. Tools: **Arrow**, **Highlighter**, **Line**, **Box**, **Pixelate**, **Step**, **Text**, **Magnify**, **color**, thickness readout, **Undo**, **Redo**, **Clear**, **Settings**, **Open in preview**, **Save**, **Copy**, **Exit**. Only the active draw tool is purple. **Ctrl+scroll** changes stroke / marker / text size.
3. **Open in preview** crops the current session into the floating HyperShot window.

### Preview window

Settings → Annotate mode → Preview window (or after **Open in preview**). HyperShot window with a solid **top** toolbar (always embedded — dock position does not apply here).

1. Top toolbar strip with high-contrast icons. On-screen dock position (left / right / top / bottom) is set under Settings → Overlay and affects **on-screen** mode only.
2. Tools: **Arrow**, **Highlighter**, **Line**, **Box**, **Pixelate**, **Step number**, **Text**, **Magnify**, **Crop**, **color picker**, thickness readout, **Undo**, **Redo**, **Clear**, **Settings**, **Save**, **Copy**, **Exit**. Only the active draw tool gets the soft purple highlight.
3. Stroke color defaults to red. Click the swatch for the color wheel. Highlighter uses a translucent marker look. Pixelate ignores stroke color.
4. **Crop**: drag/adjust a rect, **Enter** applies (undoable). Esc clears the crop draft.
5. Drag on the image to draw. **Ctrl+scroll** adjusts thickness (HUD + toolbar readout). Ctrl+Z undoes; Ctrl+Shift+Z / Ctrl+Y redoes.
6. **Settings** opens the settings sheet.
7. **Save** opens a save-file dialog. Cancel leaves the window open.
8. **Enter** runs save dialog + copy + close (Copied popup), unless Crop is active with a rect.
9. **Copy** closes annotate immediately, then shows a designed Copied to Clipboard popup.
10. **Exit** dismisses without save/copy (same as Escape when **Esc to exit** is on).
11. Re-open a PNG: `cargo run --release -- annotate ~/Pictures/hypershot/….png`.
12. Skip annotate: `cargo run --release -- region --no-annotate`.

**Display note:** Linux needs a graphical session. Headless VMs can `cargo build` but usually cannot open annotate / settings.

**Chrome:** light gray / white flat bar, charcoal icons, purple active-only highlight. Transparency / blur affect the preview; the **toolbar stays fully opaque**.

**Window placement:** default centers annotate and keeps it on top when the desktop allows. **Follow window tiling** skips those hints.

---

## Save and clipboard

After **Save** (or after capture when using `--no-annotate`), HyperShot writes a PNG and (by default) copies it to the clipboard.

1. **Default save** — settings save directory + **File** pattern (default `screenshot_%d%m%y`) and file type (PNG / JPEG / WebP). Existing names get `_2`, `_3`, …
2. **Custom file** — `cargo run --release -- region -o ~/Pictures/snip.png`.
3. **Custom directory** — `cargo run --release -- full --save-dir ~/Pictures/Screenshots`.
4. **Env override** — set `HYPERSHOT_SAVE_DIR` (legacy `LIGHTSNIP_SAVE_DIR` still works).
5. **Clipboard** — default from settings. Skip with `--no-clipboard`. Affirm with `--clipboard`.

If clipboard fails with a warning, install `wl-clipboard` (`wl-copy`) — on Fedora: `sudo dnf install wl-clipboard` — and retry. Save still succeeds; the PNG path always prints after a successful Save.

---

## Settings

```bash
cargo run --release -- settings
```

Or click **Settings** on the annotate bar. Dark two-pane window: left sidebar, right content, footer with **Open config file** + **OK** / **Cancel** / **Apply**. Apply stays disabled until you change something. Config path: `~/.config/hypershot/config.toml`.

1. **General** — run at startup, replace-default launcher helper (does **not** steal Print Screen), stay in tray, theme. All helpers off by default.
2. **Capture** — Ask Screenshot Type, Default Mode, annotate mode (on-screen vs preview window).
3. **Overlay** — on-screen toolbar dock, title bar, follow window tiling (annotate only), Esc to exit.
4. **Annotation Window** — preview window transparency / blur (toolbar stays opaque).
5. **Saving** — save directory (**Browse…**) + clipboard default.
6. **File** — default save pattern (`screenshot_%d%m%y` and tokens `%Y` `%y` `%m` `%d` `%H` `%M` `%S`) plus default file type (PNG / JPEG / WebP). About includes a Changelog.
7. **About** — HyperShot + `x.y` version.

Settings uses a **fixed**, monitor-scaled window size (not tiled; height does not grow). Checkboxes sit in one left-aligned column. CLI flags still win for a single run.

---

## Optional install

Skip unless you want `hypershot` on your `PATH`. Default stays `cargo run` from the checkout.

1. `cargo build --release`
2. Optionally: `cp target/release/hypershot ~/.local/bin/` (user PATH only, not `/usr`)
3. Then `hypershot capture` / `hypershot region` / `hypershot full` / `hypershot settings` / `hypershot tray`

---

## Hotkeys (desktop binds)

1. Prefer `cargo run --release -- capture` (or built `hypershot capture`) for the Print-style shortcut.
2. Paste that command into your desktop keyboard settings.
3. After optional install, bind `hypershot capture` (or `region` / `full` to skip the chooser).

| Desktop | Where | Sample |
| --- | --- | --- |
| Fedora COSMIC | Settings → Keyboard → Custom shortcuts | `cargo run --release --manifest-path /path/to/HyperShot/Cargo.toml -- capture` |
| Hyprland | `hyprland.conf` | `bind = SUPER, Print, exec, cargo run --release --manifest-path /path/to/HyperShot/Cargo.toml -- capture` |
| Sway | config | `bindsym $mod+Print exec cargo run --release --manifest-path /path/to/HyperShot/Cargo.toml -- capture` |
| GNOME / KDE | Custom shortcut | Same command |

Annotate keys: Enter = save + copy + close (Copied popup), or apply Crop when Crop is active; Save = dialog + close; Copy = close + Copied popup; Exit / Esc (when enabled) = cancel; Ctrl+Z / Ctrl+Shift+Z = undo / redo (crop undoable). Preview toolbar includes **Crop**.
