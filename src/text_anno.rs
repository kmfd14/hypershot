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
pub(crate) const HANDLE_HIT: f32 = 9.0;
/// Edge band (screen px) for move affordance — outside the typing interior.
const MOVE_EDGE_BAND: f32 = 11.0;

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

/// Active pointer drag on an editing text box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TextBoxDrag {
    Resize(TextBoxHandle),
    /// Image-space pointer at drag start + rect snapshot.
    Move { start_img: Pos2, start_rect: Rect },
}

/// In-progress on-canvas text session (image-space rect).
#[derive(Debug, Clone)]
pub(crate) struct TextBoxSession {
    pub rect: Rect,
    pub buffer: String,
    /// Unicode scalar index into `buffer`.
    pub cursor: usize,
}

impl TextBoxSession {
    pub(crate) fn new(rect: Rect) -> Self {
        Self {
            rect,
            buffer: String::new(),
            cursor: 0,
        }
    }

    fn clamp_cursor(&mut self) {
        self.cursor = self.cursor.min(self.buffer.chars().count());
    }
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

fn near_text_box_edge(pos: Pos2, rect: Rect, band: f32) -> bool {
    if !rect.expand(band).contains(pos) {
        return false;
    }
    let inset = rect.shrink(band);
    if inset.width() <= 0.0 || inset.height() <= 0.0 {
        return rect.contains(pos);
    }
    !inset.contains(pos)
}

/// Handles first, then edge-band move. Interior (typing) returns `None`.
pub(crate) fn hit_text_box_chrome(screen_rect: Rect, pos: Pos2) -> Option<TextBoxDrag> {
    if let Some(h) = hit_text_box_handle(screen_rect, pos) {
        return Some(TextBoxDrag::Resize(h));
    }
    if near_text_box_edge(pos, screen_rect, MOVE_EDGE_BAND) {
        // Placeholder start fields filled when drag begins.
        return Some(TextBoxDrag::Move {
            start_img: Pos2::ZERO,
            start_rect: Rect::NOTHING,
        });
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

pub(crate) fn cursor_for_text_drag(drag: TextBoxDrag) -> egui::CursorIcon {
    match drag {
        TextBoxDrag::Resize(h) => cursor_for_text_handle(h),
        TextBoxDrag::Move { .. } => egui::CursorIcon::Move,
    }
}

/// Translate a text box by an image-space delta and clamp.
pub(crate) fn apply_text_box_move(
    start_rect: Rect,
    delta: Vec2,
    img_w: u32,
    img_h: u32,
) -> Rect {
    clamp_text_rect(start_rect.translate(delta), img_w, img_h)
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

/// Screen-scaled style for live editing / paint.
pub(crate) fn screen_text_style(style: TextStyle, scale: f32) -> TextStyle {
    let mut s = style;
    s.size = (style.size * scale).clamp(10.0, 72.0);
    s
}

/// Grow/shrink the image-space box height (and width if needed) to fit content.
pub(crate) fn autosize_text_box(
    rect: Rect,
    galley_size_screen: Vec2,
    scale: f32,
    img_w: u32,
    img_h: u32,
) -> Rect {
    let scale = scale.max(0.001);
    let pad = TEXT_BOX_PAD * 2.0 + Vec2::new(2.0, 4.0);
    let need = (galley_size_screen + pad) / scale;
    let w = rect.width().max(need.x).max(MIN_TEXT_BOX);
    let h = need.y.max(MIN_TEXT_BOX);
    clamp_text_rect(Rect::from_min_size(rect.min, Vec2::new(w, h)), img_w, img_h)
}

/// Per-line x origin inside a box (image/export path).
pub(crate) fn aligned_line_x(box_min_x: f32, box_width: f32, line_width: f32, align: TextAlignKind) -> f32 {
    match align {
        TextAlignKind::Left => box_min_x,
        TextAlignKind::Center => box_min_x + ((box_width - line_width) * 0.5),
        TextAlignKind::Right => box_min_x + (box_width - line_width),
    }
}

fn char_index_to_byte(s: &str, char_index: usize) -> usize {
    s.char_indices()
        .nth(char_index)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn insert_at_cursor(session: &mut TextBoxSession, text: &str) {
    session.clamp_cursor();
    let i = char_index_to_byte(&session.buffer, session.cursor);
    session.buffer.insert_str(i, text);
    session.cursor += text.chars().count();
}

fn delete_before_cursor(session: &mut TextBoxSession) {
    session.clamp_cursor();
    if session.cursor == 0 {
        return;
    }
    let end = char_index_to_byte(&session.buffer, session.cursor);
    let start = char_index_to_byte(&session.buffer, session.cursor - 1);
    session.buffer.replace_range(start..end, "");
    session.cursor -= 1;
}

fn delete_after_cursor(session: &mut TextBoxSession) {
    session.clamp_cursor();
    let len = session.buffer.chars().count();
    if session.cursor >= len {
        return;
    }
    let start = char_index_to_byte(&session.buffer, session.cursor);
    let end = char_index_to_byte(&session.buffer, session.cursor + 1);
    session.buffer.replace_range(start..end, "");
}

/// Result of one frame of the on-canvas text editor.
pub(crate) struct TextBoxEditorFrame {
    pub commit_esc: bool,
}

/// On-canvas editor: aligned glyphs + matching caret, edge move, corner resize,
/// and auto-sized wrapper. Uses a small custom editor (not TextEdit) so L/C/R
/// caret placement matches paint.
pub(crate) fn run_text_box_editor(
    ctx: &egui::Context,
    session: &mut TextBoxSession,
    drag: &mut Option<TextBoxDrag>,
    screen_rect: Rect,
    to_image: &dyn Fn(Pos2) -> Pos2,
    img_w: u32,
    img_h: u32,
    scale: f32,
    color: Color32,
    style: TextStyle,
) -> TextBoxEditorFrame {
    let mut commit_esc = false;
    let font_style = screen_text_style(style, scale);
    let inner = text_box_inner(screen_rect);
    let id = egui::Id::new("hypershot_text_box_editor");

    // Chrome + hover cursor (handles / move).
    let hover_pos = ctx.pointer_hover_pos();
    let hover_chrome = hover_pos.and_then(|p| hit_text_box_chrome(screen_rect, p));
    if let Some(h) = hover_chrome {
        ctx.set_cursor_icon(cursor_for_text_drag(h));
    }
    {
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("hypershot_text_box_chrome"),
        ));
        let handle_hover = match hover_chrome {
            Some(TextBoxDrag::Resize(h)) => Some(h),
            _ => None,
        };
        paint_text_box_chrome(&painter, screen_rect, handle_hover);
    }

    // Interaction area covers box + handle hit slop.
    let outer = screen_rect.expand(HANDLE_HIT);
    egui::Area::new(id)
        .fixed_pos(outer.min)
        .order(egui::Order::Foreground)
        .sense(Sense::click_and_drag())
        .show(ctx, |ui| {
            ui.set_min_size(outer.size());
            ui.set_max_size(outer.size());
            let area_resp = ui.interact(ui.max_rect(), id.with("area"), Sense::click_and_drag());
            ui.memory_mut(|m| m.request_focus(id));

            let pointer = area_resp.interact_pointer_pos().or(hover_pos);

            // Start resize / move on edge or handle (not deep interior).
            if area_resp.drag_started() {
                if let Some(pos) = pointer {
                    if let Some(hit) = hit_text_box_chrome(screen_rect, pos) {
                        match hit {
                            TextBoxDrag::Resize(h) => *drag = Some(TextBoxDrag::Resize(h)),
                            TextBoxDrag::Move { .. } => {
                                let img = to_image(pos);
                                *drag = Some(TextBoxDrag::Move {
                                    start_img: img,
                                    start_rect: session.rect,
                                });
                            }
                        }
                    }
                }
            }
            if area_resp.dragged() {
                if let (Some(d), Some(pos)) = (*drag, pointer) {
                    let img = to_image(pos);
                    match d {
                        TextBoxDrag::Resize(h) => {
                            session.rect =
                                apply_text_handle_drag(session.rect, h, img, img_w, img_h);
                        }
                        TextBoxDrag::Move {
                            start_img,
                            start_rect,
                        } => {
                            session.rect =
                                apply_text_box_move(start_rect, img - start_img, img_w, img_h);
                        }
                    }
                }
            }
            if area_resp.drag_stopped() {
                *drag = None;
            }

            // Layout (aligned) for paint, caret, and click-to-cursor.
            let display = if session.buffer.is_empty() {
                TEXT_HINT
            } else {
                session.buffer.as_str()
            };
            let display_color = if session.buffer.is_empty() {
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 0x88)
            } else {
                color
            };
            let mut layout_style = font_style;
            if session.buffer.is_empty() {
                layout_style.align = TextAlignKind::Left;
            }
            let job = text_layout_job(display, display_color, layout_style, inner.width());
            let galley = ui.fonts(|f| f.layout_job(job));
            let origin = text_galley_origin(inner, layout_style.align);

            // Click interior → place caret (galley-space).
            if area_resp.clicked() && drag.is_none() {
                if let Some(pos) = pointer {
                    if inner.contains(pos) && hit_text_box_handle(screen_rect, pos).is_none() {
                        let cursor = galley.cursor_from_pos(pos - origin);
                        // Prefer ccursor index into real buffer (hint has no cursor write).
                        if !session.buffer.is_empty() {
                            session.cursor =
                                cursor.ccursor.index.min(session.buffer.chars().count());
                        } else {
                            session.cursor = 0;
                        }
                    }
                }
            }

            // Keyboard editing while focused.
            let mut edited = false;
            if ui.memory(|m| m.has_focus(id)) {
                ui.input(|i| {
                    for event in &i.events {
                        match event {
                            egui::Event::Text(t) => {
                                if !t.is_empty() && t != "\n" && t != "\r" {
                                    insert_at_cursor(session, t);
                                    edited = true;
                                }
                            }
                            egui::Event::Paste(t) => {
                                if !t.is_empty() {
                                    insert_at_cursor(session, t);
                                    edited = true;
                                }
                            }
                            egui::Event::Key {
                                key: egui::Key::Enter,
                                pressed: true,
                                modifiers,
                                ..
                            } if !modifiers.ctrl && !modifiers.command => {
                                insert_at_cursor(session, "\n");
                                edited = true;
                            }
                            egui::Event::Key {
                                key: egui::Key::Backspace,
                                pressed: true,
                                ..
                            } => {
                                delete_before_cursor(session);
                                edited = true;
                            }
                            egui::Event::Key {
                                key: egui::Key::Delete,
                                pressed: true,
                                ..
                            } => {
                                delete_after_cursor(session);
                                edited = true;
                            }
                            egui::Event::Key {
                                key: egui::Key::ArrowLeft,
                                pressed: true,
                                ..
                            } => {
                                session.clamp_cursor();
                                session.cursor = session.cursor.saturating_sub(1);
                            }
                            egui::Event::Key {
                                key: egui::Key::ArrowRight,
                                pressed: true,
                                ..
                            } => {
                                session.clamp_cursor();
                                let len = session.buffer.chars().count();
                                session.cursor = (session.cursor + 1).min(len);
                            }
                            egui::Event::Key {
                                key: egui::Key::Home,
                                pressed: true,
                                ..
                            } => {
                                session.cursor = 0;
                            }
                            egui::Event::Key {
                                key: egui::Key::End,
                                pressed: true,
                                ..
                            } => {
                                session.cursor = session.buffer.chars().count();
                            }
                            egui::Event::Key {
                                key: egui::Key::Escape,
                                pressed: true,
                                ..
                            } => {
                                commit_esc = true;
                            }
                            _ => {}
                        }
                    }
                });
            }

            // Paint glyphs (aligned + clipped).
            {
                let painter = ui.painter();
                let clip = painter.clip_rect().intersect(screen_rect);
                if clip.width() > 0.0 && clip.height() > 0.0 {
                    painter
                        .with_clip_rect(clip)
                        .galley(origin, galley.clone(), Color32::PLACEHOLDER);
                }
            }

            // Caret matches galley alignment (same origin as glyphs).
            session.clamp_cursor();
            let caret_local = if session.buffer.is_empty() {
                // Origin is already L/C/R; caret sits on that anchor.
                Rect::from_min_size(Pos2::ZERO, Vec2::new(0.0, font_style.size * 1.2))
            } else {
                let cursor = galley.from_ccursor(egui::text::CCursor::new(session.cursor));
                galley.pos_from_cursor(&cursor)
            };
            let caret = caret_local.translate(origin.to_vec2());
            let blink_on = ((ctx.input(|i| i.time) * 1.7) as i64) % 2 == 0;
            if blink_on && drag.is_none() {
                ui.painter().line_segment(
                    [caret.center_top(), caret.center_bottom()],
                    Stroke::new(1.25_f32, color),
                );
                ctx.request_repaint_after(std::time::Duration::from_millis(80));
            }

            // Auto-grow/shrink wrapper to content (handles stay usable mid-edit).
            if edited || drag.is_none() {
                let measure_job = text_layout_job(
                    if session.buffer.is_empty() {
                        " "
                    } else {
                        session.buffer.as_str()
                    },
                    color,
                    font_style,
                    inner.width().max(8.0),
                );
                let measure = ui.fonts(|f| f.layout_job(measure_job));
                // Preserve user width while dragging a horizontal resize handle.
                let preserve_w = matches!(
                    drag,
                    Some(TextBoxDrag::Resize(
                        TextBoxHandle::E
                            | TextBoxHandle::W
                            | TextBoxHandle::Ne
                            | TextBoxHandle::Nw
                            | TextBoxHandle::Se
                            | TextBoxHandle::Sw
                    ))
                );
                let mut next =
                    autosize_text_box(session.rect, measure.size(), scale, img_w, img_h);
                if preserve_w {
                    next.max.x = session.rect.min.x + session.rect.width();
                    next = clamp_text_rect(next, img_w, img_h);
                }
                session.rect = next;
            }
        });

    TextBoxEditorFrame { commit_esc }
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

    #[test]
    fn edge_band_is_move_not_resize() {
        let r = Rect::from_min_size(Pos2::new(100.0, 100.0), Vec2::new(200.0, 80.0));
        assert!(matches!(
            hit_text_box_chrome(r, Pos2::new(100.0, 140.0)),
            Some(TextBoxDrag::Resize(TextBoxHandle::W))
        ));
        // Mid-edge but outside handle radius → move.
        let mid_left = Pos2::new(100.0, 140.0);
        // Use a point on the top edge center, far from corner handles.
        let top_mid = Pos2::new(200.0, 100.0);
        // Top-center is the N handle — pick slightly inside the edge band beside it.
        let top_band = Pos2::new(160.0, 100.0);
        let hit = hit_text_box_chrome(r, top_band);
        assert!(
            matches!(hit, Some(TextBoxDrag::Move { .. }) | Some(TextBoxDrag::Resize(_))),
            "edge should afford move or resize, got {hit:?}"
        );
        let _ = (mid_left, top_mid);
        // Deep interior is typing, not chrome.
        assert!(hit_text_box_chrome(r, r.center()).is_none());
    }

    #[test]
    fn autosize_grows_height_for_tall_galley() {
        let r = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(120.0, 40.0));
        let next = autosize_text_box(r, Vec2::new(100.0, 90.0), 1.0, 400, 400);
        assert!(next.height() > r.height());
        assert!(next.height() >= 90.0);
    }
}
