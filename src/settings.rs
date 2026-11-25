//! Settings window: grouped cards, `?` tooltips, clearer controls.
//!
//! Shortcuts: click “Assign…”, then type a combination. That stores a *label*
//! and suggested bind command only — lightsnip does **not** register a global
//! hotkey on Wayland. Bind the command in your compositor / DE (COSMIC,
//! GNOME, Hyprland, …). “Replace default screenshot” only installs a user
//! `.desktop` helper; it does not make Assign… live and does not steal Print.
//! User Interface: toolbar position, theme, title bar, tiling vs floating,
//! Esc-to-exit, transparency / blur intensity (honest about egui/Wayland
//! limits). About shows name + version.

use std::path::PathBuf;

use anyhow::{bail, Result};
use eframe::egui::{self, Key, Modifiers, ViewportBuilder};

use crate::config::{
    clamp_intensity, config_path, AnnotateSessionMode, Config, ToolbarPosition, UiTheme, APP_NAME,
    APP_VERSION,
};
use crate::desktop::{apply_desktop_integrations, maybe_spawn_tray};
use crate::ui::{
    self, choice_button, contrast_checkbox, section_title_with_help, settings_button,
    settings_group_frame, settings_text_edit, SETTINGS_BG, SETTINGS_BG_DARK,
    SETTINGS_CONTROL_HEIGHT, SETTINGS_FIELD, SETTINGS_FIELD_DARK, SETTINGS_LABEL,
    SETTINGS_LABEL_DARK, SETTINGS_PRIMARY, SETTINGS_SECONDARY, SETTINGS_SECONDARY_DARK,
    SETTINGS_SEPARATOR, SETTINGS_SEPARATOR_DARK,
};

/// Open the standalone settings window. Blocks until closed.
pub fn run_settings_window() -> Result<()> {
    ensure_display_available()?;

    let cfg = Config::load().unwrap_or_default();
    let theme = cfg.theme;
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(format!("{APP_NAME} Settings"))
            .with_inner_size([520.0, 560.0])
            .with_min_inner_size([440.0, 420.0])
            .with_maximize_button(false),
        centered: true,
        ..Default::default()
    };

    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            &format!("{APP_NAME} Settings"),
            options,
            Box::new(move |cc| {
                ui::apply_settings_visuals(&cc.egui_ctx, theme);
                Ok(Box::new(SettingsApp::new(cfg)))
            }),
        )
    }));

    match run_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => bail!("settings window failed (need a display?): {e}"),
        Err(_) => bail!(
            "settings window panicked (missing display libs such as libxkbcommon?)"
        ),
    }
}

fn ensure_display_available() -> Result<()> {
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let x11 = std::env::var_os("DISPLAY").is_some();
    if !wayland && !x11 {
        bail!("no WAYLAND_DISPLAY or DISPLAY set (settings needs a graphical session)");
    }
    Ok(())
}

struct SettingsApp {
    cfg: Config,
    save_dir_text: String,
    shortcut_text: String,
    status: String,
    close_requested: bool,
    /// Listening for a key combination after “Assign…”.
    capturing_shortcut: bool,
}

impl SettingsApp {
    fn new(cfg: Config) -> Self {
        let save_dir_text = cfg.save_directory.display().to_string();
        let shortcut_text = cfg.shortcut_bind_command.clone();
        Self {
            cfg,
            save_dir_text,
            shortcut_text,
            status: config_path()
                .map(|p| format!("Config: {}", p.display()))
                .unwrap_or_else(|_| "Config path unavailable".into()),
            close_requested: false,
            capturing_shortcut: false,
        }
    }

    fn persist(&mut self) {
        self.cfg.save_directory = PathBuf::from(self.save_dir_text.trim());
        self.cfg.shortcut_bind_command = self.shortcut_text.trim().to_string();
        if self.cfg.shortcut_bind_command.is_empty() {
            self.cfg.shortcut_bind_command = crate::config::default_shortcut_bind_command();
            self.shortcut_text = self.cfg.shortcut_bind_command.clone();
        }
        if self.cfg.shortcut_keys.trim().is_empty() {
            self.cfg.shortcut_keys = crate::config::default_shortcut_keys();
        }
        self.cfg.window_transparency_intensity =
            clamp_intensity(self.cfg.window_transparency_intensity);
        self.cfg.window_background_blur_intensity =
            clamp_intensity(self.cfg.window_background_blur_intensity);

        match self.cfg.save() {
            Ok(path) => {
                let mut notes = vec![format!("Saved {}", path.display())];
                match apply_desktop_integrations(&self.cfg) {
                    Ok(extra) => notes.extend(extra),
                    Err(err) => notes.push(format!("Desktop helpers: {err:#}")),
                }
                if let Err(err) = maybe_spawn_tray(&self.cfg) {
                    notes.push(format!("Tray: {err:#}"));
                }
                self.status = notes.join(" · ");
            }
            Err(err) => {
                self.status = format!("Save failed: {err:#}");
            }
        }
    }

    fn pick_save_directory(&mut self) {
        let start = PathBuf::from(self.save_dir_text.trim());
        let mut dialog = rfd::FileDialog::new().set_title("Choose save directory");
        if start.is_dir() {
            dialog = dialog.set_directory(&start);
        } else if let Some(parent) = start.parent().filter(|p| p.is_dir()) {
            dialog = dialog.set_directory(parent);
        }
        if let Some(folder) = dialog.pick_folder() {
            self.save_dir_text = folder.display().to_string();
            self.cfg.save_directory = folder;
        }
    }

    fn poll_shortcut_capture(&mut self, ui: &mut egui::Ui) {
        if !self.capturing_shortcut {
            return;
        }
        ui.ctx().input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    if matches!(key, Key::Escape) {
                        self.capturing_shortcut = false;
                        return;
                    }
                    if is_modifier_only_key(*key) {
                        continue;
                    }
                    self.cfg.shortcut_keys = format_key_combo(*modifiers, *key);
                    self.capturing_shortcut = false;
                    return;
                }
            }
        });
    }

    fn draw_form(&mut self, ui: &mut egui::Ui, compact: bool) {
        let dark = ui.visuals().dark_mode;
        let label = if dark {
            SETTINGS_LABEL_DARK
        } else {
            SETTINGS_LABEL
        };
        let secondary = if dark {
            SETTINGS_SECONDARY_DARK
        } else {
            SETTINGS_SECONDARY
        };
        let field = if dark {
            SETTINGS_FIELD_DARK
        } else {
            SETTINGS_FIELD
        };
        let sep = if dark {
            SETTINGS_SEPARATOR_DARK
        } else {
            SETTINGS_SEPARATOR
        };

        ui::apply_settings_visuals(ui.ctx(), self.cfg.theme);

        // Content column: justified width, left-aligned controls.
        let max_w = ui.available_width().min(480.0);
        ui.set_max_width(max_w);

        if !compact {
            ui.label(
                egui::RichText::new("Settings")
                    .color(label)
                    .size(20.0)
                    .strong(),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Capture, annotate, and session defaults.")
                    .color(secondary)
                    .size(12.0),
            );
            ui.add_space(12.0);
        }

        self.poll_shortcut_capture(ui);

        // --- Shortcuts -------------------------------------------------------
        settings_group_frame(dark).show(ui, |ui| {
            section_title_with_help(
                ui,
                "Shortcuts",
                "Assign… only stores a suggested key label — it does not register a global hotkey. Wayland has no portable global grab. Copy the bind command below into your compositor or DE keyboard settings (Fedora COSMIC: Settings → Input Devices → Keyboard → Custom shortcuts). Replace default screenshot is unrelated: it only installs a user .desktop helper and never steals PrintScreen.",
                label,
                secondary,
            );

            ui.label(
                egui::RichText::new(
                    "Not a live global hotkey. Bind the command in your desktop.",
                )
                .color(SETTINGS_PRIMARY)
                .size(11.0)
                .strong(),
            );
            ui.add_space(6.0);

            ui.label(egui::RichText::new("Suggested key (label only)").color(label).size(12.0));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let chip_text = if self.capturing_shortcut {
                    "Listening… Esc to cancel".to_string()
                } else {
                    self.cfg.shortcut_keys.clone()
                };
                let chip_color = if self.capturing_shortcut {
                    SETTINGS_PRIMARY
                } else {
                    label
                };
                egui::Frame::new()
                    .fill(field)
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        if self.capturing_shortcut {
                            SETTINGS_PRIMARY
                        } else {
                            sep
                        },
                    ))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        ui.set_min_height(SETTINGS_CONTROL_HEIGHT - 8.0);
                        ui.label(
                            egui::RichText::new(chip_text)
                                .color(chip_color)
                                .size(13.0)
                                .strong()
                                .monospace(),
                        );
                    });

                let assign_label = if self.capturing_shortcut {
                    "Listening…"
                } else {
                    "Assign…"
                };
                if settings_button(
                    ui,
                    egui::RichText::new(assign_label)
                        .color(egui::Color32::WHITE)
                        .size(12.0),
                    Some(SETTINGS_PRIMARY),
                    96.0,
                )
                .on_hover_text(
                    "Stores a label for your notes. Does not grab the key system-wide.",
                )
                .clicked()
                {
                    self.capturing_shortcut = !self.capturing_shortcut;
                }
                if settings_button(ui, "Reset", None, 64.0).clicked() {
                    self.cfg.shortcut_keys = crate::config::default_shortcut_keys();
                    self.capturing_shortcut = false;
                }
            });
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Bind command (paste into DE / compositor)").color(label).size(12.0));
            settings_text_edit(
                ui,
                &mut self.shortcut_text,
                f32::INFINITY,
                "path/to/lightsnip region",
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(
                    "COSMIC: Settings → Input Devices → Keyboard → Custom shortcuts → add this command.",
                )
                .color(secondary)
                .size(10.5),
            );
        });
        ui.add_space(10.0);

        // --- Session ---------------------------------------------------------
        settings_group_frame(dark).show(ui, |ui| {
            section_title_with_help(
                ui,
                "Session",
                "Optional desktop helpers. All off by default. Autostart launches the tray; replace-default installs a user .desktop helper (does not steal PrintScreen); tray needs a StatusNotifier watcher.",
                label,
                secondary,
            );
            session_row(
                ui,
                &mut self.cfg.run_at_startup,
                "Run at startup",
                "XDG autostart → tray.",
                label,
                secondary,
            );
            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);
            session_row(
                ui,
                &mut self.cfg.replace_default_screenshot,
                "Replace default screenshot",
                "User .desktop helper only — does not bind keys or steal PrintScreen.",
                label,
                secondary,
            );
            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);
            session_row(
                ui,
                &mut self.cfg.stay_in_tray,
                "Stay in tray",
                "StatusNotifierItem. GNOME often needs AppIndicator.",
                label,
                secondary,
            );
        });
        ui.add_space(10.0);

        // --- Output ----------------------------------------------------------
        settings_group_frame(dark).show(ui, |ui| {
            section_title_with_help(
                ui,
                "Output",
                "Default PNG folder and clipboard behavior. CLI --save-dir / LIGHTSNIP_SAVE_DIR and --clipboard / --no-clipboard still override for a single run.",
                label,
                secondary,
            );
            ui.label(egui::RichText::new("Save directory").color(label).size(12.0));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                settings_text_edit(
                    ui,
                    &mut self.save_dir_text,
                    ui.available_width() - 108.0,
                    "~/Pictures/lightsnip",
                );
                if settings_button(ui, "Browse…", None, 96.0)
                    .on_hover_text("Choose preferred folder")
                    .clicked()
                {
                    self.pick_save_directory();
                }
            });
            ui.add_space(6.0);
            session_row(
                ui,
                &mut self.cfg.save_to_clipboard,
                "Save to clipboard by default",
                "When neither --clipboard nor --no-clipboard is passed.",
                label,
                secondary,
            );
        });
        ui.add_space(10.0);

        // --- User Interface --------------------------------------------------
        settings_group_frame(dark).show(ui, |ui| {
            section_title_with_help(
                ui,
                "User Interface",
                "Annotate mode (on-screen overlay vs preview window), toolbar position, theme, title bar, tiling vs floating, Esc-to-exit, and window effects. On-screen is Flameshot-style at the capture; Wayland overlay is best-effort. Transparency / blur affect the preview window; the tool rail stays opaque.",
                label,
                secondary,
            );

            ui.label(egui::RichText::new("Annotate mode").color(label).size(12.0));
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                for mode in AnnotateSessionMode::ALL {
                    if choice_button(ui, self.cfg.annotate_session_mode == mode, mode.label())
                        .clicked()
                    {
                        self.cfg.annotate_session_mode = mode;
                    }
                }
            });
            ui.label(
                egui::RichText::new(
                    "On-screen (default): Flameshot-style overlay on the capture. Preview window: floating HyperShot annotate. Wayland: fullscreen overlay is best-effort after a full capture; compositors may ignore always-on-top.",
                )
                .color(secondary)
                .size(10.5)
                .italics(),
            );
            ui.add_space(8.0);

            ui.label(egui::RichText::new("Toolbar position").color(label).size(12.0));
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                for pos in ToolbarPosition::ALL {
                    if choice_button(ui, self.cfg.toolbar_position == pos, pos.label()).clicked() {
                        self.cfg.toolbar_position = pos;
                    }
                }
            });
            ui.add_space(8.0);

            ui.label(egui::RichText::new("Theme").color(label).size(12.0));
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                for theme in UiTheme::ALL {
                    if choice_button(ui, self.cfg.theme == theme, theme.label()).clicked() {
                        self.cfg.theme = theme;
                    }
                }
            });
            ui.add_space(8.0);

            session_row(
                ui,
                &mut self.cfg.show_title_bar,
                "Show window title bar",
                "Native decorations. Off → borderless (Wayland support varies).",
                label,
                secondary,
            );
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(6.0);

            session_row(
                ui,
                &mut self.cfg.follow_window_tiling,
                "Follow window tiling",
                "On: skip center + always-on-top so a tiling WM can place annotate. Off (default): center + always-on-top. Wayland/COSMIC may still ignore always-on-top.",
                label,
                secondary,
            );
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(6.0);

            session_row(
                ui,
                &mut self.cfg.esc_to_exit,
                "Esc to exit",
                "When on (default), Escape dismisses annotate without save/copy (after clearing a draft). Exit button always works.",
                label,
                secondary,
            );
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(6.0);

            session_row(
                ui,
                &mut self.cfg.window_transparency,
                "Window transparency",
                "Applies window clear alpha + translucent preview area. Toolbar stays opaque. Many Wayland compositors ignore alpha.",
                label,
                secondary,
            );
            ui.add_space(4.0);
            ui.add_enabled_ui(self.cfg.window_transparency, |ui| {
                intensity_slider(
                    ui,
                    "Intensity",
                    &mut self.cfg.window_transparency_intensity,
                    label,
                    secondary,
                );
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(6.0);

            session_row(
                ui,
                &mut self.cfg.window_background_blur,
                "Window background blur",
                "No true desktop blur in egui/Wayland. Intensity frosts the preview area; toolbar stays a solid pastel panel.",
                label,
                secondary,
            );
            ui.add_space(4.0);
            ui.add_enabled_ui(self.cfg.window_background_blur, |ui| {
                intensity_slider(
                    ui,
                    "Intensity",
                    &mut self.cfg.window_background_blur_intensity,
                    label,
                    secondary,
                );
            });
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(
                    "Wayland / egui: transparency is compositor-dependent; blur is a frosted panel, not backdrop-blur. Always-on-top is a no-op on Wayland (winit).",
                )
                .color(secondary)
                .size(10.5)
                .italics(),
            );
        });
        ui.add_space(10.0);

        // --- About -----------------------------------------------------------
        settings_group_frame(dark).show(ui, |ui| {
            section_title_with_help(
                ui,
                "About",
                "App identity. Version uses x.y semver-lite (x = major updates, y = fixes / polish).",
                label,
                secondary,
            );
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(APP_NAME)
                        .color(label)
                        .size(15.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("v{APP_VERSION}"))
                        .color(secondary)
                        .size(13.0),
                );
            });
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Wayland screenshot tool with annotation.")
                    .color(secondary)
                    .size(11.0),
            );
        });

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            if settings_button(
                ui,
                egui::RichText::new("Save")
                    .color(egui::Color32::WHITE)
                    .size(13.0),
                Some(SETTINGS_PRIMARY),
                88.0,
            )
            .clicked()
            {
                self.persist();
            }
            if !compact && settings_button(ui, "Close", None, 72.0).clicked() {
                self.close_requested = true;
            }
        });
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(&self.status)
                .color(secondary)
                .size(11.0),
        );
    }
}

fn is_modifier_only_key(_key: Key) -> bool {
    // Modifiers arrive via `Modifiers`; egui has no standalone Ctrl/Cmd Key variants.
    false
}

fn format_key_combo(modifiers: Modifiers, key: Key) -> String {
    let mut parts = Vec::new();
    if modifiers.mac_cmd || modifiers.command {
        parts.push("Super");
    }
    if modifiers.ctrl && !modifiers.command {
        parts.push("Ctrl");
    }
    if modifiers.alt {
        parts.push("Alt");
    }
    if modifiers.shift {
        parts.push("Shift");
    }
    parts.push(key_name(key));
    parts.join("+")
}

fn key_name(key: Key) -> &'static str {
    match key {
        Key::Space => "Space",
        Key::Enter => "Enter",
        Key::Tab => "Tab",
        Key::Backspace => "Backspace",
        Key::Delete => "Delete",
        Key::ArrowUp => "Up",
        Key::ArrowDown => "Down",
        Key::ArrowLeft => "Left",
        Key::ArrowRight => "Right",
        Key::Escape => "Esc",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::A => "A",
        Key::B => "B",
        Key::C => "C",
        Key::D => "D",
        Key::E => "E",
        Key::F => "F",
        Key::G => "G",
        Key::H => "H",
        Key::I => "I",
        Key::J => "J",
        Key::K => "K",
        Key::L => "L",
        Key::M => "M",
        Key::N => "N",
        Key::O => "O",
        Key::P => "P",
        Key::Q => "Q",
        Key::R => "R",
        Key::S => "S",
        Key::T => "T",
        Key::U => "U",
        Key::V => "V",
        Key::W => "W",
        Key::X => "X",
        Key::Y => "Y",
        Key::Z => "Z",
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        // egui 0.31 has no PrintScreen key; default label stays "Print" in config.
        _ => "Key",
    }
}

fn intensity_slider(
    ui: &mut egui::Ui,
    caption: &str,
    value: &mut f32,
    label: egui::Color32,
    secondary: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(caption).color(label).size(12.0));
        ui.spacing_mut().slider_width = 160.0;
        // egui paints the rail with `inactive.bg_fill`. When Settings is embedded
        // in on-screen annotate, the Ui can still carry annotate visuals where
        // that fill is transparent — force a high-contrast track locally.
        let dark = ui.visuals().dark_mode;
        let track = if dark {
            egui::Color32::from_rgb(0x5C, 0x5C, 0x60)
        } else {
            egui::Color32::from_rgb(0xC7, 0xC7, 0xCC)
        };
        let track_hover = if dark {
            egui::Color32::from_rgb(0x6E, 0x6E, 0x72)
        } else {
            egui::Color32::from_rgb(0xB8, 0xB8, 0xBE)
        };
        {
            let v = ui.visuals_mut();
            v.widgets.inactive.bg_fill = track;
            v.widgets.inactive.weak_bg_fill = track;
            v.widgets.hovered.bg_fill = track_hover;
            v.widgets.hovered.weak_bg_fill = track_hover;
            v.widgets.active.bg_fill = track_hover;
            v.widgets.active.weak_bg_fill = track_hover;
            v.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(0x00, 0x7A, 0xFF, 0x99);
            v.slider_trailing_fill = true;
        }
        let resp = ui.add(
            egui::Slider::new(value, 0.0..=1.0)
                .show_value(false)
                .trailing_fill(true),
        );
        if resp.changed() {
            *value = clamp_intensity(*value);
        }
        ui.label(
            egui::RichText::new(format!("{:.0}%", *value * 100.0))
                .color(secondary)
                .size(12.0)
                .monospace(),
        );
    });
}

fn session_row(
    ui: &mut egui::Ui,
    value: &mut bool,
    title: &str,
    hint: &str,
    label: egui::Color32,
    secondary: egui::Color32,
) {
    // Card row: label + wrapped description on the left, checkbox column on the
    // right. Fixed trailing width keeps every checkbox in one vertical column
    // and stops long hints from overlapping the control.
    const CHECK_COL: f32 = 36.0;
    const GAP: f32 = 12.0;
    let row_w = ui.available_width();
    let text_w = (row_w - CHECK_COL - GAP).max(120.0);
    ui.horizontal(|ui| {
        ui.set_min_height(SETTINGS_CONTROL_HEIGHT.max(40.0));
        ui.allocate_ui_with_layout(
            egui::vec2(text_w, ui.available_height().max(40.0)),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.label(egui::RichText::new(title).color(label).size(12.5));
                ui.add(
                    egui::Label::new(egui::RichText::new(hint).color(secondary).size(10.5))
                        .wrap_mode(egui::TextWrapMode::Wrap),
                );
            },
        );
        ui.add_space(GAP);
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.set_min_width(CHECK_COL);
            ui.set_max_width(CHECK_COL);
            // Vertically center the checkbox against the title line.
            ui.add_space(2.0);
            contrast_checkbox(ui, value);
        });
    });
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.close_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) && !self.capturing_shortcut {
                self.close_requested = true;
            }
        });

        let dark = ctx.theme() == egui::Theme::Dark;
        let bg = if dark { SETTINGS_BG_DARK } else { SETTINGS_BG };

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg).inner_margin(16.0))
            .show(ctx, |ui| {
                // ScrollArea with a slightly shorter default viewport makes
                // overflow (and the scrollbar) obvious for the card stack.
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.with_layout(
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.with_layout(
                                    egui::Layout::top_down(egui::Align::Min).with_cross_justify(true),
                                    |ui| {
                                        self.draw_form(ui, false);
                                    },
                                );
                            },
                        );
                    });
            });
    }
}

/// Draw the settings form inside an egui window (annotate toolbar gear).
pub fn show_settings_window(ctx: &egui::Context, open: &mut bool, state: &mut EmbeddedSettings) {
    if !*open {
        return;
    }
    let theme = state.config().theme;
    // Install settings widget colors *before* the Window Ui is created so
    // slider rails inherit opaque tracks (not annotate's transparent idle fill).
    ui::apply_settings_visuals(ctx, theme);
    let dark = ctx.theme() == egui::Theme::Dark;
    let bg = if dark { SETTINGS_BG_DARK } else { SETTINGS_BG };
    let screen = ctx.screen_rect();
    let default_size = egui::vec2(480.0, 520.0);
    let default_pos = egui::pos2(
        (screen.width() - default_size.x) * 0.5,
        (screen.height() - default_size.y) * 0.5,
    );
    egui::Window::new("Settings")
        .open(open)
        .resizable(true)
        .collapsible(false)
        .default_size(default_size)
        .default_pos(default_pos)
        .min_width(400.0)
        .min_height(360.0)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(bg)
                .inner_margin(14.0),
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    state.inner.draw_form(ui, true);
                });
        });
    // Restore annotate chrome for the rest of the frame (toolbar, handles).
    ui::apply_annotate_visuals(ctx, theme);
}

/// Embeddable settings state for the annotate overlay.
pub struct EmbeddedSettings {
    inner: SettingsApp,
}

impl EmbeddedSettings {
    pub fn load() -> Self {
        let cfg = Config::load().unwrap_or_default();
        Self {
            inner: SettingsApp::new(cfg),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.cfg
    }
}

/// Resolve clipboard preference: CLI flags beat config.
pub fn want_clipboard(cli_clipboard: bool, cli_no_clipboard: bool, cfg: &Config) -> bool {
    if cli_clipboard {
        true
    } else if cli_no_clipboard {
        false
    } else {
        cfg.save_to_clipboard
    }
}

/// Resolve save dir: CLI `--save-dir` beats config/env chain.
pub fn resolve_save_dir(cli_save_dir: Option<PathBuf>, cfg: &Config) -> Result<PathBuf> {
    if let Some(dir) = cli_save_dir {
        return Ok(dir);
    }
    Ok(cfg.effective_save_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_flags_beat_config() {
        let mut cfg = Config::default();
        cfg.save_to_clipboard = false;
        assert!(want_clipboard(true, false, &cfg));
        assert!(!want_clipboard(false, true, &cfg));
        assert!(!want_clipboard(false, false, &cfg));
        cfg.save_to_clipboard = true;
        assert!(want_clipboard(false, false, &cfg));
    }

    #[test]
    fn format_combo_includes_modifiers() {
        let mut mods = Modifiers::NONE;
        mods.shift = true;
        mods.command = true;
        assert_eq!(format_key_combo(mods, Key::S), "Super+Shift+S");
    }
}
