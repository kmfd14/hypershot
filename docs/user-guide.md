# HyperShot: user guide

**Product:** HyperShot · **Crate / binary:** `hypershot` · **v1.4.0**

**Do this now** (from the repo checkout, graphical desktop):

```bash
cargo run --release -- region
```

1. Drag-select a region — live size text like `1280 x 720` updates while you drag (Esc cancels). Default is **on-screen** overlay.
2. Mark up with the bottom flat toolbar (Arrow, Highlighter, Line, Box, Pixelate, Step, Text, Magnify, color, thickness, Undo, Redo, Clear, Settings, Open in preview, Save, Copy, Exit). Only the **active** tool is purple. Ctrl+scroll adjusts stroke thickness.
3. **Save** → file dialog → close. **Enter** → file dialog + clipboard toast + close. **Copy** → toast + close. **Open in preview** opens the floating HyperShot window. Exit dismisses.

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

Pick one mode. Region is the default path. Run from the checkout. You do not need a system binary.

1. **Region** — `cargo run --release -- region` → full-screen capture, then on-screen overlay: drag-select with live `W x H`, annotate in place (Esc cancels).
2. **Fullscreen** — `cargo run --release -- full` → active output, then on-screen overlay with the full frame pre-selected.
3. Settings → **Annotate mode → Preview window** uses a separate region picker + floating HyperShot window instead.

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

1. Top toolbar strip with high-contrast icons. On-screen dock position (left / right / top / bottom) is set under Settings → User Interface and affects **on-screen** mode only.
2. Tools: **Arrow**, **Highlighter**, **Line**, **Box**, **Pixelate**, **Step number**, **Text**, **Magnify**, **color picker**, thickness readout, **Undo**, **Redo**, **Clear**, **Settings**, **Save**, **Copy**, **Exit**. Only the active draw tool gets the soft purple highlight.
3. Stroke color defaults to red. Click the swatch for the color wheel. Highlighter uses a translucent marker look. Pixelate ignores stroke color.
4. Drag on the image to draw. **Ctrl+scroll** adjusts thickness (HUD + toolbar readout). Ctrl+Z undoes; Ctrl+Shift+Z / Ctrl+Y redoes.
5. **Settings** opens the settings sheet.
6. **Save** opens a save-file dialog. Cancel leaves the window open.
7. **Enter** always runs save dialog + copy + toast + close.
8. **Copy** copies, shows a short toast, then closes.
9. **Exit** dismisses without save/copy (same as Escape when **Esc to exit** is on).
10. Re-open a PNG: `cargo run --release -- annotate ~/Pictures/hypershot/….png`.
11. Skip annotate: `cargo run --release -- region --no-annotate`.

**Display note:** Linux needs a graphical session. Headless VMs can `cargo build` but usually cannot open annotate / settings.

**Chrome:** light gray / white flat bar, charcoal icons, purple active-only highlight. Transparency / blur affect the preview; the **toolbar stays fully opaque**.

**Window placement:** default centers annotate and keeps it on top when the desktop allows. **Follow window tiling** skips those hints.

---

## Save and clipboard

After **Save** (or after capture when using `--no-annotate`), HyperShot writes a PNG and (by default) copies it to the clipboard.

1. **Default save** — settings save directory, else `~/Pictures/hypershot/hypershot-<unix>.png`.
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

Or click **Settings** on the annotate bar. Persist with **Save** inside the window. Config path: `~/.config/hypershot/config.toml`.

1. **Shortcuts** — **Assign…** stores a suggested key **label only**. Paste the bind command into your desktop keyboard settings. **Replace default screenshot** does not activate these keys.
2. **Session** — run at startup, replace-default launcher helper (does **not** steal Print Screen), stay in tray. All off by default. Section help lives on the question-mark control.
3. **Output** — save directory (text + **Browse…**) + save to clipboard by default.
4. **User Interface** — annotate mode; **on-screen** toolbar dock + theme; title bar; **follow window tiling**; **Esc to exit**; transparency and background blur (toolbar stays opaque). Preview window always docks tools at the top.
5. **About** — HyperShot + `x.y` version.

Change save dir + clipboard default here without editing files by hand. CLI flags still win for a single run.

---

## Optional install

Skip unless you want `hypershot` on your `PATH`. Default stays `cargo run` from the checkout.

1. `cargo build --release`
2. Optionally: `cp target/release/hypershot ~/.local/bin/` (user PATH only, not `/usr`)
3. Then `hypershot region` / `hypershot full` / `hypershot settings` / `hypershot tray`

---

## Hotkeys (desktop binds)

1. Copy the bind command from Settings → Shortcuts (or use `cargo run --release --manifest-path /path/to/HyperShot/Cargo.toml -- region`).
2. Paste it into your desktop keyboard settings.
3. After optional install, you can bind `hypershot region` / `hypershot full` instead.

| Desktop | Where | Sample |
| --- | --- | --- |
| Fedora COSMIC | Settings → Keyboard → Custom shortcuts | `cargo run --release --manifest-path /path/to/HyperShot/Cargo.toml -- region` |
| Hyprland | `hyprland.conf` | `bind = SUPER, Print, exec, cargo run --release --manifest-path /path/to/HyperShot/Cargo.toml -- region` |
| Sway | config | `bindsym $mod+Print exec cargo run --release --manifest-path /path/to/HyperShot/Cargo.toml -- region` |
| GNOME / KDE | Custom shortcut | Same command |

Annotate keys: Enter = save + copy + toast + close; Save = dialog + close; Copy = toast + close; Exit / Esc (when enabled) = cancel; Ctrl+Z / Ctrl+Shift+Z = undo / redo.
