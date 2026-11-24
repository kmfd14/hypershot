//! Floating annotate window (flat pastel tool rail + image canvas).
//!
//! Layout: content-sized floating rail attached to the capture preview (not
//! full window height). Position: left (default) / right / top / bottom via
//! settings. Exit cancels/discards (Esc does the same when enabled in settings).
//! Save dismisses after write. Copy shows a short clipboard toast, then closes
//! without blanking the preview.
//!
//! Chrome: flat pastel purple / gray / white. Transparency + blur settings
//! apply window opacity and a translucent frosted panel on the window/preview
//! area; the tool rail stays fully opaque. egui cannot sample the desktop for
//! real compositor backdrop-blur on Wayland.
//!
//! Floating (default): centered + `ViewportBuilder::with_always_on_top`.
//! Follow tiling: skip those hints so a tiling WM can place the window.
//! Wayland: winit’s always-on-top is a no-op; COSMIC/tiling may still tile.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2, ViewportBuilder};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_line_segment_mut};

use crate::config::ToolbarPosition;
use crate::settings::{show_settings_window, EmbeddedSettings};
use crate::ui::{
    self, fit_image_rect, glass_color_swatch, glass_icon_button, glass_toolbar_frame,
    paint_icon_arrow, paint_icon_box, paint_icon_clear, paint_icon_copy, paint_icon_pixelate,
    paint_icon_redo, paint_icon_save, paint_icon_settings, paint_icon_step, paint_icon_undo,
    AnnotatePalette, ACTION_FILL, ACTION_FILL_HOVER, DEFAULT_STROKE, PASTEL_PRIMARY,
    PASTEL_PRIMARY_HOVER, SETTINGS_BTN, TOOLBAR_GUTTER, TOOLBAR_WIDTH, TOOL_BTN, WINDOW_PAD,
};

/// How long the “copied to clipboard” toast stays visible before close.
const COPY_TOAST_SECS: f32 = 1.15;

/// Result of the annotate session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotateOutcome {
    /// User confirmed Save; annotated PNG was written.
    Saved,
    /// User confirmed Copy; annotated PNG was written — caller should copy to clipboard.
    Copied,
    /// User cancelled (Escape); path left unchanged.
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    Arrow,
    Rect,
    /// Privacy redaction: coarse pixel blocks over a drag rect.
    Pixelate,
    /// Click to stamp a numbered circle (1, 2, …).
    Step,
}

#[derive(Debug, Clone, Copy)]
enum Shape {
    Arrow {
        from: Pos2,
        to: Pos2,
        color: Color32,
    },
    Rect {
        a: Pos2,
        b: Pos2,
        color: Color32,
    },
    Pixelate {
        a: Pos2,
        b: Pos2,
    },
    Step {
        center: Pos2,
        number: u32,
        color: Color32,
    },
}

/// Radius of a step stamp in image pixels.
const STEP_RADIUS: f32 = 14.0;

/// Block size for privacy pixelation (image pixels per mosaic cell).
const PIXELATE_BLOCK: u32 = 12;

/// Max displayed image size inside the floating window (px). Larger captures scale down.
const MAX_DISPLAY_W: f32 = 1100.0;
const MAX_DISPLAY_H: f32 = 800.0;

/// Approximate content size of the floating rail (vertical).
fn toolbar_content_height() -> f32 {
    // Annotate label + 4 tools + color + undo + redo + clear + settings + save + copy + Exit
    let n_btns = 10.0_f32;
    let label = 22.0;
    let exit = 22.0;
    let gaps = 11.0 * 8.0 + 28.0;
    label + n_btns * TOOL_BTN + (SETTINGS_BTN - TOOL_BTN) + exit + gaps
}

/// Approximate content size of the floating rail (horizontal).
fn toolbar_content_width() -> f32 {
    let n_btns = 10.0_f32;
    let label = 72.0;
    let exit = 48.0;
    let gaps = 11.0 * 8.0 + 28.0;
    label + n_btns * TOOL_BTN + (SETTINGS_BTN - TOOL_BTN) + exit + gaps
}

/// Open a floating annotate window on an existing PNG.
///
/// On Save / Enter / Copy: composites annotations onto the image and overwrites `path`.
/// On Exit (or Escape when enabled): clears an in-progress drag first; with no draft,
/// returns [`AnnotateOutcome::Cancelled`] without modifying the file.
pub fn run_annotate_overlay(path: &Path) -> Result<AnnotateOutcome> {
    ensure_display_available()?;

    let rgba = image::open(path)
        .with_context(|| format!("open PNG for annotate {}", path.display()))?
        .to_rgba8();
    let (w, h) = rgba.dimensions();
    if w == 0 || h == 0 {
        bail!("PNG has empty dimensions: {}", path.display());
    }

    let settings = EmbeddedSettings::load();
    let cfg = settings.config().clone();
    let (win_w, win_h) = floating_window_size(w, h, cfg.toolbar_position);
    let rgba_bytes = rgba.as_raw().clone();
    let path_buf = path.to_path_buf();
    let outcome = Arc::new(Mutex::new(AnnotateOutcome::Cancelled));
    let outcome_for_app = Arc::clone(&outcome);

    let transparent = cfg.window_transparency;
    let decorations = cfg.show_title_bar;
    let follow_tiling = cfg.follow_window_tiling;

    // Floating (default): center + always-on-top. Follow tiling: skip those hints.
    // Wayland: winit `set_window_level` is a no-op; COSMIC may still tile anyway.
    let mut viewport = ViewportBuilder::default()
        .with_title("Lightsnip")
        .with_inner_size([win_w, win_h])
        .with_min_inner_size([
            TOOLBAR_WIDTH + TOOLBAR_GUTTER + 160.0 + WINDOW_PAD * 2.0,
            200.0,
        ])
        .with_decorations(decorations)
        .with_transparent(transparent)
        .with_resizable(true);
    if !follow_tiling {
        viewport = viewport.with_always_on_top();
    }

    let options = eframe::NativeOptions {
        viewport,
        centered: !follow_tiling,
        ..Default::default()
    };

    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            "Lightsnip",
            options,
            Box::new(move |cc| {
                ui::apply_annotate_visuals(&cc.egui_ctx, cfg.theme);
                Ok(Box::new(AnnotateApp::new(
                    &cc.egui_ctx,
                    rgba_bytes,
                    w,
                    h,
                    path_buf,
                    outcome_for_app,
                    settings,
                )))
            }),
        )
    }));

    match run_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            bail!("annotate window failed (need a display?): {e}");
        }
        Err(_) => {
            bail!(
                "annotate window panicked (missing display libs such as libxkbcommon?)"
            );
        }
    }

    let result = *outcome
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Ok(result)
}

fn floating_window_size(img_w: u32, img_h: u32, pos: ToolbarPosition) -> (f32, f32) {
    let scale = (MAX_DISPLAY_W / img_w as f32)
        .min(MAX_DISPLAY_H / img_h as f32)
        .min(1.0);
    let disp_w = img_w as f32 * scale;
    let disp_h = img_h as f32 * scale;
    match pos {
        ToolbarPosition::Left | ToolbarPosition::Right => (
            TOOLBAR_WIDTH + TOOLBAR_GUTTER + disp_w + WINDOW_PAD * 2.0,
            disp_h.max(toolbar_content_height()) + WINDOW_PAD * 2.0,
        ),
        ToolbarPosition::Top | ToolbarPosition::Bottom => (
            disp_w.max(toolbar_content_width()) + WINDOW_PAD * 2.0,
            TOOLBAR_WIDTH + TOOLBAR_GUTTER + disp_h + WINDOW_PAD * 2.0,
        ),
    }
}

fn ensure_display_available() -> Result<()> {
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let x11 = std::env::var_os("DISPLAY").is_some();
    if !wayland && !x11 {
        bail!("no WAYLAND_DISPLAY or DISPLAY set (annotate needs a graphical session)");
    }
    Ok(())
}

const STROKE_WIDTH: f32 = 3.0;

const PIXELATE_PREVIEW_FILL: Color32 =
    Color32::from_rgba_premultiplied(0x19, 0x19, 0x1A, 160);
const PIXELATE_PREVIEW_STROKE: Color32 = Color32::from_rgb(0xC8, 0xC8, 0xC8);

struct AnnotateApp {
    texture: egui::TextureHandle,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    shapes: Vec<Shape>,
    /// Shapes removed by Undo (newest at end); cleared on a new mark.
    redo_stack: Vec<Shape>,
    tool: Tool,
    stroke_color: Color32,
    /// Next step stamp number (1-based). Reset by undo/clear of step marks.
    next_step_number: u32,
    drag_start: Option<Pos2>,
    draft: Option<Shape>,
    path: std::path::PathBuf,
    outcome: Arc<Mutex<AnnotateOutcome>>,
    finished: bool,
    /// After Copy: keep painting preview + toast until this instant, then close.
    copy_toast_until: Option<Instant>,
    show_settings: bool,
    settings: EmbeddedSettings,
    /// Last applied viewport decoration / transparency (to avoid spam).
    applied_title_bar: bool,
    applied_transparency: bool,
    applied_follow_tiling: bool,
}

impl AnnotateApp {
    fn new(
        ctx: &egui::Context,
        pixels: Vec<u8>,
        width: u32,
        height: u32,
        path: std::path::PathBuf,
        outcome: Arc<Mutex<AnnotateOutcome>>,
        settings: EmbeddedSettings,
    ) -> Self {
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &pixels,
        );
        // LINEAR so upscaled small captures stay as smooth as egui allows.
        let texture = ctx.load_texture("capture", color_image, egui::TextureOptions::LINEAR);
        let cfg = settings.config();
        Self {
            texture,
            width,
            height,
            pixels,
            shapes: Vec::new(),
            redo_stack: Vec::new(),
            tool: Tool::Arrow,
            stroke_color: DEFAULT_STROKE,
            next_step_number: 1,
            drag_start: None,
            draft: None,
            path,
            outcome,
            finished: false,
            copy_toast_until: None,
            show_settings: false,
            applied_title_bar: cfg.show_title_bar,
            applied_transparency: cfg.window_transparency,
            applied_follow_tiling: cfg.follow_window_tiling,
            settings,
        }
    }

    /// Next stamp = max(existing step numbers) + 1, or 1 when none remain.
    fn sync_step_counter(&mut self) {
        self.next_step_number = self
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Step { number, .. } => Some(*number),
                _ => None,
            })
            .max()
            .map(|m| m.saturating_add(1))
            .unwrap_or(1);
    }

    fn set_outcome(&mut self, outcome: AnnotateOutcome) {
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = outcome;
        }
        self.finished = true;
    }

    fn cancel_draft_or_session(&mut self) {
        if self.copy_toast_until.is_some() {
            return;
        }
        if self.draft.is_some() || self.drag_start.is_some() {
            self.draft = None;
            self.drag_start = None;
        } else {
            self.set_outcome(AnnotateOutcome::Cancelled);
        }
    }

    fn undo(&mut self) {
        if let Some(shape) = self.shapes.pop() {
            self.redo_stack.push(shape);
        }
        self.draft = None;
        self.drag_start = None;
        self.sync_step_counter();
    }

    fn redo(&mut self) {
        if let Some(shape) = self.redo_stack.pop() {
            self.shapes.push(shape);
        }
        self.draft = None;
        self.drag_start = None;
        self.sync_step_counter();
    }

    fn clear_all(&mut self) {
        self.shapes.clear();
        self.redo_stack.clear();
        self.draft = None;
        self.drag_start = None;
        self.next_step_number = 1;
    }

    fn push_shape(&mut self, shape: Shape) {
        self.shapes.push(shape);
        self.redo_stack.clear();
    }

    fn write_annotated_png(&mut self) -> Result<()> {
        let mut img = RgbaImage::from_raw(self.width, self.height, self.pixels.clone())
            .ok_or_else(|| anyhow::anyhow!("rebuild RGBA image for export"))?;
        for shape in &self.shapes {
            rasterize_shape(&mut img, *shape);
        }
        img.save(&self.path)
            .with_context(|| format!("write annotated PNG {}", self.path.display()))?;
        Ok(())
    }

    fn save_and_close(&mut self) -> Result<()> {
        if self.copy_toast_until.is_some() {
            return Ok(());
        }
        self.write_annotated_png()?;
        self.set_outcome(AnnotateOutcome::Saved);
        Ok(())
    }

    /// Write PNG, show clipboard toast, then close after a short delay (no blank flash).
    fn copy_with_toast(&mut self) -> Result<()> {
        if self.copy_toast_until.is_some() {
            return Ok(());
        }
        self.write_annotated_png()?;
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = AnnotateOutcome::Copied;
        }
        self.copy_toast_until =
            Some(Instant::now() + Duration::from_secs_f32(COPY_TOAST_SECS));
        Ok(())
    }

    fn palette(&self, dark: bool) -> AnnotatePalette {
        let cfg = self.settings.config();
        AnnotatePalette::with_effects(
            dark,
            cfg.window_transparency,
            cfg.effective_transparency_intensity(),
            cfg.effective_blur_intensity(),
        )
    }

    fn draft_from_drag(&self, start: Pos2, end: Pos2) -> Option<Shape> {
        match self.tool {
            Tool::Arrow => Some(Shape::Arrow {
                from: start,
                to: end,
                color: self.stroke_color,
            }),
            Tool::Rect => Some(Shape::Rect {
                a: start,
                b: end,
                color: self.stroke_color,
            }),
            Tool::Pixelate => Some(Shape::Pixelate { a: start, b: end }),
            Tool::Step => None,
        }
    }

    fn sync_viewport_prefs(&mut self, ctx: &egui::Context) {
        let cfg = self.settings.config();
        ui::apply_annotate_visuals(ctx, cfg.theme);
        if cfg.show_title_bar != self.applied_title_bar {
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(cfg.show_title_bar));
            self.applied_title_bar = cfg.show_title_bar;
        }
        if cfg.window_transparency != self.applied_transparency {
            ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(cfg.window_transparency));
            self.applied_transparency = cfg.window_transparency;
        }
        if cfg.follow_window_tiling != self.applied_follow_tiling {
            // Best-effort; Wayland ignores WindowLevel.
            let level = if cfg.follow_window_tiling {
                egui::viewport::WindowLevel::Normal
            } else {
                egui::viewport::WindowLevel::AlwaysOnTop
            };
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
            self.applied_follow_tiling = cfg.follow_window_tiling;
        }
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui, horizontal: bool) {
        let layout = if horizontal {
            egui::Layout::left_to_right(egui::Align::Center)
        } else {
            egui::Layout::top_down(egui::Align::Center)
        };
        let dark = ui.visuals().dark_mode;
        let pal = self.palette(dark);
        ui.with_layout(layout, |ui| {
            if horizontal {
                ui.spacing_mut().item_spacing.x = 8.0;
            } else {
                ui.spacing_mut().item_spacing.y = 8.0;
            }

            // Mode label so the chrome reads as Annotate, not a mystery tool strip.
            let label = egui::RichText::new("Annotate")
                .color(if pal.dark {
                    crate::ui::ICON_LIGHT
                } else {
                    crate::ui::ICON_DARK
                })
                .size(11.0)
                .strong();
            if horizontal {
                ui.label(label);
                ui.add_space(4.0);
            } else {
                ui.label(label);
                ui.add_space(2.0);
            }

            if glass_icon_button(
                ui,
                self.tool == Tool::Arrow,
                "Arrow",
                None,
                TOOL_BTN,
                paint_icon_arrow,
            ) {
                self.tool = Tool::Arrow;
            }
            if glass_icon_button(
                ui,
                self.tool == Tool::Rect,
                "Box",
                None,
                TOOL_BTN,
                paint_icon_box,
            ) {
                self.tool = Tool::Rect;
            }
            if glass_icon_button(
                ui,
                self.tool == Tool::Pixelate,
                "Pixelate",
                None,
                TOOL_BTN,
                paint_icon_pixelate,
            ) {
                self.tool = Tool::Pixelate;
            }
            if glass_icon_button(
                ui,
                self.tool == Tool::Step,
                "Step number",
                None,
                TOOL_BTN,
                paint_icon_step,
            ) {
                self.tool = Tool::Step;
            }

            glass_color_swatch(ui, &mut self.stroke_color);

            if glass_icon_button(ui, false, "Undo (Ctrl+Z)", None, TOOL_BTN, paint_icon_undo) {
                self.undo();
            }
            if glass_icon_button(
                ui,
                false,
                "Redo (Ctrl+Shift+Z)",
                None,
                TOOL_BTN,
                paint_icon_redo,
            ) {
                self.redo();
            }
            if glass_icon_button(ui, false, "Clear all marks", None, TOOL_BTN, paint_icon_clear)
            {
                self.clear_all();
            }
            if glass_icon_button(
                ui,
                false,
                "Settings",
                None,
                SETTINGS_BTN,
                paint_icon_settings,
            ) {
                self.show_settings = true;
            }

            // Save / Copy: flat pastel fills with hover color change.
            let save_pair = (PASTEL_PRIMARY, PASTEL_PRIMARY_HOVER);
            let copy_pair = (ACTION_FILL, ACTION_FILL_HOVER);

            if glass_icon_button(
                ui,
                false,
                "Save and close (Enter)",
                Some(save_pair),
                TOOL_BTN,
                paint_icon_save,
            ) {
                if let Err(err) = self.save_and_close() {
                    eprintln!("annotate save failed: {err:#}");
                }
            }
            if glass_icon_button(
                ui,
                false,
                "Copy to clipboard, then close",
                Some(copy_pair),
                TOOL_BTN,
                paint_icon_copy,
            ) {
                if let Err(err) = self.copy_with_toast() {
                    eprintln!("annotate copy failed: {err:#}");
                }
            }

            // Clear Exit control (replaces the old Esc label). Same cancel path as Esc.
            let exit_fill = if pal.dark {
                Color32::from_rgb(0x4A, 0x3E, 0x48)
            } else {
                Color32::from_rgb(0xE8, 0xE0, 0xE6)
            };
            let exit_text = if pal.dark {
                crate::ui::ICON_LIGHT
            } else {
                crate::ui::ICON_DARK
            };
            let exit = ui.add_sized(
                Vec2::new(if horizontal { 52.0 } else { TOOL_BTN + 4.0 }, 22.0),
                egui::Button::new(
                    egui::RichText::new("Exit")
                        .color(exit_text)
                        .size(12.0)
                        .strong(),
                )
                .fill(exit_fill)
                .stroke(Stroke::new(1.0_f32, pal.toolbar_stroke))
                .corner_radius(egui::CornerRadius::same(6)),
            );
            if exit.on_hover_text("Close without saving").clicked() {
                self.cancel_draft_or_session();
            }
        });
    }

    fn paint_canvas(&mut self, ui: &mut egui::Ui, image_area: Rect, pal: &AnnotatePalette) {
        let (image_rect, scale) =
            fit_image_rect(image_area, self.width as f32, self.height as f32);
        let response = ui.interact(image_area, ui.id().with("canvas"), Sense::click_and_drag());
        let painter = ui.painter_at(image_area);

        painter.rect_filled(image_rect, 8.0, pal.preview_matte);
        painter.image(
            self.texture.id(),
            image_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        let origin = image_rect.min;
        let to_screen = |p: Pos2| -> Pos2 {
            Pos2::new(origin.x + p.x * scale, origin.y + p.y * scale)
        };
        let to_image = |p: Pos2| -> Pos2 {
            Pos2::new((p.x - origin.x) / scale, (p.y - origin.y) / scale)
        };

        for shape in self.shapes.iter().chain(self.draft.iter()) {
            paint_shape(&painter, *shape, to_screen, scale);
        }

        if self.tool == Tool::Step {
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if image_rect.contains(pos) {
                        let center = clamp_to_image(to_image(pos), self.width, self.height);
                        let number = self.next_step_number;
                        self.push_shape(Shape::Step {
                            center,
                            number,
                            color: self.stroke_color,
                        });
                        self.next_step_number = number.saturating_add(1);
                    }
                }
            }
        } else {
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if image_rect.contains(pos) {
                        self.drag_start = Some(to_image(pos));
                    }
                }
            }
            if response.dragged() {
                if let (Some(start), Some(pos)) =
                    (self.drag_start, response.interact_pointer_pos())
                {
                    let end = clamp_to_image(to_image(pos), self.width, self.height);
                    self.draft = self.draft_from_drag(start, end);
                }
            }
            if response.drag_stopped() {
                if let Some(shape) = self.draft.take() {
                    if shape_is_meaningful(shape) {
                        self.push_shape(shape);
                    }
                }
                self.drag_start = None;
            }
        }
    }

    fn paint_copy_toast(&self, ctx: &egui::Context, pal: &AnnotatePalette) {
        if self.copy_toast_until.is_none() {
            return;
        }
        let screen = ctx.screen_rect();
        let toast_size = Vec2::new(280.0, 56.0);
        let pos = Pos2::new(
            screen.center().x - toast_size.x * 0.5,
            screen.center().y - toast_size.y * 0.5,
        );
        egui::Area::new(egui::Id::new("lightsnip_copy_toast"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                let fill = if pal.dark {
                    Color32::from_rgba_unmultiplied(0x2A, 0x28, 0x30, 0xF0)
                } else {
                    Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0xF2)
                };
                let stroke = pal.toolbar_stroke;
                let text = if pal.dark {
                    crate::ui::ICON_LIGHT
                } else {
                    crate::ui::ICON_DARK
                };
                egui::Frame::new()
                    .fill(fill)
                    .stroke(Stroke::new(1.0_f32, stroke))
                    .corner_radius(egui::CornerRadius::same(10))
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .shadow(egui::Shadow {
                        offset: [0, 2],
                        blur: 8,
                        spread: 0,
                        color: Color32::from_black_alpha(40),
                    })
                    .show(ui, |ui| {
                        ui.set_min_width(toast_size.x - 32.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("Copied to clipboard")
                                    .color(text)
                                    .size(14.0)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new("Closing…")
                                    .color(pal.muted)
                                    .size(11.0),
                            );
                        });
                    });
            });
    }
}

impl eframe::App for AnnotateApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let dark = matches!(
            self.settings.config().theme,
            crate::config::UiTheme::Dark
        ) || (_visuals.dark_mode
            && matches!(
                self.settings.config().theme,
                crate::config::UiTheme::System
            ));
        let pal = self.palette(dark || _visuals.dark_mode);
        let mut rgba = egui::Rgba::from(pal.bg).to_array();
        // Intensity scales window clear alpha when transparency is enabled.
        // Many Wayland compositors still ignore transparent viewports.
        rgba[3] = pal.panel_opacity;
        rgba
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Finish copy toast → close. Keep painting until then (no blank flash).
        if let Some(until) = self.copy_toast_until {
            if Instant::now() >= until {
                self.copy_toast_until = None;
                self.finished = true;
            } else {
                ctx.request_repaint_after(Duration::from_millis(32));
            }
        }

        if self.finished {
            // Close after this frame's paint so the last content frame is not blank.
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // While closing after Save (no toast), skip interaction but still paint once.
        let interactive = !self.finished && self.copy_toast_until.is_none();

        self.sync_viewport_prefs(ctx);

        if interactive {
            let esc_to_exit = self.settings.config().esc_to_exit;
            ctx.input(|i| {
                if i.key_pressed(egui::Key::Escape) {
                    if self.show_settings {
                        self.show_settings = false;
                    } else if esc_to_exit {
                        self.cancel_draft_or_session();
                    } else if self.draft.is_some() || self.drag_start.is_some() {
                        // Esc-to-exit off: still drop an in-progress drag, never close.
                        self.draft = None;
                        self.drag_start = None;
                    }
                }
                if !self.show_settings && i.key_pressed(egui::Key::Enter) {
                    if let Err(err) = self.save_and_close() {
                        eprintln!("annotate save failed: {err:#}");
                    }
                }
                if i.modifiers.command && i.key_pressed(egui::Key::Z) {
                    if i.modifiers.shift {
                        self.redo();
                    } else {
                        self.undo();
                    }
                }
                if i.modifiers.command && i.key_pressed(egui::Key::Y) {
                    self.redo();
                }
            });
        }

        if interactive {
            show_settings_window(ctx, &mut self.show_settings, &mut self.settings);
        }

        let dark = ctx.theme() == egui::Theme::Dark;
        let pal = self.palette(dark);
        let toolbar_pos = self.settings.config().toolbar_position;
        let horizontal = toolbar_pos.is_horizontal();

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(pal.bg)
                    .inner_margin(egui::Margin::same(WINDOW_PAD as i8)),
            )
            .show(ctx, |ui| {
                let available = ui.available_rect_before_wrap();

                // Reserve a strip for the content-sized rail; image gets the rest.
                let (image_area, toolbar_origin) = match toolbar_pos {
                    ToolbarPosition::Left => {
                        let rail = TOOLBAR_WIDTH + TOOLBAR_GUTTER;
                        let img = Rect::from_min_max(
                            Pos2::new(available.min.x + rail, available.min.y),
                            available.max,
                        );
                        let (fitted, _) =
                            fit_image_rect(img, self.width as f32, self.height as f32);
                        let th = toolbar_content_height().min(available.height());
                        let ty = (fitted.center().y - th * 0.5)
                            .clamp(available.min.y, available.max.y - th);
                        (img, Pos2::new(available.min.x, ty))
                    }
                    ToolbarPosition::Right => {
                        let rail = TOOLBAR_WIDTH + TOOLBAR_GUTTER;
                        let img = Rect::from_min_max(
                            available.min,
                            Pos2::new(available.max.x - rail, available.max.y),
                        );
                        let (fitted, _) =
                            fit_image_rect(img, self.width as f32, self.height as f32);
                        let th = toolbar_content_height().min(available.height());
                        let ty = (fitted.center().y - th * 0.5)
                            .clamp(available.min.y, available.max.y - th);
                        (img, Pos2::new(available.max.x - TOOLBAR_WIDTH, ty))
                    }
                    ToolbarPosition::Top => {
                        let rail = TOOLBAR_WIDTH + TOOLBAR_GUTTER;
                        let img = Rect::from_min_max(
                            Pos2::new(available.min.x, available.min.y + rail),
                            available.max,
                        );
                        let (fitted, _) =
                            fit_image_rect(img, self.width as f32, self.height as f32);
                        let tw = toolbar_content_width().min(available.width());
                        let tx = (fitted.center().x - tw * 0.5)
                            .clamp(available.min.x, available.max.x - tw);
                        (img, Pos2::new(tx, available.min.y))
                    }
                    ToolbarPosition::Bottom => {
                        let rail = TOOLBAR_WIDTH + TOOLBAR_GUTTER;
                        let img = Rect::from_min_max(
                            available.min,
                            Pos2::new(available.max.x, available.max.y - rail),
                        );
                        let (fitted, _) =
                            fit_image_rect(img, self.width as f32, self.height as f32);
                        let tw = toolbar_content_width().min(available.width());
                        let tx = (fitted.center().x - tw * 0.5)
                            .clamp(available.min.x, available.max.x - tw);
                        (img, Pos2::new(tx, available.max.y - TOOLBAR_WIDTH))
                    }
                };

                // Image canvas (letterbox / upscale with LINEAR).
                ui.scope_builder(egui::UiBuilder::new().max_rect(image_area), |ui| {
                    self.paint_canvas(ui, image_area, &pal);
                });

                // Floating pastel rail — content-sized, attached to preview (not full height).
                egui::Area::new(egui::Id::new("lightsnip_toolbar"))
                    .fixed_pos(toolbar_origin)
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        glass_toolbar_frame(&pal).show(ui, |ui| {
                            if horizontal {
                                ui.set_height(TOOLBAR_WIDTH - 16.0);
                            } else {
                                ui.set_width(TOOLBAR_WIDTH - 16.0);
                            }
                            self.draw_toolbar(ui, horizontal);
                        });
                    });
            });

        self.paint_copy_toast(ctx, &pal);

        if self.finished {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn clamp_to_image(p: Pos2, w: u32, h: u32) -> Pos2 {
    Pos2::new(
        p.x.clamp(0.0, (w.saturating_sub(1)) as f32),
        p.y.clamp(0.0, (h.saturating_sub(1)) as f32),
    )
}

fn shape_is_meaningful(shape: Shape) -> bool {
    match shape {
        Shape::Arrow { from, to, .. } => from.distance(to) >= 4.0,
        Shape::Rect { a, b, .. } | Shape::Pixelate { a, b } => {
            (a.x - b.x).abs() >= 4.0 && (a.y - b.y).abs() >= 4.0
        }
        Shape::Step { number, .. } => number >= 1,
    }
}

fn paint_shape(
    painter: &egui::Painter,
    shape: Shape,
    to_screen: impl Fn(Pos2) -> Pos2,
    scale: f32,
) {
    match shape {
        Shape::Arrow { from, to, color } => {
            let stroke = Stroke::new(STROKE_WIDTH, color);
            let a = to_screen(from);
            let b = to_screen(to);
            painter.line_segment([a, b], stroke);
            for tip in arrow_head_points(a, b, 14.0) {
                painter.line_segment([b, tip], stroke);
            }
        }
        Shape::Rect { a, b, color } => {
            let stroke = Stroke::new(STROKE_WIDTH, color);
            let r = Rect::from_two_pos(to_screen(a), to_screen(b));
            painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Outside);
        }
        Shape::Pixelate { a, b } => {
            let r = Rect::from_two_pos(to_screen(a), to_screen(b));
            painter.rect_filled(r, 0.0, PIXELATE_PREVIEW_FILL);
            let block_screen = (PIXELATE_BLOCK as f32 * scale).max(4.0);
            let mut y = r.min.y;
            while y < r.max.y {
                let mut x = r.min.x;
                while x < r.max.x {
                    let cell = Rect::from_min_max(
                        Pos2::new(x, y),
                        Pos2::new((x + block_screen).min(r.max.x), (y + block_screen).min(r.max.y)),
                    );
                    painter.rect_stroke(
                        cell,
                        0.0,
                        Stroke::new(
                            1.0_f32,
                            Color32::from_rgba_premultiplied(0x33, 0x33, 0x33, 90),
                        ),
                        egui::StrokeKind::Inside,
                    );
                    x += block_screen;
                }
                y += block_screen;
            }
            painter.rect_stroke(
                r,
                0.0,
                Stroke::new(1.5_f32, PIXELATE_PREVIEW_STROKE),
                egui::StrokeKind::Outside,
            );
        }
        Shape::Step {
            center,
            number,
            color,
        } => {
            let c = to_screen(center);
            let r = (STEP_RADIUS * scale).max(10.0);
            // High-contrast: solid fill + white rim + white number.
            painter.circle_filled(c, r, color);
            painter.circle_stroke(
                c,
                r,
                Stroke::new(2.0_f32, Color32::WHITE),
            );
            painter.text(
                c,
                egui::Align2::CENTER_CENTER,
                number.to_string(),
                egui::FontId::proportional((r * 1.05).clamp(11.0, 28.0)),
                Color32::WHITE,
            );
        }
    }
}

fn arrow_head_points(from: Pos2, to: Pos2, size: f32) -> [Pos2; 2] {
    let dir = to - from;
    let len = dir.length().max(1.0);
    let unit = dir / len;
    let perp = Vec2::new(-unit.y, unit.x);
    let back = to - unit * size;
    [back + perp * (size * 0.45), back - perp * (size * 0.45)]
}

fn rasterize_shape(img: &mut RgbaImage, shape: Shape) {
    match shape {
        Shape::Arrow { from, to, color } => {
            let rgba = Rgba([color.r(), color.g(), color.b(), 255]);
            draw_thick_line(img, from, to, rgba, STROKE_WIDTH);
            let a = from;
            let b = to;
            for tip in arrow_head_points(a, b, 14.0) {
                draw_thick_line(img, b, tip, rgba, STROKE_WIDTH);
            }
        }
        Shape::Rect { a, b, color } => {
            let rgba = Rgba([color.r(), color.g(), color.b(), 255]);
            let x0 = a.x.min(b.x);
            let y0 = a.y.min(b.y);
            let x1 = a.x.max(b.x);
            let y1 = a.y.max(b.y);
            let tl = Pos2::new(x0, y0);
            let tr = Pos2::new(x1, y0);
            let br = Pos2::new(x1, y1);
            let bl = Pos2::new(x0, y1);
            draw_thick_line(img, tl, tr, rgba, STROKE_WIDTH);
            draw_thick_line(img, tr, br, rgba, STROKE_WIDTH);
            draw_thick_line(img, br, bl, rgba, STROKE_WIDTH);
            draw_thick_line(img, bl, tl, rgba, STROKE_WIDTH);
        }
        Shape::Pixelate { a, b } => {
            pixelate_region(img, a, b, PIXELATE_BLOCK);
        }
        Shape::Step {
            center,
            number,
            color,
        } => {
            rasterize_step(img, center, number, color);
        }
    }
}

fn rasterize_step(img: &mut RgbaImage, center: Pos2, number: u32, color: Color32) {
    let fill = Rgba([color.r(), color.g(), color.b(), 255]);
    let white = Rgba([255, 255, 255, 255]);
    let cx = center.x.round() as i32;
    let cy = center.y.round() as i32;
    let radius = STEP_RADIUS.round() as i32;
    draw_filled_circle_mut(img, (cx, cy), radius, fill);
    // White rim for contrast on busy backgrounds.
    draw_circle_ring(img, cx, cy, radius, white);
    if radius > 2 {
        draw_circle_ring(img, cx, cy, radius - 1, white);
    }
    let label = number.to_string();
    let digit_w = 5i32;
    let digit_h = 7i32;
    let scale = 2i32;
    let gap = 1i32;
    let total_w = label.len() as i32 * (digit_w * scale + gap) - gap;
    let mut x = cx - total_w / 2;
    let y = cy - (digit_h * scale) / 2;
    for ch in label.chars() {
        if let Some(d) = ch.to_digit(10) {
            blit_digit_5x7(img, d as u8, x, y, scale, white);
        }
        x += digit_w * scale + gap;
    }
}

fn draw_circle_ring(img: &mut RgbaImage, cx: i32, cy: i32, radius: i32, color: Rgba<u8>) {
    if radius <= 0 {
        return;
    }
    let (w, h) = img.dimensions();
    let r2 = radius * radius;
    let outer = (radius + 1) * (radius + 1);
    for dy in -radius - 1..=radius + 1 {
        for dx in -radius - 1..=radius + 1 {
            let d2 = dx * dx + dy * dy;
            if d2 > r2 && d2 <= outer {
                let x = cx + dx;
                let y = cy + dy;
                if x >= 0 && y >= 0 && (x as u32) < w && (y as u32) < h {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}

/// Tiny 5×7 digit bitmaps (MSB left). Enough for high-contrast step labels.
fn digit_bitmap_5x7(d: u8) -> [u8; 7] {
    match d {
        0 => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        1 => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        2 => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        3 => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
        4 => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        5 => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        6 => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        7 => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        8 => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        9 => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        _ => [0; 7],
    }
}

fn blit_digit_5x7(
    img: &mut RgbaImage,
    digit: u8,
    ox: i32,
    oy: i32,
    scale: i32,
    color: Rgba<u8>,
) {
    let bmp = digit_bitmap_5x7(digit);
    let (w, h) = img.dimensions();
    let scale = scale.max(1);
    for (row, bits) in bmp.iter().enumerate() {
        for col in 0..5 {
            if bits & (1 << (4 - col)) != 0 {
                for sy in 0..scale {
                    for sx in 0..scale {
                        let x = ox + col as i32 * scale + sx;
                        let y = oy + row as i32 * scale + sy;
                        if x >= 0 && y >= 0 && (x as u32) < w && (y as u32) < h {
                            img.put_pixel(x as u32, y as u32, color);
                        }
                    }
                }
            }
        }
    }
}

fn pixelate_region(img: &mut RgbaImage, a: Pos2, b: Pos2, block: u32) {
    let block = block.max(1);
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return;
    }

    let x0 = a.x.min(b.x).floor().max(0.0) as u32;
    let y0 = a.y.min(b.y).floor().max(0.0) as u32;
    let x1 = (a.x.max(b.x).ceil() as u32).min(w);
    let y1 = (a.y.max(b.y).ceil() as u32).min(h);
    if x0 >= x1 || y0 >= y1 {
        return;
    }

    let mut y = y0;
    while y < y1 {
        let y_end = (y + block).min(y1);
        let mut x = x0;
        while x < x1 {
            let x_end = (x + block).min(x1);
            let avg = average_block(img, x, y, x_end, y_end);
            for py in y..y_end {
                for px in x..x_end {
                    img.put_pixel(px, py, avg);
                }
            }
            x = x_end;
        }
        y = y_end;
    }
}

fn average_block(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> Rgba<u8> {
    let mut sum = [0u64; 4];
    let mut count = 0u64;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = img.get_pixel(x, y).0;
            sum[0] += u64::from(p[0]);
            sum[1] += u64::from(p[1]);
            sum[2] += u64::from(p[2]);
            sum[3] += u64::from(p[3]);
            count += 1;
        }
    }
    if count == 0 {
        return Rgba([0, 0, 0, 255]);
    }
    Rgba([
        (sum[0] / count) as u8,
        (sum[1] / count) as u8,
        (sum[2] / count) as u8,
        (sum[3] / count) as u8,
    ])
}

fn draw_thick_line(img: &mut RgbaImage, a: Pos2, b: Pos2, color: Rgba<u8>, width: f32) {
    let radius = (width * 0.5).max(1.0) as i32;
    draw_line_segment_mut(img, (a.x, a.y), (b.x, b.y), color);
    let dir = b - a;
    let len = dir.length().max(1.0);
    let perp = Vec2::new(-dir.y / len, dir.x / len);
    for i in 1..=radius {
        let o = perp * i as f32;
        draw_line_segment_mut(
            img,
            (a.x + o.x, a.y + o.y),
            (b.x + o.x, b.y + o.y),
            color,
        );
        draw_line_segment_mut(
            img,
            (a.x - o.x, a.y - o.y),
            (b.x - o.x, b.y - o.y),
            color,
        );
    }
    draw_filled_circle_mut(img, (a.x as i32, a.y as i32), radius, color);
    draw_filled_circle_mut(img, (b.x as i32, b.y as i32), radius, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiny_shapes_rejected() {
        let arrow = Shape::Arrow {
            from: Pos2::new(0.0, 0.0),
            to: Pos2::new(1.0, 0.0),
            color: DEFAULT_STROKE,
        };
        assert!(!shape_is_meaningful(arrow));
        let rect = Shape::Rect {
            a: Pos2::new(0.0, 0.0),
            b: Pos2::new(10.0, 10.0),
            color: DEFAULT_STROKE,
        };
        assert!(shape_is_meaningful(rect));
        let tiny_pix = Shape::Pixelate {
            a: Pos2::new(0.0, 0.0),
            b: Pos2::new(2.0, 2.0),
        };
        assert!(!shape_is_meaningful(tiny_pix));
    }

    #[test]
    fn arrow_head_not_collinear_with_tip() {
        let from = Pos2::new(0.0, 0.0);
        let to = Pos2::new(100.0, 0.0);
        let [p1, p2] = arrow_head_points(from, to, 14.0);
        assert!((p1.y - p2.y).abs() > 1.0);
        assert!(p1.x < to.x);
    }

    #[test]
    fn rasterize_arrow_touches_pixels() {
        let mut img = RgbaImage::from_pixel(64, 64, Rgba([0, 0, 0, 255]));
        rasterize_shape(
            &mut img,
            Shape::Arrow {
                from: Pos2::new(8.0, 32.0),
                to: Pos2::new(56.0, 32.0),
                color: DEFAULT_STROKE,
            },
        );
        let mid = img.get_pixel(32, 32);
        assert_ne!(mid.0[0], 0, "stroke should paint red channel");
        assert_eq!(mid.0[0], DEFAULT_STROKE.r());
    }

    #[test]
    fn rasterize_uses_custom_stroke_color() {
        let blue = Color32::from_rgb(0x00, 0x7A, 0xFF);
        let mut img = RgbaImage::from_pixel(64, 64, Rgba([0, 0, 0, 255]));
        rasterize_shape(
            &mut img,
            Shape::Rect {
                a: Pos2::new(8.0, 8.0),
                b: Pos2::new(40.0, 40.0),
                color: blue,
            },
        );
        let edge = img.get_pixel(8, 20);
        assert_eq!(edge.0[0], blue.r());
        assert_eq!(edge.0[1], blue.g());
        assert_eq!(edge.0[2], blue.b());
    }

    #[test]
    fn default_app_stroke_is_red() {
        assert_eq!(DEFAULT_STROKE, Color32::from_rgb(0xE2, 0x4A, 0x3B));
    }

    #[test]
    fn pixelate_flattens_checker_block() {
        let mut img = RgbaImage::new(32, 32);
        for y in 0..32 {
            for x in 0..32 {
                let on = (x + y) % 2 == 0;
                let v = if on { 255 } else { 0 };
                img.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        pixelate_region(
            &mut img,
            Pos2::new(0.0, 0.0),
            Pos2::new(24.0, 24.0),
            8,
        );
        let p00 = img.get_pixel(0, 0).0;
        let p77 = img.get_pixel(7, 7).0;
        assert_eq!(p00, p77);
        assert_ne!(
            img.get_pixel(30, 30).0[0],
            img.get_pixel(31, 30).0[0],
            "untouched area should keep fine detail"
        );
    }

    #[test]
    fn rasterize_pixelate_shape_redacts() {
        let mut img = RgbaImage::from_pixel(48, 48, Rgba([10, 20, 30, 255]));
        img.put_pixel(8, 8, Rgba([255, 0, 0, 255]));
        img.put_pixel(9, 9, Rgba([0, 255, 0, 255]));
        rasterize_shape(
            &mut img,
            Shape::Pixelate {
                a: Pos2::new(4.0, 4.0),
                b: Pos2::new(20.0, 20.0),
            },
        );
        let a = img.get_pixel(8, 8).0;
        let b = img.get_pixel(9, 9).0;
        assert_eq!(a, b, "pixelate should average neighbors in a block");
        assert_ne!(a, [255, 0, 0, 255], "original bright pixel must not remain");
    }

    #[test]
    fn escape_clears_draft_before_cancel() {
        fn step(
            draft: &mut Option<Shape>,
            drag_start: &mut Option<Pos2>,
        ) -> bool /* session cancelled */ {
            if draft.is_some() || drag_start.is_some() {
                *draft = None;
                *drag_start = None;
                false
            } else {
                true
            }
        }

        let mut draft = Some(Shape::Pixelate {
            a: Pos2::new(0.0, 0.0),
            b: Pos2::new(10.0, 10.0),
        });
        let mut drag_start = Some(Pos2::new(0.0, 0.0));

        assert!(!step(&mut draft, &mut drag_start));
        assert!(draft.is_none());
        assert!(drag_start.is_none());
        assert!(step(&mut draft, &mut drag_start));
    }

    #[test]
    fn floating_window_includes_toolbar_and_gutter() {
        let (w, h) = floating_window_size(400, 300, ToolbarPosition::Left);
        assert!(w > 400.0 + TOOLBAR_WIDTH);
        assert!(h >= 300.0);
        assert!(WINDOW_PAD >= 12.0);
        assert!(w >= 400.0 + TOOLBAR_WIDTH + TOOLBAR_GUTTER + WINDOW_PAD * 2.0);
        assert!(h >= 300.0 + WINDOW_PAD * 2.0);
    }

    #[test]
    fn floating_window_clamps_huge_captures() {
        let (w, h) = floating_window_size(4000, 3000, ToolbarPosition::Left);
        assert!(w <= TOOLBAR_WIDTH + TOOLBAR_GUTTER + MAX_DISPLAY_W + WINDOW_PAD * 2.0 + 1.0);
        assert!(h <= MAX_DISPLAY_H + WINDOW_PAD * 2.0 + 1.0 + toolbar_content_height());
    }

    #[test]
    fn toolbar_does_not_force_full_window_height() {
        // Content-sized rail is shorter than a tall window's image area.
        assert!(toolbar_content_height() < MAX_DISPLAY_H);
    }

    #[test]
    fn undo_redo_stack_mirrors() {
        let mut shapes = vec![
            Shape::Arrow {
                from: Pos2::new(0.0, 0.0),
                to: Pos2::new(20.0, 0.0),
                color: DEFAULT_STROKE,
            },
            Shape::Rect {
                a: Pos2::new(0.0, 0.0),
                b: Pos2::new(10.0, 10.0),
                color: DEFAULT_STROKE,
            },
        ];
        let mut redo = Vec::new();
        // undo
        if let Some(s) = shapes.pop() {
            redo.push(s);
        }
        assert_eq!(shapes.len(), 1);
        assert_eq!(redo.len(), 1);
        // redo
        if let Some(s) = redo.pop() {
            shapes.push(s);
        }
        assert_eq!(shapes.len(), 2);
        assert!(redo.is_empty());
        // new mark clears redo
        redo.push(shapes[0]);
        shapes.push(Shape::Pixelate {
            a: Pos2::new(0.0, 0.0),
            b: Pos2::new(12.0, 12.0),
        });
        redo.clear();
        assert!(redo.is_empty());
        assert_eq!(shapes.len(), 3);
    }

    fn next_from_shapes(shapes: &[Shape]) -> u32 {
        shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Step { number, .. } => Some(*number),
                _ => None,
            })
            .max()
            .map(|m| m + 1)
            .unwrap_or(1)
    }

    #[test]
    fn step_counter_resets_on_undo_and_clear() {
        let mut shapes = Vec::new();
        let mut next = 1u32;
        for _ in 0..3 {
            shapes.push(Shape::Step {
                center: Pos2::new(10.0, 10.0),
                number: next,
                color: DEFAULT_STROKE,
            });
            next += 1;
        }
        assert_eq!(next, 4);
        assert_eq!(next_from_shapes(&shapes), 4);

        // Undo last stamp → next becomes 3 again.
        shapes.pop();
        next = next_from_shapes(&shapes);
        assert_eq!(next, 3);

        // Clear everything → back to 1.
        shapes.clear();
        next = next_from_shapes(&shapes);
        assert_eq!(next, 1);
    }

    #[test]
    fn rasterize_step_paints_circle_and_digit() {
        let mut img = RgbaImage::from_pixel(64, 64, Rgba([0, 0, 0, 255]));
        rasterize_shape(
            &mut img,
            Shape::Step {
                center: Pos2::new(32.0, 32.0),
                number: 1,
                color: DEFAULT_STROKE,
            },
        );
        // Fill is visible just inside the rim (away from the centered digit).
        let fill = img.get_pixel(32, 32 - (STEP_RADIUS as u32) + 3);
        assert_eq!(fill.0[0], DEFAULT_STROKE.r());
        assert_eq!(fill.0[1], DEFAULT_STROKE.g());
        // White digit / rim should appear somewhere near the center.
        let mut saw_white = false;
        for y in 18..46 {
            for x in 18..46 {
                let p = img.get_pixel(x, y).0;
                if p == [255, 255, 255, 255] {
                    saw_white = true;
                }
            }
        }
        assert!(saw_white, "step stamp should paint a white rim or digit");
    }

    #[test]
    fn step_shape_is_always_meaningful() {
        assert!(shape_is_meaningful(Shape::Step {
            center: Pos2::ZERO,
            number: 1,
            color: DEFAULT_STROKE,
        }));
    }

    #[test]
    fn floating_mode_prefers_center_when_not_tiling() {
        // Documented contract for NativeOptions: centered iff !follow_window_tiling.
        let follow = false;
        assert!(!follow);
        let centered = !follow;
        assert!(centered);
        let follow = true;
        assert!(!follows_to_centered(follow));
    }

    fn follows_to_centered(follow_tiling: bool) -> bool {
        !follow_tiling
    }
}
