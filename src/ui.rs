//! Shared visuals for annotate + settings.
//!
//! Annotate chrome targets the liquid-glass language in
//! `/cursor/stores/self/media/liquid-glass-sample.html`: soft glass pills,
//! translucent frost fills, specular rim highlights, depth shadow, hover sheen.
//!
//! Honest limits (Wayland + egui/winit):
//! - No real `backdrop-filter` / desktop blur through the window.
//! - No SVG refraction / turbulence displacement.
//! - Window transparency is best-effort (`ViewportBuilder::with_transparent`);
//!   many Wayland compositors composite opaque or ignore alpha.
//! - “Background blur” preference is stored and labeled; we approximate frost
//!   with translucent fills + painted highlights only.
//!
//! Design dials: ENERGY 2 / RHYTHM 2 / MOTION 1 (hover scale/sheen only).

use eframe::egui::{
    self, Color32, CornerRadius, Id, Margin, Pos2, Rect, Sense, Shadow, Stroke, Theme,
    ThemePreference, Vec2,
};
use eframe::egui::color_picker::{color_picker_color32, Alpha};
use eframe::egui::epaint::{self, PathShape};
use eframe::egui::popup::{popup_below_widget, PopupCloseBehavior};

use crate::config::UiTheme;


/// Unmultiplied RGBA → premultiplied Color32 (const-safe for egui 0.31).
const fn rgba_u(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_premultiplied(
        ((r as u16 * a as u16) / 255) as u8,
        ((g as u16 * a as u16) / 255) as u8,
        ((b as u16 * a as u16) / 255) as u8,
        a,
    )
}

// --- Annotate sizes -----------------------------------------------------------

/// Circular tool button diameter (desktop ≥ 28 pt).
pub const TOOL_BTN: f32 = 36.0;

/// Settings gear is intentionally larger for discoverability.
pub const SETTINGS_BTN: f32 = 42.0;

/// Toolbar strip thickness (cross-axis).
pub const TOOLBAR_WIDTH: f32 = 56.0;

/// Gutter between image and floating tool rail.
pub const TOOLBAR_GUTTER: f32 = 12.0;

/// Outer padding inside the floating window.
pub const WINDOW_PAD: f32 = 18.0;

/// Soft glass pill radius for the rail (near-capsule, not harsh box).
pub const TOOLBAR_RADIUS: f32 = 22.0;

/// Default markup stroke (red). Accent for marks lives on the image.
pub const DEFAULT_STROKE: Color32 = Color32::from_rgb(0xE2, 0x4A, 0x3B);

// --- Glass fills (sample structure + pastel purple/gray/white palette) --------
// Structure from liquid-glass-sample.html (frost alpha ~14%/20%, specular rim,
// hover sheen). Color stays the project's restrained lilac/gray/white — not
// generic white-glass-only.

/// Rail frost: pastel gray-lilac at sample-like translucency.
pub const GLASS_FILL: Color32 = rgba_u(0xE8, 0xE4, 0xEE, 48);
/// Hover: slightly brighter lilac-white frost (~sample 20%).
pub const GLASS_FILL_HOVER: Color32 = rgba_u(0xF7, 0xF4, 0xFB, 64);
/// Selected well — soft pastel purple tint, denser frost.
pub const GLASS_FILL_SELECTED: Color32 = rgba_u(0xDD, 0xD4, 0xEC, 96);

/// Dark-appearance glass (translucent charcoal-lilac, not pure black).
pub const GLASS_FILL_DARK: Color32 = rgba_u(0x2A, 0x28, 0x30, 170);
pub const GLASS_FILL_HOVER_DARK: Color32 = rgba_u(0x42, 0x3E, 0x4C, 200);
pub const GLASS_FILL_SELECTED_DARK: Color32 = rgba_u(0x4A, 0x42, 0x5C, 220);

/// Opaque fallbacks when transparency is off / compositor refuses alpha.
pub const FROST_SOLID_LIGHT: Color32 = Color32::from_rgb(0xE8, 0xE4, 0xEE);
pub const FROST_SOLID_DARK: Color32 = Color32::from_rgb(0x2A, 0x28, 0x30);

pub const ANNOTATE_BG: Color32 = Color32::from_rgb(0xF3, 0xF1, 0xF6);
pub const ANNOTATE_BG_DARK: Color32 = Color32::from_rgb(0x1C, 0x1B, 0x20);

/// Specular top rim — cool white with a hint of lilac (not stark white-only).
pub const GLASS_SPECULAR: Color32 = rgba_u(0xFF, 0xFB, 0xFF, 200);
/// Soft inner glow (pastel-tinted).
pub const GLASS_INNER_GLOW: Color32 = rgba_u(0xF0, 0xEA, 0xF8, 48);
/// Bottom inset shadow (sample inset 0 -1px rgba(0,0,0,0.25)).
pub const GLASS_INNER_SHADOW: Color32 = rgba_u(0x3A, 0x36, 0x45, 64);
/// Gradient rim light edge (lilac-white).
pub const GLASS_RIM_LIGHT: Color32 = rgba_u(0xF7, 0xF4, 0xFB, 190);
/// Gradient rim mid/faint edge.
pub const GLASS_RIM_FAINT: Color32 = rgba_u(0xD0, 0xCA, 0xD8, 90);

pub const ICON_DARK: Color32 = Color32::from_rgb(0x3A, 0x36, 0x45);
pub const ICON_LIGHT: Color32 = Color32::from_rgb(0xF7, 0xF5, 0xFA);

/// Muted purple Save primary (restrained; sample keeps labels monochrome on glass).
pub const PASTEL_PRIMARY: Color32 = Color32::from_rgb(0x8B, 0x7C, 0xB6);
pub const PASTEL_PRIMARY_HOVER: Color32 = Color32::from_rgb(0x9A, 0x8C, 0xC4);
/// Glass-tinted primary (primary action colors the *background*, HIG).
pub const GLASS_PRIMARY: Color32 = rgba_u(0x8B, 0x7C, 0xB6, 200);
pub const GLASS_PRIMARY_HOVER: Color32 = rgba_u(0x9A, 0x8C, 0xC4, 230);

pub const ACTION_FILL: Color32 = Color32::from_rgb(0x6E, 0x68, 0x7A);
pub const ACTION_FILL_HOVER: Color32 = Color32::from_rgb(0x7E, 0x78, 0x8A);
pub const GLASS_ACTION: Color32 = rgba_u(0x6E, 0x68, 0x7A, 200);
pub const GLASS_ACTION_HOVER: Color32 = rgba_u(0x7E, 0x78, 0x8A, 230);

pub const MUTED: Color32 = Color32::from_rgb(0x8A, 0x84, 0x94);
pub const MUTED_DARK: Color32 = Color32::from_rgb(0xA8, 0xA2, 0xB0);
pub const PREVIEW_MATTE: Color32 = Color32::from_rgb(0xE0, 0xDC, 0xE8);
pub const PREVIEW_MATTE_DARK: Color32 = Color32::from_rgb(0x28, 0x26, 0x2E);

// --- Settings -----------------------------------------------------------------

pub const SETTINGS_BG: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF7);
pub const SETTINGS_CARD: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const SETTINGS_SEPARATOR: Color32 = Color32::from_rgb(0xC6, 0xC6, 0xC8);
pub const SETTINGS_LABEL: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);
pub const SETTINGS_SECONDARY: Color32 = Color32::from_rgb(0x6C, 0x6C, 0x70);
pub const SETTINGS_FIELD: Color32 = Color32::from_rgb(0xE8, 0xE8, 0xED);
pub const SETTINGS_PRIMARY: Color32 = Color32::from_rgb(0x00, 0x7A, 0xFF);

pub const SETTINGS_BG_DARK: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);
pub const SETTINGS_CARD_DARK: Color32 = Color32::from_rgb(0x2C, 0x2C, 0x2E);
pub const SETTINGS_SEPARATOR_DARK: Color32 = Color32::from_rgb(0x48, 0x48, 0x4A);
pub const SETTINGS_LABEL_DARK: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF7);
pub const SETTINGS_SECONDARY_DARK: Color32 = Color32::from_rgb(0xA1, 0xA1, 0xA6);
pub const SETTINGS_FIELD_DARK: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3C);

/// Runtime annotate colors for the active theme + transparency mode.
#[derive(Debug, Clone, Copy)]
pub struct AnnotatePalette {
    pub bg: Color32,
    pub toolbar_fill: Color32,
    pub btn_idle: Color32,
    pub btn_hover: Color32,
    pub btn_selected: Color32,
    pub muted: Color32,
    pub preview_matte: Color32,
    pub dark: bool,
    /// True when we requested window alpha (frost can stay translucent).
    pub translucent: bool,
}

impl AnnotatePalette {
    pub fn for_mode(dark: bool, translucent: bool) -> Self {
        if dark {
            Self {
                bg: if translucent {
                    rgba_u(28, 27, 32, 200)
                } else {
                    ANNOTATE_BG_DARK
                },
                toolbar_fill: if translucent {
                    GLASS_FILL_DARK
                } else {
                    FROST_SOLID_DARK
                },
                btn_idle: if translucent {
                    rgba_u(0x36, 0x34, 0x3E, 150)
                } else {
                    Color32::from_rgb(0x36, 0x34, 0x3E)
                },
                btn_hover: if translucent {
                    GLASS_FILL_HOVER_DARK
                } else {
                    Color32::from_rgb(0x42, 0x3E, 0x4C)
                },
                btn_selected: if translucent {
                    GLASS_FILL_SELECTED_DARK
                } else {
                    Color32::from_rgb(0x4A, 0x42, 0x5C)
                },
                muted: MUTED_DARK,
                preview_matte: PREVIEW_MATTE_DARK,
                dark: true,
                translucent,
            }
        } else {
            Self {
                bg: if translucent {
                    rgba_u(0xF3, 0xF1, 0xF6, 210)
                } else {
                    ANNOTATE_BG
                },
                toolbar_fill: if translucent {
                    // Pastel gray-lilac frost rail (not plain white glass).
                    rgba_u(0xE8, 0xE4, 0xEE, 72)
                } else {
                    FROST_SOLID_LIGHT
                },
                btn_idle: if translucent {
                    rgba_u(0xFF, 0xFF, 0xFF, 56)
                } else {
                    Color32::from_rgb(0xFF, 0xFF, 0xFF)
                },
                btn_hover: if translucent {
                    GLASS_FILL_HOVER
                } else {
                    Color32::from_rgb(0xF7, 0xF4, 0xFB)
                },
                btn_selected: if translucent {
                    GLASS_FILL_SELECTED
                } else {
                    Color32::from_rgb(0xDD, 0xD4, 0xEC)
                },
                muted: MUTED,
                preview_matte: PREVIEW_MATTE,
                dark: false,
                translucent,
            }
        }
    }
}

/// Approximate relative luminance of a Color32 (ignores alpha).
pub fn approx_luminance(c: Color32) -> f32 {
    0.2126 * (c.r() as f32) + 0.7152 * (c.g() as f32) + 0.0722 * (c.b() as f32)
}

/// Pick dark or light icon ink for contrast on `fill`.
pub fn icon_color_for_fill(fill: Color32) -> Color32 {
    // egui stores premultiplied RGBA; recover approx unmultiplied for luminance.
    let a = fill.a().max(1) as u16;
    let ur = ((fill.r() as u16 * 255) / a) as u8;
    let ug = ((fill.g() as u16 * 255) / a) as u8;
    let ub = ((fill.b() as u16 * 255) / a) as u8;
    let lum = approx_luminance(Color32::from_rgb(ur, ug, ub));
    if lum >= 140.0 {
        ICON_DARK
    } else {
        ICON_LIGHT
    }
}

pub fn theme_preference(theme: UiTheme) -> ThemePreference {
    match theme {
        UiTheme::Dark => ThemePreference::Dark,
        UiTheme::Light => ThemePreference::Light,
        UiTheme::System => ThemePreference::System,
    }
}

fn annotate_visuals(dark: bool) -> egui::Visuals {
    let pal = AnnotatePalette::for_mode(dark, false);
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = pal.bg;
    visuals.window_fill = pal.bg;
    visuals.extreme_bg_color = pal.toolbar_fill;
    visuals.override_text_color = Some(if dark { ICON_LIGHT } else { ICON_DARK });
    visuals.widgets.noninteractive.bg_fill = pal.bg;
    visuals.widgets.noninteractive.weak_bg_fill = pal.bg;
    visuals.widgets.inactive.bg_fill = pal.btn_idle;
    visuals.widgets.inactive.weak_bg_fill = pal.btn_idle;
    visuals.widgets.hovered.bg_fill = pal.btn_hover;
    visuals.widgets.hovered.weak_bg_fill = pal.btn_hover;
    visuals.widgets.active.bg_fill = pal.btn_selected;
    visuals.widgets.active.weak_bg_fill = pal.btn_selected;
    visuals.widgets.open.bg_fill = pal.btn_idle;
    visuals.widgets.open.weak_bg_fill = pal.btn_idle;
    visuals.selection.bg_fill = pal.btn_selected;
    visuals.selection.stroke = Stroke::new(1.0_f32, PASTEL_PRIMARY);
    visuals.text_cursor.stroke = Stroke::new(2.0_f32, PASTEL_PRIMARY);
    visuals.hyperlink_color = PASTEL_PRIMARY;
    let fg = if dark { ICON_LIGHT } else { ICON_DARK };
    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.fg_stroke = Stroke::new(1.0_f32, fg);
        w.corner_radius = CornerRadius::same(12);
    }
    visuals.window_corner_radius = CornerRadius::same(14);
    visuals.menu_corner_radius = CornerRadius::same(10);
    visuals.window_stroke = Stroke::new(1.0_f32, GLASS_RIM_FAINT);
    visuals.window_shadow = Shadow {
        offset: [0, 10],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(64),
    };
    visuals.popup_shadow = Shadow::NONE;
    visuals
}

fn settings_visuals(dark: bool) -> egui::Visuals {
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let (bg, card, sep, label, secondary, field) = if dark {
        (
            SETTINGS_BG_DARK,
            SETTINGS_CARD_DARK,
            SETTINGS_SEPARATOR_DARK,
            SETTINGS_LABEL_DARK,
            SETTINGS_SECONDARY_DARK,
            SETTINGS_FIELD_DARK,
        )
    } else {
        (
            SETTINGS_BG,
            SETTINGS_CARD,
            SETTINGS_SEPARATOR,
            SETTINGS_LABEL,
            SETTINGS_SECONDARY,
            SETTINGS_FIELD,
        )
    };
    visuals.panel_fill = bg;
    visuals.window_fill = card;
    visuals.extreme_bg_color = field;
    visuals.override_text_color = Some(label);
    visuals.widgets.noninteractive.bg_fill = bg;
    visuals.widgets.noninteractive.weak_bg_fill = bg;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, sep);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, secondary);
    visuals.widgets.inactive.bg_fill = field;
    visuals.widgets.inactive.weak_bg_fill = field;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, label);
    visuals.widgets.hovered.bg_fill = if dark {
        Color32::from_rgb(0x48, 0x48, 0x4A)
    } else {
        Color32::from_rgb(0xDC, 0xDC, 0xE0)
    };
    visuals.widgets.hovered.weak_bg_fill = visuals.widgets.hovered.bg_fill;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, label);
    visuals.widgets.active.bg_fill = if dark {
        Color32::from_rgb(0x54, 0x54, 0x56)
    } else {
        Color32::from_rgb(0xD0, 0xD0, 0xD5)
    };
    visuals.widgets.active.weak_bg_fill = visuals.widgets.active.bg_fill;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, label);
    visuals.widgets.open.bg_fill = field;
    visuals.widgets.open.weak_bg_fill = field;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, label);
    visuals.selection.bg_fill = rgba_u(0x00, 0x7A, 0xFF, 0x40);
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
    visuals.window_stroke = Stroke::new(1.0_f32, sep);
    visuals.window_shadow = Shadow {
        offset: [0, 2],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(40),
    };
    visuals.popup_shadow = Shadow::NONE;
    visuals
}

/// Install light + dark annotate styles and select theme preference.
pub fn apply_annotate_visuals(ctx: &egui::Context, theme: UiTheme) {
    ctx.set_visuals_of(Theme::Light, annotate_visuals(false));
    ctx.set_visuals_of(Theme::Dark, annotate_visuals(true));
    ctx.set_theme(theme_preference(theme));
    ctx.tessellation_options_mut(|opts| {
        opts.feathering = true;
        // Softer AA for glass rims (sample relies on blur; we feather strokes).
        opts.feathering_size_in_pixels = 2.0;
    });
}

/// Install light + dark settings styles and select theme preference.
pub fn apply_settings_visuals(ctx: &egui::Context, theme: UiTheme) {
    ctx.set_visuals_of(Theme::Light, settings_visuals(false));
    ctx.set_visuals_of(Theme::Dark, settings_visuals(true));
    ctx.set_theme(theme_preference(theme));
}

/// Frame for the floating glass tool rail (content-sized, not full window).
pub fn toolbar_frame(pal: &AnnotatePalette) -> egui::Frame {
    let shadow_a = if pal.dark { 80 } else { 64 };
    egui::Frame::new()
        .fill(pal.toolbar_fill)
        .stroke(Stroke::new(1.0_f32, GLASS_RIM_LIGHT))
        .corner_radius(CornerRadius::same(TOOLBAR_RADIUS as u8))
        .inner_margin(Margin::symmetric(8, 12))
        .outer_margin(Margin::ZERO)
        // Sample: 0 10px 24px rgba(0,0,0,0.25)
        .shadow(Shadow {
            offset: [0, 10],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(shadow_a),
        })
}

/// Grouped settings card.
pub fn settings_group_frame(dark: bool) -> egui::Frame {
    let (fill, stroke) = if dark {
        (SETTINGS_CARD_DARK, SETTINGS_SEPARATOR_DARK)
    } else {
        (SETTINGS_CARD, SETTINGS_SEPARATOR)
    };
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, stroke))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(12, 10))
}

/// Paint specular rim + inner shadow on a glass pill/circle (sample box-shadow insets).
pub fn paint_glass_surface(
    painter: &egui::Painter,
    rect: Rect,
    radius: f32,
    fill: Color32,
    hovered: bool,
    pointer: Option<Pos2>,
) {
    let center = rect.center();
    // Depth shadow under the pill.
    painter.circle_filled(
        center + Vec2::new(0.0, 3.0),
        radius + 0.5,
        Color32::from_black_alpha(40),
    );
    painter.circle_filled(center, radius, fill);

    // Soft inner glow (sample inset 0 0 12px rgba(255,255,255,0.15)).
    painter.circle_stroke(
        center,
        radius * 0.92,
        Stroke::new(radius * 0.22, GLASS_INNER_GLOW),
    );

    // Specular top rim highlight.
    let top_arc_r = radius * 0.86;
    let mut top_pts = Vec::with_capacity(18);
    for i in 0..=16 {
        let t = std::f32::consts::PI * (0.15 + 0.70 * (i as f32 / 16.0));
        // Upper semicircle-ish highlight.
        let a = -std::f32::consts::PI + t;
        top_pts.push(center + Vec2::new(top_arc_r * a.cos(), top_arc_r * a.sin() * 0.85));
    }
    if top_pts.len() >= 2 {
        painter.add(PathShape::line(
            top_pts,
            Stroke::new(1.6_f32, GLASS_SPECULAR),
        ));
    }

    // Bottom inset shadow stroke.
    painter.circle_stroke(
        center + Vec2::new(0.0, 0.6),
        radius - 0.8,
        Stroke::new(1.4_f32, GLASS_INNER_SHADOW),
    );

    // Crisp gradient rim (sample ::after mask border).
    painter.circle_stroke(center, radius - 0.4, Stroke::new(1.15_f32, GLASS_RIM_LIGHT));
    painter.circle_stroke(
        center,
        radius - 0.4,
        Stroke::new(1.0_f32, GLASS_RIM_FAINT),
    );

    // Pointer-reactive sheen (sample ::before radial-gradient), pastel-lilac not pure white.
    let sheen = rgba_u(0xF7, 0xF4, 0xFB, 72);
    if hovered {
        if let Some(p) = pointer {
            let sheen_r = radius * 1.6;
            // egui has no true radial gradient mesh; a translucent disc near the pointer
            // approximates the sample sheen.
            let offset = (p - center).normalized() * (radius * 0.35);
            painter.circle_filled(center + offset, sheen_r * 0.45, sheen);
        } else {
            painter.circle_filled(
                center + Vec2::new(-radius * 0.2, -radius * 0.25),
                radius * 0.55,
                rgba_u(0xF0, 0xEA, 0xF8, 52),
            );
        }
    }
}

/// Paint glass chrome on a rounded rect (toolbar rail). Kept for custom overlays.
#[allow(dead_code)]
pub fn paint_glass_rect(painter: &egui::Painter, rect: Rect, radius: f32, fill: Color32) {
    let rounding = CornerRadius::same(radius as u8);
    painter.rect(
        rect.translate(Vec2::new(0.0, 4.0)),
        rounding,
        Color32::from_black_alpha(36),
        Stroke::NONE,
        egui::StrokeKind::Outside,
    );
    painter.rect(
        rect,
        rounding,
        fill,
        Stroke::new(1.0_f32, GLASS_RIM_LIGHT),
        egui::StrokeKind::Outside,
    );
    // Top specular hairline.
    let top = Rect::from_min_max(
        Pos2::new(rect.left() + radius * 0.4, rect.top() + 1.0),
        Pos2::new(rect.right() - radius * 0.4, rect.top() + 2.5),
    );
    painter.rect_filled(top, 1.0, GLASS_SPECULAR);
    // Bottom inner shade.
    let bot = Rect::from_min_max(
        Pos2::new(rect.left() + radius * 0.35, rect.bottom() - 3.0),
        Pos2::new(rect.right() - radius * 0.35, rect.bottom() - 1.0),
    );
    painter.rect_filled(bot, 1.0, GLASS_INNER_SHADOW);
}

/// Circular glass tool button with painted icon and auto-contrast ink.
pub fn tool_icon_button(
    ui: &mut egui::Ui,
    selected: bool,
    tooltip: &str,
    fill_override: Option<(Color32, Color32)>,
    diameter: f32,
    paint_icon: impl FnOnce(&egui::Painter, Rect, Color32),
) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(diameter), Sense::click());
    let response = response.on_hover_text(tooltip);
    let hovered = response.hovered() || response.is_pointer_button_down_on();

    let dark = ui.visuals().dark_mode;
    let pal = AnnotatePalette::for_mode(dark, true);
    let fill = if let Some((idle, hover)) = fill_override {
        if hovered { hover } else { idle }
    } else if selected {
        pal.btn_selected
    } else if hovered {
        pal.btn_hover
    } else {
        pal.btn_idle
    };

    // Sample hover: scale 1.03; active 0.97 — subtle, MOTION 1.
    let scale = if response.is_pointer_button_down_on() {
        0.97
    } else if hovered {
        1.03
    } else {
        1.0
    };
    let draw = Rect::from_center_size(rect.center(), rect.size() * scale);
    let radius = draw.width() * 0.46;
    let painter = ui.painter_at(rect.expand(4.0));
    paint_glass_surface(
        &painter,
        draw,
        radius,
        fill,
        hovered,
        response.hover_pos(),
    );

    if selected && fill_override.is_none() {
        painter.circle_stroke(
            draw.center(),
            radius - 0.2,
            Stroke::new(2.0_f32, rgba_u(0x8B, 0x7C, 0xB6, 0xA0)),
        );
    }

    let icon = icon_color_for_fill(fill);
    let icon_rect = Rect::from_center_size(draw.center(), Vec2::splat(diameter * 0.50 * scale));
    paint_icon(&painter, icon_rect, icon);

    response.clicked()
}

/// Circular stroke-color swatch that opens egui's color wheel (opaque stroke).
pub fn tool_color_swatch(ui: &mut egui::Ui, color: &mut Color32) -> egui::Response {
    let popup_id = Id::new("lightsnip_stroke_color_popup");
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(TOOL_BTN), Sense::click());
    let response = response.on_hover_text("Stroke color");
    let hovered = response.hovered();

    let dark = ui.visuals().dark_mode;
    let pal = AnnotatePalette::for_mode(dark, true);
    let fill = if hovered { pal.btn_hover } else { pal.btn_idle };
    let radius = TOOL_BTN * 0.46;
    let painter = ui.painter_at(rect.expand(4.0));
    paint_glass_surface(&painter, rect, radius, fill, hovered, response.hover_pos());
    painter.circle_filled(rect.center(), radius * 0.58, *color);
    painter.circle_stroke(
        rect.center(),
        radius * 0.58,
        Stroke::new(1.4_f32, GLASS_RIM_LIGHT),
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

// Silence unused import if PathShape path uses epaint directly elsewhere.
#[allow(unused_imports)]
use epaint as _;

// --- Icon painters (thicker strokes + feather-friendly geometry) --------------

const ICON_STROKE: f32 = 2.35;

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

/// Clear curved undo arrow (readable at small sizes).
pub fn paint_icon_undo(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(ICON_STROKE + 0.15, color);
    let c = rect.center() + Vec2::new(rect.width() * 0.04, rect.height() * 0.06);
    let r = rect.width() * 0.30;
    let start = -0.35_f32;
    let sweep = std::f32::consts::PI * 1.35;
    let steps = 36;
    let mut prev = Pos2::new(c.x + r * start.cos(), c.y + r * start.sin());
    for i in 1..=steps {
        let t = start - sweep * (i as f32 / steps as f32);
        let p = Pos2::new(c.x + r * t.cos(), c.y + r * t.sin());
        painter.line_segment([prev, p], stroke);
        prev = p;
    }
    let tip_angle = start - sweep;
    let tip = Pos2::new(c.x + r * tip_angle.cos(), c.y + r * tip_angle.sin());
    let tangent = Vec2::new(tip_angle.sin(), -tip_angle.cos());
    let wing = rect.width() * 0.16;
    let left = tip + tangent * wing + Vec2::new(-tangent.y, tangent.x) * (wing * 0.55);
    let right = tip + tangent * wing - Vec2::new(-tangent.y, tangent.x) * (wing * 0.55);
    painter.line_segment([tip, left], stroke);
    painter.line_segment([tip, right], stroke);
}

pub fn paint_icon_clear(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(ICON_STROKE, color);
    let inset = rect.shrink(rect.width() * 0.28);
    painter.line_segment([inset.left_top(), inset.right_bottom()], stroke);
    painter.line_segment([inset.right_top(), inset.left_bottom()], stroke);
}

pub fn paint_icon_settings(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let outer = rect.width() * 0.28;
    let inner = rect.width() * 0.12;
    painter.circle_stroke(c, outer, Stroke::new(ICON_STROKE + 0.2, color));
    painter.circle_filled(c, inner, color);
    for i in 0..8 {
        let a = (i as f32) * std::f32::consts::TAU / 8.0;
        let p0 = c + Vec2::angled(a) * (outer * 0.78);
        let p1 = c + Vec2::angled(a) * (outer * 1.32);
        painter.line_segment([p0, p1], Stroke::new(2.6_f32, color));
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

/// Fit `img` into `area` (letterbox). Allows upscale so small captures stay sharp-ish when window is large.
pub fn fit_image_rect(area: Rect, img_w: f32, img_h: f32) -> (Rect, f32) {
    let img_size = Vec2::new(img_w.max(1.0), img_h.max(1.0));
    let scale = (area.width() / img_size.x).min(area.height() / img_size.y);
    let display = img_size * scale;
    let origin = area.min
        + Vec2::new(
            (area.width() - display.x) * 0.5,
            (area.height() - display.y) * 0.5,
        );
    (Rect::from_min_size(origin, display), scale)
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
        assert!(SETTINGS_BTN > TOOL_BTN);
        assert!(TOOLBAR_WIDTH >= TOOL_BTN);
        assert!(WINDOW_PAD >= 12.0);
        assert!(TOOLBAR_GUTTER >= 12.0);
    }

    #[test]
    fn glass_fill_keeps_pastel_not_pure_white() {
        // Palette must stay lilac/gray — not generic white-glass-only.
        // Colors are stored premultiplied; recover unmultiplied for the check.
        let unmul = |c: Color32| -> (u8, u8, u8) {
            let a = c.a().max(1) as u16;
            (
                ((c.r() as u16 * 255) / a) as u8,
                ((c.g() as u16 * 255) / a) as u8,
                ((c.b() as u16 * 255) / a) as u8,
            )
        };
        let (fr, fg, fb) = unmul(GLASS_FILL);
        assert!(fr < 255 || fg < 255 || fb < 255);
        assert!(fb >= fr.saturating_sub(8)); // cool/lilac bias
        let (sr, sg, sb) = unmul(GLASS_FILL_SELECTED);
        assert!(sr > 0xC0 && sb > sg); // pastel purple selected tint
        // Sample-like translucency band (not fully opaque).
        assert!(GLASS_FILL.a() > 20 && GLASS_FILL.a() < 120);
        assert!(GLASS_FILL_HOVER.a() >= GLASS_FILL.a());
    }

    #[test]
    fn light_glass_gets_dark_icons() {
        assert_eq!(icon_color_for_fill(GLASS_FILL), ICON_DARK);
        assert_eq!(icon_color_for_fill(Color32::WHITE), ICON_DARK);
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
        assert!(PASTEL_PRIMARY.r() < 0xB0);
        assert!(PASTEL_PRIMARY.b() < 0xD0);
        assert!(approx_luminance(PASTEL_PRIMARY) > 100.0);
        assert!(approx_luminance(PASTEL_PRIMARY) < 180.0);
    }

    #[test]
    fn fit_image_upscales_small_captures() {
        let area = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let (r, scale) = fit_image_rect(area, 200.0, 100.0);
        assert!(scale > 1.0, "small images should scale up with the window");
        assert!((r.width() - 800.0).abs() < 1.0 || (r.height() - 600.0).abs() < 1.0);
        assert!((r.center() - area.center()).length() < 1.0);
    }

    #[test]
    fn theme_preference_maps() {
        assert_eq!(theme_preference(UiTheme::Dark), ThemePreference::Dark);
        assert_eq!(theme_preference(UiTheme::Light), ThemePreference::Light);
        assert_eq!(theme_preference(UiTheme::System), ThemePreference::System);
    }
}
