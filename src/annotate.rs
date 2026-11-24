//! Fullscreen egui annotate overlay (Milestone 3).
//!
//! Quiet desktop chrome: neutral surfaces, small radius, no glow / pills / emoji.
//! Design dials: ENERGY 1 / RHYTHM 1 / MOTION 1 (tool overlay, not marketing UI).

use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2, ViewportBuilder};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_line_segment_mut};

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
}

#[derive(Debug, Clone, Copy)]
enum Shape {
    Arrow { from: Pos2, to: Pos2 },
    Rect { a: Pos2, b: Pos2 },
}

/// Open a fullscreen annotate overlay on an existing PNG.
///
/// On Save / Enter: composites annotations onto the image and overwrites `path`.
/// On Escape: returns [`AnnotateOutcome::Cancelled`] without modifying the file.
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
                apply_quiet_visuals(&cc.egui_ctx);
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

/// Neutral quiet chrome — reason: tool overlay should not compete with the screenshot.
fn apply_quiet_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    // Charcoal tool chrome: keeps attention on the capture (creative-tool dark, R-21).
    visuals.panel_fill = Color32::from_rgb(0x22, 0x22, 0x24);
    visuals.window_fill = Color32::from_rgb(0x22, 0x22, 0x24);
    visuals.extreme_bg_color = Color32::from_rgb(0x1A, 0x1A, 0x1C);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x2C, 0x2C, 0x2E);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x2C, 0x2C, 0x2E);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x38, 0x38, 0x3A);
    visuals.widgets.active.bg_fill = Color32::from_rgb(0x44, 0x44, 0x46);
    visuals.selection.bg_fill = Color32::from_rgb(0x3A, 0x3A, 0x3C);
    // Kill egui dark-default cyan/blue selection stroke (R-01 / quiet chrome).
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(0xD0, 0xD0, 0xD0));
    visuals.hyperlink_color = Color32::from_rgb(0xB0, 0xB0, 0xB0);
    visuals.widgets.inactive.fg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(0xD0, 0xD0, 0xD0));
    visuals.widgets.hovered.fg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(0xF0, 0xF0, 0xF0));
    visuals.widgets.noninteractive.fg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(0xB0, 0xB0, 0xB0));
    // Small radius, not pills (R-11).
    visuals.widgets.noninteractive.corner_radius = 3.0.into();
    visuals.widgets.inactive.corner_radius = 3.0.into();
    visuals.widgets.hovered.corner_radius = 3.0.into();
    visuals.widgets.active.corner_radius = 3.0.into();
    visuals.widgets.open.corner_radius = 3.0.into();
    // No soft floating shadows (R-12).
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.popup_shadow = egui::Shadow::NONE;
    ctx.set_visuals(visuals);
}

/// Markup stroke color — one deliberate accent on the image, not on chrome (R-29 / Part 3).
const STROKE: Color32 = Color32::from_rgb(0xE2, 0x4A, 0x3B);
const STROKE_WIDTH: f32 = 3.0;

struct AnnotateApp {
    texture: egui::TextureHandle,
    width: u32,
    height: u32,
    /// Original pixels (RGBA8).
    pixels: Vec<u8>,
    shapes: Vec<Shape>,
    tool: Tool,
    drag_start: Option<Pos2>,
    draft: Option<Shape>,
    path: std::path::PathBuf,
    outcome: Arc<Mutex<AnnotateOutcome>>,
    finished: bool,
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
            tool: Tool::Arrow,
            drag_start: None,
            draft: None,
            path,
            outcome,
            finished: false,
        }
    }

    fn set_outcome(&mut self, outcome: AnnotateOutcome) {
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = outcome;
        }
        self.finished = true;
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
}

impl eframe::App for AnnotateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.finished {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Keyboard: Escape cancel, Enter save, Ctrl+Z undo.
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.set_outcome(AnnotateOutcome::Cancelled);
            }
            if i.key_pressed(egui::Key::Enter) {
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
            .frame(egui::Frame::new().fill(Color32::from_rgb(0x22, 0x22, 0x24)).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    tool_button(ui, "Arrow", self.tool == Tool::Arrow, || {
                        self.tool = Tool::Arrow;
                    });
                    tool_button(ui, "Box", self.tool == Tool::Rect, || {
                        self.tool = Tool::Rect;
                    });

                    ui.add_space(8.0);
                    if ui.button("Undo").clicked() {
                        self.undo();
                    }
                    if ui.button("Clear").clicked() {
                        self.clear_all();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            self.set_outcome(AnnotateOutcome::Cancelled);
                        }
                        if ui
                            .add(egui::Button::new("Save").fill(Color32::from_rgb(0x3A, 0x3A, 0x3C)))
                            .clicked()
                        {
                            if let Err(err) = self.save_and_close() {
                                eprintln!("annotate save failed: {err:#}");
                            }
                        }
                        ui.label(
                            egui::RichText::new("Enter save · Esc cancel")
                                // #A0 on #222 ≈ 5.7:1 (WCAG AA for normal text).
                                .color(Color32::from_rgb(0xA0, 0xA0, 0xA0))
                                .size(12.0),
                        );
                    });
                });
            });

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
                    Pos2::new(
                        origin.x + p.x * scale,
                        origin.y + p.y * scale,
                    )
                };
                let to_image = |p: Pos2| -> Pos2 {
                    Pos2::new((p.x - origin.x) / scale, (p.y - origin.y) / scale)
                };

                let stroke = Stroke::new(STROKE_WIDTH, STROKE);
                for shape in self.shapes.iter().chain(self.draft.iter()) {
                    paint_shape(&painter, *shape, to_screen, stroke);
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
                        self.draft = Some(match self.tool {
                            Tool::Arrow => Shape::Arrow {
                                from: start,
                                to: end,
                            },
                            Tool::Rect => Shape::Rect { a: start, b: end },
                        });
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
        Shape::Rect { a, b } => (a.x - b.x).abs() >= 4.0 && (a.y - b.y).abs() >= 4.0,
    }
}

fn paint_shape(
    painter: &egui::Painter,
    shape: Shape,
    to_screen: impl Fn(Pos2) -> Pos2,
    stroke: Stroke,
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
    }
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
}
