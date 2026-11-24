//! Shared visuals for annotate + settings.
//!
//! Annotate: transparent floating window + one Liquid Glass functional strip (HIG materials /
//! liquid-glass: glass only on chrome, never on the capture). Tool chips imitate
//! liquid-glass-js circle buttons (translucent fill, hairline rim, soft depth) in egui — not a
//! webview. Settings: Apple-inspired grouped light chrome (clarity, deference, depth) — no
//! purple / glow / marketing pills.
//!
//! Design dials: ENERGY 2 / RHYTHM 2 / MOTION 1. antislop active: during (project preference).
//!
//! Why glass here: one functional strip over the capture (R-10 dose cap; HIG functional layer).
//! Why red stroke default: one deliberate accent on markup, not chrome (R-29 / Part 3).

use eframe::egui::{
    self, Color32, CornerRadius, Margin, Pos2, Rect, Sense, Shadow, Stroke, Vec2,
};

// --- Annotate (content layer) -------------------------------------------------

/// Soft frosted pad behind the capture (translucent; window clear is fully transparent).
/// Premultiplied approx of unmultiplied (0x1C,0x1C,0x1E,0xB0).
pub const ANNOTATE_BG: Color32 = Color32::from_rgba_premultiplied(0x14, 0x14, 0x15, 0xB0);

/// Regular Liquid Glass approximation: translucent fill (egui has no backdrop-filter).
/// Premultiplied approx of unmultiplied (0xF2,0xF2,0xF7,0x38) over dark — light catch + tint.
pub const GLASS_FILL: Color32 = Color32::from_rgba_premultiplied(0x3A, 0x3A, 0x3E, 0xA8);

/// Hairline light catch on glass (top/leading edge).
pub const GLASS_STROKE: Color32 = Color32::from_rgba_premultiplied(0xFF, 0xFF, 0xFF, 0x2E);

/// Selected tool chip on glass (slightly lifted fill; monochrome).
pub const GLASS_SELECTED: Color32 = Color32::from_rgba_premultiplied(0xFF, 0xFF, 0xFF, 0x2A);

/// Idle tool chip (nested glass button).
pub const GLASS_IDLE: Color32 = Color32::from_rgba_premultiplied(0xFF, 0xFF, 0xFF, 0x12);

/// Hovered tool chip.
pub const GLASS_HOVER: Color32 = Color32::from_rgba_premultiplied(0xFF, 0xFF, 0xFF, 0x1E);

/// Pill radius for the vertical glass strip only (KM sketch; intentional exception to
/// flat chrome elsewhere).
pub const GLASS_PILL_RADIUS: f32 = 22.0;

/// Circular glass tool button diameter (liquid-glass-js circle language; desktop ≥ 28 pt).
pub const GLASS_BTN: f32 = 36.0;

/// Toolbar strip width (button + inner margins).
pub const TOOLBAR_WIDTH: f32 = 56.0;

/// Gutter between glass strip and image.
pub const TOOLBAR_GUTTER: f32 = 12.0;

/// Outer padding inside the floating window.
pub const WINDOW_PAD: f32 = 14.0;

/// Default markup stroke (red). Accent lives on the image, not the chrome.
pub const DEFAULT_STROKE: Color32 = Color32::from_rgb(0xE2, 0x4A, 0x3B);

// --- Settings (grouped light) -------------------------------------------------

pub const SETTINGS_BG: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF7);
pub const SETTINGS_CARD: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const SETTINGS_SEPARATOR: Color32 = Color32::from_rgb(0xC6, 0xC6, 0xC8);
pub const SETTINGS_LABEL: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);
pub const SETTINGS_SECONDARY: Color32 = Color32::from_rgb(0x6C, 0x6C, 0x70);
pub const SETTINGS_FIELD: Color32 = Color32::from_rgb(0xE8, 0xE8, 0xED);

/// Muted helper text on dark annotate chrome.
pub const MUTED: Color32 = Color32::from_rgb(0xA0, 0xA0, 0xA0);

/// Icon / label color on glass.
pub const GLASS_ICON: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF7);

/// Soft depth under glass chips only (R-12: elevation for the control, not the page).
pub const GLASS_BTN_SHADOW: Color32 = Color32::from_black_alpha(48);

/// Primary action fill on dark chrome (lifted charcoal — not neon).
pub const ACTION_FILL: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3C);

/// Prominent Save on glass (one tinted primary per HIG buttons / liquid-glass color budget).
pub const GLASS_PRIMARY: Color32 = Color32::from_rgb(0x0A, 0x84, 0xFF);

/// Settings Save / primary on light surfaces (same accent family, no purple).
pub const SETTINGS_PRIMARY: Color32 = Color32::from_rgb(0x00, 0x7A, 0xFF);

/// Dark annotate + glass-friendly widgets.
pub fn apply_annotate_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::TRANSPARENT;
    visuals.window_fill = Color32::from_rgba_premultiplied(0x2C, 0x2C, 0x2E, 0xE0);
    visuals.extreme_bg_color = Color32::from_rgb(0x1A, 0x1A, 0x1C);
    visuals.override_text_color = Some(Color32::from_rgb(0xF2, 0xF2, 0xF7));
    visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.noninteractive.bg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(0x3A, 0x3A, 0x3C));
    visuals.widgets.inactive.bg_fill = GLASS_IDLE;
    visuals.widgets.inactive.weak_bg_fill = GLASS_IDLE;
    visuals.widgets.hovered.bg_fill = GLASS_HOVER;
    visuals.widgets.hovered.weak_bg_fill = GLASS_HOVER;
    visuals.widgets.active.bg_fill = GLASS_SELECTED;
    visuals.widgets.active.weak_bg_fill = GLASS_SELECTED;
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
        .shadow(Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(64),
        })
}

/// Grouped settings card (HIG boxes: related options in one visual group).
pub fn settings_group_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(SETTINGS_CARD)
        .stroke(Stroke::new(1.0_f32, SETTINGS_SEPARATOR))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 12))
}

/// Circular glass tool button (liquid-glass-js circle language) with painted icon.
pub fn glass_icon_button(
    ui: &mut egui::Ui,
    selected: bool,
    tooltip: &str,
    fill_override: Option<Color32>,
    paint_icon: impl FnOnce(&egui::Painter, Rect, Color32),
) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(GLASS_BTN), Sense::click());
    let response = response.on_hover_text(tooltip);

    let fill = fill_override.unwrap_or(if selected {
        GLASS_SELECTED
    } else if response.hovered() {
        GLASS_HOVER
    } else {
        GLASS_IDLE
    });

    let painter = ui.painter_at(rect);
    // Soft contact shadow under the chip (depth for the control only).
    painter.circle_filled(
        rect.center() + Vec2::new(0.0, 1.5),
        GLASS_BTN * 0.48,
        GLASS_BTN_SHADOW,
    );
    painter.circle_filled(rect.center(), GLASS_BTN * 0.48, fill);
    painter.circle_stroke(
        rect.center(),
        GLASS_BTN * 0.48,
        Stroke::new(1.0_f32, GLASS_STROKE),
    );
    // Top rim highlight (glass light catch).
    let rim = Rect::from_center_size(
        rect.center() - Vec2::new(0.0, GLASS_BTN * 0.18),
        Vec2::new(GLASS_BTN * 0.55, GLASS_BTN * 0.22),
    );
    painter.rect_filled(
        rim,
        CornerRadius::same((GLASS_BTN * 0.2) as u8),
        Color32::from_rgba_premultiplied(0xFF, 0xFF, 0xFF, 0x18),
    );

    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(GLASS_BTN * 0.48));
    paint_icon(&painter, icon_rect, GLASS_ICON);

    response.clicked()
}

/// Stroke-color swatch that opens egui's color wheel popup (opaque stroke).
pub fn glass_color_swatch(ui: &mut egui::Ui, color: &mut Color32) -> egui::Response {
    let response = ui
        .color_edit_button_srgba(color)
        .on_hover_text("Stroke color");
    // Keep markup strokes fully opaque (alpha is for chrome, not arrows/boxes).
    *color = Color32::from_rgb(color.r(), color.g(), color.b());
    response
}

// --- Icon painters (tool-relevant glyphs; not Lucide clones) ------------------

pub fn paint_icon_arrow(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.8_f32, color);
    let a = Pos2::new(rect.left() + rect.width() * 0.2, rect.bottom() - rect.height() * 0.25);
    let b = Pos2::new(rect.right() - rect.width() * 0.18, rect.top() + rect.height() * 0.22);
    painter.line_segment([a, b], stroke);
    let dir = (b - a).normalized();
    let perp = Vec2::new(-dir.y, dir.x);
    let back = b - dir * (rect.width() * 0.28);
    painter.line_segment([b, back + perp * (rect.width() * 0.18)], stroke);
    painter.line_segment([b, back - perp * (rect.width() * 0.18)], stroke);
}

pub fn paint_icon_box(painter: &egui::Painter, rect: Rect, color: Color32) {
    let inset = rect.shrink(rect.width() * 0.18);
    painter.rect_stroke(
        inset,
        1.5,
        Stroke::new(1.8_f32, color),
        egui::StrokeKind::Outside,
    );
}

pub fn paint_icon_pixelate(painter: &egui::Painter, rect: Rect, color: Color32) {
    let inset = rect.shrink(rect.width() * 0.18);
    let cell = inset.width() / 3.0;
    for row in 0..3 {
        for col in 0..3 {
            if (row + col) % 2 == 0 {
                let c = Rect::from_min_size(
                    Pos2::new(inset.min.x + col as f32 * cell, inset.min.y + row as f32 * cell),
                    Vec2::splat(cell * 0.92),
                );
                painter.rect_filled(c, 0.5, color.gamma_multiply(0.85));
            }
        }
    }
}

pub fn paint_icon_undo(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.8_f32, color);
    let c = rect.center();
    let r = rect.width() * 0.28;
    // Arc approximation (open circle segment).
    let steps = 10;
    let mut prev = Pos2::new(c.x + r, c.y);
    for i in 1..=steps {
        let t = std::f32::consts::PI * (i as f32 / steps as f32);
        let p = Pos2::new(c.x + r * t.cos(), c.y - r * t.sin());
        painter.line_segment([prev, p], stroke);
        prev = p;
    }
    let tip = Pos2::new(c.x - r, c.y);
    painter.line_segment(
        [tip, tip + Vec2::new(rect.width() * 0.14, -rect.height() * 0.12)],
        stroke,
    );
    painter.line_segment(
        [tip, tip + Vec2::new(rect.width() * 0.14, rect.height() * 0.12)],
        stroke,
    );
}

pub fn paint_icon_clear(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.8_f32, color);
    let inset = rect.shrink(rect.width() * 0.28);
    painter.line_segment([inset.left_top(), inset.right_bottom()], stroke);
    painter.line_segment([inset.right_top(), inset.left_bottom()], stroke);
}

pub fn paint_icon_settings(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let outer = rect.width() * 0.28;
    let inner = rect.width() * 0.12;
    painter.circle_stroke(c, outer, Stroke::new(1.6_f32, color));
    painter.circle_filled(c, inner, color);
    for i in 0..6 {
        let a = (i as f32) * std::f32::consts::TAU / 6.0;
        let p0 = c + Vec2::angled(a) * (outer * 0.85);
        let p1 = c + Vec2::angled(a) * (outer * 1.25);
        painter.line_segment([p0, p1], Stroke::new(2.0_f32, color));
    }
}

pub fn paint_icon_save(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.8_f32, color);
    let inset = rect.shrink(rect.width() * 0.22);
    // Tray
    painter.line_segment(
        [
            Pos2::new(inset.left(), inset.bottom() - inset.height() * 0.15),
            Pos2::new(inset.left(), inset.bottom()),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(inset.left(), inset.bottom()),
            Pos2::new(inset.right(), inset.bottom()),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(inset.right(), inset.bottom()),
            Pos2::new(inset.right(), inset.bottom() - inset.height() * 0.15),
        ],
        stroke,
    );
    // Arrow down
    let top = Pos2::new(inset.center().x, inset.top());
    let mid = Pos2::new(inset.center().x, inset.bottom() - inset.height() * 0.28);
    painter.line_segment([top, mid], stroke);
    painter.line_segment(
        [mid, mid + Vec2::new(-inset.width() * 0.22, -inset.height() * 0.2)],
        stroke,
    );
    painter.line_segment(
        [mid, mid + Vec2::new(inset.width() * 0.22, -inset.height() * 0.2)],
        stroke,
    );
}

pub fn paint_icon_copy(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.6_f32, color);
    let back = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.18, rect.top() + rect.height() * 0.18),
        Pos2::new(rect.right() - rect.width() * 0.32, rect.bottom() - rect.height() * 0.32),
    );
    let front = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.32, rect.top() + rect.height() * 0.32),
        Pos2::new(rect.right() - rect.width() * 0.18, rect.bottom() - rect.height() * 0.18),
    );
    painter.rect_stroke(back, 1.0, stroke, egui::StrokeKind::Outside);
    painter.rect_stroke(front, 1.0, stroke, egui::StrokeKind::Outside);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_stroke_is_red() {
        assert_eq!(DEFAULT_STROKE.r(), 0xE2);
        assert_eq!(DEFAULT_STROKE.g(), 0x4A);
        assert_eq!(DEFAULT_STROKE.b(), 0x3B);
        assert_eq!(DEFAULT_STROKE.a(), 255);
    }

    #[test]
    fn glass_button_meets_desktop_target() {
        assert!(GLASS_BTN >= 28.0);
        assert!(TOOLBAR_WIDTH >= GLASS_BTN);
    }
}
