# HyperShot: user guide

**Product name:** HyperShot · **Crate / binary / repo:** `lightsnip`

**Do this now** (from the repo checkout, graphical desktop):

```bash
cargo run --release -- region
```

1. Drag-select a region on the dimmed desktop — live size text like `1280 x 720` updates while you drag (Esc cancels). Default is Flameshot-style **on-screen** overlay.
2. Mark up in place with the bottom pill toolbar (Arrow, Box, Pixelate, Step, color, Undo, Redo, Clear, Settings, Open in preview, Save, Copy, Exit). Only the **active** tool is purple-highlighted.
3. **Save** → file dialog → close. **Enter** → file dialog + clipboard toast + close. **Copy** → toast + close. **Open in preview** hands the session to the floating HyperShot window. Exit dismisses.

PNG path prints to stdout. About 1–2 minutes for first capture after deps.

Open settings (~1 minute):

```bash
cargo run --release -- settings
```

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install, autostart, tray, and “replace default screenshot tool” are separate and off by default.

Default annotate is an on-screen fullscreen overlay (selection handles, resolution pill, bottom pill toolbar). Settings can switch to a floating preview window. Purple / gray / white — not neon / glow. Wayland: overlay is best-effort after a full capture; always-on-top may be ignored.

---

## Dependencies

Linux: portal first for fullscreen capture; `grim` / `slurp` fallback; `wl-clipboard` helps Wayland clipboard.
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
| Region fallback | `slurp` | Legacy geometry helper (region uses in-app picker after full capture) |
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
2. **Fullscreen** — `cargo run --release -- full` → active output via portal/`grim`, then on-screen overlay with the full frame pre-selected.
3. Settings → **Annotate mode → Preview window** uses a separate region picker + floating HyperShot window instead.

If you see `grim/slurp fallback unavailable`, install the missing tools from [Dependencies](#dependencies) and retry.

---

## Annotate

### On-screen (default)

After capture, HyperShot stays on a fullscreen overlay over the dimmed capture: selection rect with handles, resolution pill (`W x H` + origin), and a bottom pill toolbar. Tools draw on the selected region in place.

1. Drag to select. Resize with the corner/edge dots (they highlight on hover). Hover the selection border for a move cursor and drag to reposition. Live size text updates while selecting.
2. Tools: **Arrow**, **Box**, **Pixelate**, **Step**, **color**, **Undo**, **Redo**, **Clear**, **Settings**, **Open in preview**, **Save**, **Copy**, **Exit**. Only the active draw tool is purple-highlighted.
3. **Open in preview** crops the current session into the floating HyperShot window.

### Preview window

Settings → Annotate mode → Preview window (or after **Open in preview**). Floating bordered HyperShot window with a content-sized pill rail. You can also annotate a file you already have.

1. Floating pill icon rail (no “Annotate” text label), attached to the preview — not full window height. Default **left**; change to top / bottom / left / right in Settings → User Interface.
2. Tools: **Arrow**, **Box**, **Pixelate**, **Step number**, **color picker**, **Undo**, **Redo**, **Clear**, **Settings**, **Save**, **Copy**, **Exit**. Only the currently active draw tool gets the soft purple rounded-square highlight.
3. Stroke color defaults to red. Click the swatch for the color wheel; Arrow, Box, and Step use it. Pixelate ignores stroke color.
4. Drag on the image to draw or redact. While dragging Box / Pixelate, a live size label (`W x H`) appears near the selection. Pixelate uses a soft mosaic (`PIXELATE_BLOCK = 20`) so fine text is hard to read. Click with **Step** to stamp an incrementing circle (1, 2, …). Undo / Clear reset the step counter. Ctrl+Z undoes; Ctrl+Shift+Z / Ctrl+Y redoes.
5. **Settings** opens the settings sheet (shortcuts, session, output, User Interface).
6. **Save** opens a save-file dialog (defaults under the configured save dir). Cancel leaves the window open. Success writes and closes; clipboard follows settings / CLI flags.
7. **Enter** always runs save dialog + copy + toast + close (Esc-to-exit affects Esc only).
8. **Copy** writes the current work path, shows a short “Copied to clipboard” toast, then closes (always forces clipboard after close).
9. **Exit** dismisses without save/copy (same as Escape when **Esc to exit** is on). Escape first drops an in-progress drag (or closes Settings); Escape again cancels when the toggle is enabled.
10. Re-open a PNG: `cargo run --release -- annotate ~/Pictures/lightsnip/….png`.
11. Skip annotate: `cargo run --release -- region --no-annotate`.

**Display note:** Linux needs `WAYLAND_DISPLAY` or `DISPLAY` plus egui libs (for example `libxkbcommon`). Headless VMs can `cargo build` but usually cannot open those windows.

**Chrome:** light gray / white pill bar, charcoal icons, purple active-only highlight. Transparency / blur affect the window and preview area; the **toolbar stays fully opaque**. Background blur cannot sample the desktop in egui/Wayland.

**Window placement:** default floating mode centers annotate and requests always-on-top. **Follow window tiling** skips those hints. Wayland: winit always-on-top is a no-op; COSMIC may still tile. X11 usually honors always-on-top.

---

## Save and clipboard

After **Save** (or after capture when using `--no-annotate` / when the overlay cannot open), HyperShot writes a PNG and (by default) copies it to the system clipboard (`arboard`).

1. **Default save** — settings save directory, else `~/Pictures/lightsnip/lightsnip-<unix>.png` (directory is created if needed).
2. **Custom file** — `cargo run --release -- region -o ~/Pictures/snip.png`.
3. **Custom directory** — `cargo run --release -- full --save-dir ~/Pictures/Screenshots` (timestamped name inside that dir).
4. **Env override** — set `LIGHTSNIP_SAVE_DIR` to change the default directory without flags (beats config).
5. **Clipboard** — default from settings. Skip with `--no-clipboard`. Affirm with `--clipboard`.

If clipboard is skipped with a warning, install `wl-clipboard` (`wl-copy`) — on Fedora: `sudo dnf install wl-clipboard` — and retry. Save still succeeds either way; the PNG path is always printed after a successful Save.

---

## Settings

```bash
cargo run --release -- settings
```

Or click **Settings** on the annotate bar. Persist with **Save** inside the window. Config path: `~/.config/lightsnip/config.toml`.

1. **Shortcuts** — **Assign…** stores a suggested key **label only** (not a live global hotkey). Paste the bind command into your DE / compositor. Wayland has no portable global grab. **Replace default screenshot** does not activate these keys.
2. **Session** — toggles: run at startup (XDG → tray), replace-default `.desktop` helper (does **not** steal PrintScreen), stay in tray (StatusNotifierItem; GNOME often needs AppIndicator). All off by default. Section help lives on the `?` control.
3. **Output** — save directory (text + **Browse…** folder picker) + save to clipboard by default.
4. **User Interface** — **annotate mode** (on-screen / preview window); toolbar position and theme; title bar; **follow window tiling**; **Esc to exit**; transparency and background blur with intensity sliders (toolbar stays opaque). Wayland/egui: transparency is compositor-dependent; blur frosts the preview area (no true desktop backdrop-blur); on-screen overlay is best-effort fullscreen after capture.
5. **About** — HyperShot + `x.y` version (semver-lite: `x` major, `y` fixes/polish).

Change save dir + clipboard default here without editing files by hand. CLI flags still win for a single run.

---

## Optional install

Skip unless you want `lightsnip` on your `PATH`. Default stays `cargo run` from the checkout.

1. `cargo build --release`
2. Optionally: `cp target/release/lightsnip ~/.local/bin/` (user PATH only, not `/usr`)
3. Then `lightsnip region` / `lightsnip full` / `lightsnip settings` / `lightsnip tray` work the same way (UI still says HyperShot)

---

## Hotkeys (compositor binds)

1. Copy the bind command from Settings → Shortcuts (or use `cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region`).
2. Paste it into your DE / compositor shortcut settings.
3. After optional install, you can bind `lightsnip region` / `lightsnip full` instead.

| Desktop | Where | Sample |
| --- | --- | --- |
| Fedora COSMIC | Settings → Input Devices → Keyboard → Custom shortcuts | `cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
| Hyprland | `hyprland.conf` | `bind = SUPER, Print, exec, cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
| Sway | config | `bindsym $mod+Print exec cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
| GNOME / KDE | Custom shortcut | Same command |

Annotate keys: Enter = save + copy + toast + close; Save = dialog + close; Copy = toast + close; Exit / Esc (when enabled) = cancel; Ctrl+Z / Ctrl+Shift+Z = undo / redo.
