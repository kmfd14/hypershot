//! Settings window: Apple HIG–inspired grouped chrome, compact spacing.
//!
//! Shortcuts: click “Press keys…”, then type a combination (stored as a label;
//! Wayland still needs a compositor bind — no portable global grab).
//! User Interface: toolbar position, theme, title bar, transparency, blur
//! (blur is honest about egui/Wayland limits).

use std::path::PathBuf;

use anyhow::{bail, Result};
use eframe::egui::{self, Key, Modifiers, ViewportBuilder};

use crate::config::{
    config_path, Config, ToolbarPosition, UiTheme,
};
use crate::desktop::{apply_desktop_integrations, maybe_spawn_tray};
use crate::ui::{
    self, settings_group_frame, SETTINGS_BG, SETTINGS_BG_DARK, SETTINGS_LABEL,
    SETTINGS_LABEL_DARK, SETTINGS_PRIMARY, SETTINGS_SECONDARY, SETTINGS_SECONDARY_DARK,
};

/// Open the standalone settings window. Blocks until closed.
pub fn run_settings_window() -> Result<()> {
    ensure_display_available()?;

    let cfg = Config::load().unwrap_or_default();
    let theme = cfg.theme;
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Lightsnip Settings")
            .with_inner_size([480.0, 640.0])
            .with_min_inner_size([400.0, 480.0])
            .with_maximize_button(false),
        ..Default::default()
    };

    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            "Lightsnip Settings",
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
    /// Listening for a key combination after “Press keys…”.
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

    fn poll_shortcut_capture(&mut self, ui: &mut egui::Ui) {
        if !self.capturing_shortcut {
            return;
        }
        ui.ctx().input(|i| {
            // Prefer a non-modifier key press with current modifiers.
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    if matches!(
                        key,
                        Key::Escape
                    ) {
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
        let label = if dark { SETTINGS_LABEL_DARK } else { SETTINGS_LABEL };
        let secondary = if dark {
            SETTINGS_SECONDARY_DARK
        } else {
            SETTINGS_SECONDARY
        };

        ui::apply_settings_visuals(ui.ctx(), self.cfg.theme);

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
            ui.add_space(10.0);
        }

        self.poll_shortcut_capture(ui);

        // Shortcuts
        settings_group_frame(dark).show(ui, |ui| {
            group_title(ui, "Shortcuts", label);
            ui.label(
                egui::RichText::new(
                    "Wayland has no portable global hotkey API. Assign a combo here, then bind it to the command in your compositor (Sway/Hyprland) or DE keyboard settings.",
                )
                .color(secondary)
                .size(11.0),
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Keys").color(label).size(12.0));
                let display = if self.capturing_shortcut {
                    "Press keys… (Esc cancel)".to_string()
                } else {
                    self.cfg.shortcut_keys.clone()
                };
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(display)
                            .color(if self.capturing_shortcut {
                                SETTINGS_PRIMARY
                            } else {
                                label
                            })
                            .strong(),
                    )
                    .sense(egui::Sense::click()),
                );
                let btn = if self.capturing_shortcut {
                    "Listening…"
                } else {
                    "Press keys…"
                };
                if ui
                    .add(egui::Button::new(btn).min_size(egui::vec2(96.0, 24.0)))
                    .clicked()
                {
                    self.capturing_shortcut = !self.capturing_shortcut;
                }
            });
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Bind command").color(label).size(12.0));
            ui.add(
                egui::TextEdit::singleline(&mut self.shortcut_text)
                    .desired_width(f32::INFINITY)
                    .hint_text("path/to/lightsnip region"),
            );
        });
        ui.add_space(8.0);

        // Session — compact toggle rows
        settings_group_frame(dark).show(ui, |ui| {
            group_title(ui, "Session", label);
            session_row(
                ui,
                &mut self.cfg.run_at_startup,
                "Run at startup",
                "XDG autostart → tray. Off by default.",
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
                "User .desktop helper only — does not steal PrintScreen.",
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
        ui.add_space(8.0);

        // Output
        settings_group_frame(dark).show(ui, |ui| {
            group_title(ui, "Output", label);
            ui.label(egui::RichText::new("Save directory").color(label).size(12.0));
            ui.add(
                egui::TextEdit::singleline(&mut self.save_dir_text)
                    .desired_width(f32::INFINITY)
                    .hint_text("~/Pictures/lightsnip"),
            );
            helper(ui, "CLI --save-dir and LIGHTSNIP_SAVE_DIR still override.", secondary);
            ui.add_space(4.0);
            session_row(
                ui,
                &mut self.cfg.save_to_clipboard,
                "Save to clipboard by default",
                "When neither --clipboard nor --no-clipboard is passed.",
                label,
                secondary,
            );
        });
        ui.add_space(8.0);

        // User Interface
        settings_group_frame(dark).show(ui, |ui| {
            group_title(ui, "User Interface", label);

            ui.label(egui::RichText::new("Toolbar position").color(label).size(12.0));
            ui.horizontal(|ui| {
                for pos in ToolbarPosition::ALL {
                    ui.selectable_value(&mut self.cfg.toolbar_position, pos, pos.label());
                }
            });
            helper(
                ui,
                "Floating rail attached to the preview (not full window height).",
                secondary,
            );
            ui.add_space(6.0);

            ui.label(egui::RichText::new("Theme").color(label).size(12.0));
            ui.horizontal(|ui| {
                for theme in UiTheme::ALL {
                    ui.selectable_value(&mut self.cfg.theme, theme, theme.label());
                }
            });
            ui.add_space(6.0);

            session_row(
                ui,
                &mut self.cfg.show_title_bar,
                "Show window title bar",
                "Native decorations. Off → borderless (Wayland support varies).",
                label,
                secondary,
            );
            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);
            session_row(
                ui,
                &mut self.cfg.window_transparency,
                "Window transparency",
                "Best-effort egui/winit alpha. Many Wayland compositors ignore it.",
                label,
                secondary,
            );
            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);
            session_row(
                ui,
                &mut self.cfg.window_background_blur,
                "Window background blur",
                "Not available: egui cannot backdrop-blur the desktop on Wayland. Preference is saved; chrome uses pastel frost fills instead.",
                label,
                secondary,
            );
        });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new("Save")
                        .fill(SETTINGS_PRIMARY)
                        .min_size(egui::vec2(80.0, 26.0)),
                )
                .clicked()
            {
                self.persist();
            }
            if !compact
                && ui
                    .add(egui::Button::new("Close").min_size(egui::vec2(64.0, 26.0)))
                    .clicked()
            {
                self.close_requested = true;
            }
        });
        ui.add_space(6.0);
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

fn group_title(ui: &mut egui::Ui, title: &str, color: egui::Color32) {
    ui.label(
        egui::RichText::new(title)
            .color(color)
            .size(13.0)
            .strong(),
    );
    ui.add_space(4.0);
}

fn helper(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    ui.label(egui::RichText::new(text).color(color).size(11.0));
}

fn session_row(
    ui: &mut egui::Ui,
    value: &mut bool,
    title: &str,
    hint: &str,
    label: egui::Color32,
    secondary: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).color(label).size(12.5));
            ui.label(egui::RichText::new(hint).color(secondary).size(10.5));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(value, "");
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.draw_form(ui, false);
                });
            });
    }
}

/// Draw the settings form inside an egui window (annotate toolbar gear).
pub fn show_settings_window(ctx: &egui::Context, open: &mut bool, state: &mut EmbeddedSettings) {
    if !*open {
        return;
    }
    let dark = ctx.theme() == egui::Theme::Dark;
    let bg = if dark { SETTINGS_BG_DARK } else { SETTINGS_BG };
    egui::Window::new("Settings")
        .open(open)
        .resizable(true)
        .default_size([440.0, 560.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(bg)
                .inner_margin(12.0),
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                state.inner.draw_form(ui, true);
            });
        });
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
