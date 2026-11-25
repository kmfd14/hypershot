//! Settings window: dark two-pane chrome (sidebar + content + footer).
//!
//! Fixed, monitor-scaled size — never tiled / never auto-grows. Matches the
//! dark reference look: lavender accent, OK / Cancel / Apply.
//!
//! Sidebar keeps General, Capture, Overlay, Annotation Window, Saving,
//! File, About. Upload / S3 / History / Recording / Camera and the
//! entire SHORTCUTS group are intentionally omitted.

use std::path::PathBuf;

use anyhow::{bail, Result};
use eframe::egui::{self, Sense, Stroke, Vec2, ViewportBuilder};

use crate::config::{
    clamp_intensity, config_path, default_filename_pattern, AnnotateSessionMode, Config,
    SaveFileType, ToolbarPosition, UiTheme, APP_NAME, APP_VERSION,
};
use crate::desktop::{apply_desktop_integrations, maybe_spawn_tray};
use crate::icons::{paint_iconoir, IconId};
use crate::output::render_filename_pattern;
use crate::ui::{
    self, choice_button, contrast_checkbox, settings_button, settings_text_edit, SETTINGS_BG_DARK,
    SETTINGS_CONTROL_HEIGHT, SETTINGS_FOOTER_BTN, SETTINGS_FOOTER_H, SETTINGS_LABEL_DARK,
    SETTINGS_NAV_ACTIVE, SETTINGS_PRIMARY, SETTINGS_SECONDARY_DARK, SETTINGS_SEPARATOR_DARK,
    SETTINGS_SIDEBAR_W,
};

/// Design size at ~1080p; scaled to the monitor, then locked.
const SETTINGS_DESIGN: Vec2 = Vec2::new(820.0, 560.0);

/// Open the standalone settings window. Blocks until closed.
pub fn run_settings_window() -> Result<()> {
    ensure_display_available()?;

    let cfg = Config::load().unwrap_or_default();
    // Start at design size; first frame re-scales to the monitor and locks it.
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(format!("{APP_NAME} Settings"))
            .with_inner_size([SETTINGS_DESIGN.x, SETTINGS_DESIGN.y])
            .with_min_inner_size([SETTINGS_DESIGN.x, SETTINGS_DESIGN.y])
            .with_max_inner_size([SETTINGS_DESIGN.x, SETTINGS_DESIGN.y])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_always_on_top()
            // Hint floating dialog (helps some WMs; Wayland may still need size lock).
            .with_window_type(egui::X11WindowType::Dialog),
        centered: true,
        ..Default::default()
    };

    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            &format!("{APP_NAME} Settings"),
            options,
            Box::new(move |cc| {
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

/// Fixed Settings size: design aspect scaled to the monitor so content fits.
fn settings_fixed_size(screen: egui::Rect) -> Vec2 {
    let sw = screen.width().max(1.0);
    let sh = screen.height().max(1.0);
    // Fit design into ~72% of the work area; clamp so small/large monitors stay usable.
    let scale = (sw * 0.72 / SETTINGS_DESIGN.x)
        .min(sh * 0.72 / SETTINGS_DESIGN.y)
        .clamp(0.78, 1.15);
    Vec2::new(
        (SETTINGS_DESIGN.x * scale).round(),
        (SETTINGS_DESIGN.y * scale).round(),
    )
}

/// Keep Settings out of tiling: fixed size + not resizable + always-on-top.
fn enforce_fixed_settings_viewport(ctx: &egui::Context, size: Vec2) {
    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(size));
    ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(size));
    ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
        egui::WindowLevel::AlwaysOnTop,
    ));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavId {
    General,
    Capture,
    Overlay,
    AnnotationWindow,
    Saving,
    File,
    About,
}

impl NavId {
    fn breadcrumb(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Capture => "Capture",
            Self::Overlay => "Overlay",
            Self::AnnotationWindow => "Annotation Window",
            Self::Saving => "Saving",
            Self::File => "File",
            Self::About => "About",
        }
    }
}

#[derive(Clone, Copy)]
enum NavIcon {
    General,
    Capture,
    Overlay,
    AnnotationWindow,
    Saving,
    File,
    Info,
}

struct SettingsApp {
    cfg: Config,
    /// Snapshot used for dirty detection + Cancel restore.
    baseline: Config,
    save_dir_text: String,
    filename_pattern_text: String,
    status: String,
    close_requested: bool,
    nav: NavId,
    /// Locked outer size (monitor-scaled). Set on first frame.
    fixed_size: Option<Vec2>,
}

impl SettingsApp {
    fn new(cfg: Config) -> Self {
        let save_dir_text = cfg.save_directory.display().to_string();
        let filename_pattern_text = cfg.filename_pattern.clone();
        Self {
            baseline: cfg.clone(),
            cfg,
            save_dir_text,
            filename_pattern_text,
            status: String::new(),
            close_requested: false,
            nav: NavId::General,
            fixed_size: None,
        }
    }

    fn is_dirty(&self) -> bool {
        let mut draft = self.cfg.clone();
        draft.save_directory = PathBuf::from(self.save_dir_text.trim());
        draft.filename_pattern = self.filename_pattern_text.trim().to_string();
        draft != self.baseline
    }

    fn sync_text_into_cfg(&mut self) {
        self.cfg.save_directory = PathBuf::from(self.save_dir_text.trim());
        let pattern = self.filename_pattern_text.trim();
        self.cfg.filename_pattern = if pattern.is_empty() {
            default_filename_pattern()
        } else {
            pattern.to_string()
        };
        self.filename_pattern_text = self.cfg.filename_pattern.clone();
        if self.cfg.shortcut_bind_command.is_empty() {
            self.cfg.shortcut_bind_command = crate::config::default_shortcut_bind_command();
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
        self.filename_pattern_text = self.cfg.filename_pattern.clone();
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
            }
            NavId::AnnotationWindow => {
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
            NavId::File => {
                self.cfg.filename_pattern = defaults.filename_pattern.clone();
                self.filename_pattern_text = self.cfg.filename_pattern.clone();
                self.cfg.default_file_type = defaults.default_file_type;
            }
            NavId::About => {}
        }
    }

    fn draw_shell(&mut self, ui: &mut egui::Ui) {
        ui::apply_settings_visuals(ui.ctx(), UiTheme::Dark);

        // Fixed footer height (status may paint inside; does not grow the shell).
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
                (
                    NavId::AnnotationWindow,
                    "Annotation Window",
                    NavIcon::AnnotationWindow,
                ),
                (NavId::Saving, "Saving", NavIcon::Saving),
                (NavId::File, "File", NavIcon::File),
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
            }
        });
    }

    fn draw_content(&mut self, ui: &mut egui::Ui) {
        let label = SETTINGS_LABEL_DARK;
        let secondary = SETTINGS_SECONDARY_DARK;

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(ui.available_width());
            ui.set_max_height(ui.available_height());

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

            let body_h = ui.available_height().max(40.0);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(body_h)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        match self.nav {
                            NavId::General => self.page_general(ui, label, secondary),
                            NavId::Capture => self.page_capture(ui, label, secondary),
                            NavId::Overlay => self.page_overlay(ui, label, secondary),
                            NavId::AnnotationWindow => {
                                self.page_annotation_window(ui, label, secondary)
                            }
                            NavId::Saving => self.page_saving(ui, label, secondary),
                            NavId::File => self.page_file(ui, label, secondary),
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
                    egui::RichText::new("Open Config File")
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
        // Status fits inside the fixed footer; truncate visually via single line.
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
            "Run at Startup",
            "Start the tray when you log in.",
            label,
            secondary,
        );
        row_sep(ui);
        session_row(
            ui,
            &mut self.cfg.replace_default_screenshot,
            "Replace Default Screenshot",
            "Adds a launcher helper only — does not bind keys or steal Print Screen.",
            label,
            secondary,
        );
        row_sep(ui);
        session_row(
            ui,
            &mut self.cfg.stay_in_tray,
            "Stay in Tray",
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
            egui::RichText::new("Annotate Mode")
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
            egui::RichText::new("On-Screen Toolbar Position")
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
            "Show Window Title Bar",
            "Show the normal window title bar. Off makes the window borderless.",
            label,
            secondary,
        );
        row_sep(ui);
        session_row(
            ui,
            &mut self.cfg.follow_window_tiling,
            "Follow Window Tiling",
            "On: let your window manager place annotate. Off (default): center the window and keep it on top when the desktop allows. Settings itself never tiles.",
            label,
            secondary,
        );
        row_sep(ui);
        session_row(
            ui,
            &mut self.cfg.esc_to_exit,
            "Esc to Exit",
            "When on (default), Escape closes annotate without save/copy (after clearing a draft). Exit always works.",
            label,
            secondary,
        );
    }

    fn page_annotation_window(
        &mut self,
        ui: &mut egui::Ui,
        label: egui::Color32,
        secondary: egui::Color32,
    ) {
        ui.label(
            egui::RichText::new(
                "Preview annotate window only. The toolbar stays solid.",
            )
            .color(secondary)
            .size(11.0),
        );
        ui.add_space(12.0);
        session_row(
            ui,
            &mut self.cfg.window_transparency,
            "Window Transparency",
            "Softens the preview window. Some desktops ignore transparency.",
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
            "Window Background Blur",
            "Frosts the preview area. Does not sample the desktop behind the window.",
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
            egui::RichText::new("Save Directory")
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
            "Save to Clipboard by Default",
            "Copy each save unless you pass --clipboard or --no-clipboard.",
            label,
            secondary,
        );
    }

    fn page_file(
        &mut self,
        ui: &mut egui::Ui,
        label: egui::Color32,
        secondary: egui::Color32,
    ) {
        ui.label(
            egui::RichText::new("Default File Name")
                .color(label)
                .size(12.5)
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "Pattern used when HyperShot picks a save name. Write plain text plus date/time codes below.",
            )
            .color(secondary)
            .size(11.0),
        );
        ui.add_space(8.0);
        settings_text_edit(
            ui,
            &mut self.filename_pattern_text,
            f32::INFINITY,
            "screenshot_%d%m%y",
        );
        ui.add_space(8.0);
        let preview =
            render_filename_pattern(self.filename_pattern_text.trim(), self.cfg.default_file_type);
        ui.label(
            egui::RichText::new(format!("Preview right now: {preview}"))
                .color(SETTINGS_PRIMARY)
                .size(12.0)
                .monospace(),
        );
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new("Default File Type")
                .color(label)
                .size(12.5)
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Format used for automatic names and the Save dialog default.")
                .color(secondary)
                .size(11.0),
        );
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            for file_type in SaveFileType::ALL {
                if choice_button(ui, self.cfg.default_file_type == file_type, file_type.label())
                    .clicked()
                {
                    self.cfg.default_file_type = file_type;
                }
            }
        });
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new("Codes You Can Use")
                .color(label)
                .size(12.5)
                .strong(),
        );
        ui.add_space(6.0);
        for (code, meaning) in [
            ("%d", "Day of the month (01-31)"),
            ("%m", "Month (01-12)"),
            ("%y", "Year, two digits (26 for 2026)"),
            ("%Y", "Year, four digits (2026)"),
            ("%H", "Hour, 24-hour clock (00-23)"),
            ("%M", "Minute (00-59)"),
            ("%S", "Second (00-59)"),
            ("%%", "A literal % character"),
        ] {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(code)
                        .color(label)
                        .size(12.0)
                        .monospace()
                        .strong(),
                );
                ui.label(egui::RichText::new(meaning).color(secondary).size(11.5));
            });
        }
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new("Example")
                .color(label)
                .size(12.5)
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Pattern:  screenshot_%d%m%y")
                .color(secondary)
                .size(11.5)
                .monospace(),
        );
        let becomes = format!(
            "Becomes:  screenshot_251125.{}",
            self.cfg.default_file_type.extension()
        );
        ui.label(
            egui::RichText::new(becomes)
                .color(secondary)
                .size(11.5)
                .monospace(),
        );
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(
                "The chosen file type is added if you leave the extension off. If that name already exists, HyperShot tries _2, then _3, and so on.",
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
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new("Changelog")
                .color(label)
                .size(14.0)
                .strong(),
        );
        ui.add_space(8.0);
        for (ver, notes) in CHANGELOG_ENTRIES {
            ui.label(
                egui::RichText::new(*ver)
                    .color(label)
                    .size(12.5)
                    .strong(),
            );
            ui.add_space(2.0);
            for note in *notes {
                ui.label(
                    egui::RichText::new(format!("· {note}"))
                        .color(secondary)
                        .size(11.5),
                );
            }
            ui.add_space(10.0);
        }
    }
}

/// Recent version notes for Settings → About (Title Case headers; scannable bullets).
const CHANGELOG_ENTRIES: &[(&str, &[&str])] = &[
    (
        "1.9.0",
        &[
            "File page with Default File Type (PNG / JPEG / WebP)",
            "About Changelog section",
            "Title Case settings titles and sidebar labels",
            "Centered tools in the annotate toolbar",
        ],
    ),
    (
        "1.8.0",
        &[
            "Annotation Window for transparency and blur",
            "Clearer file name tokens and examples",
            "Iconoir sidebar navigation icons",
        ],
    ),
    (
        "1.7.0",
        &[
            "Fixed Settings window size (no tiling)",
            "Default file name pattern wiring",
            "Settings polish and left-aligned checkboxes",
        ],
    ),
    (
        "1.6.0",
        &["Settings two-pane restack and layout polish"],
    ),
    (
        "1.5.0",
        &[
            "Dark two-pane Settings chrome",
            "High-contrast Iconoir toolbar textures",
        ],
    ),
];

fn open_path_in_desktop(path: &std::path::Path) -> bool {
    use std::process::Command;
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
    const ICON: f32 = 18.0;
    let height = 32.0;
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
        egui::pos2(rect.left() + 14.0 + ICON * 0.5, rect.center().y),
        egui::vec2(ICON, ICON),
    );
    let ink = if active { label } else { secondary };
    paint_iconoir(&painter, icon_rect, ink, nav_icon_id(icon));

    painter.text(
        egui::pos2(rect.left() + 14.0 + ICON + 10.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(if active { 12.5 } else { 12.0 }),
        if active { label } else { secondary },
    );

    response.clicked()
}

fn nav_icon_id(icon: NavIcon) -> IconId {
    match icon {
        NavIcon::General => IconId::Settings,
        NavIcon::Capture => IconId::Camera,
        NavIcon::Overlay => IconId::Crop,
        NavIcon::AnnotationWindow => IconId::Preview,
        NavIcon::Saving => IconId::Folder,
        NavIcon::File => IconId::Text,
        NavIcon::Info => IconId::InfoCircle,
    }
}

fn reset_defaults_button(ui: &mut egui::Ui) -> egui::Response {
    let label = egui::RichText::new("Reset to Defaults")
        .color(SETTINGS_LABEL_DARK)
        .size(12.0);
    let btn = egui::Button::new(label)
        .fill(egui::Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0_f32, egui::Color32::from_rgb(0x5A, 0x5A, 0x60)))
        .corner_radius(egui::CornerRadius::same(14))
        .min_size(egui::vec2(128.0, 26.0));
    ui.add(btn).on_hover_text("Reset this section to defaults")
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

/// Checkbox in a fixed left column; title + hint wrap in the remaining width.
fn session_row(
    ui: &mut egui::Ui,
    value: &mut bool,
    title: &str,
    hint: &str,
    label: egui::Color32,
    secondary: egui::Color32,
) {
    const CHECK_COL: f32 = 28.0;
    const GAP: f32 = 12.0;
    let row_w = ui.available_width();
    let text_w = (row_w - CHECK_COL - GAP).max(120.0);
    ui.horizontal(|ui| {
        ui.set_min_height(SETTINGS_CONTROL_HEIGHT.max(40.0));
        // Left column: one aligned checkbox edge for every row.
        ui.allocate_ui_with_layout(
            egui::vec2(CHECK_COL, ui.available_height().max(40.0)),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.add_space(2.0);
                contrast_checkbox(ui, value);
            },
        );
        ui.add_space(GAP);
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
    });
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.close_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Lock size every frame so a tiling WM cannot slowly stretch height.
        let size = *self
            .fixed_size
            .get_or_insert_with(|| settings_fixed_size(ctx.screen_rect()));
        enforce_fixed_settings_viewport(ctx, size);

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.cancel_and_close();
            }
        });

        ui::apply_settings_visuals(ctx, UiTheme::Dark);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(SETTINGS_BG_DARK).inner_margin(0.0))
            .show(ctx, |ui| {
                // Clip shell to the fixed viewport so content never requests growth.
                ui.set_min_size(ui.available_size());
                ui.set_max_size(ui.available_size());
                self.draw_shell(ui);
            });
    }
}

/// Draw the settings form inside an egui window (annotate toolbar gear).
pub fn show_settings_window(ctx: &egui::Context, open: &mut bool, state: &mut EmbeddedSettings) {
    if !*open {
        return;
    }
    ui::apply_settings_visuals(ctx, UiTheme::Dark);
    let screen = ctx.screen_rect();
    let size = settings_fixed_size(screen);
    let default_pos = egui::pos2(
        (screen.width() - size.x) * 0.5,
        (screen.height() - size.y) * 0.5,
    );
    egui::Window::new("Settings")
        .open(open)
        .resizable(false)
        .collapsible(false)
        .default_size(size)
        .min_size(size)
        .max_size(size)
        .default_pos(default_pos)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(SETTINGS_BG_DARK)
                .inner_margin(0.0),
        )
        .show(ctx, |ui| {
            ui.set_min_size(size);
            ui.set_max_size(size);
            state.inner.draw_shell(ui);
        });
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
    fn dirty_detection_tracks_baseline() {
        let mut app = SettingsApp::new(Config::default());
        assert!(!app.is_dirty());
        app.cfg.stay_in_tray = true;
        assert!(app.is_dirty());
        app.baseline = app.cfg.clone();
        assert!(!app.is_dirty());
    }

    #[test]
    fn dirty_tracks_filename_pattern() {
        let mut app = SettingsApp::new(Config::default());
        assert!(!app.is_dirty());
        app.filename_pattern_text = "shot_%Y%m%d".into();
        assert!(app.is_dirty());
    }

    #[test]
    fn nav_pages_are_trimmed() {
        assert_eq!(NavId::General.breadcrumb(), "General");
        assert_eq!(NavId::Overlay.breadcrumb(), "Overlay");
        assert_eq!(
            NavId::AnnotationWindow.breadcrumb(),
            "Annotation Window"
        );
        assert_eq!(NavId::File.breadcrumb(), "File");
        assert_eq!(NavId::About.breadcrumb(), "About");
    }

    #[test]
    fn dirty_tracks_default_file_type() {
        let mut app = SettingsApp::new(Config::default());
        assert!(!app.is_dirty());
        app.cfg.default_file_type = crate::config::SaveFileType::Jpeg;
        assert!(app.is_dirty());
    }

    #[test]
    fn changelog_lists_current_version() {
        assert!(
            CHANGELOG_ENTRIES
                .iter()
                .any(|(ver, _)| *ver == APP_VERSION),
            "About Changelog must include APP_VERSION {APP_VERSION}"
        );
    }

    #[test]
    fn fixed_size_scales_and_clamps() {
        let large = settings_fixed_size(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(3840.0, 2160.0),
        ));
        assert!(large.x <= SETTINGS_DESIGN.x * 1.15 + 1.0);
        assert!(large.y <= SETTINGS_DESIGN.y * 1.15 + 1.0);
        let small = settings_fixed_size(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1280.0, 720.0),
        ));
        assert!(small.x >= SETTINGS_DESIGN.x * 0.78 - 1.0);
        assert!(small.y >= SETTINGS_DESIGN.y * 0.78 - 1.0);
    }
}
