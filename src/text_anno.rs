//! In-place text annotation box + quiet style strip.
//!
//! Click places a dashed, resizable box; type inside it (Enter = newline).
//! Esc or click-outside commits. Style strip appears under the main toolbar
//! while the Text tool is active or a box is being edited. egui text only.

use eframe::egui::{
    self, Align, Color32, FontFamily, FontId, Frame, Pos2, Rect, Sense, Stroke, TextFormat, Vec2,
};
use eframe::egui::text::{LayoutJob, TextWrapping};

use crate::ui::{
    AnnotatePalette, BTN_HOVER, BTN_HOVER_DARK, BTN_SELECTED, BTN_SELECTED_DARK, ICON_DARK,
    ICON_LIGHT, MUTED, MUTED_DARK, PASTEL_PRIMARY, TOOL_WELL_RADIUS,
};

/// Hint shown in an empty text box.
pub(crate) const TEXT_HINT: &str = "Type here";

/// Inner padding inside the dashed text box (matches TextEdit margin).
pub(crate) const TEXT_BOX_PAD: Vec2 = Vec2::new(4.0, 2.0);

/// Default image-space box size on click.
pub(crate) const DEFAULT_TEXT_BOX_W: f32 = 180.0;
pub(crate) const DEFAULT_TEXT_BOX_H: f32 = 48.0;
pub(crate) const MIN_TEXT_BOX: f32 = 36.0;

/// Secondary strip height under the main toolbar.
pub(crate) const TEXT_STYLE_STRIP_H: f32 = 34.0;

const HANDLE_R: f32 = 4.5;
const HANDLE_HIT: f32 = 9.0;

/// Short font list for the style strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextFontKind {
    Sans,
    Serif,
    Mono,
}

impl TextFontKind {
    pub(crate) const ALL: [Self; 3] = [Self::Sans, Self::Serif, Self::Mono];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Sans => "Sans",
            Self::Serif => "Serif",
            Self::Mono => "Mono",
        }
    }

    pub(crate) fn family(self) -> FontFamily {
        match self {
            Self::Sans => FontFamily::Proportional,
            // egui falls back if a named face is missing.
            Self::Serif => FontFamily::Name("serif".into()),
            Self::Mono => FontFamily::Monospace,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextAlignKind {
    Left,
    Center,
    Right,
}

impl TextAlignKind {
    pub(crate) const ALL: [Self; 3] = [Self::Left, Self::Center, Self::Right];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Left => "L",
            Self::Center => "C",
            Self::Right => "R",
        }
    }

    pub(crate) fn egui_align(self) -> Align {
        match self {
            Self::Left => Align::LEFT,
            Self::Center => Align::Center,
            Self::Right => Align::RIGHT,
        }
    }
}

/// Persisted with each text annotation (undo / export).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TextStyle {
    pub font: TextFontKind,
    pub size: f32,
    pub align: TextAlignKind,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: TextFontKind::Sans,
            size: 18.0,
            align: TextAlignKind::Left,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

impl TextStyle {
    pub(crate) fn font_id(self, px: f32) -> FontId {
        let size = if self.bold {
            (px * 1.06).clamp(8.0, 96.0)
        } else {
            px.clamp(8.0, 96.0)
        };
        FontId::new(size, self.font.family())
    }
}

/// Corner / edge handles for the in-progress text box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextBoxHandle {
    Nw,
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
}

/// Place a default text box with its top-left near `click` (image coords).
pub(crate) fn default_text_box_at(click: Pos2, img_w: u32, img_h: u32) -> Rect {
    let max_x = (img_w.saturating_sub(1)) as f32;
    let max_y = (img_h.saturating_sub(1)) as f32;
    let w = DEFAULT_TEXT_BOX_W.min(max_x.max(MIN_TEXT_BOX));
    let h = DEFAULT_TEXT_BOX_H.min(max_y.max(MIN_TEXT_BOX));
    let mut min = click;
    min.x = min.x.clamp(0.0, (max_x - w).max(0.0));
    min.y = min.y.clamp(0.0, (max_y - h).max(0.0));
    Rect::from_min_size(min, Vec2::new(w, h))
}

pub(crate) fn clamp_text_rect(rect: Rect, img_w: u32, img_h: u32) -> Rect {
    let max_x = (img_w.saturating_sub(1)) as f32;
    let max_y = (img_h.saturating_sub(1)) as f32;
    let mut r = rect;
    if r.width() < MIN_TEXT_BOX {
        r.max.x = r.min.x + MIN_TEXT_BOX;
    }
    if r.height() < MIN_TEXT_BOX {
        r.max.y = r.min.y + MIN_TEXT_BOX;
    }
    let w = r.width().min(max_x.max(1.0));
    let h = r.height().min(max_y.max(1.0));
    let mut min = r.min;
    min.x = min.x.clamp(0.0, (max_x - w).max(0.0));
    min.y = min.y.clamp(0.0, (max_y - h).max(0.0));
    Rect::from_min_size(min, Vec2::new(w, h))
}

fn handle_points(rect: Rect) -> [(TextBoxHandle, Pos2); 8] {
    let c = rect.center();
    [
        (TextBoxHandle::Nw, rect.left_top()),
        (TextBoxHandle::N, Pos2::new(c.x, rect.top())),
        (TextBoxHandle::Ne, rect.right_top()),
        (TextBoxHandle::E, Pos2::new(rect.right(), c.y)),
        (TextBoxHandle::Se, rect.right_bottom()),
        (TextBoxHandle::S, Pos2::new(c.x, rect.bottom())),
        (TextBoxHandle::Sw, rect.left_bottom()),
        (TextBoxHandle::W, Pos2::new(rect.left(), c.y)),
    ]
}

pub(crate) fn hit_text_box_handle(screen_rect: Rect, pos: Pos2) -> Option<TextBoxHandle> {
    for (h, p) in handle_points(screen_rect) {
        if p.distance(pos) <= HANDLE_HIT {
            return Some(h);
        }
    }
    None
}

pub(crate) fn cursor_for_text_handle(h: TextBoxHandle) -> egui::CursorIcon {
    match h {
        TextBoxHandle::N | TextBoxHandle::S => egui::CursorIcon::ResizeVertical,
        TextBoxHandle::E | TextBoxHandle::W => egui::CursorIcon::ResizeHorizontal,
        TextBoxHandle::Nw | TextBoxHandle::Se => egui::CursorIcon::ResizeNwSe,
        TextBoxHandle::Ne | TextBoxHandle::Sw => egui::CursorIcon::ResizeNeSw,
    }
}

/// Resize `rect` (image coords) by dragging `handle` to `img_pos`.
pub(crate) fn apply_text_handle_drag(
    rect: Rect,
    handle: TextBoxHandle,
    img_pos: Pos2,
    img_w: u32,
    img_h: u32,
) -> Rect {
    let mut min = rect.min;
    let mut max = rect.max;
    match handle {
        TextBoxHandle::Nw => {
            min = img_pos;
        }
        TextBoxHandle::N => {
            min.y = img_pos.y;
        }
        TextBoxHandle::Ne => {
            max.x = img_pos.x;
            min.y = img_pos.y;
        }
        TextBoxHandle::E => {
            max.x = img_pos.x;
        }
        TextBoxHandle::Se => {
            max = img_pos;
        }
        TextBoxHandle::S => {
            max.y = img_pos.y;
        }
        TextBoxHandle::Sw => {
            min.x = img_pos.x;
            max.y = img_pos.y;
        }
        TextBoxHandle::W => {
            min.x = img_pos.x;
        }
    }
    if max.x < min.x {
        std::mem::swap(&mut min.x, &mut max.x);
    }
    if max.y < min.y {
        std::mem::swap(&mut min.y, &mut max.y);
    }
    clamp_text_rect(Rect::from_min_max(min, max), img_w, img_h)
}

/// Dashed highlight + corner/edge handles while editing.
pub(crate) fn paint_text_box_chrome(
    painter: &egui::Painter,
    screen_rect: Rect,
    hover: Option<TextBoxHandle>,
) {
    let stroke = Stroke::new(1.5_f32, PASTEL_PRIMARY);
    paint_dashed_rect(painter, screen_rect, stroke);
    // Soft fill so the placement reads clearly.
    painter.rect_filled(
        screen_rect,
        0.0,
        Color32::from_rgba_unmultiplied(0x8E, 0x74, 0xC2, 0x18),
    );

    for (h, p) in handle_points(screen_rect) {
        let hovered = hover == Some(h);
        let r = if hovered { HANDLE_R + 1.5 } else { HANDLE_R };
        let fill = if hovered {
            Color32::from_rgb(0xED, 0xE7, 0xF6)
        } else {
            Color32::WHITE
        };
        let ring = if hovered {
            Stroke::new(1.75_f32, PASTEL_PRIMARY)
        } else {
            Stroke::new(1.2_f32, Color32::from_rgb(0x6A, 0x68, 0x72))
        };
        painter.circle_filled(p, r, fill);
        painter.circle_stroke(p, r, ring);
    }
}

fn paint_dashed_rect(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    let dash = 6.0_f32;
    let gap = 4.0_f32;
    dash_segment(
        painter,
        rect.left_top(),
        rect.right_top(),
        dash,
        gap,
        stroke,
    );
    dash_segment(
        painter,
        rect.right_top(),
        rect.right_bottom(),
        dash,
        gap,
        stroke,
    );
    dash_segment(
        painter,
        rect.right_bottom(),
        rect.left_bottom(),
        dash,
        gap,
        stroke,
    );
    dash_segment(
        painter,
        rect.left_bottom(),
        rect.left_top(),
        dash,
        gap,
        stroke,
    );
}

fn dash_segment(
    painter: &egui::Painter,
    a: Pos2,
    b: Pos2,
    dash: f32,
    gap: f32,
    stroke: Stroke,
) {
    let delta = b - a;
    let len = delta.length();
    if len < 0.5 {
        return;
    }
    let dir = delta / len;
    let mut t = 0.0_f32;
    let mut draw = true;
    while t < len {
        let seg = if draw { dash } else { gap };
        let t1 = (t + seg).min(len);
        if draw {
            painter.line_segment([a + dir * t, a + dir * t1], stroke);
        }
        t = t1;
        draw = !draw;
    }
}

/// Quiet purple/gray secondary strip: Font, Size, Align, B/I/U.
pub(crate) fn draw_text_style_strip(ui: &mut egui::Ui, style: &mut TextStyle, pal: &AnnotatePalette) {
    let ink = if pal.dark { ICON_LIGHT } else { ICON_DARK };
    let muted = if pal.dark { MUTED_DARK } else { MUTED };
    let selected = if pal.dark { BTN_SELECTED_DARK } else { BTN_SELECTED };
    let hover = if pal.dark { BTN_HOVER_DARK } else { BTN_HOVER };

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.label(
            egui::RichText::new("Font")
                .size(11.0)
                .color(muted),
        );
        egui::ComboBox::from_id_salt("hypershot_text_font")
            .selected_text(style.font.label())
            .width(72.0)
            .show_ui(ui, |ui| {
                for font in TextFontKind::ALL {
                    ui.selectable_value(&mut style.font, font, font.label());
                }
            });

        ui.add_space(4.0);
        ui.label(egui::RichText::new("Size").size(11.0).color(muted));
        let mut size_i = style.size.round() as i32;
        let resp = ui.add(
            egui::DragValue::new(&mut size_i)
                .range(10..=72)
                .speed(0.4)
                .suffix(" px"),
        );
        if resp.changed() {
            style.size = size_i as f32;
        }

        ui.add_space(6.0);
        ui.label(egui::RichText::new("Align").size(11.0).color(muted));
        for align in TextAlignKind::ALL {
            let on = style.align == align;
            if quiet_toggle(ui, align.label(), on, ink, selected, hover) {
                style.align = align;
            }
        }

        ui.add_space(6.0);
        if quiet_toggle(ui, "B", style.bold, ink, selected, hover) {
            style.bold = !style.bold;
        }
        if quiet_toggle(ui, "I", style.italic, ink, selected, hover) {
            style.italic = !style.italic;
        }
        if quiet_toggle(ui, "U", style.underline, ink, selected, hover) {
            style.underline = !style.underline;
        }
    });
}

fn quiet_toggle(
    ui: &mut egui::Ui,
    label: &str,
    on: bool,
    ink: Color32,
    selected: Color32,
    hover: Color32,
) -> bool {
    let size = Vec2::new(22.0, 22.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fill = if on {
        selected
    } else if resp.hovered() {
        hover
    } else {
        Color32::TRANSPARENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, TOOL_WELL_RADIUS, fill);
    if on {
        painter.rect_stroke(
            rect,
            TOOL_WELL_RADIUS,
            Stroke::new(1.0_f32, PASTEL_PRIMARY),
            egui::StrokeKind::Inside,
        );
    }
    let mut job = LayoutJob::default();
    job.append(
        label,
        0.0,
        TextFormat {
            font_id: FontId::proportional(if label == "B" { 12.5 } else { 12.0 }),
            color: ink,
            italics: label == "I",
            underline: if label == "U" {
                Stroke::new(1.0_f32, ink)
            } else {
                Stroke::NONE
            },
            ..Default::default()
        },
    );
    let galley = painter.layout_job(job);
    painter.galley(rect.center() - galley.size() * 0.5, galley, Color32::PLACEHOLDER);
    resp.clicked()
}

/// Frame for the secondary text style strip.
pub(crate) fn text_style_strip_frame(pal: &AnnotatePalette) -> Frame {
    Frame::NONE
        .fill(pal.toolbar_fill)
        .inner_margin(egui::Margin::symmetric(10, 4))
}

/// Build a wrapped layout job for preview / paint.
///
/// egui’s `halign` positions each row relative to x=0:
/// - Left → row starts at 0
/// - Center → row centered on 0
/// - Right → row ends at 0
/// So the galley must be placed with [`text_galley_origin`], not the box’s
/// left edge, or Center/Right will clip away (Right looked empty).
pub(crate) fn text_layout_job(text: &str, color: Color32, style: TextStyle, wrap_width: f32) -> LayoutJob {
    let mut job = LayoutJob {
        halign: style.align.egui_align(),
        wrap: TextWrapping {
            max_width: wrap_width.max(8.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let fmt = TextFormat {
        font_id: style.font_id(style.size),
        color,
        italics: style.italic,
        underline: if style.underline {
            Stroke::new(1.25_f32, color)
        } else {
            Stroke::NONE
        },
        extra_letter_spacing: if style.bold { 0.35 } else { 0.0 },
        ..Default::default()
    };
    job.append(text, 0.0, fmt);
    job
}

/// Screen-space origin for a galley laid out with [`text_layout_job`].
pub(crate) fn text_galley_origin(inner: Rect, align: TextAlignKind) -> Pos2 {
    match align {
        TextAlignKind::Left => inner.left_top(),
        TextAlignKind::Center => Pos2::new(inner.center().x, inner.top()),
        TextAlignKind::Right => Pos2::new(inner.right(), inner.top()),
    }
}

/// Content rect inside the dashed wrapper.
pub(crate) fn text_box_inner(box_rect: Rect) -> Rect {
    let mut inner = box_rect.shrink2(TEXT_BOX_PAD);
    if inner.width() < 1.0 {
        inner = Rect::from_center_size(box_rect.center(), Vec2::new(1.0, inner.height().max(1.0)));
    }
    if inner.height() < 1.0 {
        inner = Rect::from_center_size(box_rect.center(), Vec2::new(inner.width().max(1.0), 1.0));
    }
    inner
}

/// Paint wrapped, L/C/R-aligned text clipped to `box_rect`.
pub(crate) fn paint_boxed_text(
    painter: &egui::Painter,
    box_rect: Rect,
    text: &str,
    color: Color32,
    style: TextStyle,
) {
    if text.is_empty() {
        return;
    }
    let inner = text_box_inner(box_rect);
    let job = text_layout_job(text, color, style, inner.width());
    let galley = painter.layout_job(job);
    let origin = text_galley_origin(inner, style.align);
    let clip = painter.clip_rect().intersect(box_rect);
    if clip.width() <= 0.0 || clip.height() <= 0.0 {
        return;
    }
    painter.with_clip_rect(clip).galley(origin, galley, Color32::PLACEHOLDER);
}

/// Left-aligned layout job for TextEdit (cursor metrics); visuals use [`paint_boxed_text`].
pub(crate) fn text_edit_layouter_job(
    text: &str,
    color: Color32,
    style: TextStyle,
    wrap_width: f32,
    font_px: f32,
) -> LayoutJob {
    let mut edit_style = style;
    edit_style.align = TextAlignKind::Left;
    let mut job = text_layout_job(text, color, edit_style, wrap_width);
    for section in &mut job.sections {
        section.format.font_id = style.font_id(font_px);
    }
    job
}

/// Per-line x origin inside a box (image/export path).
pub(crate) fn aligned_line_x(box_min_x: f32, box_width: f32, line_width: f32, align: TextAlignKind) -> f32 {
    match align {
        TextAlignKind::Left => box_min_x,
        TextAlignKind::Center => box_min_x + ((box_width - line_width) * 0.5),
        TextAlignKind::Right => box_min_x + (box_width - line_width),
    }
}

/// Word-wrap `text` to roughly `max_width` image pixels for export.
pub(crate) fn wrap_text_for_export(text: &str, max_width: f32, size: f32, bold: bool) -> Vec<String> {
    // Approximate advance: ~0.52em for Ubuntu Light; bold a bit wider.
    let em = size.max(8.0) * if bold { 0.56 } else { 0.52 };
    let max_chars = ((max_width / em).floor() as usize).max(1);
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut rest = paragraph;
        while !rest.is_empty() {
            if rest.chars().count() <= max_chars {
                lines.push(rest.to_string());
                break;
            }
            let mut end = rest
                .char_indices()
                .nth(max_chars)
                .map(|(i, _)| i)
                .unwrap_or(rest.len());
            if let Some(space) = rest[..end].rfind(char::is_whitespace) {
                if space > 0 {
                    end = space;
                }
            }
            let (chunk, next) = rest.split_at(end);
            lines.push(chunk.trim_end().to_string());
            rest = next.trim_start();
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_box_clamps_to_image() {
        let r = default_text_box_at(Pos2::new(500.0, 500.0), 100, 80);
        assert!(r.min.x >= 0.0);
        assert!(r.min.y >= 0.0);
        assert!(r.max.x <= 99.0 + 0.1);
        assert!(r.max.y <= 79.0 + 0.1);
        assert!(r.width() >= MIN_TEXT_BOX - 0.1);
    }

    #[test]
    fn wrap_breaks_long_lines() {
        let lines = wrap_text_for_export("hello world from hypershot", 40.0, 18.0, false);
        assert!(lines.len() >= 2, "expected wrap, got {lines:?}");
    }

    #[test]
    fn wrap_keeps_explicit_newlines() {
        let lines = wrap_text_for_export("a\nb\nc", 400.0, 18.0, false);
        assert_eq!(lines, vec!["a", "b", "c"]);
    }

    #[test]
    fn galley_origin_follows_box_edges() {
        let inner = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(100.0, 40.0));
        assert_eq!(
            text_galley_origin(inner, TextAlignKind::Left),
            Pos2::new(10.0, 20.0)
        );
        assert_eq!(
            text_galley_origin(inner, TextAlignKind::Center),
            Pos2::new(60.0, 20.0)
        );
        assert_eq!(
            text_galley_origin(inner, TextAlignKind::Right),
            Pos2::new(110.0, 20.0)
        );
    }

    #[test]
    fn aligned_line_x_within_box() {
        assert!((aligned_line_x(0.0, 100.0, 40.0, TextAlignKind::Left) - 0.0).abs() < 0.01);
        assert!((aligned_line_x(0.0, 100.0, 40.0, TextAlignKind::Center) - 30.0).abs() < 0.01);
        assert!((aligned_line_x(0.0, 100.0, 40.0, TextAlignKind::Right) - 60.0).abs() < 0.01);
    }
}
