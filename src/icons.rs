//! Vendored [Iconoir](https://iconoir.com/) toolbar glyphs.
//!
//! SVGs live under `assets/icons/` (MIT — see `LICENSE-ICONOIR.md`). At paint
//! time each glyph is rasterized with `resvg` at
//! `logical_size × pixels_per_point × ICON_SUPERSAMPLE` so strokes stay sharp
//! on 1080p (1.0) and fractional scales (~1.25–1.5), then drawn at the logical
//! rect (GPU downsamples the supersampled bitmap — never soft-upscales a tiny
//! 1× raster).

use std::collections::HashMap;

use eframe::egui::{self, Color32, ColorImage, Id, Pos2, Rect, TextureHandle, TextureOptions, Vec2};
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
    Help,
}

impl IconId {
    pub const fn iconoir_name(self) -> &'static str {
        match self {
            Self::Arrow => "arrow-up-right",
            Self::Box => "square",
            // Iconoir has no plain `pen`; `edit-pencil` is the clearest pen glyph.
            Self::Highlighter => "edit-pencil",
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
            Self::Help => "question-mark",
        }
    }

    fn svg(self) -> &'static str {
        match self {
            Self::Arrow => include_str!("../assets/icons/arrow-up-right.svg"),
            Self::Box => include_str!("../assets/icons/square.svg"),
            Self::Highlighter => include_str!("../assets/icons/edit-pencil.svg"),
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
            Self::Help => include_str!("../assets/icons/question-mark.svg"),
        }
    }
}

/// Cache key is (icon, device px). Color is applied at draw time via tint so one
/// white alpha-mask texture works on light and dark chrome.
type CacheKey = (IconId, u32);

/// Per-`egui::Context` TextureHandle map. Never process-wide: each eframe window
/// has its own TextureManager, and onscreen → preview reuses the process.
type IconTextureCache = HashMap<CacheKey, TextureHandle>;

fn icon_texture_cache_id() -> Id {
    Id::new("hypershot_iconoir_textures")
}

fn hex_color(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

/// Prepare Iconoir SVG: force opaque white strokes (alpha mask), bump stroke weight.
///
/// Baking ink into the texture + multiplying `Color32::WHITE` at draw time made
/// charcoal glyphs disappear on dark on-screen chrome. White mask + tint is the
/// reliable egui pattern and keeps contrast under caller control.
fn tinted_svg(raw: &str, color: Color32) -> String {
    let hex = hex_color(color);
    raw.replace("currentColor", &hex)
        // Slightly heavier than Iconoir's 1.5 so 15–16 px logical icons stay readable.
        .replace("stroke-width=\"1.5\"", "stroke-width=\"1.75\"")
        .replace("stroke-width=\"1.49993\"", "stroke-width=\"1.75\"")
}

/// Device-pixel size for a logical icon side at the given `pixels_per_point`.
pub fn icon_device_pixels(logical: f32, pixels_per_point: f32) -> u32 {
    let ppp = pixels_per_point.max(1.0);
    let logical = logical.max(1.0);
    // Ceil so fractional scales (1.25 / 1.5) never undersample.
    ((logical * ppp * ICON_SUPERSAMPLE).ceil() as u32).clamp(24, 256)
}

/// Opaque white used for alpha-mask rasterization (tint at draw time).
const MASK_WHITE: Color32 = Color32::WHITE;

/// Rasterize an Iconoir SVG to a white alpha-mask at `px × px` device pixels.
///
/// `color` is accepted for API stability / tests; the mask is always white so
/// callers tint with `painter.image(..., color)`.
pub fn rasterize_icon(id: IconId, color: Color32, px: u32) -> Result<ColorImage, String> {
    let _ = color; // tint happens at draw time
    let px = px.clamp(24, 256);
    let svg = tinted_svg(id.svg(), MASK_WHITE);
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
        // Force RGB to white and keep coverage in alpha so tint multiplies cleanly.
        let a = pixel.alpha();
        if a == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            rgba.extend_from_slice(&[255, 255, 255, a]);
        }
    }

    Ok(ColorImage::from_rgba_unmultiplied(
        [px as usize, px as usize],
        &rgba,
    ))
}

fn texture_for(ctx: &egui::Context, id: IconId, px: u32) -> Option<TextureHandle> {
    let key = (id, px);
    let cache_id = icon_texture_cache_id();

    if let Some(tex) = ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<IconTextureCache>(cache_id)
            .get(&key)
            .cloned()
    }) {
        return Some(tex);
    }

    let image = match rasterize_icon(id, MASK_WHITE, px) {
        Ok(img) => img,
        Err(err) => {
            eprintln!("hypershot icon {}: {err}", id.iconoir_name());
            return None;
        }
    };

    let name = format!("iconoir/{}/mask/{px}", id.iconoir_name());
    // LINEAR minifies the supersampled glyph cleanly; we never upscale a 1× bitmap.
    // Always load into *this* context's TextureManager (never a handle from another window).
    let handle = ctx.load_texture(name, image, TextureOptions::LINEAR);

    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<IconTextureCache>(cache_id)
            .insert(key, handle.clone());
    });
    Some(handle)
}

/// Paint a DPI-aware Iconoir glyph into `rect` (logical coordinates).
///
/// Texture is a white alpha mask; `color` tints it (must contrast the chrome).
pub fn paint_iconoir(painter: &egui::Painter, rect: Rect, color: Color32, id: IconId) {
    let ctx = painter.ctx();
    let ppp = ctx.pixels_per_point().max(1.0);
    let logical = rect.width().min(rect.height()).max(1.0);
    let px = icon_device_pixels(logical, ppp);
    let Some(tex) = texture_for(ctx, id, px) else {
        return;
    };

    // Snap the destination to the pixel grid for crisp edges.
    let size = Vec2::splat(logical);
    let center = Pos2::new(
        (rect.center().x * ppp).round() / ppp,
        (rect.center().y * ppp).round() / ppp,
    );
    let dest = Rect::from_center_size(center, size);
    // Ensure opaque tint — transparent ink would make glyphs vanish.
    let tint = Color32::from_rgb(color.r(), color.g(), color.b());
    painter.image(
        tex.id(),
        dest,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        tint,
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

pub fn paint_icon_help(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_iconoir(painter, rect, color, IconId::Help);
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
        IconId::Help,
    ];

    #[test]
    fn iconoir_names_are_stable() {
        assert_eq!(IconId::Arrow.iconoir_name(), "arrow-up-right");
        assert_eq!(IconId::Highlighter.iconoir_name(), "edit-pencil");
        assert_eq!(IconId::Line.iconoir_name(), "slash");
        assert_eq!(IconId::Pixelate.iconoir_name(), "dots-grid-3x3");
        assert_eq!(IconId::Step.iconoir_name(), "number-1-square");
        assert_eq!(IconId::Text.iconoir_name(), "text");
        assert_eq!(IconId::Magnify.iconoir_name(), "search");
        assert_eq!(IconId::Preview.iconoir_name(), "app-window");
        assert_eq!(IconId::Clear.iconoir_name(), "erase");
        assert_eq!(IconId::Exit.iconoir_name(), "xmark");
        assert_eq!(IconId::Save.iconoir_name(), "download");
        assert_eq!(IconId::Help.iconoir_name(), "question-mark");
    }

    #[test]
    fn every_toolbar_svg_rasterizes_at_hidpi() {
        let ink = Color32::from_rgb(0x3A, 0x36, 0x45);
        for id in ALL {
            // 15 logical @ 2× SSS ≈ toolbar icon on HiDPI.
            let img = rasterize_icon(*id, ink, 30).expect(id.iconoir_name());
            assert_eq!(img.size, [30, 30]);
            let opaque: Vec<_> = img.pixels.iter().filter(|c| c.a() > 0).collect();
            assert!(
                opaque.len() > 20,
                "{} should paint visible ink (got {} px)",
                id.iconoir_name(),
                opaque.len()
            );
            // White alpha-mask: after undoing any premultiply, RGB must be white.
            for c in opaque {
                let a = u32::from(c.a()).max(1);
                let r = (u32::from(c.r()) * 255) / a;
                let g = (u32::from(c.g()) * 255) / a;
                let b = (u32::from(c.b()) * 255) / a;
                assert!(
                    r >= 250 && g >= 250 && b >= 250,
                    "{} mask must be white (got unmultiplied {r},{g},{b} a={})",
                    id.iconoir_name(),
                    c.a()
                );
            }
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

    #[test]
    fn mask_raster_ignores_requested_ink_rgb() {
        // API still takes a color, but the mask is always white.
        let a = rasterize_icon(IconId::Arrow, Color32::RED, 32).unwrap();
        let b = rasterize_icon(IconId::Arrow, Color32::BLUE, 32).unwrap();
        assert_eq!(a.pixels, b.pixels);
    }

    #[test]
    fn icon_textures_belong_to_the_calling_egui_context() {
        // Process-wide TextureHandle cache reuses ids from a prior eframe window
        // (onscreen → preview). The new context's tex_manager then has no meta for
        // that id → blank but clickable toolbar icons.
        let ctx_a = egui::Context::default();
        let handle_a = texture_for(&ctx_a, IconId::Exit, 32).expect("load in A");
        assert!(
            ctx_a.tex_manager().read().meta(handle_a.id()).is_some(),
            "texture must register in context A"
        );
        // Same context must reuse the handle (per-window cache hit).
        let handle_a2 = texture_for(&ctx_a, IconId::Exit, 32).expect("cache hit in A");
        assert_eq!(handle_a.id(), handle_a2.id());
        drop(handle_a);
        drop(handle_a2);
        drop(ctx_a);

        let ctx_b = egui::Context::default();
        let handle_b = texture_for(&ctx_b, IconId::Exit, 32).expect("load in B");
        assert!(
            ctx_b.tex_manager().read().meta(handle_b.id()).is_some(),
            "must load into context B's TextureManager, not reuse a stale handle from A"
        );
        let handle_b2 = texture_for(&ctx_b, IconId::Exit, 32).expect("cache hit in B");
        assert_eq!(handle_b.id(), handle_b2.id());
    }
}
