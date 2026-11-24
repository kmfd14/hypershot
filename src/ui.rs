//! Shared visuals for annotate + settings.
//!
//! Annotate: flat pastel chrome (restrained purple / gray / white). egui on Wayland cannot
//! deliver convincing liquid-glass (no desktop backdrop-blur), so we use opaque flat fills
//! instead of frosted imitation. Toolbar sits to the RIGHT of the capture with even outer pad.
//!
//! Contrast: dark icons on light pastel wells. Soft feathering + thicker strokes for AA.
//! One muted purple accent on Save only (not neon, no gradient, no glow).
//!
//! Design dials: ENERGY 2 / RHYTHM 2 / MOTION 1. antislop active: during (project preference).
//! Palette note: KM requested restrained pastel purple/gray/white as the glass fallback
//! (R-37 owner override of the purple-default caution).

use eframe::egui::{
    self, Color32, CornerRadius, Id, Margin, Pos2, Rect, Sense, Shadow, Stroke, Vec2,
};
use eframe::egui::color_picker::{color_picker_color32, Alpha};
use eframe::egui::popup::{popup_below_widget, PopupCloseBehavior};

// --- Annotate (flat pastel) ---------------------------------------------------

/// Opaque window / panel fill (soft cool gray-lilac, not pure white).
pub const ANNOTATE_BG: Color32 = Color32::from_rgb(0xF3, 0xF1, 0xF6);

/// Vertical tool rail fill (flat pastel gray-lilac).
pub const TOOLBAR_FILL: Color32 = Color32::from_rgb(0xE8, 0xE4, 0xEE);

/// Soft rail edge (muted, not harsh white).
pub const TOOLBAR_STROKE: Color32 = Color32::from_rgb(0xD0, 0xCA, 0xD8);

/// Selected tool well (soft pastel purple tint — flat, no glow).
pub const BTN_SELECTED: Color32 = Color32::from_rgb(0xDD, 0xD4, 0xEC);

/// Idle tool well (white).
pub const BTN_IDLE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);

/// Hovered tool well.
pub const BTN_HOVER: Color32 = Color32::from_rgb(0xF7, 0xF4, 0xFB);

/// Rail corner radius (moderate; not pill-everything).
pub const TOOLBAR_RADIUS: f32 = 14.0;

/// Circular tool button diameter (desktop ≥ 28 pt).
pub const TOOL_BTN: f32 = 36.0;

/// Toolbar strip width.
pub const TOOLBAR_WIDTH: f32 = 56.0;

/// Gutter between image and right tool rail.
pub const TOOLBAR_GUTTER: f32 = 16.0;

/// Outer padding inside the floating window (top / left / right / bottom).
pub const WINDOW_PAD: f32 = 18.0;

/// Default markup stroke (red). Accent for marks lives on the image.
pub const DEFAULT_STROKE: Color32 = Color32::from_rgb(0xE2, 0x4A, 0x3B);

/// Dark icon ink on light pastel wells.
pub const ICON_DARK: Color32 = Color32::from_rgb(0x3A, 0x36, 0x45);

/// Light icon ink on tinted / darker fills (Save / Copy).
pub const ICON_LIGHT: Color32 = Color32::from_rgb(0xF7, 0xF5, 0xFA);

/// Soft contact shadow under tool wells only.
pub const BTN_SHADOW: Color32 = Color32::from_black_alpha(28);

/// Muted pastel purple (Save primary — restrained, not neon).
pub const PASTEL_PRIMARY: Color32 = Color32::from_rgb(0x8B, 0x7C, 0xB6);

/// Secondary action (Copy): muted gray-violet, flat.
pub const ACTION_FILL: Color32 = Color32::from_rgb(0x6E, 0x68, 0x7A);

/// Helper text on pastel chrome.
pub const MUTED: Color32 = Color32::from_rgb(0x8A, 0x84, 0x94);

// --- Settings (grouped light) -------------------------------------------------

pub const SETTINGS_BG: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF7);
pub const SETTINGS_CARD: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const SETTINGS_SEPARATOR: Color32 = Color32::from_rgb(0xC6, 0xC6, 0xC8);
pub const SETTINGS_LABEL: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);
pub const SETTINGS_SECONDARY: Color32 = Color32::from_rgb(0x6C, 0x6C, 0x70);
pub const SETTINGS_FIELD: Color32 = Color32::from_rgb(0xE8, 0xE8, 0xED);
pub const SETTINGS_PRIMARY: Color32 = Color32::from_rgb(0x00, 0x7A, 0xFF);

/// Approximate relative luminance of a Color32 (ignores alpha).
pub fn approx_luminance(c: Color32) -> f32 {
    0.2126 * (c.r() as f32) + 0.7152 * (c.g() as f32) + 0.0722 * (c.b() as f32)
}

/// Pick dark or light icon ink for contrast on `fill`.
pub fn icon_color_for_fill(fill: Color32) -> Color32 {
    if approx_luminance(fill) >= 140.0 {
        ICON_DARK
    } else {
        ICON_LIGHT
    }
}

/// Pastel annotate visuals; feathering on for soft circle/rail edges.
pub fn apply_annotate_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = ANNOTATE_BG;
    visuals.window_fill = ANNOTATE_BG;
    visuals.extreme_bg_color = Color32::from_rgb(0xE8, 0xE4, 0xEE);
    visuals.override_text_color = Some(ICON_DARK);
    visuals.widgets.noninteractive.bg_fill = ANNOTATE_BG;
    visuals.widgets.noninteractive.weak_bg_fill = ANNOTATE_BG;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, TOOLBAR_STROKE);
    visuals.widgets.inactive.bg_fill = BTN_IDLE;
    visuals.widgets.inactive.weak_bg_fill = BTN_IDLE;
    visuals.widgets.hovered.bg_fill = BTN_HOVER;
    visuals.widgets.hovered.weak_bg_fill = BTN_HOVER;
    visuals.widgets.active.bg_fill = BTN_SELECTED;
    visuals.widgets.active.weak_bg_fill = BTN_SELECTED;
    visuals.widgets.open.bg_fill = BTN_IDLE;
    visuals.widgets.open.weak_bg_fill = BTN_IDLE;
    visuals.selection.bg_fill = BTN_SELECTED;
    visuals.selection.stroke = Stroke::new(1.0_f32, PASTEL_PRIMARY);
    visuals.text_cursor.stroke = Stroke::new(2.0_f32, PASTEL_PRIMARY);
    visuals.hyperlink_color = PASTEL_PRIMARY;
    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.fg_stroke = Stroke::new(1.0_f32, ICON_DARK);
        w.corner_radius = CornerRadius::same(8);
    }
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);
    visuals.window_stroke = Stroke::new(1.0_f32, TOOLBAR_STROKE);
    visuals.window_shadow = Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(48),
    };
    visuals.popup_shadow = Shadow::NONE;
    ctx.set_visuals(visuals);

    ctx.tessellation_options_mut(|opts| {
        opts.feathering = true;
        opts.feathering_size_in_pixels = 1.5;
    });
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
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0x00, 0x7A, 0xFF, 0x40);
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

/// Frame for the right-side pastel tool rail.
pub fn toolbar_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(TOOLBAR_FILL)
        .stroke(Stroke::new(1.25_f32, TOOLBAR_STROKE))
        .corner_radius(CornerRadius::same(TOOLBAR_RADIUS as u8))
        .inner_margin(Margin::symmetric(6, 12))
        .outer_margin(Margin::ZERO)
        .shadow(Shadow {
            offset: [0, 4],
            blur: 14,
            spread: 0,
            color: Color32::from_black_alpha(28),
        })
}

/// Grouped settings card.
pub fn settings_group_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(SETTINGS_CARD)
        .stroke(Stroke::new(1.0_f32, SETTINGS_SEPARATOR))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 12))
}

fn paint_soft_circle(painter: &egui::Painter, center: Pos2, radius: f32, fill: Color32) {
    painter.circle_filled(center + Vec2::new(0.0, 1.0), radius + 0.4, BTN_SHADOW);
    painter.circle_filled(center, radius, fill);
    // Soft muted rim (wide + low contrast → feathers; no harsh 1px white).
    painter.circle_stroke(
        center,
        radius - 0.5,
        Stroke::new(1.75_f32, Color32::from_rgba_unmultiplied(0x3A, 0x36, 0x45, 0x28)),
    );
}

/// Circular pastel tool button with painted icon and auto-contrast ink.
pub fn tool_icon_button(
    ui: &mut egui::Ui,
    selected: bool,
    tooltip: &str,
    fill_override: Option<Color32>,
    paint_icon: impl FnOnce(&egui::Painter, Rect, Color32),
) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(TOOL_BTN), Sense::click());
    let response = response.on_hover_text(tooltip);

    let fill = fill_override.unwrap_or(if selected {
        BTN_SELECTED
    } else if response.hovered() {
        BTN_HOVER
    } else {
        BTN_IDLE
    });
    let icon = icon_color_for_fill(fill);
    let radius = TOOL_BTN * 0.46;

    let painter = ui.painter_at(rect);
    paint_soft_circle(&painter, rect.center(), radius, fill);

    if selected && fill_override.is_none() {
        painter.circle_stroke(
            rect.center(),
            radius - 0.2,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0x8B, 0x7C, 0xB6, 0x90)),
        );
    }

    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(TOOL_BTN * 0.50));
    paint_icon(&painter, icon_rect, icon);

    response.clicked()
}

/// Circular stroke-color swatch that opens egui's color wheel (opaque stroke).
pub fn tool_color_swatch(ui: &mut egui::Ui, color: &mut Color32) -> egui::Response {
    let popup_id = Id::new("lightsnip_stroke_color_popup");
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(TOOL_BTN), Sense::click());
    let response = response.on_hover_text("Stroke color");

    let painter = ui.painter_at(rect);
    let radius = TOOL_BTN * 0.46;
    paint_soft_circle(&painter, rect.center(), radius, BTN_IDLE);
    painter.circle_filled(rect.center(), radius * 0.62, *color);
    painter.circle_stroke(
        rect.center(),
        radius * 0.62,
        Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0x3A, 0x36, 0x45, 0x40)),
    );

    if response.clicked() {
        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
    }

    popup_below_widget(
        ui,
        popup_id,
        &response,
        PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            ui.set_min_width(220.0);
            ui.spacing_mut().slider_width = 180.0;
            color_picker_color32(ui, color, Alpha::Opaque);
        },
    );

    *color = Color32::from_rgb(color.r(), color.g(), color.b());
    response
}

// Aliases so annotate can keep short call sites while names stay accurate.
pub use tool_icon_button as glass_icon_button;
pub use tool_color_swatch as glass_color_swatch;
pub use toolbar_frame as glass_toolbar_frame;

// --- Icon painters (thicker strokes for AA) -----------------------------------

const ICON_STROKE: f32 = 2.25;

pub fn paint_icon_arrow(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(ICON_STROKE, color);
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
        2.0,
        Stroke::new(ICON_STROKE, color),
        egui::StrokeKind::Middle,
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
                    Vec2::splat(cell * 0.88),
                );
                painter.rect_filled(c, 1.5, color);
            }
        }
    }
}

pub fn paint_icon_undo(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(ICON_STROKE, color);
    let c = rect.center();
    let r = rect.width() * 0.28;
    let steps = 28;
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
    let stroke = Stroke::new(ICON_STROKE, color);
    let inset = rect.shrink(rect.width() * 0.28);
    painter.line_segment([inset.left_top(), inset.right_bottom()], stroke);
    painter.line_segment([inset.right_top(), inset.left_bottom()], stroke);
}

pub fn paint_icon_settings(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let outer = rect.width() * 0.26;
    let inner = rect.width() * 0.11;
    painter.circle_stroke(c, outer, Stroke::new(ICON_STROKE, color));
    painter.circle_filled(c, inner, color);
    for i in 0..6 {
        let a = (i as f32) * std::f32::consts::TAU / 6.0;
        let p0 = c + Vec2::angled(a) * (outer * 0.82);
        let p1 = c + Vec2::angled(a) * (outer * 1.28);
        painter.line_segment([p0, p1], Stroke::new(2.4_f32, color));
    }
}

pub fn paint_icon_save(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(ICON_STROKE, color);
    let inset = rect.shrink(rect.width() * 0.22);
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
    let stroke = Stroke::new(ICON_STROKE, color);
    let back = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.18, rect.top() + rect.height() * 0.18),
        Pos2::new(rect.right() - rect.width() * 0.32, rect.bottom() - rect.height() * 0.32),
    );
    let front = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.32, rect.top() + rect.height() * 0.32),
        Pos2::new(rect.right() - rect.width() * 0.18, rect.bottom() - rect.height() * 0.18),
    );
    painter.rect_stroke(back, 2.0, stroke, egui::StrokeKind::Middle);
    painter.rect_stroke(front, 2.0, stroke, egui::StrokeKind::Middle);
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
    fn tool_button_meets_desktop_target() {
        assert!(TOOL_BTN >= 28.0);
        assert!(TOOLBAR_WIDTH >= TOOL_BTN);
        assert!(WINDOW_PAD >= 12.0);
        assert!(TOOLBAR_GUTTER >= 12.0);
    }

    #[test]
    fn light_wells_get_dark_icons() {
        assert_eq!(icon_color_for_fill(BTN_IDLE), ICON_DARK);
        assert_eq!(icon_color_for_fill(BTN_SELECTED), ICON_DARK);
        assert_eq!(icon_color_for_fill(BTN_HOVER), ICON_DARK);
        assert!(approx_luminance(ICON_DARK) < 100.0);
    }

    #[test]
    fn dark_and_primary_fills_get_light_icons() {
        assert_eq!(icon_color_for_fill(ACTION_FILL), ICON_LIGHT);
        assert_eq!(icon_color_for_fill(PASTEL_PRIMARY), ICON_LIGHT);
        assert!(approx_luminance(ICON_LIGHT) > 200.0);
    }

    #[test]
    fn pastel_primary_is_muted_not_neon() {
        // Restrained purple: mid saturation / mid value — not electric neon.
        assert!(PASTEL_PRIMARY.r() < 0xB0);
        assert!(PASTEL_PRIMARY.b() < 0xD0);
        assert!(approx_luminance(PASTEL_PRIMARY) > 100.0);
        assert!(approx_luminance(PASTEL_PRIMARY) < 180.0);
    }
}
