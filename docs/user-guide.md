# lightsnip: user guide

**Do this now** (from the repo checkout, on a Wayland desktop):

```bash
cargo run --release -- region
```

1. Select a region.
2. Mark up in the **Lightsnip** window: floating flat pastel rail labeled **Annotate** (Arrow, Box, Pixelate, color, Undo, Redo, Clear, Settings, Save, Copy).
3. Press **Save** / Enter, or **Copy** (toast, then clipboard + close). Escape clears a draft drag, then cancels and discards.

PNG path prints to stdout. About 1–2 minutes for first capture after deps.

Open settings (~1 minute):

```bash
cargo run --release -- settings
```

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install, autostart, tray, and “replace default screenshot tool” are separate and off by default.

Floating annotate window (not fullscreen) with a content-sized flat pastel rail (default left). Settings uses card sections with `?` tooltips, folder picker, intensity sliders, and About (name + version). Pastel purple / gray / white — not neon / glow, no frosted glass imitation.

---

## Dependencies

Portal first. `grim` / `slurp` when the portal fails. `wl-copy` for clipboard.

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
| Region fallback | `slurp` | Region geometry for `grim` |
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

1. **Region** — `cargo run --release -- region` → portal interactive picker, or `slurp` → `grim` if the portal fails.
2. **Fullscreen** — `cargo run --release -- full` → active output via portal, or `grim` fallback.
3. Escape / cancel in the picker aborts with a non-zero exit.

If you see `grim/slurp fallback unavailable`, install the missing tools from [Dependencies](#dependencies) and retry.

---

## Annotate

After capture, a floating bordered **Lightsnip** window opens (sized to the image + floating toolbar). You can also annotate a file you already have.

1. Floating flat pastel tool rail with an **Annotate** label (icons + hover tooltips), attached to the preview — not full window height. Default **left**; change to top / bottom / left / right in Settings → User Interface.
2. Tools: **Arrow**, **Box**, **Pixelate**, **Step number**, **color picker**, **Undo**, **Redo**, **Clear**, larger **Settings**, **Save**, **Copy**, **Exit**.
3. Stroke color defaults to red. Click the swatch for the color wheel; Arrow, Box, and Step use it. Pixelate ignores stroke color.
4. Drag on the image to draw or redact. Click with **Step** to stamp an incrementing circle (1, 2, …). Undo / Clear reset the step counter. Ctrl+Z undoes; Ctrl+Shift+Z / Ctrl+Y redoes. The preview letterboxes and can upscale small captures when you enlarge the window (LINEAR filtering).
5. **Settings** opens the settings sheet (shortcuts, session, output, User Interface).
6. **Save** / Enter writes the PNG and dismisses; clipboard follows settings / CLI flags.
7. **Copy** writes the PNG, shows a short “Copied to clipboard” toast, then closes cleanly (always forces clipboard after close).
8. **Exit** dismisses without save/copy (same as Escape when **Esc to exit** is on). Escape first drops an in-progress drag (or closes Settings); Escape again cancels when the toggle is enabled.
9. Re-open a PNG: `cargo run --release -- annotate ~/Pictures/lightsnip/….png`.
10. Skip annotate: `cargo run --release -- region --no-annotate`.

**Display note:** annotate / settings need `WAYLAND_DISPLAY` or `DISPLAY` and egui native libs (for example `libxkbcommon`). Headless / cloud VMs can `cargo build` but usually cannot open those windows. On annotate failure after capture, lightsnip keeps the raw PNG and continues to clipboard/save.

**Chrome:** flat pastel purple / gray / white (no specular / glow). Transparency / blur affect the window and preview area; the **toolbar stays fully opaque**. Background blur cannot sample the desktop in egui/Wayland.

**Window placement:** default floating mode centers annotate and requests always-on-top. **Follow window tiling** skips those hints. Wayland: winit always-on-top is a no-op; COSMIC may still tile. X11 usually honors always-on-top.

---

## Save and clipboard

After **Save** (or after capture when using `--no-annotate` / when the overlay cannot open), lightsnip writes a PNG and (by default) copies it to the Wayland clipboard.

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
4. **User Interface** — toolbar position and theme; title bar; **follow window tiling**; **Esc to exit**; transparency and background blur with intensity sliders (toolbar stays opaque). Wayland/egui: transparency is compositor-dependent; blur frosts the preview area (no true desktop backdrop-blur).
5. **About** — app name + `x.y` version (semver-lite: `x` major, `y` fixes/polish).

Change save dir + clipboard default here without editing files by hand. CLI flags still win for a single run.

---

## Optional install

Skip unless you want `lightsnip` on your `PATH`. Default stays `cargo run` from the checkout.

1. `cargo build --release`
2. Optionally: `cp target/release/lightsnip ~/.local/bin/` (user PATH only, not `/usr`)
3. Then `lightsnip region` / `lightsnip full` / `lightsnip settings` / `lightsnip tray` work the same way

---

## Hotkeys (compositor binds)

**Assign… is not a global hotkey.** It only remembers a label. Bind the command yourself.

**Replace default screenshot** only writes a `.desktop` helper. It does not steal PrintScreen and does not make Assign… live.

1. Copy the bind command from Settings → Shortcuts (or use `cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region`).
2. Add a custom shortcut in your desktop that runs that command.
3. After optional install, you can bind `lightsnip region` / `lightsnip full` instead.

| Desktop | Where | Sample |
| --- | --- | --- |
| Fedora COSMIC | Settings → Input Devices → Keyboard → Custom shortcuts | `cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
| Hyprland | `hyprland.conf` | `bind = SUPER, Print, exec, cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
| Sway | config | `bindsym $mod+Print exec cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
| GNOME | Settings → Keyboard → Custom Shortcut | Same command (does not steal PrintScreen by itself) |
| KDE | System Settings → Shortcuts → Custom | Same command |

Inside annotate: Enter = Save, Copy = save + clipboard + dismiss, **Exit** = cancel, Escape = same when Esc-to-exit is on, Ctrl+Z = Undo.

---

## Next

1. From the checkout on a Wayland desktop: `cargo run --release -- region`
2. Draw → Save → paste into another app (or open the printed PNG path)
3. `cargo run --release -- settings` → set save dir + clipboard once
4. Bind one compositor key to that same `cargo run …` command (optional install later if you want)
