//! Settings window (Milestone 5): six fields, quiet charcoal chrome, TOML persist.
//!
//! Design dials: ENERGY 1 / RHYTHM 1 / MOTION 1. Focal point: Save button + form fields.
//! Accent: lifted charcoal action fill only (no purple/cyan/glow).

use std::path::PathBuf;

use anyhow::{bail, Result};
use eframe::egui::{self, Color32, ViewportBuilder};

use crate::config::{config_path, Config};
use crate::desktop::{apply_desktop_integrations, maybe_spawn_tray};
use crate::ui::{self, ACTION_FILL, MUTED};

/// Open the standalone settings window. Blocks until closed.
pub fn run_settings_window() -> Result<()> {
    ensure_display_available()?;

    let cfg = Config::load().unwrap_or_default();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("lightsnip settings")
            .with_inner_size([520.0, 560.0])
            .with_min_inner_size([420.0, 420.0]),
        ..Default::default()
    };

    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            "lightsnip settings",
            options,
            Box::new(move |cc| {
                ui::apply_quiet_visuals(&cc.egui_ctx);
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
                    .fill(Color32::from_rgb(0x22, 0x22, 0x24))
                    .inner_margin(16.0),
            )
            .show(ctx, |ui| {
                ui.heading(
                    egui::RichText::new("lightsnip settings")
                        .color(Color32::from_rgb(0xE8, 0xE8, 0xE8))
                        .size(18.0),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Quiet defaults. Optional install stays separate.")
                        .color(MUTED)
                        .size(12.0),
                );
                ui.add_space(12.0);

                ui.label(egui::RichText::new("Shortcuts").strong());
                ui.label(
                    egui::RichText::new(
                        "Wayland has no portable global hotkey API. Bind this command in your compositor (Sway/Hyprland) or DE keyboard settings (GNOME/KDE).",
                    )
                    .color(MUTED)
                    .size(12.0),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.shortcut_text)
                        .desired_width(f32::INFINITY)
                        .hint_text("bind command"),
                );
                ui.add_space(10.0);

                ui.label(egui::RichText::new("Session").strong());
                ui.checkbox(&mut self.cfg.run_at_startup, "Run at startup (XDG autostart)");
                ui.label(
                    egui::RichText::new(
                        "Writes ~/.config/autostart/lightsnip.desktop → launches tray. Off by default.",
                    )
                    .color(MUTED)
                    .size(12.0),
                );
                ui.add_space(4.0);
                ui.checkbox(
                    &mut self.cfg.replace_default_screenshot,
                    "Replace default screenshot tool (helper only)",
                );
                ui.label(
                    egui::RichText::new(
                        "Installs a user .desktop entry. Does not steal PrintScreen on GNOME/KDE; bind the key yourself. Off by default.",
                    )
                    .color(MUTED)
                    .size(12.0),
                );
                ui.add_space(4.0);
                ui.checkbox(
                    &mut self.cfg.stay_in_tray,
                    "Stay in taskbar / tray (StatusNotifierItem)",
                );
                ui.label(
                    egui::RichText::new(
                        "Needs a working StatusNotifier watcher (KDE, many trays). GNOME often needs an AppIndicator extension. Off by default.",
                    )
                    .color(MUTED)
                    .size(12.0),
                );
                ui.add_space(10.0);

                ui.label(egui::RichText::new("Output").strong());
                ui.label(egui::RichText::new("Save directory").size(12.0));
                ui.add(
                    egui::TextEdit::singleline(&mut self.save_dir_text)
                        .desired_width(f32::INFINITY)
                        .hint_text("~/Pictures/lightsnip"),
                );
                ui.label(
                    egui::RichText::new(
                        "CLI --save-dir and LIGHTSNIP_SAVE_DIR still override this.",
                    )
                    .color(MUTED)
                    .size(12.0),
                );
                ui.add_space(4.0);
                ui.checkbox(
                    &mut self.cfg.save_to_clipboard,
                    "Save to clipboard by default",
                );
                ui.label(
                    egui::RichText::new(
                        "Used when neither --clipboard nor --no-clipboard is passed.",
                    )
                    .color(MUTED)
                    .size(12.0),
                );

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new("Save")
                                .fill(ACTION_FILL)
                                .min_size(egui::vec2(72.0, 28.0)),
                        )
                        .clicked()
                    {
                        self.persist();
                    }
                    if ui.button("Close").clicked() {
                        self.close_requested = true;
                    }
                });
                ui.add_space(8.0);
                ui.label(egui::RichText::new(&self.status).color(MUTED).size(12.0));
            });
    }
}

/// Draw the settings form inside an egui window (annotate toolbar gear).
pub fn show_settings_window(ctx: &egui::Context, open: &mut bool, state: &mut EmbeddedSettings) {
    if !*open {
        return;
    }
    egui::Window::new("Settings")
        .open(open)
        .resizable(true)
        .default_size([480.0, 520.0])
        .show(ctx, |ui| {
            state.ui(ui);
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

    fn ui(&mut self, ui: &mut egui::Ui) {
        // Reuse the same fields via a compact layout matching the standalone window.
        ui.label(
            egui::RichText::new("Wayland shortcuts are compositor binds (no portable global grab).")
                .color(MUTED)
                .size(12.0),
        );
        ui.add(
            egui::TextEdit::singleline(&mut self.inner.shortcut_text)
                .desired_width(f32::INFINITY),
        );
        ui.checkbox(&mut self.inner.cfg.run_at_startup, "Run at startup");
        ui.checkbox(
            &mut self.inner.cfg.replace_default_screenshot,
            "Replace default (helper .desktop)",
        );
        ui.checkbox(&mut self.inner.cfg.stay_in_tray, "Stay in tray");
        ui.label("Save directory");
        ui.add(
            egui::TextEdit::singleline(&mut self.inner.save_dir_text)
                .desired_width(f32::INFINITY),
        );
        ui.checkbox(
            &mut self.inner.cfg.save_to_clipboard,
            "Save to clipboard by default",
        );
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new("Save").fill(ACTION_FILL))
                .clicked()
            {
                self.inner.persist();
            }
        });
        ui.label(
            egui::RichText::new(&self.inner.status)
                .color(MUTED)
                .size(12.0),
        );
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
