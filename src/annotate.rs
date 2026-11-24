//! Floating annotate window (flat pastel tool rail + image canvas).
//!
//! Layout: content-sized floating rail attached to the capture preview (not
//! full window height). Position: left (default) / right / top / bottom via
//! settings. Esc cancels/discards; Save and Copy dismiss after a successful write.
//!
//! Chrome: flat pastel purple / gray / white — no liquid-glass imitation
//! (no frost, specular rim, or hover sheen). egui on Wayland cannot backdrop-blur.
//!
//! Stay-on-top: `ViewportBuilder::with_always_on_top`. Many Wayland compositors
//! ignore or gate always-on-top.

use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2, ViewportBuilder};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_line_segment_mut};

use crate::config::ToolbarPosition;
use crate::settings::{show_settings_window, EmbeddedSettings};
use crate::ui::{
    self, fit_image_rect, glass_color_swatch, glass_icon_button, glass_toolbar_frame,
    paint_icon_arrow, paint_icon_box, paint_icon_clear, paint_icon_copy, paint_icon_pixelate,
    paint_icon_save, paint_icon_settings, paint_icon_undo, AnnotatePalette, ACTION_FILL,
    ACTION_FILL_HOVER, DEFAULT_STROKE, PASTEL_PRIMARY, PASTEL_PRIMARY_HOVER, SETTINGS_BTN,
    TOOLBAR_GUTTER, TOOLBAR_WIDTH, TOOL_BTN, WINDOW_PAD,
};

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
}

/// Block size for privacy pixelation (image pixels per mosaic cell).
const PIXELATE_BLOCK: u32 = 12;

/// Max displayed image size inside the floating window (px). Larger captures scale down.
const MAX_DISPLAY_W: f32 = 1100.0;
const MAX_DISPLAY_H: f32 = 800.0;

/// Approximate content size of the floating rail (vertical).
fn toolbar_content_height() -> f32 {
    // 3 tools + color + undo + clear + settings + save + copy + gaps + margins
    let n_btns = 8.0_f32;
    let gaps = 7.0 * 8.0 + 24.0;
    n_btns * TOOL_BTN + (SETTINGS_BTN - TOOL_BTN) + gaps
}

/// Approximate content size of the floating rail (horizontal).
fn toolbar_content_width() -> f32 {
    let n_btns = 8.0_f32;
    let gaps = 7.0 * 8.0 + 24.0;
    n_btns * TOOL_BTN + (SETTINGS_BTN - TOOL_BTN) + gaps
}

/// Open a floating annotate window on an existing PNG.
///
/// On Save / Enter / Copy: composites annotations onto the image and overwrites `path`.
/// On Escape: clears an in-progress drag first; with no draft, returns
/// [`AnnotateOutcome::Cancelled`] without modifying the file.
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

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Lightsnip")
            .with_inner_size([win_w, win_h])
            .with_min_inner_size([
                TOOLBAR_WIDTH + TOOLBAR_GUTTER + 160.0 + WINDOW_PAD * 2.0,
                200.0,
            ])
            .with_decorations(decorations)
            .with_transparent(transparent)
            .with_resizable(true)
            // Best-effort; Wayland compositors often ignore or restrict this.
            .with_always_on_top(),
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
    tool: Tool,
    stroke_color: Color32,
    drag_start: Option<Pos2>,
    draft: Option<Shape>,
    path: std::path::PathBuf,
    outcome: Arc<Mutex<AnnotateOutcome>>,
    finished: bool,
    show_settings: bool,
    settings: EmbeddedSettings,
    /// Last applied viewport decoration / transparency (to avoid spam).
    applied_title_bar: bool,
    applied_transparency: bool,
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
            tool: Tool::Arrow,
            stroke_color: DEFAULT_STROKE,
            drag_start: None,
            draft: None,
            path,
            outcome,
            finished: false,
            show_settings: false,
            applied_title_bar: cfg.show_title_bar,
            applied_transparency: cfg.window_transparency,
            settings,
        }
    }

    fn set_outcome(&mut self, outcome: AnnotateOutcome) {
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = outcome;
        }
        self.finished = true;
    }

    fn cancel_draft_or_session(&mut self) {
        if self.draft.is_some() || self.drag_start.is_some() {
            self.draft = None;
            self.drag_start = None;
        } else {
            self.set_outcome(AnnotateOutcome::Cancelled);
        }
    }

    fn undo(&mut self) {
        self.shapes.pop();
        self.draft = None;
        self.drag_start = None;
    }

    fn clear_all(&mut self) {
        self.shapes.clear();
        self.draft = None;
        self.drag_start = None;
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
        self.write_annotated_png()?;
        self.set_outcome(AnnotateOutcome::Saved);
        Ok(())
    }

    fn copy_and_close(&mut self) -> Result<()> {
        self.write_annotated_png()?;
        self.set_outcome(AnnotateOutcome::Copied);
        Ok(())
    }

    fn draft_from_drag(&self, start: Pos2, end: Pos2) -> Shape {
        match self.tool {
            Tool::Arrow => Shape::Arrow {
                from: start,
                to: end,
                color: self.stroke_color,
            },
            Tool::Rect => Shape::Rect {
                a: start,
                b: end,
                color: self.stroke_color,
            },
            Tool::Pixelate => Shape::Pixelate { a: start, b: end },
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
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui, horizontal: bool) {
        let layout = if horizontal {
            egui::Layout::left_to_right(egui::Align::Center)
        } else {
            egui::Layout::top_down(egui::Align::Center)
        };
        ui.with_layout(layout, |ui| {
            if horizontal {
                ui.spacing_mut().item_spacing.x = 8.0;
            } else {
                ui.spacing_mut().item_spacing.y = 8.0;
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

            glass_color_swatch(ui, &mut self.stroke_color);

            if glass_icon_button(ui, false, "Undo (Ctrl+Z)", None, TOOL_BTN, paint_icon_undo) {
                self.undo();
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
                "Save, copy to clipboard, and close",
                Some(copy_pair),
                TOOL_BTN,
                paint_icon_copy,
            ) {
                if let Err(err) = self.copy_and_close() {
                    eprintln!("annotate copy failed: {err:#}");
                }
            }

            let dark = ui.visuals().dark_mode;
            let muted = AnnotatePalette::for_mode(dark, false).muted;
            ui.label(egui::RichText::new("Esc").color(muted).size(10.0));
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

        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                if image_rect.contains(pos) {
                    self.drag_start = Some(to_image(pos));
                }
            }
        }
        if response.dragged() {
            if let (Some(start), Some(pos)) = (self.drag_start, response.interact_pointer_pos()) {
                let end = clamp_to_image(to_image(pos), self.width, self.height);
                self.draft = Some(self.draft_from_drag(start, end));
            }
        }
        if response.drag_stopped() {
            if let Some(shape) = self.draft.take() {
                if shape_is_meaningful(shape) {
                    self.shapes.push(shape);
                }
            }
            self.drag_start = None;
        }
    }
}

impl eframe::App for AnnotateApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let dark = matches!(
            self.settings.config().theme,
            crate::config::UiTheme::Dark
        ) || (_visuals.dark_mode && matches!(self.settings.config().theme, crate::config::UiTheme::System));
        let translucent = self.settings.config().window_transparency;
        let pal = AnnotatePalette::for_mode(dark || _visuals.dark_mode, translucent);
        egui::Rgba::from(pal.bg).to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.finished {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.sync_viewport_prefs(ctx);

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                if self.show_settings {
                    self.show_settings = false;
                } else {
                    self.cancel_draft_or_session();
                }
            }
            if !self.show_settings && i.key_pressed(egui::Key::Enter) {
                if let Err(err) = self.save_and_close() {
                    eprintln!("annotate save failed: {err:#}");
                }
            }
            if i.modifiers.command && i.key_pressed(egui::Key::Z) {
                self.undo();
            }
        });

        if self.finished {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        show_settings_window(ctx, &mut self.show_settings, &mut self.settings);

        let dark = ctx.theme() == egui::Theme::Dark;
        let translucent = self.settings.config().window_transparency;
        let pal = AnnotatePalette::for_mode(dark, translucent);
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

                // Floating glass rail — content-sized, attached to preview (not full height).
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
}
