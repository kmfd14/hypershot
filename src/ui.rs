//! Shared visuals for annotate + settings.
//!
//! Annotate: floating content layer + one Liquid Glass functional strip (HIG materials /
//! liquid-glass: glass only on chrome, never on the capture). Settings: Apple-inspired
//! grouped light chrome (clarity, deference, depth) — no purple / glow / marketing pills.
//!
//! Design dials: ENERGY 2 / RHYTHM 2 / MOTION 1. antislop active: during (session override).

use eframe::egui::{self, Color32, CornerRadius, Margin, Shadow, Stroke};

// --- Annotate (content layer) -------------------------------------------------

/// Floating window fill behind the capture (opaque content layer).
pub const ANNOTATE_BG: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);

/// Regular Liquid Glass approximation: translucent fill (egui has no backdrop-filter).
/// Premultiplied approx of unmultiplied (0x2C,0x2C,0x2E,0xB8).
pub const GLASS_FILL: Color32 = Color32::from_rgba_premultiplied(0x1F, 0x1F, 0x21, 0xB8);

/// Hairline light catch on glass (top/leading edge).
pub const GLASS_STROKE: Color32 = Color32::from_rgba_premultiplied(0x38, 0x38, 0x38, 0x38);

/// Selected tool chip on glass (monochrome; not a brand tint).
pub const GLASS_SELECTED: Color32 = Color32::from_rgba_premultiplied(0x28, 0x28, 0x28, 0x28);

/// Idle tool chip.
pub const GLASS_IDLE: Color32 = Color32::from_rgba_premultiplied(0x10, 0x10, 0x10, 0x10);

/// Pill radius for the vertical glass strip only (KM sketch; intentional exception to
/// flat chrome elsewhere).
pub const GLASS_PILL_RADIUS: f32 = 22.0;

/// Toolbar strip width (desktop targets ≥ 28 pt).
pub const TOOLBAR_WIDTH: f32 = 56.0;

/// Gutter between glass strip and image.
pub const TOOLBAR_GUTTER: f32 = 12.0;

/// Outer padding inside the floating window.
pub const WINDOW_PAD: f32 = 14.0;

// --- Settings (grouped light) -------------------------------------------------

pub const SETTINGS_BG: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF7);
pub const SETTINGS_CARD: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const SETTINGS_SEPARATOR: Color32 = Color32::from_rgb(0xC6, 0xC6, 0xC8);
pub const SETTINGS_LABEL: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);
pub const SETTINGS_SECONDARY: Color32 = Color32::from_rgb(0x6C, 0x6C, 0x70);
pub const SETTINGS_FIELD: Color32 = Color32::from_rgb(0xE8, 0xE8, 0xED);

/// Muted helper text on dark annotate chrome.
pub const MUTED: Color32 = Color32::from_rgb(0xA0, 0xA0, 0xA0);

/// Primary action fill on dark chrome (lifted charcoal — not neon).
pub const ACTION_FILL: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3C);

/// Prominent Save on glass (one tinted primary per HIG buttons / liquid-glass color budget).
pub const GLASS_PRIMARY: Color32 = Color32::from_rgb(0x0A, 0x84, 0xFF);

/// Settings Save / primary on light surfaces (same accent family, no purple).
pub const SETTINGS_PRIMARY: Color32 = Color32::from_rgb(0x00, 0x7A, 0xFF);

/// Dark annotate + glass-friendly widgets.
pub fn apply_annotate_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = ANNOTATE_BG;
    visuals.window_fill = Color32::from_rgb(0x2C, 0x2C, 0x2E);
    visuals.extreme_bg_color = Color32::from_rgb(0x1A, 0x1A, 0x1C);
    visuals.override_text_color = Some(Color32::from_rgb(0xF2, 0xF2, 0xF7));
    visuals.widgets.noninteractive.bg_fill = ANNOTATE_BG;
    visuals.widgets.noninteractive.weak_bg_fill = ANNOTATE_BG;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0x3A, 0x3A, 0x3C));
    visuals.widgets.inactive.bg_fill = GLASS_IDLE;
    visuals.widgets.inactive.weak_bg_fill = GLASS_IDLE;
    visuals.widgets.hovered.bg_fill = GLASS_SELECTED;
    visuals.widgets.hovered.weak_bg_fill = GLASS_SELECTED;
    visuals.widgets.active.bg_fill = Color32::from_rgba_premultiplied(0x38, 0x38, 0x38, 0x38);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgba_premultiplied(0x38, 0x38, 0x38, 0x38);
    visuals.widgets.open.bg_fill = GLASS_IDLE;
    visuals.widgets.open.weak_bg_fill = GLASS_IDLE;
    visuals.selection.bg_fill = Color32::from_rgb(0x3A, 0x3A, 0x3C);
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(0xD0, 0xD0, 0xD0));
    visuals.text_cursor.stroke = Stroke::new(2.0_f32, Color32::from_rgb(0xD0, 0xD0, 0xD0));
    visuals.hyperlink_color = Color32::from_rgb(0xB0, 0xB0, 0xB0);
    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0xF2, 0xF2, 0xF7));
        w.corner_radius = CornerRadius::same(8);
    }
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);
    visuals.window_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0x3A, 0x3A, 0x3C));
    // Soft depth under the floating settings sheet only — not a page-wide glow.
    visuals.window_shadow = Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(80),
    };
    visuals.popup_shadow = Shadow::NONE;
    ctx.set_visuals(visuals);
}

/// Light Apple-inspired settings (grouped cards, deference to content).
pub fn apply_settings_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = SETTINGS_BG;
    visuals.window_fill = SETTINGS_CARD;
    visuals.extreme_bg_color = SETTINGS_FIELD;
    visuals.override_text_color = Some(SETTINGS_LABEL);
    visuals.widgets.noninteractive.bg_fill = SETTINGS_BG;
    visuals.widgets.noninteractive.weak_bg_fill = SETTINGS_BG;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, SETTINGS_SEPARATOR);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, SETTINGS_SECONDARY);
    visuals.widgets.inactive.bg_fill = SETTINGS_FIELD;
    visuals.widgets.inactive.weak_bg_fill = SETTINGS_FIELD;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, SETTINGS_LABEL);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0xDC, 0xDC, 0xE0);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0xDC, 0xDC, 0xE0);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, SETTINGS_LABEL);
    visuals.widgets.active.bg_fill = Color32::from_rgb(0xD0, 0xD0, 0xD5);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(0xD0, 0xD0, 0xD5);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, SETTINGS_LABEL);
    visuals.widgets.open.bg_fill = SETTINGS_FIELD;
    visuals.widgets.open.weak_bg_fill = SETTINGS_FIELD;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, SETTINGS_LABEL);
    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(0x00, 0x1E, 0x40, 0x40);
    visuals.selection.stroke = Stroke::new(1.0_f32, SETTINGS_PRIMARY);
    visuals.text_cursor.stroke = Stroke::new(2.0_f32, SETTINGS_PRIMARY);
    visuals.hyperlink_color = SETTINGS_PRIMARY;
    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(6);
    }
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(6);
    visuals.window_stroke = Stroke::new(1.0_f32, SETTINGS_SEPARATOR);
    visuals.window_shadow = Shadow {
        offset: [0, 2],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(40),
    };
    visuals.popup_shadow = Shadow::NONE;
    ctx.set_visuals(visuals);
}


/// Frame for the vertical Liquid Glass tool strip.
pub fn glass_toolbar_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(GLASS_FILL)
        .stroke(Stroke::new(1.0_f32, GLASS_STROKE))
        .corner_radius(CornerRadius::same(GLASS_PILL_RADIUS as u8))
        .inner_margin(Margin::symmetric(6, 10))
        .outer_margin(Margin::ZERO)
}

/// Grouped settings card (HIG boxes: related options in one visual group).
pub fn settings_group_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(SETTINGS_CARD)
        .stroke(Stroke::new(1.0_f32, SETTINGS_SEPARATOR))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 12))
}
