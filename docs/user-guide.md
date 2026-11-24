# lightsnip: user guide

**Do this now** (from the repo checkout, on a Wayland desktop):

```bash
cargo run --release -- region
```

1. Select a region.
2. Mark up in the floating window: left pastel tool rail has icon tools (Arrow, Box, Pixelate), a stroke color picker (default red), Undo, Clear, Settings, Save, Copy.
3. Press **Save** / Enter, or **Copy** (always clipboard). Escape clears a draft drag, then cancels and discards.

PNG path prints to stdout. About 1–2 minutes for first capture after deps.

Open settings (~1 minute):

```bash
cargo run --release -- settings
```

No install. No copy into `/usr` or your PATH. Other screenshot apps stay untouched. Optional install, autostart, tray, and “replace default screenshot tool” are separate and off by default.

Floating annotate window (not fullscreen) with a left pastel tool rail and icon tools. Settings uses light grouped cards. Flat pastel purple / gray / white (no neon / glow).

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

After capture, a floating bordered window opens (sized to the image + left toolbar). You can also annotate a file you already have.

1. Left pastel tool rail (icons + hover tooltips), inset from the window on all sides: **Arrow**, **Box**, **Pixelate**, **color picker**, **Undo**, **Clear**, **Settings**, **Save**, **Copy**.
2. Stroke color defaults to red. Click the swatch for the color wheel; Arrow and Box use it. Pixelate ignores stroke color.
3. Drag on the image (right) to draw or redact. Ctrl+Z undoes.
4. **Settings** opens the settings sheet (same six fields as `lightsnip settings`).
5. **Save** / Enter writes the PNG and dismisses; clipboard follows settings / CLI flags.
6. **Copy** writes the PNG, copies to the clipboard (always), and dismisses.
7. Escape first drops an in-progress drag (or closes Settings); Escape again cancels without keeping the capture (region/full) or without changing the file (`annotate`).
8. Re-open a PNG: `cargo run --release -- annotate ~/Pictures/lightsnip/….png`.
9. Skip annotate: `cargo run --release -- region --no-annotate`.

**Display note:** annotate / settings need `WAYLAND_DISPLAY` or `DISPLAY` and egui native libs (for example `libxkbcommon`). Headless / cloud VMs can `cargo build` but usually cannot open those windows. On annotate failure after capture, lightsnip keeps the raw PNG and continues to clipboard/save.

**Chrome:** flat pastel (restrained purple / gray / white). egui on Wayland cannot do real liquid-glass backdrop-blur, so annotate uses opaque flat fills with dark icons on light wells (readable contrast, soft feathering).

**Stay on top:** annotate requests always-on-top through egui/winit. Wayland compositors often ignore or restrict this (no portable layer-shell API in this stack); X11 usually honors it. If the window goes under others, raise it from the task switcher.

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

1. **Shortcuts** — edit the suggested bind command. Wayland has no portable global hotkey API; bind that command in your compositor or DE.
2. **Run at startup** — opt-in XDG autostart → launches `tray`. Off by default.
3. **Replace default screenshot tool** — writes a user `.desktop` helper only. Does **not** steal PrintScreen on GNOME/KDE; assign a custom shortcut yourself. Off by default.
4. **Stay in taskbar / tray** — use StatusNotifierItem via `cargo run --release -- tray` (Region / Full / Settings / Quit). Needs a tray watcher; GNOME often needs an AppIndicator extension. Off by default.
5. **Save directory** — default PNG folder.
6. **Save to clipboard** — default on/off after Save.

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

| Desktop | Sample |
| --- | --- |
| Hyprland | `bind = SUPER, Print, exec, cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
| Sway | `bindsym $mod+Print exec cargo run --release --manifest-path /path/to/lightsnip/Cargo.toml -- region` |
| GNOME | Settings → Keyboard → Custom Shortcut → same command (does not steal PrintScreen by itself) |
| KDE | System Settings → Shortcuts → Custom → same command |

Inside annotate: Enter = Save, Copy = save + clipboard + dismiss, Escape = clear draft / close Settings then Cancel, Ctrl+Z = Undo.

---

## Next

1. From the checkout on a Wayland desktop: `cargo run --release -- region`
2. Draw → Save → paste into another app (or open the printed PNG path)
3. `cargo run --release -- settings` → set save dir + clipboard once
4. Bind one compositor key to that same `cargo run …` command (optional install later if you want)
