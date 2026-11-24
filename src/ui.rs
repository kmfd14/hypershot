//! Shared quiet charcoal visuals for annotate + settings (ENERGY 1 / RHYTHM 1 / MOTION 1).
//!
//! Reasons (R-31): charcoal chrome so the capture stays focal; no purple/cyan/glow;
//! small radius, not pills; flat surfaces (no soft floating shadows).

use eframe::egui::{self, Color32, Stroke};

/// Neutral quiet chrome shared by overlay and settings windows.
pub fn apply_quiet_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    // Creative-tool dark (R-21): charcoal ladder, attention on content not chrome.
    // Kill egui dark defaults that still leak into Settings: cyan selection/cursor,
    // blue hyperlink, soft window shadow, and window/menu radius 6 (pill-adjacent).
    visuals.panel_fill = Color32::from_rgb(0x22, 0x22, 0x24);
    visuals.window_fill = Color32::from_rgb(0x22, 0x22, 0x24);
    visuals.extreme_bg_color = Color32::from_rgb(0x1A, 0x1A, 0x1C);
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(0x22, 0x22, 0x24);
    visuals.widgets.noninteractive.weak_bg_fill = Color32::from_rgb(0x22, 0x22, 0x24);
    visuals.widgets.noninteractive.bg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(0x3A, 0x3A, 0x3C));
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x2C, 0x2C, 0x2E);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x2C, 0x2C, 0x2E);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x38, 0x38, 0x3A);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x38, 0x38, 0x3A);
    visuals.widgets.active.bg_fill = Color32::from_rgb(0x44, 0x44, 0x46);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(0x44, 0x44, 0x46);
    visuals.widgets.open.bg_fill = Color32::from_rgb(0x2C, 0x2C, 0x2E);
    visuals.widgets.open.weak_bg_fill = Color32::from_rgb(0x2C, 0x2C, 0x2E);
    visuals.selection.bg_fill = Color32::from_rgb(0x3A, 0x3A, 0x3C);
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(0xD0, 0xD0, 0xD0));
    visuals.text_cursor.stroke = Stroke::new(2.0_f32, Color32::from_rgb(0xD0, 0xD0, 0xD0));
    visuals.hyperlink_color = Color32::from_rgb(0xB0, 0xB0, 0xB0);
    visuals.widgets.inactive.fg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(0xD0, 0xD0, 0xD0));
    visuals.widgets.hovered.fg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(0xF0, 0xF0, 0xF0));
    visuals.widgets.active.fg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(0xF0, 0xF0, 0xF0));
    visuals.widgets.open.fg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(0xD0, 0xD0, 0xD0));
    visuals.widgets.noninteractive.fg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(0xB0, 0xB0, 0xB0));
    visuals.widgets.noninteractive.corner_radius = 3.0.into();
    visuals.widgets.inactive.corner_radius = 3.0.into();
    visuals.widgets.hovered.corner_radius = 3.0.into();
    visuals.widgets.active.corner_radius = 3.0.into();
    visuals.widgets.open.corner_radius = 3.0.into();
    // Settings Window / menus: match widget radius (R-11), not egui's default 6.
    visuals.window_corner_radius = 3.0.into();
    visuals.menu_corner_radius = 3.0.into();
    visuals.window_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0x3A, 0x3A, 0x3C));
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.popup_shadow = egui::Shadow::NONE;
    ctx.set_visuals(visuals);
}

/// Muted helper text (#A0 on #222 ≈ 5.7:1 WCAG AA).
pub const MUTED: Color32 = Color32::from_rgb(0xA0, 0xA0, 0xA0);

/// Primary action fill (slightly lifted charcoal, not a neon accent).
pub const ACTION_FILL: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3C);
