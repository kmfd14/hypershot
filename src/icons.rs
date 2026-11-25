//! Vendored [Iconoir](https://iconoir.com/) toolbar glyphs.
//!
//! SVGs live under `assets/icons/` (MIT — see `LICENSE-ICONOIR.md`). At paint
//! time each glyph is rasterized with `resvg` at
//! `logical_size × pixels_per_point × ICON_SUPERSAMPLE` so strokes stay sharp
//! on 1080p (1.0) and fractional scales (~1.25–1.5), then drawn at the logical
//! rect (GPU downsamples the supersampled bitmap — never soft-upscales a tiny
//! 1× raster).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, TextureHandle, TextureOptions, Vec2};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};

/// Extra device-pixel density when rasterizing Iconoir SVGs.
///
/// At `pixels_per_point == 1.0` (common 1080p), a 15–17 px logical icon is too
/// small for thin 1.5-width strokes — LINEAR sampling then looks soft/pixelated.
/// Always render at ≥2× device pixels and let the GPU minify.
pub const ICON_SUPERSAMPLE: f32 = 2.0;

/// Stable id for cache keys + texture names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconId {
    Arrow,
    Box,
    Highlighter,
    Line,
    Pixelate,
    Step,
    Text,
    Magnify,
    Undo,
    Redo,
    Clear,
    Settings,
    Save,
    Copy,
    Exit,
    Preview,
}

impl IconId {
    pub const fn iconoir_name(self) -> &'static str {
        match self {
            Self::Arrow => "arrow-up-right",
            Self::Box => "square",
            Self::Highlighter => "design-nib",
            Self::Line => "slash",
            Self::Pixelate => "dots-grid-3x3",
            Self::Step => "number-1-square",
            Self::Text => "text",
            Self::Magnify => "search",
            Self::Undo => "undo",
            Self::Redo => "redo",
            Self::Clear => "erase",
            Self::Settings => "settings",
            Self::Save => "download",
            Self::Copy => "copy",
            Self::Exit => "xmark",
            Self::Preview => "app-window",
        }
    }

    fn svg(self) -> &'static str {
        match self {
            Self::Arrow => include_str!("../assets/icons/arrow-up-right.svg"),
            Self::Box => include_str!("../assets/icons/square.svg"),
            Self::Highlighter => include_str!("../assets/icons/design-nib.svg"),
            Self::Line => include_str!("../assets/icons/slash.svg"),
            Self::Pixelate => include_str!("../assets/icons/dots-grid-3x3.svg"),
            Self::Step => include_str!("../assets/icons/number-1-square.svg"),
            Self::Text => include_str!("../assets/icons/text.svg"),
            Self::Magnify => include_str!("../assets/icons/search.svg"),
            Self::Undo => include_str!("../assets/icons/undo.svg"),
            Self::Redo => include_str!("../assets/icons/redo.svg"),
            Self::Clear => include_str!("../assets/icons/erase.svg"),
            Self::Settings => include_str!("../assets/icons/settings.svg"),
            Self::Save => include_str!("../assets/icons/download.svg"),
            Self::Copy => include_str!("../assets/icons/copy.svg"),
            Self::Exit => include_str!("../assets/icons/xmark.svg"),
            Self::Preview => include_str!("../assets/icons/app-window.svg"),
        }
    }
}

type CacheKey = (IconId, u32, u32);

fn texture_cache() -> &'static Mutex<HashMap<CacheKey, TextureHandle>> {
    static CACHE: OnceLock<Mutex<HashMap<CacheKey, TextureHandle>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn color_key(c: Color32) -> u32 {
    u32::from_be_bytes([c.r(), c.g(), c.b(), c.a()])
}

fn hex_color(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

/// Prepare Iconoir SVG: tint `currentColor`, bump stroke slightly for 24→toolbar scale.
fn tinted_svg(raw: &str, color: Color32) -> String {
    let hex = hex_color(color);
    raw.replace("currentColor", &hex)
        // Slightly heavier than Iconoir's 1.5 so 15–16 px logical icons stay readable.
        .replace("stroke-width=\"1.5\"", "stroke-width=\"1.75\"")
}

/// Device-pixel size for a logical icon side at the given `pixels_per_point`.
pub fn icon_device_pixels(logical: f32, pixels_per_point: f32) -> u32 {
    let ppp = pixels_per_point.max(1.0);
    let logical = logical.max(1.0);
    // Ceil so fractional scales (1.25 / 1.5) never undersample.
    ((logical * ppp * ICON_SUPERSAMPLE).ceil() as u32).clamp(24, 256)
}

/// Rasterize an Iconoir SVG to premultiplied RGBA at `px × px` device pixels.
pub fn rasterize_icon(id: IconId, color: Color32, px: u32) -> Result<ColorImage, String> {
    let px = px.clamp(24, 256);
    let svg = tinted_svg(id.svg(), color);
    let opts = Options::default();
    let tree = Tree::from_str(&svg, &opts).map_err(|e| format!("parse {}: {e}", id.iconoir_name()))?;

    let mut pixmap =
        Pixmap::new(px, px).ok_or_else(|| format!("pixmap {px}x{px} for {}", id.iconoir_name()))?;

    // Iconoir viewBox is 24×24; scale into the pixmap.
    let size = tree.size();
    let sx = px as f32 / size.width().max(1.0);
    let sy = px as f32 / size.height().max(1.0);
    let transform = Transform::from_scale(sx, sy);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let mut rgba = Vec::with_capacity((px * px * 4) as usize);
    for pixel in pixmap.pixels() {
        // tiny-skia is premultiplied; egui ColorImage expects unmultiplied.
        let a = pixel.alpha();
        if a == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else if a == 255 {
            rgba.extend_from_slice(&[pixel.red(), pixel.green(), pixel.blue(), 255]);
        } else {
            let a32 = u32::from(a);
            rgba.push(((u32::from(pixel.red()) * 255) / a32).min(255) as u8);
            rgba.push(((u32::from(pixel.green()) * 255) / a32).min(255) as u8);
            rgba.push(((u32::from(pixel.blue()) * 255) / a32).min(255) as u8);
            rgba.push(a);
        }
    }

    Ok(ColorImage::from_rgba_unmultiplied(
        [px as usize, px as usize],
        &rgba,
    ))
}

fn texture_for(
    ctx: &egui::Context,
    id: IconId,
    color: Color32,
    px: u32,
) -> Option<TextureHandle> {
    let key = (id, px, color_key(color));
    if let Ok(cache) = texture_cache().lock() {
        if let Some(tex) = cache.get(&key) {
            return Some(tex.clone());
        }
    }

    let image = match rasterize_icon(id, color, px) {
        Ok(img) => img,
        Err(err) => {
            eprintln!("hypershot icon {}: {err}", id.iconoir_name());
            return None;
        }
    };

    let name = format!(
        "iconoir/{}/{px}/{:08x}",
        id.iconoir_name(),
        color_key(color)
    );
    // LINEAR minifies the supersampled glyph cleanly; we never upscale a 1× bitmap.
    let handle = ctx.load_texture(name, image, TextureOptions::LINEAR);

    if let Ok(mut cache) = texture_cache().lock() {
        cache.insert(key, handle.clone());
    }
    Some(handle)
}

/// Paint a DPI-aware Iconoir glyph into `rect` (logical coordinates).
pub fn paint_iconoir(painter: &egui::Painter, rect: Rect, color: Color32, id: IconId) {
    let ctx = painter.ctx();
    let ppp = ctx.pixels_per_point().max(1.0);
    let logical = rect.width().min(rect.height()).max(1.0);
    let px = icon_device_pixels(logical, ppp);
    let Some(tex) = texture_for(ctx, id, color, px) else {
        return;
    };

    // Snap the destination to the pixel grid for crisp edges.
    let size = Vec2::splat(logical);
    let center = Pos2::new(
        (rect.center().x * ppp).round() / ppp,
        (rect.center().y * ppp).round() / ppp,
    );
    let dest = Rect::from_center_size(center, size);
    painter.image(
        tex.id(),
        dest,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
}

// --- Toolbar paint helpers (same signatures as the old hand-stroked icons) ----

pub fn paint_icon_arrow(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Arrow);
}

pub fn paint_icon_box(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Box);
}

pub fn paint_icon_highlighter(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Highlighter);
}

pub fn paint_icon_line(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Line);
}

pub fn paint_icon_pixelate(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Pixelate);
}

pub fn paint_icon_step(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Step);
}

pub fn paint_icon_text(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Text);
}

pub fn paint_icon_magnify(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Magnify);
}

pub fn paint_icon_undo(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Undo);
}

pub fn paint_icon_redo(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Redo);
}

pub fn paint_icon_clear(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Clear);
}

pub fn paint_icon_settings(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Settings);
}

pub fn paint_icon_save(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Save);
}

pub fn paint_icon_copy(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Copy);
}

pub fn paint_icon_exit(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Exit);
}

pub fn paint_icon_preview(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Preview);
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: &[IconId] = &[
        IconId::Arrow,
        IconId::Box,
        IconId::Highlighter,
        IconId::Line,
        IconId::Pixelate,
        IconId::Step,
        IconId::Text,
        IconId::Magnify,
        IconId::Undo,
        IconId::Redo,
        IconId::Clear,
        IconId::Settings,
        IconId::Save,
        IconId::Copy,
        IconId::Exit,
        IconId::Preview,
    ];

    #[test]
    fn iconoir_names_are_stable() {
        assert_eq!(IconId::Arrow.iconoir_name(), "arrow-up-right");
        assert_eq!(IconId::Highlighter.iconoir_name(), "design-nib");
        assert_eq!(IconId::Line.iconoir_name(), "slash");
        assert_eq!(IconId::Pixelate.iconoir_name(), "dots-grid-3x3");
        assert_eq!(IconId::Step.iconoir_name(), "number-1-square");
        assert_eq!(IconId::Text.iconoir_name(), "text");
        assert_eq!(IconId::Magnify.iconoir_name(), "search");
        assert_eq!(IconId::Preview.iconoir_name(), "app-window");
        assert_eq!(IconId::Clear.iconoir_name(), "erase");
        assert_eq!(IconId::Exit.iconoir_name(), "xmark");
        assert_eq!(IconId::Save.iconoir_name(), "download");
    }

    #[test]
    fn every_toolbar_svg_rasterizes_at_hidpi() {
        let ink = Color32::from_rgb(0x3A, 0x36, 0x45);
        for id in ALL {
            // 15 logical @ 2× SSS ≈ toolbar icon on Wayland HiDPI.
            let img = rasterize_icon(*id, ink, 30).expect(id.iconoir_name());
            assert_eq!(img.size, [30, 30]);
            let opaque = img.pixels.iter().filter(|c| c.a() > 0).count();
            assert!(
                opaque > 20,
                "{} should paint visible ink (got {opaque} px)",
                id.iconoir_name()
            );
        }
    }

    #[test]
    fn supersample_covers_common_display_scales() {
        // ~15.6–16.8 logical (TOOL_BTN * 0.56) at 1.0 / 1.25 / 1.5.
        let logical = 16.0_f32;
        let px_1 = icon_device_pixels(logical, 1.0);
        let px_125 = icon_device_pixels(logical, 1.25);
        let px_15 = icon_device_pixels(logical, 1.5);
        assert_eq!(px_1, 32); // 16 * 1 * 2
        assert_eq!(px_125, 40); // ceil(16 * 1.25 * 2)
        assert_eq!(px_15, 48); // 16 * 1.5 * 2
        // Always denser than the destination device pixels (no soft upscale).
        assert!(px_1 as f32 >= logical * 1.0 * 1.9);
        assert!(px_125 as f32 >= logical * 1.25 * 1.9);
        assert!(px_15 as f32 >= logical * 1.5 * 1.9);
    }

    #[test]
    fn tint_replaces_current_color() {
        let raw = IconId::Exit.svg();
        let out = tinted_svg(raw, Color32::from_rgb(0xAB, 0xCD, 0xEF));
        assert!(out.contains("#ABCDEF"));
        assert!(!out.contains("currentColor"));
    }
}
