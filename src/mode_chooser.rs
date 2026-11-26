//! Capture mode chooser: dimmed/blurred full-screen with two centered tiles.
//!
//! Left = Region Select · Right = Full Screen. Esc cancels.
//! Shown when Settings → Capture → Ask Screenshot Type is on.

use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2, ViewportBuilder};
use image::imageops::{self, FilterType};
use image::RgbaImage;

use crate::capture::CaptureMode;
use crate::config::{UiTheme, APP_NAME};
use crate::ui::{self, fit_image_rect, PASTEL_PRIMARY};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeChooserOutcome {
    Region,
    Full,
    Cancelled,
}

impl ModeChooserOutcome {
    pub fn as_capture_mode(self) -> Option<CaptureMode> {
        match self {
            Self::Region => Some(CaptureMode::Region),
            Self::Full => Some(CaptureMode::Full),
            Self::Cancelled => None,
        }
    }
}

/// Fullscreen chooser over an already-captured full-screen PNG.
pub fn run_mode_chooser(png_path: &std::path::Path) -> Result<ModeChooserOutcome> {
    ensure_display_available()?;

    let rgba = image::open(png_path)
        .with_context(|| format!("open capture for mode chooser {}", png_path.display()))?
        .to_rgba8();
    let (img_w, img_h) = rgba.dimensions();
    if img_w == 0 || img_h == 0 {
        bail!("capture has empty dimensions: {}", png_path.display());
    }

    let backdrop = soft_blur_backdrop(&rgba);
    let outcome = Arc::new(Mutex::new(ModeChooserOutcome::Cancelled));
    let outcome_for_app = Arc::clone(&outcome);
    let rgba_bytes = backdrop.into_raw();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(format!("{APP_NAME} — capture mode"))
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
                Ok(Box::new(ModeChooserApp::new(
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
        Ok(Err(e)) => bail!("mode chooser window failed (need a display?): {e}"),
        Err(_) => bail!("mode chooser panicked (missing display libs?)"),
    }

    Ok(outcome
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone())
}

/// Cheap soft backdrop: downscale → slight blur → upscale (reads as frosted/dim).
fn soft_blur_backdrop(src: &RgbaImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let tw = (w / 8).max(1);
    let th = (h / 8).max(1);
    let small = imageops::resize(src, tw, th, FilterType::Triangle);
    let soft = imageops::blur(&small, 1.2);
    let mut full = imageops::resize(&soft, w, h, FilterType::Triangle);
    // Extra dim so tiles stay readable.
    for px in full.pixels_mut() {
        px.0[0] = (u16::from(px.0[0]) * 55 / 100) as u8;
        px.0[1] = (u16::from(px.0[1]) * 55 / 100) as u8;
        px.0[2] = (u16::from(px.0[2]) * 55 / 100) as u8;
    }
    full
}

fn ensure_display_available() -> Result<()> {
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let x11 = std::env::var_os("DISPLAY").is_some();
    if !wayland && !x11 {
        bail!("no WAYLAND_DISPLAY or DISPLAY set (mode chooser needs a graphical session)");
    }
    Ok(())
}

struct ModeChooserApp {
    texture: egui::TextureHandle,
    img_w: u32,
    img_h: u32,
    outcome: Arc<Mutex<ModeChooserOutcome>>,
    close: bool,
    hover: Option<CaptureMode>,
}

impl ModeChooserApp {
    fn new(
        ctx: &egui::Context,
        rgba: Vec<u8>,
        img_w: u32,
        img_h: u32,
        outcome: Arc<Mutex<ModeChooserOutcome>>,
    ) -> Self {
        let color = egui::ColorImage::from_rgba_unmultiplied([img_w as usize, img_h as usize], &rgba);
        let texture = ctx.load_texture("hypershot_mode_chooser", color, Default::default());
        Self {
            texture,
            img_w,
            img_h,
            outcome,
            close: false,
            hover: None,
        }
    }

    fn set_outcome(&mut self, outcome: ModeChooserOutcome) {
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = outcome;
        }
        self.close = true;
    }
}

impl eframe::App for ModeChooserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(0x10, 0x10, 0x12)))
            .show(ctx, |ui| {
                let area = ui.max_rect();
                let (image_rect, _) = fit_image_rect(area, self.img_w as f32, self.img_h as f32);
                let painter = ui.painter_at(area);

                painter.image(
                    self.texture.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                // Extra veil so the control pops.
                painter.rect_filled(
                    area,
                    0.0,
                    Color32::from_rgba_unmultiplied(0x08, 0x08, 0x0A, 0x66),
                );

                let control = chooser_control_rect(area.center());
                let mid_x = control.center().x;
                let left = Rect::from_min_max(control.min, Pos2::new(mid_x, control.max.y));
                let right = Rect::from_min_max(Pos2::new(mid_x, control.min.y), control.max);

                // Panel chrome.
                painter.rect_filled(
                    control,
                    10.0,
                    Color32::from_rgba_unmultiplied(0xF4, 0xF2, 0xF7, 0xF2),
                );
                painter.rect_stroke(
                    control,
                    10.0,
                    Stroke::new(1.0_f32, Color32::from_rgb(0xD0, 0xCC, 0xD8)),
                    egui::StrokeKind::Inside,
                );
                // Vertical divider.
                painter.line_segment(
                    [
                        Pos2::new(mid_x, control.min.y + 14.0),
                        Pos2::new(mid_x, control.max.y - 14.0),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgb(0xC8, 0xC4, 0xD0)),
                );

                let pointer = ui.input(|i| i.pointer.hover_pos());
                self.hover = None;
                if let Some(pos) = pointer {
                    if left.contains(pos) {
                        self.hover = Some(CaptureMode::Region);
                    } else if right.contains(pos) {
                        self.hover = Some(CaptureMode::Full);
                    }
                }

                paint_tile(
                    &painter,
                    left,
                    self.hover == Some(CaptureMode::Region),
                    TileKind::Region,
                );
                paint_tile(
                    &painter,
                    right,
                    self.hover == Some(CaptureMode::Full),
                    TileKind::Full,
                );

                let left_resp = ui.interact(left, ui.id().with("chooser_region"), Sense::click());
                let right_resp = ui.interact(right, ui.id().with("chooser_full"), Sense::click());
                if left_resp.clicked() {
                    self.set_outcome(ModeChooserOutcome::Region);
                }
                if right_resp.clicked() {
                    self.set_outcome(ModeChooserOutcome::Full);
                }
            });

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.set_outcome(ModeChooserOutcome::Cancelled);
            }
            if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::R) {
                self.set_outcome(ModeChooserOutcome::Region);
            }
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::F) {
                self.set_outcome(ModeChooserOutcome::Full);
            }
        });
    }
}

#[derive(Clone, Copy)]
enum TileKind {
    Region,
    Full,
}

fn chooser_control_rect(center: Pos2) -> Rect {
    let size = Vec2::new(320.0, 120.0);
    Rect::from_center_size(center, size)
}

fn paint_tile(painter: &egui::Painter, tile: Rect, hovered: bool, kind: TileKind) {
    if hovered {
        painter.rect_filled(
            tile.shrink(4.0),
            8.0,
            Color32::from_rgba_unmultiplied(0x8E, 0x74, 0xC2, 0x28),
        );
    }
    let ink = if hovered {
        PASTEL_PRIMARY
    } else {
        Color32::from_rgb(0x2A, 0x28, 0x30)
    };
    let icon = Rect::from_center_size(tile.center(), Vec2::splat(56.0));
    match kind {
        TileKind::Region => paint_region_icon(painter, icon, ink),
        TileKind::Full => paint_fullscreen_icon(painter, icon, ink),
    }
}

/// Monitor frame with inner corner brackets (region select).
fn paint_region_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(2.0_f32, color);
    let frame = rect.shrink(6.0);
    painter.rect_stroke(frame, 3.0, stroke, egui::StrokeKind::Outside);
    // Inner selection brackets.
    let inner = frame.shrink(12.0);
    paint_corner_brackets(painter, inner, 10.0, stroke);
}

/// Outer corner brackets only (full screen).
fn paint_fullscreen_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(2.25_f32, color);
    let frame = rect.shrink(8.0);
    paint_corner_brackets(painter, frame, 14.0, stroke);
}

fn paint_corner_brackets(painter: &egui::Painter, rect: Rect, arm: f32, stroke: Stroke) {
    let tl = rect.min;
    let tr = Pos2::new(rect.max.x, rect.min.y);
    let bl = Pos2::new(rect.min.x, rect.max.y);
    let br = rect.max;
    // Top-left
    painter.line_segment([tl, Pos2::new(tl.x + arm, tl.y)], stroke);
    painter.line_segment([tl, Pos2::new(tl.x, tl.y + arm)], stroke);
    // Top-right
    painter.line_segment([tr, Pos2::new(tr.x - arm, tr.y)], stroke);
    painter.line_segment([tr, Pos2::new(tr.x, tr.y + arm)], stroke);
    // Bottom-left
    painter.line_segment([bl, Pos2::new(bl.x + arm, bl.y)], stroke);
    painter.line_segment([bl, Pos2::new(bl.x, bl.y - arm)], stroke);
    // Bottom-right
    painter.line_segment([br, Pos2::new(br.x - arm, br.y)], stroke);
    painter.line_segment([br, Pos2::new(br.x, br.y - arm)], stroke);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_maps_to_capture_mode() {
        assert_eq!(
            ModeChooserOutcome::Region.as_capture_mode(),
            Some(CaptureMode::Region)
        );
        assert_eq!(
            ModeChooserOutcome::Full.as_capture_mode(),
            Some(CaptureMode::Full)
        );
        assert_eq!(ModeChooserOutcome::Cancelled.as_capture_mode(), None);
    }

    #[test]
    fn soft_blur_keeps_dimensions() {
        let src = RgbaImage::from_pixel(64, 48, image::Rgba([200, 180, 160, 255]));
        let out = soft_blur_backdrop(&src);
        assert_eq!(out.dimensions(), (64, 48));
    }

    #[test]
    fn control_rect_is_centered_two_tile_size() {
        let r = chooser_control_rect(Pos2::new(960.0, 540.0));
        assert!((r.width() - 320.0).abs() < 0.1);
        assert!((r.height() - 120.0).abs() < 0.1);
        assert!((r.center().x - 960.0).abs() < 0.1);
    }
}
