//! Shared visuals for annotate + settings.
//!
//! Annotate: flat rounded-rect chrome (light gray / white bar, charcoal icons).
//! Only the *active* tool gets a soft purple rounded-square highlight
//! (pale fill + purple border + purple icon). No neon, no multi-color accents
//! beyond the stroke color picker. When Window Transparency / Background Blur
//! are on, we apply real window opacity plus a translucent frosted panel on the
//! window/preview area; the tool rail stays fully opaque.
//!
//! Design dials: ENERGY 2 / RHYTHM 2 / MOTION 1 (hover fill only).
//! Palette note: restrained purple/gray/white is an owner override (R-37).

use eframe::egui::{
    self, Color32, CornerRadius, Id, Margin, Pos2, Rect, Sense, Shadow, Stroke, Theme,
    ThemePreference, Vec2,
};
use eframe::egui::color_picker::{color_picker_color32, Alpha};
use eframe::egui::popup::{popup_below_widget, PopupCloseBehavior};

use crate::config::UiTheme;

// --- Annotate sizes -----------------------------------------------------------

/// Tool button hit target (desktop ≥ 28 pt). Rounded square, not a circle.
pub const TOOL_BTN: f32 = 30.0;

/// Settings gear matches other tools for compact bar chrome.
pub const SETTINGS_BTN: f32 = 30.0;

/// Toolbar strip thickness (cross-axis).
pub const TOOLBAR_WIDTH: f32 = 44.0;

/// Gutter between image and floating tool rail.
pub const TOOLBAR_GUTTER: f32 = 12.0;

/// Outer padding inside the floating window.
pub const WINDOW_PAD: f32 = 18.0;

/// Modest bar corner radius — matches tool wells, not a capsule/pill.
pub const TOOLBAR_RADIUS: f32 = 8.0;

/// Rounded-square highlight radius on tool wells.
pub const TOOL_WELL_RADIUS: f32 = 7.0;

/// Universal settings control row height (buttons, text fields, checkbox wells).
pub const SETTINGS_CONTROL_HEIGHT: f32 = 28.0;

/// Checkbox mark size inside the universal control height.
pub const SETTINGS_CHECKBOX_SIZE: f32 = 18.0;

/// Default markup stroke (red). Accent for marks lives on the image.
pub const DEFAULT_STROKE: Color32 = Color32::from_rgb(0xE2, 0x4A, 0x3B);

// --- Annotate light palette (flat purple / gray / white) ----------------------

pub const ANNOTATE_BG: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF4);
pub const TOOLBAR_FILL: Color32 = Color32::from_rgb(0xF7, 0xF7, 0xF8);
pub const TOOLBAR_STROKE: Color32 = Color32::from_rgb(0xD8, 0xD8, 0xDC);
/// Active tool only: pale purple fill.
pub const BTN_SELECTED: Color32 = Color32::from_rgb(0xED, 0xE7, 0xF6);
/// Idle wells stay transparent against the bar (no white disc on every icon).
pub const BTN_IDLE: Color32 = Color32::TRANSPARENT;
pub const BTN_HOVER: Color32 = Color32::from_rgb(0xEC, 0xEC, 0xF0);

/// Idle icon ink on light chrome (near-black — must beat charcoal-on-dark misses).
pub const ICON_DARK: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);
/// Idle icon ink on dark chrome (near-white).
pub const ICON_LIGHT: Color32 = Color32::from_rgb(0xFA, 0xFA, 0xFC);

/// High-contrast ink for thickness HUD / toolbar readout on light chrome.
pub const THICKNESS_INK_ON_LIGHT: Color32 = ICON_DARK;
/// High-contrast ink for thickness HUD / toolbar readout on dark chrome.
pub const THICKNESS_INK_ON_DARK: Color32 = ICON_LIGHT;

/// Active-tool icon on dark selected wells (light lavender — purple-on-purple was invisible).
pub const ICON_ACTIVE_ON_DARK: Color32 = Color32::from_rgb(0xE4, 0xD8, 0xF8);

/// Active-tool purple (border + icon). Soft, not neon.
pub const PASTEL_PRIMARY: Color32 = Color32::from_rgb(0x8E, 0x74, 0xC2);
/// Hover companion for solid purple fills (settings / future action wells).
#[allow(dead_code)]
pub const PASTEL_PRIMARY_HOVER: Color32 = Color32::from_rgb(0x9A, 0x84, 0xCA);

/// Kept for settings/action accents that still need a solid gray fill.
#[allow(dead_code)]
pub const ACTION_FILL: Color32 = Color32::from_rgb(0x6E, 0x68, 0x7A);
#[allow(dead_code)]
pub const ACTION_FILL_HOVER: Color32 = Color32::from_rgb(0x7E, 0x78, 0x8A);

pub const MUTED: Color32 = Color32::from_rgb(0x8A, 0x8A, 0x92);
pub const PREVIEW_MATTE: Color32 = Color32::from_rgb(0xE4, 0xE4, 0xE8);
pub const TOOLBAR_SEPARATOR: Color32 = Color32::from_rgb(0xD0, 0xD0, 0xD4);

// --- Annotate dark palette ----------------------------------------------------

pub const ANNOTATE_BG_DARK: Color32 = Color32::from_rgb(0x1C, 0x1B, 0x20);
pub const TOOLBAR_FILL_DARK: Color32 = Color32::from_rgb(0x2C, 0x2B, 0x32);
pub const TOOLBAR_STROKE_DARK: Color32 = Color32::from_rgb(0x48, 0x44, 0x52);
pub const BTN_SELECTED_DARK: Color32 = Color32::from_rgb(0x3D, 0x34, 0x52);
pub const BTN_IDLE_DARK: Color32 = Color32::TRANSPARENT;
pub const BTN_HOVER_DARK: Color32 = Color32::from_rgb(0x3A, 0x38, 0x42);
pub const MUTED_DARK: Color32 = Color32::from_rgb(0xA8, 0xA2, 0xB0);
pub const PREVIEW_MATTE_DARK: Color32 = Color32::from_rgb(0x28, 0x26, 0x2E);
pub const TOOLBAR_SEPARATOR_DARK: Color32 = Color32::from_rgb(0x54, 0x50, 0x5C);

// --- Settings -----------------------------------------------------------------

pub const SETTINGS_BG: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF7);
pub const SETTINGS_CARD: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const SETTINGS_SEPARATOR: Color32 = Color32::from_rgb(0xC6, 0xC6, 0xC8);
pub const SETTINGS_LABEL: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);
pub const SETTINGS_SECONDARY: Color32 = Color32::from_rgb(0x6C, 0x6C, 0x70);
pub const SETTINGS_FIELD: Color32 = Color32::from_rgb(0xE8, 0xE8, 0xED);
/// Lavender accent (OK, links, active nav) — matches Settings reference chrome.
pub const SETTINGS_PRIMARY: Color32 = Color32::from_rgb(0x7C, 0x72, 0xC3);

/// Deep charcoal shell for the two-pane Settings window.
pub const SETTINGS_BG_DARK: Color32 = Color32::from_rgb(0x1A, 0x1B, 0x1E);
pub const SETTINGS_CARD_DARK: Color32 = Color32::from_rgb(0x22, 0x23, 0x27);
pub const SETTINGS_SEPARATOR_DARK: Color32 = Color32::from_rgb(0x32, 0x33, 0x38);
pub const SETTINGS_LABEL_DARK: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF7);
pub const SETTINGS_SECONDARY_DARK: Color32 = Color32::from_rgb(0x9A, 0x9A, 0xA0);
pub const SETTINGS_FIELD_DARK: Color32 = Color32::from_rgb(0x2A, 0x2B, 0x2F);
/// Subtle dark-purple rounded highlight for the active sidebar item.
pub const SETTINGS_NAV_ACTIVE: Color32 = Color32::from_rgb(0x2C, 0x28, 0x3C);
pub const SETTINGS_KEYCAP: Color32 = Color32::from_rgb(0x2A, 0x2B, 0x2E);
pub const SETTINGS_FOOTER_BTN: Color32 = Color32::from_rgb(0x2E, 0x2F, 0x33);
pub const SETTINGS_SIDEBAR_W: f32 = 188.0;
pub const SETTINGS_FOOTER_H: f32 = 52.0;

/// Runtime annotate colors for the active theme.
#[derive(Debug, Clone, Copy)]
pub struct AnnotatePalette {
    pub bg: Color32,
    pub toolbar_fill: Color32,
    pub toolbar_stroke: Color32,
    pub btn_idle: Color32,
    pub btn_hover: Color32,
    pub btn_selected: Color32,
    pub muted: Color32,
    pub preview_matte: Color32,
    pub dark: bool,
    /// Window / panel opacity from transparency intensity (`1.0` = fully opaque).
    pub panel_opacity: f32,
    /// Frost strength from blur intensity (`0.0` = solid flat, higher = more frost on preview).
    /// Retained for settings/debug; toolbar no longer samples it (stays opaque).
    #[allow(dead_code)]
    pub frost: f32,
}

impl AnnotatePalette {
    pub fn for_mode(dark: bool, translucent: bool) -> Self {
        Self::with_effects(dark, translucent, 0.0, 0.0)
    }

    /// Build palette with transparency + frosted-panel intensities (`0.0..=1.0`).
    ///
    /// Transparency lowers panel/window alpha. Blur cannot sample the desktop in
    /// egui; we approximate with a lighter translucent frost fill on chrome.
    pub fn with_effects(
        dark: bool,
        translucent: bool,
        transparency_intensity: f32,
        blur_intensity: f32,
    ) -> Self {
        let t = if translucent {
            transparency_intensity.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let frost = blur_intensity.clamp(0.0, 1.0);
        // Keep chrome readable: never drop below ~0.42 when transparency is on.
        let panel_opacity = if translucent {
            (1.0 - t * 0.55).clamp(0.42, 1.0)
        } else {
            1.0
        };
        let mut pal = if dark {
            Self {
                bg: ANNOTATE_BG_DARK,
                toolbar_fill: TOOLBAR_FILL_DARK,
                toolbar_stroke: TOOLBAR_STROKE_DARK,
                btn_idle: BTN_IDLE_DARK,
                btn_hover: BTN_HOVER_DARK,
                btn_selected: BTN_SELECTED_DARK,
                muted: MUTED_DARK,
                preview_matte: PREVIEW_MATTE_DARK,
                dark: true,
                panel_opacity,
                frost,
            }
        } else {
            Self {
                bg: ANNOTATE_BG,
                toolbar_fill: TOOLBAR_FILL,
                toolbar_stroke: TOOLBAR_STROKE,
                btn_idle: BTN_IDLE,
                btn_hover: BTN_HOVER,
                btn_selected: BTN_SELECTED,
                muted: MUTED,
                preview_matte: PREVIEW_MATTE,
                dark: false,
                panel_opacity,
                frost,
            }
        };

        // Transparency / frost affect the window clear color and preview matte only.
        // The tool rail stays a solid pastel panel so intensity sliders never wash it out.
        if translucent || frost > 0.0 {
            pal.bg = with_alpha(pal.bg, panel_opacity);
            if frost > 0.0 {
                let matte_a = (panel_opacity * (1.0 - frost * 0.25)).clamp(0.45, 1.0);
                pal.preview_matte = with_alpha(pal.preview_matte, matte_a);
                pal.preview_matte = frost_lift(pal.preview_matte, frost * 0.5, dark);
            }
        }
        // Toolbar fill/stroke remain fully opaque (alpha 255).
        debug_assert_eq!(pal.toolbar_fill.a(), 255);
        debug_assert_eq!(pal.toolbar_stroke.a(), 255);
        pal
    }
}

fn with_alpha(c: Color32, a: f32) -> Color32 {
    let a = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

fn frost_lift(c: Color32, frost: f32, dark: bool) -> Color32 {
    let frost = frost.clamp(0.0, 1.0);
    let lift = if dark { 28.0 } else { 18.0 } * frost;
    let r = (c.r() as f32 + lift).min(255.0) as u8;
    let g = (c.g() as f32 + lift).min(255.0) as u8;
    let b = (c.b() as f32 + lift).min(255.0) as u8;
    Color32::from_rgba_unmultiplied(r, g, b, c.a())
}

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
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, pal.toolbar_stroke);
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
        w.corner_radius = CornerRadius::same(8);
    }
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);
    visuals.window_stroke = Stroke::new(1.0_f32, pal.toolbar_stroke);
    visuals.window_shadow = Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: Color32::from_black_alpha(if dark { 64 } else { 36 }),
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
    // Slider rails use `inactive.bg_fill` — keep clearly above card/bg contrast
    // (dark card 0x2C2C2E / light card white) so intensity tracks stay visible
    // when Settings is layered over the on-screen annotate overlay.
    let slider_track = if dark {
        Color32::from_rgb(0x5C, 0x5C, 0x60)
    } else {
        Color32::from_rgb(0xC7, 0xC7, 0xCC)
    };
    visuals.panel_fill = bg;
    visuals.window_fill = card;
    visuals.extreme_bg_color = slider_track;
    visuals.override_text_color = Some(label);
    visuals.widgets.noninteractive.bg_fill = bg;
    visuals.widgets.noninteractive.weak_bg_fill = bg;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, sep);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, secondary);
    visuals.widgets.inactive.bg_fill = slider_track;
    visuals.widgets.inactive.weak_bg_fill = field;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, label);
    visuals.widgets.hovered.bg_fill = if dark {
        Color32::from_rgb(0x6E, 0x6E, 0x72)
    } else {
        Color32::from_rgb(0xB8, 0xB8, 0xBE)
    };
    visuals.widgets.hovered.weak_bg_fill = visuals.widgets.hovered.bg_fill;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, label);
    visuals.widgets.active.bg_fill = if dark {
        Color32::from_rgb(0x7A, 0x7A, 0x7E)
    } else {
        Color32::from_rgb(0xA8, 0xA8, 0xAE)
    };
    visuals.widgets.active.weak_bg_fill = visuals.widgets.active.bg_fill;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, label);
    visuals.widgets.open.bg_fill = field;
    visuals.widgets.open.weak_bg_fill = field;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, label);
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0x7C, 0x72, 0xC3, 0x99);
    visuals.slider_trailing_fill = true;
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
        offset: [0, 1],
        blur: 6,
        spread: 0,
        color: Color32::from_black_alpha(32),
    };
    visuals.popup_shadow = Shadow::NONE;
    // High-contrast checkbox marks on light and dark cards.
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.5_f32, label);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, label);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, label);
    visuals
}

/// Install light + dark annotate styles and select theme preference.
pub fn apply_annotate_visuals(ctx: &egui::Context, theme: UiTheme) {
    ctx.set_visuals_of(Theme::Light, annotate_visuals(false));
    ctx.set_visuals_of(Theme::Dark, annotate_visuals(true));
    ctx.set_theme(theme_preference(theme));
    // Low feathering keeps vector icons crisp (less soft/pixelated AA).
    ctx.tessellation_options_mut(|opts| {
        opts.feathering = true;
        opts.feathering_size_in_pixels = 0.28;
    });
}

/// Install light + dark settings styles and select theme preference.
pub fn apply_settings_visuals(ctx: &egui::Context, theme: UiTheme) {
    ctx.set_visuals_of(Theme::Light, settings_visuals(false));
    ctx.set_visuals_of(Theme::Dark, settings_visuals(true));
    ctx.set_theme(theme_preference(theme));
    ctx.tessellation_options_mut(|opts| {
        opts.feathering = true;
        opts.feathering_size_in_pixels = 0.28;
    });
}

/// Frame for the on-screen floating flat tool bar (content-sized pill).
///
/// Flat fill only — no border stroke. Modest radius matches tool wells.
/// Window transparency / blur never wash the fill out.
pub fn toolbar_frame(pal: &AnnotatePalette) -> egui::Frame {
    let shadow_a = if pal.dark { 36 } else { 20 };
    egui::Frame::new()
        .fill(Color32::from_rgb(
            pal.toolbar_fill.r(),
            pal.toolbar_fill.g(),
            pal.toolbar_fill.b(),
        ))
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(TOOLBAR_RADIUS as u8))
        .inner_margin(Margin::symmetric(5, 5))
        .outer_margin(Margin::ZERO)
        .shadow(Shadow {
            offset: [0, 1],
            blur: 6,
            spread: 0,
            color: Color32::from_black_alpha(shadow_a),
        })
}

/// Full-width top strip for the preview/window annotate chrome.
///
/// Embedded in the window (not a floating `Area`) so Iconoir glyphs stay in the
/// normal paint path and remain high-contrast on the solid bar.
pub fn embedded_top_toolbar_frame(pal: &AnnotatePalette) -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(
            pal.toolbar_fill.r(),
            pal.toolbar_fill.g(),
            pal.toolbar_fill.b(),
        ))
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::ZERO)
        .inner_margin(Margin::symmetric(8, 6))
        .outer_margin(Margin::ZERO)
}

/// Subtle separator between toolbar groups (vertical for horizontal bars).
pub fn toolbar_separator(ui: &mut egui::Ui, horizontal: bool) {
    let dark = ui.visuals().dark_mode;
    let color = if dark {
        TOOLBAR_SEPARATOR_DARK
    } else {
        TOOLBAR_SEPARATOR
    };
    let (size, stroke_len) = if horizontal {
        (Vec2::new(8.0, TOOL_BTN), TOOL_BTN * 0.55)
    } else {
        (Vec2::new(TOOL_BTN, 8.0), TOOL_BTN * 0.55)
    };
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    if horizontal {
        let x = rect.center().x;
        let half = stroke_len * 0.5;
        painter.line_segment(
            [
                Pos2::new(x, rect.center().y - half),
                Pos2::new(x, rect.center().y + half),
            ],
            Stroke::new(1.0_f32, color),
        );
    } else {
        let y = rect.center().y;
        let half = stroke_len * 0.5;
        painter.line_segment(
            [
                Pos2::new(rect.center().x - half, y),
                Pos2::new(rect.center().x + half, y),
            ],
            Stroke::new(1.0_f32, color),
        );
    }
}

/// Grouped settings card (legacy light-card layout; kept for embed helpers).
#[allow(dead_code)]
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
        .inner_margin(Margin::symmetric(14, 12))
        .shadow(Shadow {
            offset: [0, 1],
            blur: 4,
            spread: 0,
            color: Color32::from_black_alpha(if dark { 48 } else { 18 }),
        })
}

/// Icon ink that stays readable on the well behind the glyph.
pub fn tool_icon_ink(selected: bool, dark_chrome: bool, well_fill: Color32) -> Color32 {
    if selected {
        // Purple-on-dark-purple (PASTEL on BTN_SELECTED_DARK) reads as empty.
        if dark_chrome || approx_luminance(well_fill) < 140.0 {
            ICON_ACTIVE_ON_DARK
        } else {
            PASTEL_PRIMARY
        }
    } else {
        icon_color_for_fill(well_fill)
    }
}

/// Rounded-square tool button. Only `selected` tools get the purple highlight;
/// idle icons use high-contrast ink against the bar; hover is a faint gray well.
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
    let pal = AnnotatePalette::for_mode(dark, false);
    // Snap wells/icons to half-pixels for cleaner stroke AA.
    let center = Pos2::new(rect.center().x.round() + 0.5, rect.center().y.round() + 0.5);
    let well = Rect::from_center_size(center, Vec2::splat(diameter - 6.0));
    let radius = TOOL_WELL_RADIUS;

    let painter = ui.painter_at(rect);

    // Effective fill behind the glyph (toolbar shows through transparent idle wells).
    let well_fill = if selected && fill_override.is_none() {
        pal.btn_selected
    } else if let Some((idle, hover)) = fill_override {
        if hovered { hover } else { idle }
    } else if hovered {
        pal.btn_hover
    } else {
        pal.toolbar_fill
    };

    // Active tool: pale purple fill + purple border. Never highlight non-tools via selected.
    if selected && fill_override.is_none() {
        painter.rect_filled(well, radius, pal.btn_selected);
        painter.rect_stroke(
            well,
            radius,
            Stroke::new(1.25_f32, PASTEL_PRIMARY),
            egui::StrokeKind::Inside,
        );
    } else if let Some((idle, hover)) = fill_override {
        let fill = if hovered { hover } else { idle };
        painter.rect_filled(well, radius, fill);
    } else if hovered {
        painter.rect_filled(well, radius, pal.btn_hover);
    }

    let icon = tool_icon_ink(selected && fill_override.is_none(), pal.dark, well_fill);
    // Slightly larger glyph well; Iconoir is supersampled for crisp HiDPI.
    let icon_rect = Rect::from_center_size(center, Vec2::splat(diameter * 0.60));
    paint_icon(&painter, icon_rect, icon);

    response.clicked()
}

/// Stroke-color swatch that opens egui's color wheel (alpha supported for highlighter).
pub fn tool_color_swatch(ui: &mut egui::Ui, color: &mut Color32) -> egui::Response {
    let popup_id = Id::new("hypershot_stroke_color_popup");
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(TOOL_BTN), Sense::click());
    let response = response.on_hover_text("Stroke color (alpha for highlighter)");
    let hovered = response.hovered();

    let dark = ui.visuals().dark_mode;
    let pal = AnnotatePalette::for_mode(dark, false);
    let well = Rect::from_center_size(rect.center(), Vec2::splat(TOOL_BTN - 4.0));
    let painter = ui.painter_at(rect);
    if hovered {
        painter.rect_filled(well, TOOL_WELL_RADIUS, pal.btn_hover);
    }
    let swatch_r = TOOL_BTN * 0.28;
    // Checker under translucent swatches so alpha is visible.
    if color.a() < 255 {
        let c0 = Color32::from_rgb(0xE8, 0xE8, 0xEC);
        let c1 = Color32::from_rgb(0xC8, 0xC8, 0xD0);
        painter.circle_filled(rect.center(), swatch_r, c0);
        painter.circle_filled(
            rect.center() + Vec2::new(swatch_r * 0.35, -swatch_r * 0.35),
            swatch_r * 0.45,
            c1,
        );
    }
    painter.circle_filled(rect.center(), swatch_r, *color);
    painter.circle_stroke(
        rect.center(),
        swatch_r,
        Stroke::new(1.25_f32, Color32::from_rgba_premultiplied(0x3A, 0x3A, 0x40, 0x50)),
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
            color_picker_color32(ui, color, Alpha::OnlyBlend);
        },
    );

    response
}

// Aliases so annotate can keep short call sites.
pub use tool_icon_button as glass_icon_button;
pub use tool_color_swatch as glass_color_swatch;
pub use toolbar_frame as glass_toolbar_frame;

/// Segmented choice button (toolbar position, theme, etc.).
pub fn choice_button(ui: &mut egui::Ui, selected: bool, label: &str) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let (fill, stroke, text) = if selected {
        (
            SETTINGS_PRIMARY,
            SETTINGS_PRIMARY,
            Color32::WHITE,
        )
    } else if dark {
        (
            SETTINGS_FIELD_DARK,
            SETTINGS_SEPARATOR_DARK,
            SETTINGS_LABEL_DARK,
        )
    } else {
        (
            SETTINGS_FIELD,
            SETTINGS_SEPARATOR,
            SETTINGS_LABEL,
        )
    };
    let btn = egui::Button::new(egui::RichText::new(label).color(text).size(12.0).strong())
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, stroke))
        .corner_radius(CornerRadius::same(6))
        .min_size(Vec2::new(64.0, SETTINGS_CONTROL_HEIGHT));
    ui.add(btn)
}

/// Primary / secondary settings button sized to the universal control height.
pub fn settings_button(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    fill: Option<Color32>,
    min_width: f32,
) -> egui::Response {
    let mut btn = egui::Button::new(label).min_size(Vec2::new(min_width, SETTINGS_CONTROL_HEIGHT));
    if let Some(fill) = fill {
        btn = btn.fill(fill);
    }
    ui.add(btn)
}

/// Single-line text field with the universal control height.
pub fn settings_text_edit(ui: &mut egui::Ui, text: &mut String, width: f32, hint: &str) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(text)
            .desired_width(width)
            .min_size(Vec2::new(0.0, SETTINGS_CONTROL_HEIGHT))
            .margin(Margin::symmetric(8, 4))
            .hint_text(hint),
    )
}

/// High-contrast checkbox for light and dark settings cards.
///
/// Painted with an Inside stroke and field-colored unchecked fill so rounded
/// corners do not leave white AA artifacts on card backgrounds.
pub fn contrast_checkbox(ui: &mut egui::Ui, value: &mut bool) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let box_size = SETTINGS_CHECKBOX_SIZE;
    // Allocate the universal control height so the well aligns with buttons/fields.
    let (row, mut response) =
        ui.allocate_exact_size(Vec2::new(box_size, SETTINGS_CONTROL_HEIGHT), Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    let rect = Rect::from_center_size(row.center(), Vec2::splat(box_size));
    let painter = ui.painter_at(row);
    let radius = 3.0_f32;
    let border = if dark {
        SETTINGS_LABEL_DARK
    } else {
        Color32::from_rgb(0x3A, 0x3A, 0x3C)
    };
    // Unchecked: match card field tone (not pure white) to avoid corner speckles.
    let fill = if *value {
        SETTINGS_PRIMARY
    } else if dark {
        SETTINGS_FIELD_DARK
    } else {
        SETTINGS_FIELD
    };
    let stroke_color = if *value { SETTINGS_PRIMARY } else { border };

    // Fill first, then Inside stroke so AA stays within the rounded rect.
    painter.rect_filled(rect, radius, fill);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.5_f32, stroke_color),
        egui::StrokeKind::Inside,
    );
    if *value {
        let stroke = Stroke::new(2.0_f32, Color32::WHITE);
        let a = Pos2::new(rect.left() + box_size * 0.22, rect.center().y + box_size * 0.02);
        let b = Pos2::new(
            rect.center().x - box_size * 0.04,
            rect.bottom() - box_size * 0.24,
        );
        let c = Pos2::new(rect.right() - box_size * 0.20, rect.top() + box_size * 0.22);
        painter.line_segment([a, b], stroke);
        painter.line_segment([b, c], stroke);
    }
    response
}

/// Card section title with an Iconoir question-mark help control (tooltip = section help).
///
/// Avoids RichText `"?"` / unicode tofus that can show as empty boxes on some fonts.
#[allow(dead_code)]
pub fn section_title_with_help(
    ui: &mut egui::Ui,
    title: &str,
    help: &str,
    label: Color32,
    secondary: Color32,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(title)
                .color(label)
                .size(13.0)
                .strong(),
        );
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
        let hovered = response.hovered();
        let painter = ui.painter_at(rect);
        let fill = if hovered {
            Color32::from_rgba_unmultiplied(secondary.r(), secondary.g(), secondary.b(), 28)
        } else {
            Color32::TRANSPARENT
        };
        painter.rect(
            rect,
            CornerRadius::same(9),
            fill,
            Stroke::new(1.0_f32, secondary),
            egui::StrokeKind::Inside,
        );
        let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(12.0));
        crate::icons::paint_icon_help(&painter, icon_rect, secondary);
        response.on_hover_text(help);
    });
    ui.add_space(6.0);
}

// --- Icon painters (Iconoir SVGs via `icons`, DPI-aware) ---------------------

pub use crate::icons::{
    paint_icon_arrow, paint_icon_box, paint_icon_clear, paint_icon_copy, paint_icon_exit,
    paint_icon_highlighter, paint_icon_line, paint_icon_magnify, paint_icon_pixelate,
    paint_icon_preview, paint_icon_redo, paint_icon_save, paint_icon_settings, paint_icon_step,
    paint_icon_text, paint_icon_undo,
};

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
        assert_eq!(SETTINGS_BTN, TOOL_BTN);
        assert!(TOOLBAR_WIDTH >= TOOL_BTN);
        assert!(WINDOW_PAD >= 12.0);
        assert!(TOOLBAR_GUTTER >= 12.0);
        // Flat bar: modest radius near tool wells — not a capsule (≥ half bar height).
        assert!(TOOLBAR_RADIUS <= TOOL_WELL_RADIUS + 2.0);
        assert!(TOOLBAR_RADIUS < TOOLBAR_WIDTH * 0.35);
        assert!(TOOL_WELL_RADIUS >= 6.0);
    }

    #[test]
    fn flat_toolbar_is_light_gray_with_purple_active_only() {
        assert_eq!(TOOLBAR_FILL, Color32::from_rgb(0xF7, 0xF7, 0xF8));
        assert_eq!(BTN_IDLE, Color32::TRANSPARENT);
        assert!(BTN_SELECTED.b() > BTN_SELECTED.g());
        // Selected well is pale (high luminance); primary accent is purple.
        assert!(approx_luminance(BTN_SELECTED) > 180.0);
        assert!(PASTEL_PRIMARY.b() > PASTEL_PRIMARY.r());
    }

    #[test]
    fn settings_slider_track_contrasts_card() {
        for dark in [false, true] {
            let v = settings_visuals(dark);
            let track = v.widgets.inactive.bg_fill;
            let card = v.window_fill;
            assert!(
                track.a() == 255,
                "slider track must be opaque (dark={dark})"
            );
            let lum_delta = (approx_luminance(track) - approx_luminance(card)).abs();
            assert!(
                lum_delta >= 40.0,
                "slider track must contrast card (dark={dark}, ΔL={lum_delta:.1})"
            );
        }
    }

    #[test]
    fn light_wells_get_dark_icons() {
        assert_eq!(icon_color_for_fill(BTN_SELECTED), ICON_DARK);
        assert_eq!(icon_color_for_fill(BTN_HOVER), ICON_DARK);
        assert_eq!(icon_color_for_fill(Color32::WHITE), ICON_DARK);
        assert_eq!(icon_color_for_fill(TOOLBAR_FILL), ICON_DARK);
        assert!(approx_luminance(ICON_DARK) < 100.0);
    }

    #[test]
    fn dark_and_primary_fills_get_light_icons() {
        assert_eq!(icon_color_for_fill(ACTION_FILL), ICON_LIGHT);
        assert_eq!(icon_color_for_fill(TOOLBAR_FILL_DARK), ICON_LIGHT);
        assert_eq!(icon_color_for_fill(BTN_SELECTED_DARK), ICON_LIGHT);
        // Primary purple is mid-luminance; used as icon ink when selected on light chrome.
        assert!(approx_luminance(ICON_LIGHT) > 200.0);
        assert!(approx_luminance(PASTEL_PRIMARY) > 80.0);
        assert!(approx_luminance(PASTEL_PRIMARY) < 160.0);
    }

    #[test]
    fn tool_icon_ink_contrasts_dark_and_light_chrome() {
        // Idle on dark bar → near-white (charcoal would vanish).
        assert_eq!(
            tool_icon_ink(false, true, TOOLBAR_FILL_DARK),
            ICON_LIGHT
        );
        // Idle on light bar → near-black.
        assert_eq!(tool_icon_ink(false, false, TOOLBAR_FILL), ICON_DARK);
        // Selected on dark well → light lavender, not mid purple-on-purple.
        assert_eq!(
            tool_icon_ink(true, true, BTN_SELECTED_DARK),
            ICON_ACTIVE_ON_DARK
        );
        // Selected on light well → brand purple.
        assert_eq!(
            tool_icon_ink(true, false, BTN_SELECTED),
            PASTEL_PRIMARY
        );
        // Luminance delta must stay readable.
        let dark_delta =
            (approx_luminance(ICON_LIGHT) - approx_luminance(TOOLBAR_FILL_DARK)).abs();
        let light_delta =
            (approx_luminance(ICON_DARK) - approx_luminance(TOOLBAR_FILL)).abs();
        assert!(dark_delta >= 120.0, "dark chrome ΔL={dark_delta:.1}");
        assert!(light_delta >= 120.0, "light chrome ΔL={light_delta:.1}");
    }

    #[test]
    fn pastel_primary_is_muted_not_neon() {
        assert!(PASTEL_PRIMARY.r() < 0xB0);
        assert!(PASTEL_PRIMARY.b() < 0xD0);
        assert!(approx_luminance(PASTEL_PRIMARY) > 80.0);
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

    #[test]
    fn settings_controls_share_height() {
        assert_eq!(SETTINGS_CONTROL_HEIGHT, 28.0);
        assert!(SETTINGS_CHECKBOX_SIZE < SETTINGS_CONTROL_HEIGHT);
        assert!(SETTINGS_CHECKBOX_SIZE >= 16.0);
    }

    #[test]
    fn palette_effects_lower_opacity_and_frost() {
        let solid = AnnotatePalette::with_effects(false, false, 0.0, 0.0);
        assert!((solid.panel_opacity - 1.0).abs() < f32::EPSILON);
        assert_eq!(solid.frost, 0.0);
        assert_eq!(solid.bg.a(), 255);

        let clear = AnnotatePalette::with_effects(false, true, 1.0, 0.8);
        assert!(clear.panel_opacity < 1.0);
        assert!(clear.bg.a() < 255);
        // Toolbar stays fully opaque even when transparency / blur are maxed.
        assert_eq!(clear.toolbar_fill.a(), 255);
        assert_eq!(clear.toolbar_stroke.a(), 255);
        assert!((clear.frost - 0.8).abs() < f32::EPSILON);
    }
}
