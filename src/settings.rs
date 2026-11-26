//! Settings window: dark two-pane chrome (sidebar + content + footer).
//!
//! Monitor-scaled default size; user-resizable. Content stays clipped to the
//! viewport so tiling WMs cannot auto-grow the shell. Dialog + always-on-top
//! keep it out of tiling. Matches the dark reference look: lavender accent,
//! OK / Cancel / Apply, rounded corners, flat minimal outline.
//!
//! Sidebar keeps General, Capture, Overlay, Annotation Window, Saving,
//! File, About. Upload / S3 / History / Recording / Camera and the
//! entire SHORTCUTS group are intentionally omitted.

use std::path::PathBuf;

use anyhow::{bail, Result};
use eframe::egui::{self, CornerRadius, Sense, Stroke, Vec2, ViewportBuilder};

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

/// Design size at ~1080p; scaled to the monitor as the opening default.
const SETTINGS_DESIGN: Vec2 = Vec2::new(820.0, 560.0);

/// Shared dark title strip — same for `hypershot settings` and annotate-toolbar Settings.
const SETTINGS_TITLE_H: f32 = 36.0;

/// Minimal square close control in the title strip.
const SETTINGS_CLOSE_BTN: f32 = 22.0;

/// Outer shell radius — matches annotate / settings visuals (`window_corner_radius`).
const SETTINGS_CORNER: u8 = 10;

fn settings_window_title() -> String {
    format!("{APP_NAME} Settings")
}

fn settings_corner_radius() -> CornerRadius {
    CornerRadius::same(SETTINGS_CORNER)
}

/// Shared viewport chrome for the standalone Settings window (and any deferred host).
fn settings_viewport_builder(size: Vec2, min_size: Vec2) -> ViewportBuilder {
    ViewportBuilder::default()
        .with_title(settings_window_title())
        .with_inner_size([size.x, size.y])
        .with_min_inner_size([min_size.x, min_size.y])
        .with_resizable(true)
        .with_maximize_button(false)
        .with_minimize_button(false)
        // Custom title strip inside the shell — keep OS chrome off so both entry
        // points match (standalone CLI and annotate gear). Transparent so the
        // painted rounded shell can show through (no hard rectangular outline).
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_window_type(egui::X11WindowType::Dialog)
}

/// Open the standalone settings window. Blocks until closed.
pub fn run_settings_window() -> Result<()> {
    ensure_display_available()?;

    let cfg = Config::load().unwrap_or_default();
    // Start at design size; first frame re-scales to the monitor as the default.
    let options = eframe::NativeOptions {
        viewport: settings_viewport_builder(SETTINGS_DESIGN, settings_min_size(SETTINGS_DESIGN)),
        centered: true,
        ..Default::default()
    };

    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            &settings_window_title(),
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

/// Default Settings size: design aspect scaled to the monitor so content fits.
fn settings_default_size(screen: egui::Rect) -> Vec2 {
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

/// Smallest useful Settings size (still fits sidebar + footer).
fn settings_min_size(default: Vec2) -> Vec2 {
    Vec2::new(
        (default.x * 0.82).max(560.0).round(),
        (default.y * 0.82).max(420.0).round(),
    )
}

/// Keep Settings out of tiling without fighting user resize: dialog chrome +
/// always-on-top + min size. Do **not** re-lock InnerSize/MaxInnerSize each frame
/// (that blocked resize and was the old “fixed size” approach).
fn apply_settings_viewport_chrome(ctx: &egui::Context, min_size: Vec2) {
    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(min_size));
    ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
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
    /// Opening default size (monitor-scaled). Computed once on first frame.
    default_size: Option<Vec2>,
    /// Whether the one-shot `InnerSize` open command was already sent.
    initial_size_applied: bool,
    /// When true, title-bar drag issues `ViewportCommand::StartDrag` (standalone
    /// Settings). When false (annotate overlay), the egui `Window` owns dragging.
    native_viewport: bool,
}

impl SettingsApp {
    fn new(cfg: Config) -> Self {
        Self::new_with_host(cfg, true)
    }

    fn new_with_host(cfg: Config, native_viewport: bool) -> Self {
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
            default_size: None,
            initial_size_applied: false,
            native_viewport,
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

        // Panel fills stay transparent so the outer rounded shell is what you see
        // in the corners (no square overpaint on the transparent viewport).
        egui::TopBottomPanel::top("hypershot_settings_title")
            .exact_height(SETTINGS_TITLE_H)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show_inside(ui, |ui| {
                if draw_settings_title_bar(ui, self.native_viewport) {
                    self.cancel_and_close();
                }
            });

        // Fixed footer height (status may paint inside; does not grow the shell).
        egui::TopBottomPanel::bottom("hypershot_settings_footer")
            .exact_height(SETTINGS_FOOTER_H + 8.0)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
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
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
                    .inner_margin(egui::Margin::symmetric(10, 12)),
            )
            .show_inside(ui, |ui| {
                self.draw_sidebar(ui);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
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
        "1.11.0",
        &[
            "Resizable Settings window with title-bar drag",
            "Rounded Settings corners and flatter minimal outline",
        ],
    ),
    (
        "1.10.0",
        &[
            "Copy closes annotate immediately (no toast delay)",
            "Compact system-themed Copied to Clipboard popup (Copy / Enter only)",
        ],
    ),
    (
        "1.9.0",
        &[
            "File page with Default File Type (PNG / JPEG / WebP)",
            "About Changelog section",
            "Title Case settings titles and sidebar labels",
            "Centered tools in the annotate toolbar",
            "Matching Settings chrome from CLI and annotate gear",
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
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Transparent clear so rounded corners are not boxed by a hard rect.
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.close_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        let default = *self
            .default_size
            .get_or_insert_with(|| settings_default_size(ctx.screen_rect()));
        let min_size = settings_min_size(default);
        apply_settings_viewport_chrome(ctx, min_size);
        // First frame only: open at the monitor-scaled default. Later frames
        // leave InnerSize alone so the user can resize freely.
        if !self.initial_size_applied {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(default));
            self.initial_size_applied = true;
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.cancel_and_close();
            }
        });

        ui::apply_settings_visuals(ctx, UiTheme::Dark);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(SETTINGS_BG_DARK)
                    .corner_radius(settings_corner_radius())
                    .stroke(Stroke::NONE)
                    .inner_margin(0.0),
            )
            .show(ctx, |ui| {
                // Clip to the current viewport so content never requests growth
                // (avoids the old tiling auto-grow bug without freezing size).
                let avail = ui.available_size();
                ui.set_min_size(avail);
                ui.set_max_size(avail);
                let clip = ui.max_rect();
                ui.set_clip_rect(clip);
                self.draw_shell(ui);
            });
    }
}

/// Shared title strip + minimal close (same chrome for CLI and annotate entry).
///
/// Returns `true` when the close control is clicked (caller should Cancel).
fn draw_settings_title_bar(ui: &mut egui::Ui, native_viewport: bool) -> bool {
    let full = ui.available_rect_before_wrap();
    // Native: Sense::drag so StartDrag fires on press (Wayland needs the
    // pointer serial; click_and_drag delays past the compositor timeout).
    // Embedded: Sense::hover so the egui Window Area can own drag-to-move.
    let sense = if native_viewport {
        Sense::drag()
    } else {
        Sense::hover()
    };
    let (bar, bar_resp) = ui.allocate_exact_size(
        egui::vec2(full.width(), SETTINGS_TITLE_H),
        sense,
    );
    let painter = ui.painter_at(bar);
    // Title strip sits on the outer rounded shell — no extra fill (avoids squaring corners).
    painter.hline(
        bar.x_range(),
        bar.bottom() - 0.5,
        Stroke::new(1.0_f32, SETTINGS_SEPARATOR_DARK),
    );

    let close_rect = egui::Rect::from_center_size(
        egui::pos2(bar.right() - 14.0 - SETTINGS_CLOSE_BTN * 0.5, bar.center().y),
        egui::vec2(SETTINGS_CLOSE_BTN, SETTINGS_CLOSE_BTN),
    );
    let close_id = ui.id().with("hypershot_settings_close");
    let close_resp = ui.interact(close_rect, close_id, Sense::click());

    // Standalone borderless window: drag from the title strip on press.
    // Skip when the pointer is on the close control.
    if native_viewport
        && bar_resp.drag_started_by(egui::PointerButton::Primary)
        && !close_resp.contains_pointer()
    {
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    let title = settings_window_title();
    painter.text(
        egui::pos2(bar.left() + 14.0, bar.center().y),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(13.0),
        SETTINGS_LABEL_DARK,
    );

    let close_hover = close_resp.hovered();
    if close_hover {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        painter.rect_filled(
            close_rect,
            4.0,
            egui::Color32::from_rgb(0x2A, 0x2B, 0x30),
        );
    }
    // Minimal × — quiet stroke, lavender on hover (matches Settings accent).
    let ink = if close_hover {
        SETTINGS_PRIMARY
    } else {
        SETTINGS_SECONDARY_DARK
    };
    let pad = 6.5_f32;
    let stroke = Stroke::new(1.25_f32, ink);
    painter.line_segment(
        [
            egui::pos2(close_rect.left() + pad, close_rect.top() + pad),
            egui::pos2(close_rect.right() - pad, close_rect.bottom() - pad),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(close_rect.right() - pad, close_rect.top() + pad),
            egui::pos2(close_rect.left() + pad, close_rect.bottom() - pad),
        ],
        stroke,
    );
    close_resp.on_hover_text("Close").clicked()
}

/// Draw the settings form inside an egui window (annotate toolbar gear).
///
/// Uses the same default size, dark chrome, and custom title strip as
/// [`run_settings_window`] so both entry points look identical.
pub fn show_settings_window(ctx: &egui::Context, open: &mut bool, state: &mut EmbeddedSettings) {
    if !*open {
        return;
    }

    ui::apply_settings_visuals(ctx, UiTheme::Dark);
    let screen = ctx.screen_rect();
    let size = settings_default_size(screen);
    let min_size = settings_min_size(size);
    let default_pos = egui::pos2(
        (screen.width() - size.x) * 0.5,
        (screen.height() - size.y) * 0.5,
    );
    egui::Window::new(settings_window_title())
        .id(egui::Id::new("hypershot_settings_embedded"))
        .title_bar(false)
        .resizable(true)
        .collapsible(false)
        .default_size(size)
        .min_size(min_size)
        .default_pos(default_pos)
        .frame(
            egui::Frame::new()
                .fill(SETTINGS_BG_DARK)
                .stroke(Stroke::NONE)
                .corner_radius(settings_corner_radius())
                .inner_margin(0.0),
        )
        .show(ctx, |ui| {
            // Clip to the window so content cannot force auto-grow.
            let avail = ui.available_size();
            ui.set_min_size(avail);
            ui.set_max_size(avail);
            state.inner.draw_shell(ui);
        });

    if state.inner.close_requested {
        *open = false;
        state.inner.close_requested = false;
    }

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
            // Hosted inside annotate/onscreen — do not StartDrag the capture viewport.
            inner: SettingsApp::new_with_host(cfg, false),
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
    fn settings_entry_points_share_title_and_title_bar() {
        assert_eq!(settings_window_title(), format!("{APP_NAME} Settings"));
        assert!(SETTINGS_TITLE_H >= 32.0);
        assert!(SETTINGS_CLOSE_BTN < SETTINGS_TITLE_H);
        let standalone = SettingsApp::new(Config::default());
        assert!(standalone.native_viewport);
        let embedded = EmbeddedSettings::load();
        assert!(!embedded.inner.native_viewport);
    }

    #[test]
    fn default_size_scales_and_clamps() {
        let large = settings_default_size(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(3840.0, 2160.0),
        ));
        assert!(large.x <= SETTINGS_DESIGN.x * 1.15 + 1.0);
        assert!(large.y <= SETTINGS_DESIGN.y * 1.15 + 1.0);
        let small = settings_default_size(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1280.0, 720.0),
        ));
        assert!(small.x >= SETTINGS_DESIGN.x * 0.78 - 1.0);
        assert!(small.y >= SETTINGS_DESIGN.y * 0.78 - 1.0);
        let min = settings_min_size(small);
        assert!(min.x <= small.x);
        assert!(min.y <= small.y);
        assert!(min.x >= 560.0);
        assert!(min.y >= 420.0);
    }
}
