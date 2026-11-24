//! Fullscreen egui annotate overlay (Milestone 4 + settings gear).
//!
//! Quiet desktop chrome: neutral surfaces, small radius, no glow / pills / emoji.
//! Design dials: ENERGY 1 / RHYTHM 1 / MOTION 1 (tool overlay, not marketing UI).
//!
//! Design reasons (R-31): charcoal chrome so the capture stays focal; one red accent
//! for markup strokes only; pixelate preview uses muted fill (privacy, not decoration).

use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2, ViewportBuilder};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_line_segment_mut};

use crate::settings::{show_settings_window, EmbeddedSettings};
use crate::ui::{self, ACTION_FILL, MUTED};

/// Result of the annotate session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotateOutcome {
    /// User confirmed; annotated PNG was written to the path.
    Saved,
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
    Arrow { from: Pos2, to: Pos2 },
    Rect { a: Pos2, b: Pos2 },
    Pixelate { a: Pos2, b: Pos2 },
}

/// Block size for privacy pixelation (image pixels per mosaic cell).
const PIXELATE_BLOCK: u32 = 12;

/// Open a fullscreen annotate overlay on an existing PNG.
///
/// On Save / Enter: composites annotations onto the image and overwrites `path`.
/// On Escape: clears an in-progress drag first; with no draft, returns
/// [`AnnotateOutcome::Cancelled`] without modifying the file.
///
/// Needs a working Wayland (or X11) display plus egui/winit native libs
/// (`libxkbcommon`, etc.). Cloud / headless VMs usually cannot run this.
pub fn run_annotate_overlay(path: &Path) -> Result<AnnotateOutcome> {
    ensure_display_available()?;

    let rgba = image::open(path)
        .with_context(|| format!("open PNG for annotate {}", path.display()))?
        .to_rgba8();
    let (w, h) = rgba.dimensions();
    if w == 0 || h == 0 {
        bail!("PNG has empty dimensions: {}", path.display());
    }

    let rgba_bytes = rgba.as_raw().clone();
    let path_buf = path.to_path_buf();
    let outcome = Arc::new(Mutex::new(AnnotateOutcome::Cancelled));
    let outcome_for_app = Arc::clone(&outcome);

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("lightsnip annotate")
            .with_fullscreen(true)
            .with_decorations(false),
        ..Default::default()
    };

    // winit can abort via panic when native libs are missing; catch so capture
    // can keep the raw PNG instead of killing the process.
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            "lightsnip annotate",
            options,
            Box::new(move |cc| {
                ui::apply_quiet_visuals(&cc.egui_ctx);
                Ok(Box::new(AnnotateApp::new(
                    &cc.egui_ctx,
                    rgba_bytes,
                    w,
                    h,
                    path_buf,
                    outcome_for_app,
                )))
            }),
        )
    }));

    match run_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            bail!("annotate overlay failed (need a display?): {e}");
        }
        Err(_) => {
            bail!(
                "annotate overlay panicked (missing display libs such as libxkbcommon?)"
            );
        }
    }

    let result = *outcome
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Ok(result)
}

fn ensure_display_available() -> Result<()> {
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let x11 = std::env::var_os("DISPLAY").is_some();
    if !wayland && !x11 {
        bail!("no WAYLAND_DISPLAY or DISPLAY set (annotate needs a graphical session)");
    }
    Ok(())
}

/// Markup stroke color - one deliberate accent on the image, not on chrome (R-29 / Part 3).
const STROKE: Color32 = Color32::from_rgb(0xE2, 0x4A, 0x3B);
const STROKE_WIDTH: f32 = 3.0;

/// Privacy region outline - muted, not the markup accent (R-31: redaction ≠ callout).
/// Premultiplied approx of unmultiplied (0x28,0x28,0x2A,160).
const PIXELATE_PREVIEW_FILL: Color32 =
    Color32::from_rgba_premultiplied(0x19, 0x19, 0x1A, 160);
const PIXELATE_PREVIEW_STROKE: Color32 = Color32::from_rgb(0xC8, 0xC8, 0xC8);

struct AnnotateApp {
    texture: egui::TextureHandle,
    width: u32,
    height: u32,
    /// Original pixels (RGBA8). Unchanged until Save.
    pixels: Vec<u8>,
    shapes: Vec<Shape>,
    tool: Tool,
    drag_start: Option<Pos2>,
    draft: Option<Shape>,
    path: std::path::PathBuf,
    outcome: Arc<Mutex<AnnotateOutcome>>,
    finished: bool,
    show_settings: bool,
    settings: EmbeddedSettings,
}

impl AnnotateApp {
    fn new(
        ctx: &egui::Context,
        pixels: Vec<u8>,
        width: u32,
        height: u32,
        path: std::path::PathBuf,
        outcome: Arc<Mutex<AnnotateOutcome>>,
    ) -> Self {
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &pixels,
        );
        let texture = ctx.load_texture("capture", color_image, egui::TextureOptions::LINEAR);
        Self {
            texture,
            width,
            height,
            pixels,
            shapes: Vec::new(),
            // Default: arrow for callouts; pick Pixelate when redacting.
            tool: Tool::Arrow,
            drag_start: None,
            draft: None,
            path,
            outcome,
            finished: false,
            show_settings: false,
            settings: EmbeddedSettings::load(),
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

    fn save_and_close(&mut self) -> Result<()> {
        let mut img = RgbaImage::from_raw(self.width, self.height, self.pixels.clone())
            .ok_or_else(|| anyhow::anyhow!("rebuild RGBA image for export"))?;
        for shape in &self.shapes {
            rasterize_shape(&mut img, *shape);
        }
        img.save(&self.path)
            .with_context(|| format!("write annotated PNG {}", self.path.display()))?;
        self.set_outcome(AnnotateOutcome::Saved);
        Ok(())
    }

    fn draft_from_drag(&self, start: Pos2, end: Pos2) -> Shape {
        match self.tool {
            Tool::Arrow => Shape::Arrow {
                from: start,
                to: end,
            },
            Tool::Rect => Shape::Rect { a: start, b: end },
            Tool::Pixelate => Shape::Pixelate { a: start, b: end },
        }
    }
}

impl eframe::App for AnnotateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.finished {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Keyboard: Escape clears draft then cancels; Enter saves; Ctrl+Z undoes.
        // When Settings is open, Escape closes Settings first (does not cancel annotate).
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

        egui::TopBottomPanel::bottom("toolbar")
            .exact_height(44.0)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(0x22, 0x22, 0x24))
                    .inner_margin(8.0),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    tool_button(ui, "Arrow", self.tool == Tool::Arrow, || {
                        self.tool = Tool::Arrow;
                    });
                    tool_button(ui, "Box", self.tool == Tool::Rect, || {
                        self.tool = Tool::Rect;
                    });
                    tool_button(ui, "Pixelate", self.tool == Tool::Pixelate, || {
                        self.tool = Tool::Pixelate;
                    });

                    ui.add_space(8.0);
                    if ui.button("Undo").clicked() {
                        self.undo();
                    }
                    if ui.button("Clear").clicked() {
                        self.clear_all();
                    }
                    if ui.button("Settings").clicked() {
                        self.show_settings = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            self.set_outcome(AnnotateOutcome::Cancelled);
                        }
                        if ui
                            .add(egui::Button::new("Save").fill(ACTION_FILL))
                            .clicked()
                        {
                            if let Err(err) = self.save_and_close() {
                                eprintln!("annotate save failed: {err:#}");
                            }
                        }
                        ui.label(
                            egui::RichText::new("Enter save · Esc cancel")
                                .color(MUTED)
                                .size(12.0),
                        );
                    });
                });
            });

        show_settings_window(ctx, &mut self.show_settings, &mut self.settings);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(0x12, 0x12, 0x14)))
            .show(ctx, |ui| {
                let available = ui.available_size();
                let img_size = Vec2::new(self.width as f32, self.height as f32);
                let scale = (available.x / img_size.x)
                    .min(available.y / img_size.y)
                    .min(1.0);
                let display = img_size * scale;
                let (response, painter) =
                    ui.allocate_painter(available, Sense::click_and_drag());

                let origin = response.rect.min
                    + Vec2::new(
                        (available.x - display.x) * 0.5,
                        (available.y - display.y) * 0.5,
                    );
                let image_rect = Rect::from_min_size(origin, display);

                painter.image(
                    self.texture.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );

                let to_screen = |p: Pos2| -> Pos2 {
                    Pos2::new(origin.x + p.x * scale, origin.y + p.y * scale)
                };
                let to_image = |p: Pos2| -> Pos2 {
                    Pos2::new((p.x - origin.x) / scale, (p.y - origin.y) / scale)
                };

                let stroke = Stroke::new(STROKE_WIDTH, STROKE);
                for shape in self.shapes.iter().chain(self.draft.iter()) {
                    paint_shape(&painter, *shape, to_screen, stroke, scale);
                }

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
            });

        if self.finished {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn tool_button(ui: &mut egui::Ui, label: &str, selected: bool, on_click: impl FnOnce()) {
    let fill = if selected {
        Color32::from_rgb(0x44, 0x44, 0x46)
    } else {
        Color32::from_rgb(0x2C, 0x2C, 0x2E)
    };
    if ui.add(egui::Button::new(label).fill(fill)).clicked() {
        on_click();
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
        Shape::Arrow { from, to } => from.distance(to) >= 4.0,
        Shape::Rect { a, b } | Shape::Pixelate { a, b } => {
            (a.x - b.x).abs() >= 4.0 && (a.y - b.y).abs() >= 4.0
        }
    }
}

fn paint_shape(
    painter: &egui::Painter,
    shape: Shape,
    to_screen: impl Fn(Pos2) -> Pos2,
    stroke: Stroke,
    scale: f32,
) {
    match shape {
        Shape::Arrow { from, to } => {
            let a = to_screen(from);
            let b = to_screen(to);
            painter.line_segment([a, b], stroke);
            for tip in arrow_head_points(a, b, 14.0) {
                painter.line_segment([b, tip], stroke);
            }
        }
        Shape::Rect { a, b } => {
            let r = Rect::from_two_pos(to_screen(a), to_screen(b));
            painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Outside);
        }
        Shape::Pixelate { a, b } => {
            let r = Rect::from_two_pos(to_screen(a), to_screen(b));
            // Preview: muted fill + light outline (actual mosaic applied on Save).
            painter.rect_filled(r, 0.0, PIXELATE_PREVIEW_FILL);
            // Screen-space block hint so the region reads as redaction, not a box stroke.
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
    let color = Rgba([STROKE.r(), STROKE.g(), STROKE.b(), 255]);
    match shape {
        Shape::Arrow { from, to } => {
            draw_thick_line(img, from, to, color, STROKE_WIDTH);
            let a = from;
            let b = to;
            for tip in arrow_head_points(a, b, 14.0) {
                draw_thick_line(img, b, tip, color, STROKE_WIDTH);
            }
        }
        Shape::Rect { a, b } => {
            let x0 = a.x.min(b.x);
            let y0 = a.y.min(b.y);
            let x1 = a.x.max(b.x);
            let y1 = a.y.max(b.y);
            let tl = Pos2::new(x0, y0);
            let tr = Pos2::new(x1, y0);
            let br = Pos2::new(x1, y1);
            let bl = Pos2::new(x0, y1);
            draw_thick_line(img, tl, tr, color, STROKE_WIDTH);
            draw_thick_line(img, tr, br, color, STROKE_WIDTH);
            draw_thick_line(img, br, bl, color, STROKE_WIDTH);
            draw_thick_line(img, bl, tl, color, STROKE_WIDTH);
        }
        Shape::Pixelate { a, b } => {
            pixelate_region(img, a, b, PIXELATE_BLOCK);
        }
    }
}

/// Coarse mosaic over `[a, b]` for privacy. Block size is in image pixels.
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
    // Core line
    draw_line_segment_mut(img, (a.x, a.y), (b.x, b.y), color);
    // Approximate thickness with parallel offsets + end caps.
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
        };
        assert!(!shape_is_meaningful(arrow));
        let rect = Shape::Rect {
            a: Pos2::new(0.0, 0.0),
            b: Pos2::new(10.0, 10.0),
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
            },
        );
        let mid = img.get_pixel(32, 32);
        assert_ne!(mid.0[0], 0, "stroke should paint red channel");
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
        // Within one mosaic cell, all pixels match.
        let p00 = img.get_pixel(0, 0).0;
        let p77 = img.get_pixel(7, 7).0;
        assert_eq!(p00, p77);
        // Outside the region stays checkerboard.
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
        // Mirrors AnnotateApp::cancel_draft_or_session without needing a GPU texture.
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
}
