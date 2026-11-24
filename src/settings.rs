//! Settings window: Apple HIG–inspired grouped light chrome.
//!
//! Clarity / deference / depth: grouped cards (HIG boxes), restrained materials,
//! one primary Save. No purple / glow / marketing pills.
//!
//! Design dials: ENERGY 2 / RHYTHM 2 / MOTION 1.

use std::path::PathBuf;

use anyhow::{bail, Result};
use eframe::egui::{self, ViewportBuilder};

use crate::config::{config_path, Config};
use crate::desktop::{apply_desktop_integrations, maybe_spawn_tray};
use crate::ui::{
    self, settings_group_frame, SETTINGS_BG, SETTINGS_LABEL, SETTINGS_PRIMARY, SETTINGS_SECONDARY,
};

/// Open the standalone settings window. Blocks until closed.
pub fn run_settings_window() -> Result<()> {
    ensure_display_available()?;

    let cfg = Config::load().unwrap_or_default();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("lightsnip Settings")
            .with_inner_size([520.0, 620.0])
            .with_min_inner_size([420.0, 480.0])
            // Settings is infrequently opened; dim maximize per HIG settings.md.
            .with_maximize_button(false),
        ..Default::default()
    };

    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            "lightsnip settings",
            options,
            Box::new(move |cc| {
                ui::apply_settings_visuals(&cc.egui_ctx);
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
        }
    }

    fn persist(&mut self) {
        self.cfg.save_directory = PathBuf::from(self.save_dir_text.trim());
        self.cfg.shortcut_bind_command = self.shortcut_text.trim().to_string();
        if self.cfg.shortcut_bind_command.is_empty() {
            self.cfg.shortcut_bind_command = crate::config::default_shortcut_bind_command();
            self.shortcut_text = self.cfg.shortcut_bind_command.clone();
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

    fn draw_form(&mut self, ui: &mut egui::Ui, compact: bool) {
        if !compact {
            ui.label(
                egui::RichText::new("Settings")
                    .color(SETTINGS_LABEL)
                    .size(22.0)
                    .strong(),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Defaults for capture, annotate, and session helpers.")
                    .color(SETTINGS_SECONDARY)
                    .size(12.0),
            );
            ui.add_space(14.0);
        }

        // Shortcuts
        settings_group_frame().show(ui, |ui| {
            group_title(ui, "Shortcuts");
            ui.label(
                egui::RichText::new(
                    "Wayland has no portable global hotkey API. Bind this command in your compositor (Sway/Hyprland) or DE keyboard settings (GNOME/KDE).",
                )
                .color(SETTINGS_SECONDARY)
                .size(12.0),
            );
            ui.add_space(6.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.shortcut_text)
                    .desired_width(f32::INFINITY)
                    .hint_text("bind command"),
            );
        });
        ui.add_space(10.0);

        // Session
        settings_group_frame().show(ui, |ui| {
            group_title(ui, "Session");
            ui.checkbox(&mut self.cfg.run_at_startup, "Run at startup (XDG autostart)");
            helper(ui, "Writes ~/.config/autostart/lightsnip.desktop → tray. Off by default.");
            ui.add_space(4.0);
            ui.checkbox(
                &mut self.cfg.replace_default_screenshot,
                "Replace default screenshot tool (helper only)",
            );
            helper(
                ui,
                "Installs a user .desktop entry. Does not steal PrintScreen on GNOME/KDE; bind the key yourself. Off by default.",
            );
            ui.add_space(4.0);
            ui.checkbox(
                &mut self.cfg.stay_in_tray,
                "Stay in taskbar / tray (StatusNotifierItem)",
            );
            helper(
                ui,
                "Needs a StatusNotifier watcher (KDE, many trays). GNOME often needs an AppIndicator extension. Off by default.",
            );
        });
        ui.add_space(10.0);

        // Output
        settings_group_frame().show(ui, |ui| {
            group_title(ui, "Output");
            ui.label(
                egui::RichText::new("Save directory")
                    .color(SETTINGS_LABEL)
                    .size(12.0),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.save_dir_text)
                    .desired_width(f32::INFINITY)
                    .hint_text("~/Pictures/lightsnip"),
            );
            helper(ui, "CLI --save-dir and LIGHTSNIP_SAVE_DIR still override this.");
            ui.add_space(4.0);
            ui.checkbox(
                &mut self.cfg.save_to_clipboard,
                "Save to clipboard by default",
            );
            helper(
                ui,
                "Used when neither --clipboard nor --no-clipboard is passed. Annotate Copy always copies.",
            );
        });

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new("Save")
                        .fill(SETTINGS_PRIMARY)
                        .min_size(egui::vec2(88.0, 28.0)),
                )
                .clicked()
            {
                self.persist();
            }
            if !compact
                && ui
                    .add(egui::Button::new("Close").min_size(egui::vec2(72.0, 28.0)))
                    .clicked()
            {
                self.close_requested = true;
            }
        });
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(&self.status)
                .color(SETTINGS_SECONDARY)
                .size(12.0),
        );
    }
}

fn group_title(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .color(SETTINGS_LABEL)
            .size(13.0)
            .strong(),
    );
    ui.add_space(4.0);
}

fn helper(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .color(SETTINGS_SECONDARY)
            .size(11.0),
    );
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.close_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.close_requested = true;
            }
        });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(SETTINGS_BG)
                    .inner_margin(20.0),
            )
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
    // Embedded sheet uses the annotate dark visuals; restyle frame locally.
    egui::Window::new("Settings")
        .open(open)
        .resizable(true)
        .default_size([460.0, 560.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(SETTINGS_BG)
                .inner_margin(16.0),
        )
        .show(ctx, |ui| {
            // Force light labels inside the dark annotate context.
            ui.style_mut().visuals.override_text_color = Some(SETTINGS_LABEL);
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
}
