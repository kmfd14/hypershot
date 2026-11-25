//! Fullscreen region picker with live `W x H` resolution text.
//!
//! Region capture: take a full-screen PNG first, then let the user drag-select
//! a crop. While dragging, a label like `1280 x 720` tracks the selection.
//! Esc / empty selection cancels. Release confirms when the rect is meaningful.

use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2, ViewportBuilder};
use image::RgbaImage;

use crate::config::{UiTheme, APP_NAME};
use crate::ui::{self, fit_image_rect, AnnotatePalette, PASTEL_PRIMARY};

/// Pixel crop chosen by the user (inclusive origin, exclusive-ish size).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionCrop {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionSelectOutcome {
    Selected(RegionCrop),
    Cancelled,
}

/// Open a fullscreen picker over `png_path`. On confirm, crops the file in place.
pub fn run_region_select_and_crop(png_path: &Path) -> Result<RegionSelectOutcome> {
    ensure_display_available()?;

    let rgba = image::open(png_path)
        .with_context(|| format!("open capture for region select {}", png_path.display()))?
        .to_rgba8();
    let (img_w, img_h) = rgba.dimensions();
    if img_w == 0 || img_h == 0 {
        bail!("capture has empty dimensions: {}", png_path.display());
    }

    let outcome = Arc::new(Mutex::new(RegionSelectOutcome::Cancelled));
    let outcome_for_app = Arc::clone(&outcome);
    let rgba_bytes = rgba.as_raw().clone();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(format!("{APP_NAME} — select region"))
            .with_fullscreen(true)
            .with_decorations(false)
            .with_transparent(false)
            .with_always_on_top(),
        centered: true,
        ..Default::default()
    };

    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            APP_NAME,
            options,
            Box::new(move |cc| {
                ui::apply_annotate_visuals(&cc.egui_ctx, UiTheme::System);
                Ok(Box::new(RegionSelectApp::new(
                    &cc.egui_ctx,
                    rgba_bytes,
                    img_w,
                    img_h,
                    outcome_for_app,
                )))
            }),
        )
    }));

    match run_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => bail!("region select window failed (need a display?): {e}"),
        Err(_) => bail!("region select panicked (missing display libs?)"),
    }

    let result = outcome
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();

    if let RegionSelectOutcome::Selected(crop) = result {
        crop_png_in_place(png_path, &rgba, crop)?;
        Ok(RegionSelectOutcome::Selected(crop))
    } else {
        Ok(RegionSelectOutcome::Cancelled)
    }
}

fn crop_png_in_place(path: &Path, src: &RgbaImage, crop: RegionCrop) -> Result<()> {
    let cropped = image::imageops::crop_imm(src, crop.x, crop.y, crop.w, crop.h).to_image();
    cropped
        .save(path)
        .with_context(|| format!("write cropped region to {}", path.display()))?;
    Ok(())
}

fn ensure_display_available() -> Result<()> {
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let x11 = std::env::var_os("DISPLAY").is_some();
    if !wayland && !x11 {
        bail!("no WAYLAND_DISPLAY or DISPLAY set (region select needs a graphical session)");
    }
    Ok(())
}

struct RegionSelectApp {
    texture: egui::TextureHandle,
    img_w: u32,
    img_h: u32,
    drag_start: Option<Pos2>,
    /// Image-space corners while dragging / after drag.
    selection: Option<(Pos2, Pos2)>,
    outcome: Arc<Mutex<RegionSelectOutcome>>,
    close: bool,
}

impl RegionSelectApp {
    fn new(
        ctx: &egui::Context,
        rgba: Vec<u8>,
        img_w: u32,
        img_h: u32,
        outcome: Arc<Mutex<RegionSelectOutcome>>,
    ) -> Self {
        let color = egui::ColorImage::from_rgba_unmultiplied([img_w as usize, img_h as usize], &rgba);
        let texture = ctx.load_texture("hypershot_region_full", color, Default::default());
        Self {
            texture,
            img_w,
            img_h,
            drag_start: None,
            selection: None,
            outcome,
            close: false,
        }
    }

    fn set_outcome(&mut self, outcome: RegionSelectOutcome) {
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = outcome;
        }
        self.close = true;
    }

    fn confirm_if_valid(&mut self) {
        if let Some(crop) = self.selection.and_then(|(a, b)| corners_to_crop(a, b, self.img_w, self.img_h))
        {
            self.set_outcome(RegionSelectOutcome::Selected(crop));
        }
    }
}

impl eframe::App for RegionSelectApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        let dark = ctx.style().visuals.dark_mode;
        let pal = AnnotatePalette::for_mode(dark, false);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(0x10, 0x10, 0x12)))
            .show(ctx, |ui| {
                let area = ui.max_rect();
                let (image_rect, scale) =
                    fit_image_rect(area, self.img_w as f32, self.img_h as f32);
                let response = ui.interact(area, ui.id().with("region_select"), Sense::click_and_drag());
                let painter = ui.painter_at(area);

                painter.image(
                    self.texture.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );

                // Dim everything; clear the selection hole when present.
                let dim = Color32::from_rgba_unmultiplied(0x10, 0x10, 0x12, 0xB0);
                if let Some((a, b)) = self.selection {
                    let sel = Rect::from_two_pos(
                        image_to_screen(a, image_rect.min, scale),
                        image_to_screen(b, image_rect.min, scale),
                    )
                    .intersect(image_rect);
                    // Four panels around the selection.
                    paint_dim_around(&painter, image_rect, sel, dim);
                    painter.rect_stroke(
                        sel,
                        0.0,
                        Stroke::new(2.0_f32, PASTEL_PRIMARY),
                        egui::StrokeKind::Outside,
                    );

                    let w = (a.x - b.x).abs().round().max(1.0) as u32;
                    let h = (a.y - b.y).abs().round().max(1.0) as u32;
                    let label_pos = Pos2::new(sel.right() + 8.0, sel.top());
                    paint_size_label(&painter, label_pos, w, h, &pal);
                } else {
                    painter.rect_filled(image_rect, 0.0, dim);
                }

                let origin = image_rect.min;
                let to_image = |p: Pos2| -> Pos2 {
                    Pos2::new(
                        ((p.x - origin.x) / scale).clamp(0.0, (self.img_w.saturating_sub(1)) as f32),
                        ((p.y - origin.y) / scale).clamp(0.0, (self.img_h.saturating_sub(1)) as f32),
                    )
                };

                if response.drag_started() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if image_rect.contains(pos) {
                            let img = to_image(pos);
                            self.drag_start = Some(img);
                            self.selection = Some((img, img));
                        }
                    }
                }
                if response.dragged() {
                    if let (Some(start), Some(pos)) =
                        (self.drag_start, response.interact_pointer_pos())
                    {
                        self.selection = Some((start, to_image(pos)));
                    }
                }
                if response.drag_stopped() {
                    self.drag_start = None;
                    self.confirm_if_valid();
                }

                // Hint when idle.
                if self.selection.is_none() {
                    painter.text(
                        area.center(),
                        egui::Align2::CENTER_CENTER,
                        "Drag to select a region  ·  Esc to cancel",
                        egui::FontId::proportional(16.0),
                        Color32::from_rgba_unmultiplied(0xF7, 0xF5, 0xFA, 0xE0),
                    );
                }
            });

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.set_outcome(RegionSelectOutcome::Cancelled);
            }
            if i.key_pressed(egui::Key::Enter) {
                self.confirm_if_valid();
            }
        });
    }
}

fn image_to_screen(p: Pos2, origin: Pos2, scale: f32) -> Pos2 {
    Pos2::new(origin.x + p.x * scale, origin.y + p.y * scale)
}

fn corners_to_crop(a: Pos2, b: Pos2, img_w: u32, img_h: u32) -> Option<RegionCrop> {
    let x0 = a.x.min(b.x).floor().max(0.0) as u32;
    let y0 = a.y.min(b.y).floor().max(0.0) as u32;
    let x1 = a.x.max(b.x).ceil().min(img_w as f32) as u32;
    let y1 = a.y.max(b.y).ceil().min(img_h as f32) as u32;
    let w = x1.saturating_sub(x0);
    let h = y1.saturating_sub(y0);
    if w < 2 || h < 2 {
        return None;
    }
    Some(RegionCrop { x: x0, y: y0, w, h })
}

fn paint_dim_around(painter: &egui::Painter, outer: Rect, hole: Rect, dim: Color32) {
    let hole = hole.intersect(outer);
    if hole.width() <= 0.0 || hole.height() <= 0.0 {
        painter.rect_filled(outer, 0.0, dim);
        return;
    }
    // Top
    painter.rect_filled(
        Rect::from_min_max(outer.min, Pos2::new(outer.max.x, hole.min.y)),
        0.0,
        dim,
    );
    // Bottom
    painter.rect_filled(
        Rect::from_min_max(Pos2::new(outer.min.x, hole.max.y), outer.max),
        0.0,
        dim,
    );
    // Left
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(outer.min.x, hole.min.y),
            Pos2::new(hole.min.x, hole.max.y),
        ),
        0.0,
        dim,
    );
    // Right
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(hole.max.x, hole.min.y),
            Pos2::new(outer.max.x, hole.max.y),
        ),
        0.0,
        dim,
    );
}

fn paint_size_label(painter: &egui::Painter, pos: Pos2, w: u32, h: u32, pal: &AnnotatePalette) {
    let text = format!("{w} x {h}");
    let font = egui::FontId::proportional(13.0);
    let galley = painter.layout_no_wrap(text, font, crate::ui::ICON_DARK);
    let pad = Vec2::new(10.0, 5.0);
    let mut rect = Rect::from_min_size(pos, galley.size() + pad * 2.0);
    // Keep label on-screen if the selection is near the right edge.
    let screen = painter.clip_rect();
    if rect.right() > screen.right() - 4.0 {
        rect = rect.translate(Vec2::new(screen.right() - 4.0 - rect.right(), 0.0));
    }
    if rect.top() < screen.top() + 4.0 {
        rect = rect.translate(Vec2::new(0.0, screen.top() + 4.0 - rect.top()));
    }
    painter.rect_filled(rect, 6.0, Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0xF2));
    painter.rect_stroke(
        rect,
        6.0,
        Stroke::new(1.0_f32, pal.toolbar_stroke),
        egui::StrokeKind::Inside,
    );
    painter.galley(rect.min + pad, galley, Color32::PLACEHOLDER);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_to_crop_orders_and_rejects_tiny() {
        let crop = corners_to_crop(
            Pos2::new(100.0, 50.0),
            Pos2::new(10.0, 200.0),
            1920,
            1080,
        )
        .expect("valid crop");
        assert_eq!(crop.x, 10);
        assert_eq!(crop.y, 50);
        assert_eq!(crop.w, 90);
        assert_eq!(crop.h, 150);

        assert!(corners_to_crop(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0), 100, 100).is_none());
    }

    #[test]
    fn format_matches_preference_example() {
        let label = format!("{} x {}", 1280, 720);
        assert_eq!(label, "1280 x 720");
    }
}
