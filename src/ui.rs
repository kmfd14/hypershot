//! Shared visuals for annotate + settings.
//!
//! Annotate: flat pastel chrome (restrained purple / gray / white). No liquid-glass
//! imitation — no specular rims or glow. When Window Transparency / Background Blur
//! are on, we apply real window opacity plus a translucent frosted panel on the
//! window/preview area (closest effect egui can do). The tool rail stays fully
//! opaque. True compositor backdrop-blur is unavailable on Wayland/egui.
//!
//! Design dials: ENERGY 2 / RHYTHM 2 / MOTION 1 (hover fill only).
//! Palette note: restrained pastel purple/gray/white is an owner override (R-37).

use eframe::egui::{
    self, Color32, CornerRadius, Id, Margin, Pos2, Rect, Sense, Shadow, Stroke, Theme,
    ThemePreference, Vec2,
};
use eframe::egui::color_picker::{color_picker_color32, Alpha};
use eframe::egui::popup::{popup_below_widget, PopupCloseBehavior};

use crate::config::UiTheme;

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

/// Rail corner radius (moderate; not pill-everything).
pub const TOOLBAR_RADIUS: f32 = 14.0;

/// Universal settings control row height (buttons, text fields, checkbox wells).
pub const SETTINGS_CONTROL_HEIGHT: f32 = 28.0;

/// Checkbox mark size inside the universal control height.
pub const SETTINGS_CHECKBOX_SIZE: f32 = 18.0;

/// Default markup stroke (red). Accent for marks lives on the image.
pub const DEFAULT_STROKE: Color32 = Color32::from_rgb(0xE2, 0x4A, 0x3B);

/// Soft contact shadow under tool wells only (kept light for a sharper chrome).
pub const BTN_SHADOW: Color32 = Color32::from_black_alpha(20);

// --- Annotate light palette ---------------------------------------------------

pub const ANNOTATE_BG: Color32 = Color32::from_rgb(0xF3, 0xF1, 0xF6);
pub const TOOLBAR_FILL: Color32 = Color32::from_rgb(0xE8, 0xE4, 0xEE);
pub const TOOLBAR_STROKE: Color32 = Color32::from_rgb(0xD0, 0xCA, 0xD8);
pub const BTN_SELECTED: Color32 = Color32::from_rgb(0xDD, 0xD4, 0xEC);
pub const BTN_IDLE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const BTN_HOVER: Color32 = Color32::from_rgb(0xF7, 0xF4, 0xFB);

pub const ICON_DARK: Color32 = Color32::from_rgb(0x3A, 0x36, 0x45);
pub const ICON_LIGHT: Color32 = Color32::from_rgb(0xF7, 0xF5, 0xFA);

/// Muted purple Save primary (restrained, not neon).
pub const PASTEL_PRIMARY: Color32 = Color32::from_rgb(0x8B, 0x7C, 0xB6);
pub const PASTEL_PRIMARY_HOVER: Color32 = Color32::from_rgb(0x9A, 0x8C, 0xC4);

/// Secondary action (Copy): muted gray-violet, flat.
pub const ACTION_FILL: Color32 = Color32::from_rgb(0x6E, 0x68, 0x7A);
pub const ACTION_FILL_HOVER: Color32 = Color32::from_rgb(0x7E, 0x78, 0x8A);

pub const MUTED: Color32 = Color32::from_rgb(0x8A, 0x84, 0x94);
pub const PREVIEW_MATTE: Color32 = Color32::from_rgb(0xE0, 0xDC, 0xE8);

// --- Annotate dark palette ----------------------------------------------------

pub const ANNOTATE_BG_DARK: Color32 = Color32::from_rgb(0x1C, 0x1B, 0x20);
pub const TOOLBAR_FILL_DARK: Color32 = Color32::from_rgb(0x2A, 0x28, 0x30);
pub const TOOLBAR_STROKE_DARK: Color32 = Color32::from_rgb(0x48, 0x44, 0x52);
pub const BTN_SELECTED_DARK: Color32 = Color32::from_rgb(0x4A, 0x42, 0x5C);
pub const BTN_IDLE_DARK: Color32 = Color32::from_rgb(0x36, 0x34, 0x3E);
pub const BTN_HOVER_DARK: Color32 = Color32::from_rgb(0x42, 0x3E, 0x4C);
pub const MUTED_DARK: Color32 = Color32::from_rgb(0xA8, 0xA2, 0xB0);
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
    // Tighter feathering keeps vector icons crisp without jagged stair-steps.
    ctx.tessellation_options_mut(|opts| {
        opts.feathering = true;
        opts.feathering_size_in_pixels = 0.55;
    });
}

/// Install light + dark settings styles and select theme preference.
pub fn apply_settings_visuals(ctx: &egui::Context, theme: UiTheme) {
    ctx.set_visuals_of(Theme::Light, settings_visuals(false));
    ctx.set_visuals_of(Theme::Dark, settings_visuals(true));
    ctx.set_theme(theme_preference(theme));
    ctx.tessellation_options_mut(|opts| {
        opts.feathering = true;
        opts.feathering_size_in_pixels = 0.55;
    });
}

/// Frame for the floating pastel tool rail (content-sized, not full window).
///
/// Always uses a solid opaque fill — window transparency / blur never wash it out.
pub fn toolbar_frame(pal: &AnnotatePalette) -> egui::Frame {
    let shadow_a = if pal.dark { 40 } else { 22 };
    egui::Frame::new()
        .fill(Color32::from_rgb(
            pal.toolbar_fill.r(),
            pal.toolbar_fill.g(),
            pal.toolbar_fill.b(),
        ))
        .stroke(Stroke::new(
            1.0_f32,
            Color32::from_rgb(
                pal.toolbar_stroke.r(),
                pal.toolbar_stroke.g(),
                pal.toolbar_stroke.b(),
            ),
        ))
        .corner_radius(CornerRadius::same(TOOLBAR_RADIUS as u8))
        .inner_margin(Margin::symmetric(6, 12))
        .outer_margin(Margin::ZERO)
        .shadow(Shadow {
            offset: [0, 2],
            blur: 6,
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
        .inner_margin(Margin::symmetric(14, 12))
        .shadow(Shadow {
            offset: [0, 1],
            blur: 4,
            spread: 0,
            color: Color32::from_black_alpha(if dark { 48 } else { 18 }),
        })
}

fn paint_soft_circle(painter: &egui::Painter, center: Pos2, radius: f32, fill: Color32) {
    painter.circle_filled(center + Vec2::new(0.0, 0.75), radius, BTN_SHADOW);
    painter.circle_filled(center, radius, fill);
    // Crisp rim (no wide soft glass stroke).
    painter.circle_stroke(
        center,
        radius - 0.5,
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(0x3A, 0x36, 0x45, 0x38)),
    );
}

/// Circular pastel tool button with painted icon and auto-contrast ink.
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
    let fill = if let Some((idle, hover)) = fill_override {
        if hovered {
            hover
        } else {
            idle
        }
    } else if selected {
        pal.btn_selected
    } else if hovered {
        pal.btn_hover
    } else {
        pal.btn_idle
    };

    let radius = diameter * 0.46;
    let painter = ui.painter_at(rect);
    paint_soft_circle(&painter, rect.center(), radius, fill);

    if selected && fill_override.is_none() {
        painter.circle_stroke(
            rect.center(),
            radius - 0.2,
            Stroke::new(2.0_f32, Color32::from_rgba_premultiplied(0x8B, 0x7C, 0xB6, 0x90)),
        );
    }

    let icon = icon_color_for_fill(fill);
    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(diameter * 0.50));
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
    let pal = AnnotatePalette::for_mode(dark, false);
    let fill = if hovered { pal.btn_hover } else { pal.btn_idle };
    let radius = TOOL_BTN * 0.46;
    let painter = ui.painter_at(rect);
    paint_soft_circle(&painter, rect.center(), radius, fill);
    painter.circle_filled(rect.center(), radius * 0.62, *color);
    painter.circle_stroke(
        rect.center(),
        radius * 0.62,
        Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0x3A, 0x36, 0x45, 0x40)),
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

/// Card section title with a `?` help control (tooltip = section help).
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
        let help_btn = egui::Button::new(
            egui::RichText::new("?")
                .color(secondary)
                .size(11.0)
                .strong(),
        )
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0_f32, secondary))
        .corner_radius(CornerRadius::same(9))
        .min_size(Vec2::splat(18.0));
        ui.add(help_btn).on_hover_text(help);
    });
    ui.add_space(6.0);
}

// --- Icon painters (vector strokes; keep AA feathering low in apply_*_visuals) ---

const ICON_STROKE: f32 = 2.35;

/// `direction`: +1 = undo (opens left), -1 = redo (opens right).
fn paint_arc_arrow(painter: &egui::Painter, rect: Rect, color: Color32, direction: f32) {
    let stroke = Stroke::new(ICON_STROKE + 0.15, color);
    let c = rect.center() + Vec2::new(-direction * rect.width() * 0.06, rect.height() * 0.04);
    let r = rect.width() * 0.32;
    let start = if direction > 0.0 {
        std::f32::consts::FRAC_PI_4 * 0.35
    } else {
        std::f32::consts::PI - std::f32::consts::FRAC_PI_4 * 0.35
    };
    let sweep = direction * std::f32::consts::PI * 1.45;
    let steps = 32;
    let mut prev = Pos2::new(c.x + r * start.cos(), c.y + r * start.sin());
    for i in 1..=steps {
        let t = start - sweep * (i as f32 / steps as f32);
        let p = Pos2::new(c.x + r * t.cos(), c.y + r * t.sin());
        painter.line_segment([prev, p], stroke);
        prev = p;
    }
    let tip_angle = start - sweep;
    let tip = Pos2::new(c.x + r * tip_angle.cos(), c.y + r * tip_angle.sin());
    // Unit tangent at tip, oriented toward the arc travel end.
    let travel = -sweep.signum();
    let tangent = Vec2::new(-travel * tip_angle.sin(), travel * tip_angle.cos()).normalized();
    let normal = Vec2::new(-tangent.y, tangent.x);
    let head = rect.width() * 0.22;
    let wing = rect.width() * 0.16;
    let base = tip + tangent * head;
    let left = tip - tangent * (head * 0.15) + normal * wing;
    let right = tip - tangent * (head * 0.15) - normal * wing;
    painter.add(egui::Shape::convex_polygon(
        vec![tip, left, base, right],
        color,
        Stroke::NONE,
    ));
}

pub fn paint_icon_arrow(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(ICON_STROKE, color);
    let a = Pos2::new(rect.left() + rect.width() * 0.18, rect.bottom() - rect.height() * 0.22);
    let b = Pos2::new(rect.right() - rect.width() * 0.16, rect.top() + rect.height() * 0.20);
    painter.line_segment([a, b], stroke);
    let dir = (b - a).normalized();
    let perp = Vec2::new(-dir.y, dir.x);
    let head = rect.width() * 0.30;
    let wing = rect.width() * 0.17;
    let back = b - dir * head;
    // Filled head reads sharper than two thin strokes at 36px.
    painter.add(egui::Shape::convex_polygon(
        vec![b, back + perp * wing, back - perp * wing],
        color,
        Stroke::NONE,
    ));
}

pub fn paint_icon_box(painter: &egui::Painter, rect: Rect, color: Color32) {
    let inset = rect.shrink(rect.width() * 0.20);
    painter.rect_stroke(
        inset,
        2.5,
        Stroke::new(ICON_STROKE, color),
        egui::StrokeKind::Middle,
    );
}

pub fn paint_icon_pixelate(painter: &egui::Painter, rect: Rect, color: Color32) {
    let inset = rect.shrink(rect.width() * 0.18);
    let cell = inset.width() / 3.0;
    let gap = cell * 0.12;
    for row in 0..3 {
        for col in 0..3 {
            if (row + col) % 2 == 0 {
                let c = Rect::from_min_size(
                    Pos2::new(
                        inset.min.x + col as f32 * cell + gap * 0.5,
                        inset.min.y + row as f32 * cell + gap * 0.5,
                    ),
                    Vec2::splat(cell - gap),
                );
                painter.rect_filled(c, 1.0, color);
            }
        }
    }
}

/// Numbered step stamp icon: high-contrast circle with a “1”.
pub fn paint_icon_step(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let r = rect.width() * 0.36;
    painter.circle_filled(c, r, color);
    painter.circle_stroke(
        c,
        r,
        Stroke::new(ICON_STROKE * 0.85, Color32::from_rgba_unmultiplied(255, 255, 255, 220)),
    );
    // Simple “1” glyph (vertical stem + top serif + base).
    let ink = Color32::WHITE;
    let stem_x = c.x + r * 0.05;
    let top = c.y - r * 0.42;
    let bot = c.y + r * 0.42;
    let stroke = Stroke::new(ICON_STROKE + 0.2, ink);
    painter.line_segment(
        [Pos2::new(stem_x - r * 0.28, top + r * 0.18), Pos2::new(stem_x, top)],
        stroke,
    );
    painter.line_segment([Pos2::new(stem_x, top), Pos2::new(stem_x, bot)], stroke);
    painter.line_segment(
        [
            Pos2::new(stem_x - r * 0.28, bot),
            Pos2::new(stem_x + r * 0.28, bot),
        ],
        stroke,
    );
}

/// Curved undo arrow with a filled arrowhead (readable at small sizes).
pub fn paint_icon_undo(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_arc_arrow(painter, rect, color, 1.0);
}

/// Curved redo arrow (mirror of undo).
pub fn paint_icon_redo(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_arc_arrow(painter, rect, color, -1.0);
}

pub fn paint_icon_clear(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(ICON_STROKE + 0.15, color);
    let inset = rect.shrink(rect.width() * 0.26);
    painter.line_segment([inset.left_top(), inset.right_bottom()], stroke);
    painter.line_segment([inset.right_top(), inset.left_bottom()], stroke);
}

pub fn paint_icon_settings(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let outer = rect.width() * 0.26;
    let hub = rect.width() * 0.11;
    let tooth_len = rect.width() * 0.16;
    let tooth_w = rect.width() * 0.11;
    // Hub ring + filled center.
    painter.circle_stroke(c, outer, Stroke::new(ICON_STROKE, color));
    painter.circle_filled(c, hub, color);
    // Six trapezoid teeth (sharper than stubby spokes).
    for i in 0..6 {
        let a = (i as f32) * std::f32::consts::TAU / 6.0;
        let dir = Vec2::angled(a);
        let perp = Vec2::new(-dir.y, dir.x);
        let base = c + dir * (outer * 0.72);
        let tip = c + dir * (outer + tooth_len);
        let half = tooth_w * 0.5;
        painter.add(egui::Shape::convex_polygon(
            vec![
                base + perp * half,
                tip + perp * (half * 0.55),
                tip - perp * (half * 0.55),
                base - perp * half,
            ],
            color,
            Stroke::NONE,
        ));
    }
}

pub fn paint_icon_save(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(ICON_STROKE, color);
    let inset = rect.shrink(rect.width() * 0.20);
    // Tray
    let tray_y = inset.bottom() - inset.height() * 0.08;
    painter.line_segment(
        [
            Pos2::new(inset.left(), tray_y - inset.height() * 0.18),
            Pos2::new(inset.left(), tray_y),
        ],
        stroke,
    );
    painter.line_segment(
        [Pos2::new(inset.left(), tray_y), Pos2::new(inset.right(), tray_y)],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(inset.right(), tray_y),
            Pos2::new(inset.right(), tray_y - inset.height() * 0.18),
        ],
        stroke,
    );
    // Down arrow
    let tip = Pos2::new(inset.center().x, tray_y - inset.height() * 0.12);
    let top = Pos2::new(inset.center().x, inset.top() + inset.height() * 0.05);
    painter.line_segment([top, tip], stroke);
    let wing = inset.width() * 0.22;
    painter.add(egui::Shape::convex_polygon(
        vec![
            tip,
            tip + Vec2::new(-wing, -wing * 0.95),
            tip + Vec2::new(wing, -wing * 0.95),
        ],
        color,
        Stroke::NONE,
    ));
}

pub fn paint_icon_copy(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(ICON_STROKE, color);
    let back = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.16, rect.top() + rect.height() * 0.16),
        Pos2::new(rect.right() - rect.width() * 0.34, rect.bottom() - rect.height() * 0.34),
    );
    let front = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.34, rect.top() + rect.height() * 0.34),
        Pos2::new(rect.right() - rect.width() * 0.16, rect.bottom() - rect.height() * 0.16),
    );
    // Fill front sheet so the glyph reads as two pages, not two empty frames.
    painter.rect_filled(front, 2.0, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 28));
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
    fn flat_pastel_keeps_lilac_not_pure_white() {
        assert_eq!(TOOLBAR_FILL, Color32::from_rgb(0xE8, 0xE4, 0xEE));
        assert!(BTN_SELECTED.b() > BTN_SELECTED.g());
        assert_eq!(BTN_IDLE, Color32::WHITE);
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
