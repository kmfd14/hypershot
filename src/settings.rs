//! Settings window: dark two-pane chrome (sidebar + content + footer).
//!
//! Matches `/media/hypershot-settings-window-reference.png`: lavender accent,
//! Iconoir-style nav icons, keycap shortcut rows, OK / Cancel / Apply.
//!
//! Shortcuts store *suggested* key labels and a bind command only — HyperShot
//! does **not** register a live global hotkey. Bind the command in your desktop
//! keyboard settings. “Replace default screenshot” only installs a user
//! `.desktop` helper.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Result};
use eframe::egui::{self, Key, Modifiers, Sense, Stroke, ViewportBuilder};

use crate::config::{
    clamp_intensity, config_path, AnnotateSessionMode, Config, ToolbarPosition, UiTheme, APP_NAME,
    APP_VERSION,
};
use crate::desktop::{apply_desktop_integrations, maybe_spawn_tray};
use crate::ui::{
    self, choice_button, contrast_checkbox, settings_button, settings_text_edit, SETTINGS_BG_DARK,
    SETTINGS_CONTROL_HEIGHT, SETTINGS_FOOTER_BTN, SETTINGS_FOOTER_H, SETTINGS_KEYCAP,
    SETTINGS_LABEL_DARK, SETTINGS_NAV_ACTIVE, SETTINGS_PRIMARY, SETTINGS_SECONDARY_DARK,
    SETTINGS_SEPARATOR_DARK, SETTINGS_SIDEBAR_W,
};

/// Open the standalone settings window. Blocks until closed.
pub fn run_settings_window() -> Result<()> {
    ensure_display_available()?;

    let cfg = Config::load().unwrap_or_default();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(format!("{APP_NAME} Settings"))
            .with_inner_size([820.0, 560.0])
            .with_min_inner_size([640.0, 420.0])
            .with_maximize_button(false),
        centered: true,
        ..Default::default()
    };

    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            &format!("{APP_NAME} Settings"),
            options,
            Box::new(move |cc| {
                // Settings chrome is always the dark two-pane reference look.
                ui::apply_settings_visuals(&cc.egui_ctx, UiTheme::Dark);
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
        bail!("no display available (settings needs a graphical session)");
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavId {
    General,
    Capture,
    Overlay,
    Saving,
    FileNames,
    History,
    Recording,
    Camera,
    Upload,
    S3Storage,
    ShortcutsGlobal,
    ShortcutsRecording,
    ShortcutsOverlayTools,
    ShortcutsOverlayEditing,
    ShortcutsOverlayActions,
    ShortcutsEditWindow,
    About,
}

impl NavId {
    fn breadcrumb(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Capture => "Capture",
            Self::Overlay => "Overlay",
            Self::Saving => "Saving",
            Self::FileNames => "File names",
            Self::History => "History",
            Self::Recording => "Recording",
            Self::Camera => "Camera",
            Self::Upload => "Upload",
            Self::S3Storage => "S3 storage",
            Self::ShortcutsGlobal => "Shortcuts · Global",
            Self::ShortcutsRecording => "Shortcuts · Recording",
            Self::ShortcutsOverlayTools => "Shortcuts · Overlay tools",
            Self::ShortcutsOverlayEditing => "Shortcuts · Overlay editing",
            Self::ShortcutsOverlayActions => "Shortcuts · Overlay actions",
            Self::ShortcutsEditWindow => "Shortcuts · Edit window",
            Self::About => "About",
        }
    }

    #[cfg(test)]
    fn is_shortcuts(self) -> bool {
        matches!(
            self,
            Self::ShortcutsGlobal
                | Self::ShortcutsRecording
                | Self::ShortcutsOverlayTools
                | Self::ShortcutsOverlayEditing
                | Self::ShortcutsOverlayActions
                | Self::ShortcutsEditWindow
        )
    }
}

#[derive(Clone, Copy)]
enum NavIcon {
    General,
    Capture,
    Overlay,
    Saving,
    FileNames,
    History,
    Recording,
    Camera,
    Upload,
    S3,
    Keyboard,
    Grid,
    Pencil,
    Cursor,
    Send,
    Window,
    Info,
}

struct SettingsApp {
    cfg: Config,
    /// Snapshot used for dirty detection + Cancel restore.
    baseline: Config,
    save_dir_text: String,
    shortcut_text: String,
    status: String,
    close_requested: bool,
    /// Listening for a key combination after clicking a shortcut row.
    capturing: Option<&'static str>,
    nav: NavId,
}

impl SettingsApp {
    fn new(cfg: Config) -> Self {
        let save_dir_text = cfg.save_directory.display().to_string();
        let shortcut_text = cfg.shortcut_bind_command.clone();
        Self {
            baseline: cfg.clone(),
            cfg,
            save_dir_text,
            shortcut_text,
            status: String::new(),
            close_requested: false,
            capturing: None,
            nav: NavId::ShortcutsGlobal,
        }
    }

    fn is_dirty(&self) -> bool {
        let mut draft = self.cfg.clone();
        draft.save_directory = PathBuf::from(self.save_dir_text.trim());
        draft.shortcut_bind_command = self.shortcut_text.trim().to_string();
        draft != self.baseline
    }

    fn sync_text_into_cfg(&mut self) {
        self.cfg.save_directory = PathBuf::from(self.save_dir_text.trim());
        self.cfg.shortcut_bind_command = self.shortcut_text.trim().to_string();
        if self.cfg.shortcut_bind_command.is_empty() {
            self.cfg.shortcut_bind_command = crate::config::default_shortcut_bind_command();
            self.shortcut_text = self.cfg.shortcut_bind_command.clone();
        }
        if self.cfg.shortcut_keys.trim().is_empty() {
            self.cfg.shortcut_keys = crate::config::default_shortcut_keys();
        }
        if self.cfg.shortcut_full_keys.trim().is_empty() {
            self.cfg.shortcut_full_keys = crate::config::default_shortcut_full_keys();
        }
        if self.cfg.shortcut_quit_keys.trim().is_empty() {
            self.cfg.shortcut_quit_keys = crate::config::default_shortcut_quit_keys();
        }
        self.cfg.window_transparency_intensity =
            clamp_intensity(self.cfg.window_transparency_intensity);
        self.cfg.window_background_blur_intensity =
            clamp_intensity(self.cfg.window_background_blur_intensity);
    }

    fn persist(&mut self) -> bool {
        self.sync_text_into_cfg();
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
                self.baseline = self.cfg.clone();
                true
            }
            Err(err) => {
                self.status = format!("Save failed: {err:#}");
                false
            }
        }
    }

    fn cancel_and_close(&mut self) {
        self.cfg = self.baseline.clone();
        self.save_dir_text = self.cfg.save_directory.display().to_string();
        self.shortcut_text = self.cfg.shortcut_bind_command.clone();
        self.capturing = None;
        self.close_requested = true;
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

    fn open_config_file(&mut self) {
        match config_path() {
            Ok(path) => {
                // Ensure the file exists so the desktop can open it.
                if !path.is_file() {
                    let _ = self.cfg.save();
                }
                let opened = open_path_in_desktop(&path);
                self.status = if opened {
                    format!("Opened {}", path.display())
                } else {
                    format!("Could not open {}", path.display())
                };
            }
            Err(err) => {
                self.status = format!("Config path unavailable: {err:#}");
            }
        }
    }

    fn reset_current_section(&mut self) {
        let defaults = Config::default();
        match self.nav {
            NavId::General => {
                self.cfg.run_at_startup = defaults.run_at_startup;
                self.cfg.replace_default_screenshot = defaults.replace_default_screenshot;
                self.cfg.stay_in_tray = defaults.stay_in_tray;
                self.cfg.theme = defaults.theme;
            }
            NavId::Capture => {
                self.cfg.annotate_session_mode = defaults.annotate_session_mode;
            }
            NavId::Overlay => {
                self.cfg.toolbar_position = defaults.toolbar_position;
                self.cfg.show_title_bar = defaults.show_title_bar;
                self.cfg.follow_window_tiling = defaults.follow_window_tiling;
                self.cfg.esc_to_exit = defaults.esc_to_exit;
                self.cfg.window_transparency = defaults.window_transparency;
                self.cfg.window_transparency_intensity = defaults.window_transparency_intensity;
                self.cfg.window_background_blur = defaults.window_background_blur;
                self.cfg.window_background_blur_intensity =
                    defaults.window_background_blur_intensity;
            }
            NavId::Saving => {
                self.cfg.save_directory = defaults.save_directory.clone();
                self.save_dir_text = self.cfg.save_directory.display().to_string();
                self.cfg.save_to_clipboard = defaults.save_to_clipboard;
            }
            NavId::ShortcutsGlobal => {
                self.cfg.shortcut_keys = defaults.shortcut_keys;
                self.cfg.shortcut_full_keys = defaults.shortcut_full_keys;
                self.cfg.shortcut_window_keys = defaults.shortcut_window_keys;
                self.cfg.shortcut_repeat_keys = defaults.shortcut_repeat_keys;
                self.cfg.shortcut_copy_text_keys = defaults.shortcut_copy_text_keys;
                self.cfg.shortcut_qr_keys = defaults.shortcut_qr_keys;
                self.cfg.shortcut_history_keys = defaults.shortcut_history_keys;
                self.cfg.shortcut_quit_keys = defaults.shortcut_quit_keys;
                self.cfg.shortcut_bind_command = crate::config::default_shortcut_bind_command();
                self.shortcut_text = self.cfg.shortcut_bind_command.clone();
                self.capturing = None;
            }
            _ => {}
        }
    }

    fn poll_shortcut_capture(&mut self, ui: &mut egui::Ui) {
        let Some(target) = self.capturing else {
            return;
        };
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
                        self.capturing = None;
                        return;
                    }
                    let combo = format_key_combo(*modifiers, *key);
                    match target {
                        "region" => self.cfg.shortcut_keys = combo,
                        "full" => self.cfg.shortcut_full_keys = combo,
                        "window" => self.cfg.shortcut_window_keys = combo,
                        "repeat" => self.cfg.shortcut_repeat_keys = combo,
                        "copy_text" => self.cfg.shortcut_copy_text_keys = combo,
                        "qr" => self.cfg.shortcut_qr_keys = combo,
                        "history" => self.cfg.shortcut_history_keys = combo,
                        "quit" => self.cfg.shortcut_quit_keys = combo,
                        _ => {}
                    }
                    self.capturing = None;
                    return;
                }
            }
        });
    }

    fn draw_shell(&mut self, ui: &mut egui::Ui) {
        ui::apply_settings_visuals(ui.ctx(), UiTheme::Dark);
        self.poll_shortcut_capture(ui);

        egui::TopBottomPanel::bottom("hypershot_settings_footer")
            .exact_height(SETTINGS_FOOTER_H + 8.0)
            .frame(
                egui::Frame::new()
                    .fill(SETTINGS_BG_DARK)
                    .stroke(Stroke::new(1.0_f32, SETTINGS_SEPARATOR_DARK))
                    .inner_margin(egui::Margin::symmetric(16, 8)),
            )
            .show_inside(ui, |ui| {
                self.draw_footer(ui);
            });

        egui::SidePanel::left("hypershot_settings_nav")
            .exact_width(SETTINGS_SIDEBAR_W)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(SETTINGS_BG_DARK)
                    .stroke(Stroke::new(1.0_f32, SETTINGS_SEPARATOR_DARK))
                    .inner_margin(egui::Margin::symmetric(10, 12)),
            )
            .show_inside(ui, |ui| {
                self.draw_sidebar(ui);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(SETTINGS_BG_DARK)
                    .inner_margin(egui::Margin::symmetric(20, 14)),
            )
            .show_inside(ui, |ui| {
                self.draw_content(ui);
            });
    }

    fn draw_sidebar(&mut self, ui: &mut egui::Ui) {
        let label = SETTINGS_LABEL_DARK;
        let secondary = SETTINGS_SECONDARY_DARK;

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(ui.available_width());

            let top_items: &[(NavId, &str, NavIcon)] = &[
                (NavId::General, "General", NavIcon::General),
                (NavId::Capture, "Capture", NavIcon::Capture),
                (NavId::Overlay, "Overlay", NavIcon::Overlay),
                (NavId::Saving, "Saving", NavIcon::Saving),
                (NavId::FileNames, "File names", NavIcon::FileNames),
                (NavId::History, "History", NavIcon::History),
                (NavId::Recording, "Recording", NavIcon::Recording),
                (NavId::Camera, "Camera", NavIcon::Camera),
                (NavId::Upload, "Upload", NavIcon::Upload),
                (NavId::S3Storage, "S3 storage", NavIcon::S3),
            ];
            let shortcut_items: &[(NavId, &str, NavIcon)] = &[
                (NavId::ShortcutsGlobal, "Global", NavIcon::Keyboard),
                (NavId::ShortcutsRecording, "Recording", NavIcon::Grid),
                (NavId::ShortcutsOverlayTools, "Overlay tools", NavIcon::Pencil),
                (
                    NavId::ShortcutsOverlayEditing,
                    "Overlay editing",
                    NavIcon::Cursor,
                ),
                (
                    NavId::ShortcutsOverlayActions,
                    "Overlay actions",
                    NavIcon::Send,
                ),
                (NavId::ShortcutsEditWindow, "Edit window", NavIcon::Window),
            ];

            let about_h = 32.0;
            let scroll_h = (ui.available_height() - about_h - 8.0).max(80.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(scroll_h)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        for &(id, name, icon) in top_items {
                            if nav_item(ui, self.nav == id, name, icon, label, secondary) {
                                self.nav = id;
                                self.capturing = None;
                            }
                        }
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new("SHORTCUTS")
                                .color(secondary)
                                .size(10.0)
                                .strong(),
                        );
                        ui.add_space(4.0);
                        for &(id, name, icon) in shortcut_items {
                            if nav_item(ui, self.nav == id, name, icon, label, secondary) {
                                self.nav = id;
                                self.capturing = None;
                            }
                        }
                    });
                });

            ui.add_space(4.0);
            if nav_item(
                ui,
                self.nav == NavId::About,
                "About",
                NavIcon::Info,
                label,
                secondary,
            ) {
                self.nav = NavId::About;
                self.capturing = None;
            }
        });
    }

    fn draw_content(&mut self, ui: &mut egui::Ui) {
        let label = SETTINGS_LABEL_DARK;
        let secondary = SETTINGS_SECONDARY_DARK;

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(ui.available_width());

            // Header: breadcrumb + reset
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(self.nav.breadcrumb())
                        .color(label)
                        .size(16.0)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if reset_defaults_button(ui).clicked() {
                        self.reset_current_section();
                    }
                });
            });
            ui.add_space(8.0);
            let (div, _) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
            ui.painter().rect_filled(div, 0.0, SETTINGS_SEPARATOR_DARK);
            ui.add_space(10.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        match self.nav {
                            NavId::General => self.page_general(ui, label, secondary),
                            NavId::Capture => self.page_capture(ui, label, secondary),
                            NavId::Overlay => self.page_overlay(ui, label, secondary),
                            NavId::Saving => self.page_saving(ui, label, secondary),
                            NavId::FileNames => stub_page(
                                ui,
                                label,
                                secondary,
                                "File names",
                                "Custom file-name patterns are not available yet. Captures use a timestamped PNG name.",
                            ),
                            NavId::History => stub_page(
                                ui,
                                label,
                                secondary,
                                "History",
                                "Capture history is not available yet.",
                            ),
                            NavId::Recording => stub_page(
                                ui,
                                label,
                                secondary,
                                "Recording",
                                "Screen recording is not available yet.",
                            ),
                            NavId::Camera => stub_page(
                                ui,
                                label,
                                secondary,
                                "Camera",
                                "Camera capture is not available yet.",
                            ),
                            NavId::Upload => stub_page(
                                ui,
                                label,
                                secondary,
                                "Upload",
                                "Upload destinations are not available yet.",
                            ),
                            NavId::S3Storage => stub_page(
                                ui,
                                label,
                                secondary,
                                "S3 storage",
                                "S3 storage is not available yet.",
                            ),
                            NavId::ShortcutsGlobal => {
                                self.page_shortcuts_global(ui, label, secondary)
                            }
                            NavId::ShortcutsRecording
                            | NavId::ShortcutsOverlayTools
                            | NavId::ShortcutsOverlayEditing
                            | NavId::ShortcutsOverlayActions
                            | NavId::ShortcutsEditWindow => {
                                stub_shortcut_page(ui, label, secondary, self.nav);
                            }
                            NavId::About => self.page_about(ui, label, secondary),
                        }
                    });
                });
        });
    }

    fn draw_footer(&mut self, ui: &mut egui::Ui) {
        let dirty = self.is_dirty();
        ui.horizontal(|ui| {
            ui.set_min_height(SETTINGS_FOOTER_H - 16.0);
            let link = ui.add(
                egui::Label::new(
                    egui::RichText::new("Open config file")
                        .color(SETTINGS_PRIMARY)
                        .size(13.0),
                )
                .sense(Sense::click()),
            );
            if link.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if link.clicked() {
                self.open_config_file();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                let apply_enabled = dirty;
                ui.add_enabled_ui(apply_enabled, |ui| {
                    let fill = if apply_enabled {
                        SETTINGS_FOOTER_BTN
                    } else {
                        egui::Color32::from_rgb(0x24, 0x25, 0x28)
                    };
                    let text = if apply_enabled {
                        SETTINGS_LABEL_DARK
                    } else {
                        egui::Color32::from_rgb(0x5A, 0x5A, 0x60)
                    };
                    if footer_button(ui, "Apply", fill, text, 72.0).clicked() {
                        self.persist();
                    }
                });

                if footer_button(ui, "Cancel", SETTINGS_FOOTER_BTN, SETTINGS_LABEL_DARK, 72.0)
                    .clicked()
                {
                    self.cancel_and_close();
                }

                if footer_button(ui, "OK", SETTINGS_PRIMARY, egui::Color32::WHITE, 72.0).clicked()
                {
                    if self.persist() || !dirty {
                        self.close_requested = true;
                    }
                }
            });
        });
        if !self.status.is_empty() {
            ui.label(
                egui::RichText::new(&self.status)
                    .color(SETTINGS_SECONDARY_DARK)
                    .size(11.0),
            );
        }
    }

    fn page_general(&mut self, ui: &mut egui::Ui, label: egui::Color32, secondary: egui::Color32) {
        session_row(
            ui,
            &mut self.cfg.run_at_startup,
            "Run at startup",
            "Start the tray when you log in.",
            label,
            secondary,
        );
        row_sep(ui);
        session_row(
            ui,
            &mut self.cfg.replace_default_screenshot,
            "Replace default screenshot",
            "Adds a launcher helper only — does not bind keys or steal Print Screen.",
            label,
            secondary,
        );
        row_sep(ui);
        session_row(
            ui,
            &mut self.cfg.stay_in_tray,
            "Stay in tray",
            "Keep a tray icon while HyperShot is running.",
            label,
            secondary,
        );
        ui.add_space(14.0);
        ui.label(egui::RichText::new("Theme").color(label).size(12.5).strong());
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            for theme in UiTheme::ALL {
                if choice_button(ui, self.cfg.theme == theme, theme.label()).clicked() {
                    self.cfg.theme = theme;
                }
            }
        });
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Applies to annotate chrome. This Settings window stays dark.")
                .color(secondary)
                .size(11.0),
        );
    }

    fn page_capture(&mut self, ui: &mut egui::Ui, label: egui::Color32, secondary: egui::Color32) {
        ui.label(
            egui::RichText::new("Annotate mode")
                .color(label)
                .size(12.5)
                .strong(),
        );
        ui.add_space(6.0);
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
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(
                "On-screen (default): mark up on the capture. Preview window: HyperShot window with a top toolbar.",
            )
            .color(secondary)
            .size(11.0),
        );
    }

    fn page_overlay(&mut self, ui: &mut egui::Ui, label: egui::Color32, secondary: egui::Color32) {
        ui.label(
            egui::RichText::new("On-screen toolbar position")
                .color(label)
                .size(12.5)
                .strong(),
        );
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            for pos in ToolbarPosition::ALL {
                if choice_button(ui, self.cfg.toolbar_position == pos, pos.label()).clicked() {
                    self.cfg.toolbar_position = pos;
                }
            }
        });
        ui.label(
            egui::RichText::new(
                "Applies only to on-screen annotate. Preview window always uses a top toolbar.",
            )
            .color(secondary)
            .size(11.0),
        );
        ui.add_space(12.0);
        session_row(
            ui,
            &mut self.cfg.show_title_bar,
            "Show window title bar",
            "Show the normal window title bar. Off makes the window borderless.",
            label,
            secondary,
        );
        row_sep(ui);
        session_row(
            ui,
            &mut self.cfg.follow_window_tiling,
            "Follow window tiling",
            "On: let your window manager place annotate. Off (default): center the window and keep it on top when the desktop allows.",
            label,
            secondary,
        );
        row_sep(ui);
        session_row(
            ui,
            &mut self.cfg.esc_to_exit,
            "Esc to exit",
            "When on (default), Escape closes annotate without save/copy (after clearing a draft). Exit always works.",
            label,
            secondary,
        );
        row_sep(ui);
        session_row(
            ui,
            &mut self.cfg.window_transparency,
            "Window transparency",
            "Softens the preview window. The toolbar stays solid.",
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
        row_sep(ui);
        session_row(
            ui,
            &mut self.cfg.window_background_blur,
            "Window background blur",
            "Frosts the preview area. The toolbar stays a solid pastel panel.",
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
    }

    fn page_saving(&mut self, ui: &mut egui::Ui, label: egui::Color32, secondary: egui::Color32) {
        ui.label(
            egui::RichText::new("Save directory")
                .color(label)
                .size(12.5)
                .strong(),
        );
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            settings_text_edit(
                ui,
                &mut self.save_dir_text,
                ui.available_width() - 108.0,
                "~/Pictures/hypershot",
            );
            if settings_button(ui, "Browse…", None, 96.0)
                .on_hover_text("Choose preferred folder")
                .clicked()
            {
                self.pick_save_directory();
            }
        });
        ui.add_space(10.0);
        session_row(
            ui,
            &mut self.cfg.save_to_clipboard,
            "Save to clipboard by default",
            "Copy each save unless you pass --clipboard or --no-clipboard.",
            label,
            secondary,
        );
    }

    fn page_shortcuts_global(
        &mut self,
        ui: &mut egui::Ui,
        label: egui::Color32,
        secondary: egui::Color32,
    ) {
        ui.label(
            egui::RichText::new(
                "Suggested keys only — bind the command in your desktop keyboard settings.",
            )
            .color(SETTINGS_PRIMARY)
            .size(11.5)
            .strong(),
        );
        ui.add_space(10.0);

        let region = self.cfg.shortcut_keys.clone();
        let full = self.cfg.shortcut_full_keys.clone();
        let window = self.cfg.shortcut_window_keys.clone();
        let repeat = self.cfg.shortcut_repeat_keys.clone();
        let copy_text = self.cfg.shortcut_copy_text_keys.clone();
        let qr = self.cfg.shortcut_qr_keys.clone();
        let history = self.cfg.shortcut_history_keys.clone();
        let quit = self.cfg.shortcut_quit_keys.clone();
        let capturing = self.capturing;
        let quit_label = format!("Quit {APP_NAME}");

        let rows = [
            ("Capture region", region.as_str(), "region"),
            ("Capture full screen", full.as_str(), "full"),
            ("Capture window", window.as_str(), "window"),
            ("Repeat last capture", repeat.as_str(), "repeat"),
            ("Copy text from area", copy_text.as_str(), "copy_text"),
            ("Read QR code from area", qr.as_str(), "qr"),
            ("Open History", history.as_str(), "history"),
            (quit_label.as_str(), quit.as_str(), "quit"),
        ];
        let mut clicked: Option<&'static str> = None;
        for (name, keys, id) in rows {
            if shortcut_row(ui, name, keys, capturing == Some(id), label) {
                clicked = Some(id);
            }
        }
        if let Some(id) = clicked {
            self.capturing = Some(id);
        }

        ui.add_space(16.0);
        ui.label(
            egui::RichText::new("Bind command (region)")
                .color(label)
                .size(12.5)
                .strong(),
        );
        ui.add_space(4.0);
        settings_text_edit(
            ui,
            &mut self.shortcut_text,
            f32::INFINITY,
            "path/to/hypershot region",
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "Paste into your desktop’s custom shortcuts. Full capture uses the same binary with `full`.",
            )
            .color(secondary)
            .size(11.0),
        );
    }

    fn page_about(&mut self, ui: &mut egui::Ui, label: egui::Color32, secondary: egui::Color32) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(APP_NAME)
                    .color(label)
                    .size(18.0)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(format!("v{APP_VERSION}"))
                    .color(secondary)
                    .size(14.0),
            );
        });
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new("Screenshot tool with annotation.")
                .color(secondary)
                .size(12.0),
        );
    }
}

fn open_path_in_desktop(path: &std::path::Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn().is_ok()
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .spawn()
            .is_ok()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Command::new("xdg-open").arg(path).spawn().is_ok()
            || Command::new("gio")
                .args(["open", &path.display().to_string()])
                .spawn()
                .is_ok()
    }
}

fn stub_page(
    ui: &mut egui::Ui,
    label: egui::Color32,
    secondary: egui::Color32,
    title: &str,
    body: &str,
) {
    ui.label(egui::RichText::new(title).color(label).size(13.0).strong());
    ui.add_space(6.0);
    ui.label(egui::RichText::new(body).color(secondary).size(12.0));
}

fn stub_shortcut_page(
    ui: &mut egui::Ui,
    label: egui::Color32,
    secondary: egui::Color32,
    nav: NavId,
) {
    let hint = match nav {
        NavId::ShortcutsRecording => "Recording shortcuts will appear here when recording ships.",
        NavId::ShortcutsOverlayTools => {
            "Overlay tool shortcuts use the on-screen annotate toolbar for now."
        }
        NavId::ShortcutsOverlayEditing => {
            "Editing shortcuts (undo/redo) work in annotate; global binds are not configured here yet."
        }
        NavId::ShortcutsOverlayActions => {
            "Overlay action shortcuts (save/copy/exit) work in annotate; global binds are not configured here yet."
        }
        NavId::ShortcutsEditWindow => {
            "Edit-window shortcuts will appear here when more window binds are added."
        }
        _ => "No shortcuts in this group yet.",
    };
    ui.label(
        egui::RichText::new(nav.breadcrumb())
            .color(label)
            .size(13.0)
            .strong(),
    );
    ui.add_space(6.0);
    ui.label(egui::RichText::new(hint).color(secondary).size(12.0));
    ui.add_space(12.0);
    // Empty dash rows keep the layout familiar.
    for name in ["Action A", "Action B", "Action C"] {
        let _ = shortcut_row(ui, name, "", false, label);
    }
}

fn row_sep(ui: &mut egui::Ui) {
    ui.add_space(4.0);
    let (div, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(
        div,
        0.0,
        egui::Color32::from_rgb(0x2A, 0x2B, 0x2F),
    );
    ui.add_space(6.0);
}

fn nav_item(
    ui: &mut egui::Ui,
    active: bool,
    name: &str,
    icon: NavIcon,
    label: egui::Color32,
    secondary: egui::Color32,
) -> bool {
    let height = 28.0;
    let width = ui.available_width().max(SETTINGS_SIDEBAR_W - 20.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
    let painter = ui.painter_at(rect);

    if active {
        painter.rect_filled(rect, 6.0, SETTINGS_NAV_ACTIVE);
    } else if response.hovered() {
        painter.rect_filled(
            rect,
            6.0,
            egui::Color32::from_rgb(0x22, 0x23, 0x28),
        );
    }

    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        egui::vec2(14.0, 14.0),
    );
    let ink = if active { label } else { secondary };
    paint_nav_icon(&painter, icon_rect, ink, icon);

    painter.text(
        egui::pos2(rect.left() + 30.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(if active { 12.5 } else { 12.0 }),
        if active { label } else { secondary },
    );

    response.clicked()
}

fn paint_nav_icon(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32, icon: NavIcon) {
    let stroke = Stroke::new(1.25_f32, color);
    let c = rect.center();
    let r = rect.width() * 0.42;
    match icon {
        NavIcon::General => {
            // Fader / levels
            for (i, h) in [0.55_f32, 0.9, 0.4].iter().enumerate() {
                let x = rect.left() + rect.width() * (0.22 + i as f32 * 0.28);
                let top = c.y - r * h;
                let bot = c.y + r * 0.85;
                painter.line_segment(
                    [egui::pos2(x, top), egui::pos2(x, bot)],
                    stroke,
                );
                painter.circle_filled(egui::pos2(x, top + (bot - top) * 0.35), 1.8, color);
            }
        }
        NavIcon::Capture => {
            painter.rect_stroke(
                egui::Rect::from_center_size(c, egui::vec2(r * 1.7, r * 1.2)),
                2.0,
                stroke,
                egui::StrokeKind::Outside,
            );
            painter.circle_stroke(c, r * 0.35, stroke);
        }
        NavIcon::Overlay => {
            let rr = egui::Rect::from_center_size(c, egui::vec2(r * 1.6, r * 1.6));
            // Dashed square
            let pts = [
                rr.left_top(),
                rr.right_top(),
                rr.right_bottom(),
                rr.left_bottom(),
            ];
            for i in 0..4 {
                let a = pts[i];
                let b = pts[(i + 1) % 4];
                let mid = egui::pos2((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
                painter.line_segment([a, mid], stroke);
            }
        }
        NavIcon::Saving => {
            let folder = egui::Rect::from_center_size(
                egui::pos2(c.x, c.y + 1.0),
                egui::vec2(r * 1.7, r * 1.2),
            );
            painter.rect_stroke(folder, 1.5, stroke, egui::StrokeKind::Outside);
            painter.line_segment(
                [
                    egui::pos2(folder.left(), folder.top()),
                    egui::pos2(folder.left() + r * 0.55, folder.top() - r * 0.35),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(folder.left() + r * 0.55, folder.top() - r * 0.35),
                    egui::pos2(folder.left() + r * 0.95, folder.top()),
                ],
                stroke,
            );
        }
        NavIcon::FileNames => {
            painter.text(
                c,
                egui::Align2::CENTER_CENTER,
                "T",
                egui::FontId::proportional(13.0),
                color,
            );
        }
        NavIcon::History => {
            painter.circle_stroke(c, r * 0.85, stroke);
            painter.line_segment([c, egui::pos2(c.x, c.y - r * 0.5)], stroke);
            painter.line_segment([c, egui::pos2(c.x + r * 0.45, c.y + 0.2)], stroke);
            // small undo tick
            painter.line_segment(
                [
                    egui::pos2(c.x - r * 0.7, c.y - r * 0.55),
                    egui::pos2(c.x - r * 0.35, c.y - r * 0.85),
                ],
                stroke,
            );
        }
        NavIcon::Recording => {
            painter.rect_stroke(
                egui::Rect::from_center_size(c, egui::vec2(r * 1.5, r * 1.1)),
                2.0,
                stroke,
                egui::StrokeKind::Outside,
            );
            painter.circle_filled(c, r * 0.28, color);
        }
        NavIcon::Camera => {
            painter.circle_stroke(c, r * 0.85, stroke);
            painter.circle_stroke(c, r * 0.4, stroke);
        }
        NavIcon::Upload | NavIcon::S3 => {
            // Cloud-ish arcs
            painter.circle_stroke(
                egui::pos2(c.x - r * 0.35, c.y + r * 0.1),
                r * 0.45,
                stroke,
            );
            painter.circle_stroke(
                egui::pos2(c.x + r * 0.25, c.y),
                r * 0.55,
                stroke,
            );
            if matches!(icon, NavIcon::S3) {
                painter.rect_stroke(
                    egui::Rect::from_center_size(
                        egui::pos2(c.x + r * 0.55, c.y + r * 0.55),
                        egui::vec2(r * 0.55, r * 0.45),
                    ),
                    1.0,
                    stroke,
                    egui::StrokeKind::Outside,
                );
            }
        }
        NavIcon::Keyboard => {
            painter.rect_stroke(
                egui::Rect::from_center_size(c, egui::vec2(r * 1.8, r * 1.1)),
                2.0,
                stroke,
                egui::StrokeKind::Outside,
            );
            for i in 0..3 {
                let x = c.x - r * 0.5 + i as f32 * r * 0.5;
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        egui::pos2(x, c.y - r * 0.15),
                        egui::vec2(r * 0.28, r * 0.28),
                    ),
                    1.0,
                    color,
                );
            }
        }
        NavIcon::Grid => {
            for row in 0..2 {
                for col in 0..2 {
                    let p = egui::pos2(
                        c.x - r * 0.4 + col as f32 * r * 0.8,
                        c.y - r * 0.4 + row as f32 * r * 0.8,
                    );
                    painter.rect_stroke(
                        egui::Rect::from_center_size(p, egui::vec2(r * 0.55, r * 0.55)),
                        1.0,
                        stroke,
                        egui::StrokeKind::Outside,
                    );
                }
            }
        }
        NavIcon::Pencil => {
            painter.line_segment(
                [
                    egui::pos2(c.x - r * 0.55, c.y + r * 0.55),
                    egui::pos2(c.x + r * 0.55, c.y - r * 0.55),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(c.x + r * 0.35, c.y - r * 0.75),
                    egui::pos2(c.x + r * 0.75, c.y - r * 0.35),
                ],
                stroke,
            );
        }
        NavIcon::Cursor => {
            let tip = egui::pos2(c.x - r * 0.4, c.y - r * 0.7);
            painter.line_segment([tip, egui::pos2(c.x - r * 0.4, c.y + r * 0.6)], stroke);
            painter.line_segment([tip, egui::pos2(c.x + r * 0.55, c.y + r * 0.15)], stroke);
            painter.line_segment(
                [
                    egui::pos2(c.x - r * 0.05, c.y + r * 0.05),
                    egui::pos2(c.x + r * 0.2, c.y + r * 0.7),
                ],
                stroke,
            );
        }
        NavIcon::Send => {
            painter.line_segment(
                [
                    egui::pos2(c.x - r * 0.7, c.y + r * 0.5),
                    egui::pos2(c.x + r * 0.7, c.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(c.x - r * 0.7, c.y - r * 0.5),
                    egui::pos2(c.x + r * 0.7, c.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(c.x - r * 0.7, c.y - r * 0.5),
                    egui::pos2(c.x - r * 0.15, c.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(c.x - r * 0.7, c.y + r * 0.5),
                    egui::pos2(c.x - r * 0.15, c.y),
                ],
                stroke,
            );
        }
        NavIcon::Window => {
            painter.rect_stroke(
                egui::Rect::from_center_size(c, egui::vec2(r * 1.7, r * 1.3)),
                2.0,
                stroke,
                egui::StrokeKind::Outside,
            );
            painter.line_segment(
                [
                    egui::pos2(c.x - r * 0.85, c.y - r * 0.35),
                    egui::pos2(c.x + r * 0.85, c.y - r * 0.35),
                ],
                stroke,
            );
            painter.rect_stroke(
                egui::Rect::from_center_size(
                    egui::pos2(c.x + r * 0.35, c.y + r * 0.25),
                    egui::vec2(r * 0.55, r * 0.45),
                ),
                1.0,
                stroke,
                egui::StrokeKind::Outside,
            );
        }
        NavIcon::Info => {
            painter.circle_stroke(c, r * 0.85, stroke);
            painter.circle_filled(egui::pos2(c.x, c.y - r * 0.35), 1.4, color);
            painter.line_segment(
                [
                    egui::pos2(c.x, c.y - r * 0.05),
                    egui::pos2(c.x, c.y + r * 0.45),
                ],
                Stroke::new(1.6_f32, color),
            );
        }
    }
}

fn reset_defaults_button(ui: &mut egui::Ui) -> egui::Response {
    let label = egui::RichText::new("Reset to defaults")
        .color(SETTINGS_LABEL_DARK)
        .size(12.0);
    let btn = egui::Button::new(label)
        .fill(egui::Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0_f32, egui::Color32::from_rgb(0x5A, 0x5A, 0x60)))
        .corner_radius(egui::CornerRadius::same(14))
        .min_size(egui::vec2(128.0, 26.0));
    let response = ui.add(btn);
    // Small circular arrow drawn left of text would need custom layout; the
    // bordered pill already matches the reference affordance.
    response.on_hover_text("Reset this section to defaults")
}

fn footer_button(
    ui: &mut egui::Ui,
    text: &str,
    fill: egui::Color32,
    text_color: egui::Color32,
    min_width: f32,
) -> egui::Response {
    let btn = egui::Button::new(egui::RichText::new(text).color(text_color).size(13.0))
        .fill(fill)
        .stroke(Stroke::NONE)
        .corner_radius(egui::CornerRadius::same(14))
        .min_size(egui::vec2(min_width, 30.0));
    ui.add(btn)
}

/// Returns true when the row was clicked (start listening).
fn shortcut_row(
    ui: &mut egui::Ui,
    name: &str,
    keys: &str,
    listening: bool,
    label: egui::Color32,
) -> bool {
    let height = 34.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        Sense::click(),
    );
    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            4.0,
            egui::Color32::from_rgb(0x20, 0x21, 0x25),
        );
    }
    if listening {
        ui.painter().rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0_f32, SETTINGS_PRIMARY),
            egui::StrokeKind::Inside,
        );
    }

    ui.painter().text(
        egui::pos2(rect.left() + 4.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        if listening {
            "Listening… (Esc cancels)"
        } else {
            name
        },
        egui::FontId::proportional(13.0),
        if listening { SETTINGS_PRIMARY } else { label },
    );

    // Keycaps on the right
    let parts = parse_keycaps(keys);
    let mut x = rect.right() - 4.0;
    if parts.is_empty() {
        ui.painter().text(
            egui::pos2(x, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            "—",
            egui::FontId::proportional(13.0),
            SETTINGS_SECONDARY_DARK,
        );
    } else {
        for part in parts.iter().rev() {
            let galley = ui.painter().layout_no_wrap(
                (*part).to_string(),
                egui::FontId::proportional(11.5),
                SETTINGS_LABEL_DARK,
            );
            let pad_x = 7.0;
            let w = galley.size().x + pad_x * 2.0;
            let h = 22.0;
            let key_rect = egui::Rect::from_min_size(
                egui::pos2(x - w, rect.center().y - h * 0.5),
                egui::vec2(w, h),
            );
            ui.painter().rect_filled(key_rect, 4.0, SETTINGS_KEYCAP);
            ui.painter().galley(
                egui::pos2(
                    key_rect.left() + pad_x,
                    key_rect.center().y - galley.size().y * 0.5,
                ),
                galley,
                SETTINGS_LABEL_DARK,
            );
            x = key_rect.left() - 4.0;
        }
    }

    ui.add_space(2.0);
    response.clicked()
}

/// Split `Shift+Super+X` into display keycaps (`Shift`, `Win`, `X`).
fn parse_keycaps(keys: &str) -> Vec<String> {
    let trimmed = keys.trim();
    if trimmed.is_empty() || trimmed == "—" || trimmed == "-" {
        return Vec::new();
    }
    trimmed
        .split('+')
        .map(|p| {
            let p = p.trim();
            match p {
                "Super" | "Meta" | "Cmd" | "Command" => "Win".to_string(),
                "Control" => "Ctrl".to_string(),
                other => other.to_string(),
            }
        })
        .filter(|p| !p.is_empty())
        .collect()
}

fn format_key_combo(modifiers: Modifiers, key: Key) -> String {
    let mut parts = Vec::new();
    if modifiers.ctrl && !modifiers.command {
        parts.push("Ctrl");
    }
    if modifiers.alt {
        parts.push("Alt");
    }
    if modifiers.shift {
        parts.push("Shift");
    }
    if modifiers.mac_cmd || modifiers.command {
        parts.push("Super");
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
        let track = egui::Color32::from_rgb(0x5C, 0x5C, 0x60);
        let track_hover = egui::Color32::from_rgb(0x6E, 0x6E, 0x72);
        {
            let v = ui.visuals_mut();
            v.widgets.inactive.bg_fill = track;
            v.widgets.inactive.weak_bg_fill = track;
            v.widgets.hovered.bg_fill = track_hover;
            v.widgets.hovered.weak_bg_fill = track_hover;
            v.widgets.active.bg_fill = track_hover;
            v.widgets.active.weak_bg_fill = track_hover;
            v.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(0x7C, 0x72, 0xC3, 0x99);
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
            if i.key_pressed(egui::Key::Escape) && self.capturing.is_none() {
                self.cancel_and_close();
            }
        });

        ui::apply_settings_visuals(ctx, UiTheme::Dark);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(SETTINGS_BG_DARK).inner_margin(0.0))
            .show(ctx, |ui| {
                self.draw_shell(ui);
            });
    }
}

/// Draw the settings form inside an egui window (annotate toolbar gear).
pub fn show_settings_window(ctx: &egui::Context, open: &mut bool, state: &mut EmbeddedSettings) {
    if !*open {
        return;
    }
    // Install settings widget colors *before* the Window Ui is created so
    // slider rails inherit opaque tracks (not annotate's transparent idle fill).
    ui::apply_settings_visuals(ctx, UiTheme::Dark);
    let screen = ctx.screen_rect();
    let default_size = egui::vec2(780.0, 520.0);
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
        .min_width(560.0)
        .min_height(360.0)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(SETTINGS_BG_DARK)
                .inner_margin(0.0),
        )
        .show(ctx, |ui| {
            state.inner.draw_shell(ui);
        });
    // Restore annotate chrome for the rest of the frame (toolbar, handles).
    let theme = state.config().theme;
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
        assert_eq!(format_key_combo(mods, Key::S), "Shift+Super+S");
    }

    #[test]
    fn parse_keycaps_maps_super_to_win() {
        assert_eq!(
            parse_keycaps("Shift+Super+X"),
            vec!["Shift".to_string(), "Win".to_string(), "X".to_string()]
        );
        assert!(parse_keycaps("").is_empty());
        assert!(parse_keycaps("—").is_empty());
    }

    #[test]
    fn dirty_detection_tracks_baseline() {
        let mut app = SettingsApp::new(Config::default());
        assert!(!app.is_dirty());
        app.cfg.stay_in_tray = true;
        assert!(app.is_dirty());
        app.baseline = app.cfg.clone();
        assert!(!app.is_dirty());
    }

    #[test]
    fn nav_breadcrumb_global() {
        assert_eq!(NavId::ShortcutsGlobal.breadcrumb(), "Shortcuts · Global");
        assert!(NavId::ShortcutsGlobal.is_shortcuts());
        assert!(!NavId::General.is_shortcuts());
    }
}
