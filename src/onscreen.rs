//! On-screen annotate overlay.
//!
//! Fullscreen capture → dimmed desktop → drag-select with live `W x H` →
//! docked flat toolbar (position from Settings) draws on the region in place.
//! Save / Copy / Enter / Exit match the preview window. A preview button hands
//! the session off to the HyperShot window (top-embedded toolbar).
//!
//! Best-effort fullscreen egui overlay after a full-screen capture. Some
//! desktops may tile or refuse always-on-top.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2, ViewportBuilder};

use crate::annotate::{
    clamp_to_image, compose_selection_png, draft_region_corners, draft_shape_for_tool,
    paint_resolution_label, paint_shape, shape_is_meaningful, AnnotateOutcome, Shape, Tool,
    HIGHLIGHTER_DEFAULT_ALPHA, MAX_STROKE_WIDTH, MIN_STROKE_WIDTH, STROKE_WIDTH, THICKNESS_HUD_SECS,
};
use crate::config::{ToolbarPosition, APP_NAME};
use crate::output::{
    configured_save_file_type, default_capture_filename, ensure_save_extension, save_rgba_image,
};
use crate::settings::{show_settings_window, EmbeddedSettings};
use crate::ui::{
    self, fit_image_rect, glass_color_swatch, glass_icon_button, glass_toolbar_frame,
    paint_icon_arrow, paint_icon_box, paint_icon_clear, paint_icon_copy, paint_icon_exit,
    paint_icon_highlighter, paint_icon_line, paint_icon_magnify, paint_icon_pixelate,
    paint_icon_preview, paint_icon_redo, paint_icon_save, paint_icon_settings, paint_icon_step,
    paint_icon_text, paint_icon_undo, toolbar_separator, AnnotatePalette, DEFAULT_STROKE,
    PASTEL_PRIMARY, TOOLBAR_WIDTH, TOOL_BTN,
};

const HANDLE_R: f32 = 5.5;
const HANDLE_HIT: f32 = 10.0;
/// Border band (screen px) that affords move — not the interior (drawing).
const EDGE_BAND: f32 = 12.0;
const MIN_SEL: f32 = 8.0;

/// Fullscreen on-screen annotate over a full-display capture PNG.
///
/// On Save/Copy/Enter: writes the cropped+annotated region to `path` (or a
/// chosen Save path) and returns the matching [`AnnotateOutcome`].
///
/// When `preselect_full` is true (fullscreen capture mode), the selection
/// starts as the entire image; the user can still resize it.
#[allow(dead_code)]
pub fn run_onscreen_annotate(path: &Path) -> Result<AnnotateOutcome> {
    run_onscreen_annotate_with_options(path, false)
}

/// Same as [`run_onscreen_annotate`], optionally pre-selecting the full frame.
pub fn run_onscreen_annotate_with_options(
    path: &Path,
    preselect_full: bool,
) -> Result<AnnotateOutcome> {
    ensure_display_available()?;

    let rgba = image::open(path)
        .with_context(|| format!("open capture for on-screen annotate {}", path.display()))?
        .to_rgba8();
    let (img_w, img_h) = rgba.dimensions();
    if img_w == 0 || img_h == 0 {
        bail!("capture has empty dimensions: {}", path.display());
    }

    let settings = EmbeddedSettings::load();
    let theme = settings.config().theme;
    let rgba_bytes = rgba.as_raw().clone();
    let path_buf = path.to_path_buf();
    let outcome = Arc::new(Mutex::new(AnnotateOutcome::Cancelled));
    let outcome_for_app = Arc::clone(&outcome);

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_fullscreen(true)
            .with_decorations(false)
            .with_transparent(false)
            .with_always_on_top(),
        centered: true,
        ..Default::default()
    };

    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            APP_NAME,
            options,
            Box::new(move |cc| {
                ui::apply_annotate_visuals(&cc.egui_ctx, theme);
                Ok(Box::new(OnscreenApp::new(
                    &cc.egui_ctx,
                    rgba_bytes,
                    img_w,
                    img_h,
                    path_buf,
                    outcome_for_app,
                    settings,
                    preselect_full,
                )))
            }),
        )
    }));

    match run_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => bail!("on-screen annotate failed (need a display?): {e}"),
        Err(_) => bail!("on-screen annotate panicked (missing display libs?)"),
    }

    let result = outcome
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    Ok(result)
}

fn ensure_display_available() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
        let x11 = std::env::var_os("DISPLAY").is_some();
        if !wayland && !x11 {
            bail!("no WAYLAND_DISPLAY or DISPLAY set (on-screen annotate needs a graphical session)");
        }
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Handle {
    Nw,
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
    Move,
}

struct OnscreenApp {
    texture: egui::TextureHandle,
    img_w: u32,
    img_h: u32,
    pixels: Vec<u8>,
    /// Image-space selection (inclusive min, exclusive-ish max via width/height).
    sel_a: Option<Pos2>,
    sel_b: Option<Pos2>,
    selecting: bool,
    resize_handle: Option<Handle>,
    shapes: Vec<Shape>,
    redo_stack: Vec<Shape>,
    tool: Tool,
    stroke_color: Color32,
    stroke_width: f32,
    next_step_number: u32,
    drag_start: Option<Pos2>,
    draft: Option<Shape>,
    text_edit: Option<OnscreenTextEdit>,
    path: PathBuf,
    outcome: Arc<Mutex<AnnotateOutcome>>,
    finished: bool,
    thickness_hud_until: Option<Instant>,
    show_settings: bool,
    settings: EmbeddedSettings,
}

#[derive(Debug, Clone)]
struct OnscreenTextEdit {
    pos: Pos2,
    buffer: String,
}

impl OnscreenApp {
    fn new(
        ctx: &egui::Context,
        pixels: Vec<u8>,
        img_w: u32,
        img_h: u32,
        path: PathBuf,
        outcome: Arc<Mutex<AnnotateOutcome>>,
        settings: EmbeddedSettings,
        preselect_full: bool,
    ) -> Self {
        let color = egui::ColorImage::from_rgba_unmultiplied([img_w as usize, img_h as usize], &pixels);
        let texture = ctx.load_texture("hypershot_onscreen", color, egui::TextureOptions::LINEAR);
        let (sel_a, sel_b) = if preselect_full {
            (
                Some(Pos2::new(0.0, 0.0)),
                Some(Pos2::new(
                    (img_w.saturating_sub(1)) as f32,
                    (img_h.saturating_sub(1)) as f32,
                )),
            )
        } else {
            (None, None)
        };
        Self {
            texture,
            img_w,
            img_h,
            pixels,
            sel_a,
            sel_b,
            selecting: false,
            resize_handle: None,
            shapes: Vec::new(),
            redo_stack: Vec::new(),
            tool: Tool::Arrow,
            stroke_color: DEFAULT_STROKE,
            stroke_width: STROKE_WIDTH,
            next_step_number: 1,
            drag_start: None,
            draft: None,
            text_edit: None,
            path,
            outcome,
            finished: false,
            thickness_hud_until: None,
            show_settings: false,
            settings,
        }
    }

    fn adjust_stroke_width(&mut self, delta: f32) {
        let before = self.stroke_width;
        self.stroke_width = (self.stroke_width + delta).clamp(MIN_STROKE_WIDTH, MAX_STROKE_WIDTH);
        if (self.stroke_width - before).abs() > 0.01 {
            self.thickness_hud_until =
                Some(Instant::now() + Duration::from_secs_f32(THICKNESS_HUD_SECS));
        }
    }

    fn text_size_for_width(&self) -> f32 {
        (self.stroke_width * 6.0).clamp(10.0, 72.0)
    }

    fn highlighter_color(&self) -> Color32 {
        let c = self.stroke_color;
        if c.a() < 255 {
            c
        } else {
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), HIGHLIGHTER_DEFAULT_ALPHA)
        }
    }

    fn commit_text_edit(&mut self) {
        let Some(edit) = self.text_edit.take() else {
            return;
        };
        let text = edit.buffer.trim().to_string();
        if text.is_empty() {
            return;
        }
        self.push_shape(Shape::Text {
            pos: edit.pos,
            text,
            color: self.stroke_color,
            size: self.text_size_for_width(),
        });
    }

    fn selection_rect(&self) -> Option<Rect> {
        let a = self.sel_a?;
        let b = self.sel_b?;
        let r = Rect::from_two_pos(a, b);
        if r.width() < MIN_SEL || r.height() < MIN_SEL {
            None
        } else {
            Some(r)
        }
    }

    fn set_outcome(&mut self, outcome: AnnotateOutcome) {
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = outcome;
        }
        self.finished = true;
    }

    fn sync_step_counter(&mut self) {
        self.next_step_number = self
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Step { number, .. } => Some(*number),
                _ => None,
            })
            .max()
            .map(|m| m.saturating_add(1))
            .unwrap_or(1);
    }

    fn push_shape(&mut self, shape: Shape) {
        self.shapes.push(shape);
        self.redo_stack.clear();
        self.sync_step_counter();
    }

    fn undo(&mut self) {
        if let Some(s) = self.shapes.pop() {
            self.redo_stack.push(s);
            self.sync_step_counter();
        }
    }

    fn redo(&mut self) {
        if let Some(s) = self.redo_stack.pop() {
            self.shapes.push(s);
            self.sync_step_counter();
        }
    }

    fn clear_all(&mut self) {
        self.shapes.clear();
        self.redo_stack.clear();
        self.draft = None;
        self.drag_start = None;
        self.text_edit = None;
        self.next_step_number = 1;
    }

    fn write_selection_png(&mut self, dest: &Path) -> Result<()> {
        let sel = self
            .selection_rect()
            .context("no region selected")?;
        let cropped = compose_selection_png(
            &self.pixels,
            self.img_w,
            self.img_h,
            &self.shapes,
            sel,
        )?;
        save_rgba_image(&cropped, dest)
    }

    fn pick_save_path(&self) -> Option<PathBuf> {
        let default_dir = self
            .settings
            .config()
            .effective_save_dir()
            .to_string_lossy()
            .into_owned();
        let default_name = default_capture_filename();
        let file_type = configured_save_file_type();
        let (filter_name, filter_exts) = file_type.dialog_filter();
        rfd::FileDialog::new()
            .set_title("Save screenshot")
            .set_directory(&default_dir)
            .set_file_name(&default_name)
            .add_filter(filter_name, filter_exts)
            .add_filter("PNG", &["png"])
            .add_filter("JPEG", &["jpg", "jpeg"])
            .add_filter("WebP", &["webp"])
            .save_file()
            .map(|path| ensure_save_extension(path, file_type))
    }

    fn save_and_close(&mut self) -> Result<()> {
        let sel = self
            .selection_rect()
            .context("select a region before saving")?;
        let _ = sel;
        let Some(dest) = self.pick_save_path() else {
            return Ok(());
        };
        self.write_selection_png(&dest)?;
        self.set_outcome(AnnotateOutcome::Saved(dest));
        Ok(())
    }

    fn copy_and_close(&mut self) -> Result<()> {
        let dest = self.path.clone();
        self.write_selection_png(&dest)?;
        self.set_outcome(AnnotateOutcome::Copied(dest));
        Ok(())
    }

    fn enter_save_and_copy(&mut self) -> Result<()> {
        let Some(dest) = self.pick_save_path() else {
            return Ok(());
        };
        self.write_selection_png(&dest)?;
        self.path = dest.clone();
        self.set_outcome(AnnotateOutcome::Copied(dest));
        Ok(())
    }

    fn open_in_preview(&mut self) -> Result<()> {
        let sel = self
            .selection_rect()
            .context("select a region before opening preview")?;
        let cropped = compose_selection_png(
            &self.pixels,
            self.img_w,
            self.img_h,
            &self.shapes,
            sel,
        )?;
        // Write into the capture path so the floating session continues the same file.
        cropped
            .save(&self.path)
            .with_context(|| format!("write preview handoff {}", self.path.display()))?;
        let marker = preview_handoff_marker(&self.path);
        std::fs::write(&marker, b"1").with_context(|| {
            format!("write preview handoff marker {}", marker.display())
        })?;
        // Saved path tells main to continue; marker requests floating preview.
        self.set_outcome(AnnotateOutcome::Saved(self.path.clone()));
        Ok(())
    }

    fn cancel_draft_or_session(&mut self) {
        if self.draft.is_some() || self.drag_start.is_some() {
            self.draft = None;
            self.drag_start = None;
        } else if self.show_settings {
            self.show_settings = false;
        } else {
            self.set_outcome(AnnotateOutcome::Cancelled);
        }
    }

    fn draw_top_chrome(&self, painter: &egui::Painter, sel_screen: Rect, pal: &AnnotatePalette) {
        let (a, b) = match (self.sel_a, self.sel_b) {
            (Some(a), Some(b)) => (a, b),
            _ => return,
        };
        let w = (a.x - b.x).abs().round().max(1.0) as u32;
        let h = (a.y - b.y).abs().round().max(1.0) as u32;
        let x = a.x.min(b.x).round() as i32;
        let y = a.y.min(b.y).round() as i32;

        let pill_pos = Pos2::new(sel_screen.left(), sel_screen.top() - 36.0);
        paint_size_coord_pill(painter, pill_pos, w, h, x, y, pal);
    }

    fn draw_handles(
        &self,
        painter: &egui::Painter,
        sel_screen: Rect,
        hover: Option<Handle>,
        edge_hover: bool,
    ) {
        // Slightly brighter border when the edge affords move.
        let border = if edge_hover {
            Color32::from_rgb(0xF0, 0xEC, 0xFF)
        } else {
            Color32::WHITE
        };
        painter.rect_stroke(
            sel_screen,
            0.0,
            Stroke::new(if edge_hover { 2.0_f32 } else { 1.5_f32 }, border),
            egui::StrokeKind::Outside,
        );

        let pts = handle_labeled_points(sel_screen);
        for (h, p) in pts {
            let hovered = hover == Some(h);
            let r = if hovered { HANDLE_R + 2.0 } else { HANDLE_R };
            let fill = if hovered {
                Color32::from_rgb(0xED, 0xE7, 0xF6)
            } else {
                Color32::WHITE
            };
            let stroke = if hovered {
                Stroke::new(2.0_f32, PASTEL_PRIMARY)
            } else {
                Stroke::new(1.35_f32, Color32::from_rgb(0x55, 0x55, 0x5C))
            };
            // Soft outer ring so the hit target reads as interactive before hover.
            if hovered {
                painter.circle_filled(p, r + 3.0, Color32::from_rgba_unmultiplied(0x8E, 0x74, 0xC2, 0x40));
            }
            painter.circle_filled(p, r, fill);
            painter.circle_stroke(p, r, stroke);
        }
    }

    /// Resize handles first, then edge-band move. Interior returns `None` (draw tools).
    fn hit_selection(&self, screen_pos: Pos2, sel_screen: Rect) -> Option<Handle> {
        for (h, p) in handle_labeled_points(sel_screen) {
            if p.distance(screen_pos) <= HANDLE_HIT {
                return Some(h);
            }
        }
        if near_selection_edge(screen_pos, sel_screen, EDGE_BAND) {
            Some(Handle::Move)
        } else {
            None
        }
    }

    fn apply_handle_drag(&mut self, handle: Handle, img_pos: Pos2) {
        let (Some(a), Some(b)) = (self.sel_a, self.sel_b) else {
            return;
        };
        let mut min = Pos2::new(a.x.min(b.x), a.y.min(b.y));
        let mut max = Pos2::new(a.x.max(b.x), a.y.max(b.y));
        match handle {
            Handle::Nw => {
                min = img_pos;
            }
            Handle::N => {
                min.y = img_pos.y;
            }
            Handle::Ne => {
                max.x = img_pos.x;
                min.y = img_pos.y;
            }
            Handle::E => {
                max.x = img_pos.x;
            }
            Handle::Se => {
                max = img_pos;
            }
            Handle::S => {
                max.y = img_pos.y;
            }
            Handle::Sw => {
                min.x = img_pos.x;
                max.y = img_pos.y;
            }
            Handle::W => {
                min.x = img_pos.x;
            }
            Handle::Move => {
                // Move is handled separately with a delta.
                return;
            }
        }
        min = clamp_to_image(min, self.img_w, self.img_h);
        max = clamp_to_image(max, self.img_w, self.img_h);
        if (max.x - min.x).abs() >= MIN_SEL && (max.y - min.y).abs() >= MIN_SEL {
            self.sel_a = Some(min);
            self.sel_b = Some(max);
        }
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui, horizontal: bool) {
        // Center tools in the toolbar chrome (main + cross axis).
        let available = ui.available_size();
        let layout = if horizontal {
            egui::Layout::left_to_right(egui::Align::Center).with_main_align(egui::Align::Center)
        } else {
            egui::Layout::top_down(egui::Align::Center).with_main_align(egui::Align::Center)
        };
        ui.allocate_ui_with_layout(available, layout, |ui| {
            if horizontal {
                ui.spacing_mut().item_spacing.x = 4.0;
            } else {
                ui.spacing_mut().item_spacing.y = 4.0;
            }
            if glass_icon_button(ui, self.tool == Tool::Arrow, "Arrow", None, TOOL_BTN, paint_icon_arrow)
            {
                self.tool = Tool::Arrow;
            }
            if glass_icon_button(
                ui,
                self.tool == Tool::Highlighter,
                "Highlighter",
                None,
                TOOL_BTN,
                paint_icon_highlighter,
            ) {
                self.tool = Tool::Highlighter;
            }
            if glass_icon_button(ui, self.tool == Tool::Line, "Line", None, TOOL_BTN, paint_icon_line)
            {
                self.tool = Tool::Line;
            }
            if glass_icon_button(ui, self.tool == Tool::Rect, "Box", None, TOOL_BTN, paint_icon_box) {
                self.tool = Tool::Rect;
            }
            if glass_icon_button(
                ui,
                self.tool == Tool::Pixelate,
                "Pixelate",
                None,
                TOOL_BTN,
                paint_icon_pixelate,
            ) {
                self.tool = Tool::Pixelate;
            }
            if glass_icon_button(
                ui,
                self.tool == Tool::Step,
                "Step number",
                None,
                TOOL_BTN,
                paint_icon_step,
            ) {
                self.tool = Tool::Step;
            }
            if glass_icon_button(ui, self.tool == Tool::Text, "Text", None, TOOL_BTN, paint_icon_text)
            {
                self.tool = Tool::Text;
            }
            if glass_icon_button(
                ui,
                self.tool == Tool::Magnify,
                "Magnify",
                None,
                TOOL_BTN,
                paint_icon_magnify,
            ) {
                self.tool = Tool::Magnify;
            }
            glass_color_swatch(ui, &mut self.stroke_color);
            // Thickness readout (Ctrl+scroll) — number + unit on one line.
            {
                let size = if horizontal {
                    egui::vec2(44.0, TOOL_BTN)
                } else {
                    egui::vec2(TOOL_BTN, 36.0)
                };
                let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
                let label = format!("{:.0} px", self.stroke_width);
                let _ = response.on_hover_text(format!("Thickness {label} (Ctrl+scroll)"));
                let painter = ui.painter_at(rect);
                // Overlay toolbar is light pastel; use near-black for contrast.
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::monospace(11.0),
                    crate::ui::THICKNESS_INK_ON_LIGHT,
                );
            }

            toolbar_separator(ui, horizontal);

            if glass_icon_button(ui, false, "Undo (Ctrl+Z)", None, TOOL_BTN, paint_icon_undo) {
                self.undo();
            }
            if glass_icon_button(ui, false, "Redo (Ctrl+Shift+Z)", None, TOOL_BTN, paint_icon_redo)
            {
                self.redo();
            }
            if glass_icon_button(ui, false, "Clear all marks", None, TOOL_BTN, paint_icon_clear) {
                self.clear_all();
            }

            toolbar_separator(ui, horizontal);

            if glass_icon_button(ui, false, "Settings", None, TOOL_BTN, paint_icon_settings) {
                self.show_settings = true;
            }
            if glass_icon_button(
                ui,
                false,
                "Open in preview window",
                None,
                TOOL_BTN,
                paint_icon_preview,
            ) {
                if let Err(err) = self.open_in_preview() {
                    eprintln!("open preview failed: {err:#}");
                }
            }
            if glass_icon_button(
                ui,
                false,
                "Save (file dialog), then close",
                None,
                TOOL_BTN,
                paint_icon_save,
            ) {
                if let Err(err) = self.save_and_close() {
                    eprintln!("save failed: {err:#}");
                }
            }
            if glass_icon_button(
                ui,
                false,
                "Copy to clipboard, then close",
                None,
                TOOL_BTN,
                paint_icon_copy,
            ) {
                if let Err(err) = self.copy_and_close() {
                    eprintln!("copy failed: {err:#}");
                }
            }

            toolbar_separator(ui, horizontal);

            if glass_icon_button(ui, false, "Exit without saving", None, TOOL_BTN, paint_icon_exit)
            {
                self.cancel_draft_or_session();
            }
        });
    }
}

impl eframe::App for OnscreenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.finished {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        ui::apply_annotate_visuals(ctx, self.settings.config().theme);
        let dark = ctx.style().visuals.dark_mode;
        let pal = AnnotatePalette::for_mode(dark, false);

        if self.show_settings {
            show_settings_window(ctx, &mut self.show_settings, &mut self.settings);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(0x12, 0x12, 0x14)))
            .show(ctx, |ui| {
                let area = ui.max_rect();
                let (image_rect, scale) =
                    fit_image_rect(area, self.img_w as f32, self.img_h as f32);
                let response =
                    ui.interact(area, ui.id().with("onscreen_canvas"), Sense::click_and_drag());
                let painter = ui.painter_at(area);

                painter.image(
                    self.texture.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );

                let origin = image_rect.min;
                let img_w = self.img_w;
                let img_h = self.img_h;
                let to_screen = |p: Pos2| Pos2::new(origin.x + p.x * scale, origin.y + p.y * scale);
                let to_image = |p: Pos2| {
                    clamp_to_image(
                        Pos2::new((p.x - origin.x) / scale, (p.y - origin.y) / scale),
                        img_w,
                        img_h,
                    )
                };

                let dim = Color32::from_rgba_unmultiplied(0x10, 0x10, 0x12, 0xB8);
                let sel_img = self.selection_rect();
                let sel_screen = sel_img.map(|r| {
                    Rect::from_two_pos(to_screen(r.min), to_screen(r.max)).intersect(image_rect)
                });

                if let Some(sel) = sel_screen {
                    paint_dim_around(&painter, image_rect, sel, dim);
                } else {
                    painter.rect_filled(image_rect, 0.0, dim);
                    painter.text(
                        area.center(),
                        egui::Align2::CENTER_CENTER,
                        "Drag to select a region  ·  Esc to cancel",
                        egui::FontId::proportional(16.0),
                        Color32::from_rgba_unmultiplied(0xF7, 0xF5, 0xFA, 0xE0),
                    );
                }

                // Marks (full-image coordinates).
                for shape in self.shapes.iter().chain(self.draft.iter()) {
                    paint_shape(
                        &painter,
                        shape,
                        to_screen,
                        scale,
                        &self.pixels,
                        self.img_w,
                        self.img_h,
                    );
                }
                if let Some(edit) = &self.text_edit {
                    paint_shape(
                        &painter,
                        &Shape::Text {
                            pos: edit.pos,
                            text: if edit.buffer.is_empty() {
                                "Text".into()
                            } else {
                                edit.buffer.clone()
                            },
                            color: self.stroke_color,
                            size: self.text_size_for_width(),
                        },
                        to_screen,
                        scale,
                        &self.pixels,
                        self.img_w,
                        self.img_h,
                    );
                }
                if let Some(draft) = self.draft.clone() {
                    if let Some((a, b)) = draft_region_corners(&draft) {
                        let w = (a.x - b.x).abs().round() as u32;
                        let h = (a.y - b.y).abs().round() as u32;
                        if w > 0 && h > 0 {
                            let label_pos = Pos2::new(
                                to_screen(a).x.max(to_screen(b).x) + 8.0,
                                to_screen(a).y.min(to_screen(b).y),
                            );
                            paint_resolution_label(&painter, label_pos, w, h, &pal);
                        }
                    }
                }

                // Hover affordances: resize cursors on handles, move on edge band.
                let hover_pos = ui.ctx().pointer_hover_pos().or(pointer_from_response(&response));
                let hover_handle = sel_screen.and_then(|sel| {
                    hover_pos.and_then(|pos| self.hit_selection(pos, sel))
                });
                let edge_hover = matches!(hover_handle, Some(Handle::Move));
                if let Some(h) = hover_handle {
                    ui.ctx().set_cursor_icon(cursor_for_handle(h));
                }

                if let Some(sel) = sel_screen {
                    self.draw_handles(&painter, sel, hover_handle.filter(|h| !matches!(h, Handle::Move)), edge_hover);
                    self.draw_top_chrome(&painter, sel, &pal);
                }

                // Interaction: resize / edge-move / new select / draw inside selection.
                let pointer = response.interact_pointer_pos();
                if response.drag_started() {
                    if let Some(pos) = pointer {
                        if let Some(sel) = sel_screen {
                            if let Some(h) = self.hit_selection(pos, sel) {
                                // Edge or handle: move / resize the region.
                                self.resize_handle = Some(h);
                                self.drag_start = Some(to_image(pos));
                                self.draft = None;
                            } else if sel.contains(pos) {
                                // Interior: draw with the active tool.
                                self.drag_start = Some(to_image(pos));
                                self.resize_handle = None;
                            } else if image_rect.contains(pos) {
                                // Outside: start a new selection.
                                let img = to_image(pos);
                                self.sel_a = Some(img);
                                self.sel_b = Some(img);
                                self.selecting = true;
                                self.shapes.clear();
                                self.redo_stack.clear();
                                self.text_edit = None;
                                self.next_step_number = 1;
                            }
                        } else if image_rect.contains(pos) {
                            let img = to_image(pos);
                            self.sel_a = Some(img);
                            self.sel_b = Some(img);
                            self.selecting = true;
                        }
                    }
                }

                if response.dragged() {
                    if let Some(pos) = pointer {
                        let img = to_image(pos);
                        if self.selecting {
                            self.sel_b = Some(img);
                        } else if let Some(h) = self.resize_handle {
                            if h == Handle::Move {
                                if let (Some(start), Some(a), Some(b)) =
                                    (self.drag_start, self.sel_a, self.sel_b)
                                {
                                    let delta = img - start;
                                    let min = Pos2::new(a.x.min(b.x), a.y.min(b.y));
                                    let max = Pos2::new(a.x.max(b.x), a.y.max(b.y));
                                    let w = max.x - min.x;
                                    let hh = max.y - min.y;
                                    let mut nmin = min + delta;
                                    nmin.x = nmin.x.clamp(0.0, (self.img_w as f32) - w);
                                    nmin.y = nmin.y.clamp(0.0, (self.img_h as f32) - hh);
                                    let applied = nmin - min;
                                    self.sel_a = Some(nmin);
                                    self.sel_b = Some(nmin + Vec2::new(w, hh));
                                    if applied.length_sq() > 0.0 {
                                        translate_shapes(&mut self.shapes, applied);
                                        if let Some(d) = self.draft.as_mut() {
                                            translate_shape(d, applied);
                                        }
                                    }
                                    self.drag_start = Some(img);
                                }
                            } else {
                                self.apply_handle_drag(h, img);
                            }
                        } else if let Some(start) = self.drag_start {
                            if self.tool == Tool::Step || self.tool == Tool::Text {
                                // Click tools — ignore drag.
                            } else {
                                self.draft = draft_shape_for_tool(
                                    self.tool,
                                    start,
                                    img,
                                    self.stroke_color,
                                    self.stroke_width,
                                    self.highlighter_color(),
                                );
                            }
                        }
                    }
                }

                if response.drag_stopped() {
                    if self.selecting {
                        self.selecting = false;
                    } else if self.resize_handle.is_some() {
                        self.resize_handle = None;
                        self.drag_start = None;
                    } else if self.tool == Tool::Step || self.tool == Tool::Text {
                        // handled on click
                    } else if let Some(shape) = self.draft.take() {
                        if shape_is_meaningful(&shape) {
                            self.push_shape(shape);
                        }
                        self.drag_start = None;
                    } else {
                        self.drag_start = None;
                    }
                }

                if self.tool == Tool::Step && response.clicked() {
                    if let (Some(pos), Some(sel)) = (pointer, sel_img) {
                        let img = to_image(pos);
                        if sel.contains(img) {
                            let number = self.next_step_number;
                            self.push_shape(Shape::Step {
                                center: img,
                                number,
                                color: self.stroke_color,
                            });
                            self.next_step_number = number.saturating_add(1);
                        }
                    }
                }

                if self.tool == Tool::Text && response.clicked() {
                    if let (Some(pos), Some(sel)) = (pointer, sel_img) {
                        let img = to_image(pos);
                        if sel.contains(img) {
                            self.commit_text_edit();
                            self.text_edit = Some(OnscreenTextEdit {
                                pos: img,
                                buffer: String::new(),
                            });
                        }
                    }
                }

                // Docked toolbar around the selection (Settings → on-screen position).
                if let Some(sel) = sel_screen {
                    let dock = self.settings.config().toolbar_position;
                    let horizontal = dock.is_horizontal();
                    let bar_w = if horizontal { 620.0_f32 } else { TOOLBAR_WIDTH };
                    let bar_h = if horizontal { 42.0_f32 } else { 520.0_f32 };
                    let gap = 12.0_f32;
                    let bar_pos = match dock {
                        ToolbarPosition::Bottom => Pos2::new(
                            (sel.center().x - bar_w * 0.5)
                                .clamp(area.left() + 8.0, area.right() - bar_w - 8.0),
                            (sel.bottom() + gap).min(area.bottom() - bar_h - 8.0),
                        ),
                        ToolbarPosition::Top => Pos2::new(
                            (sel.center().x - bar_w * 0.5)
                                .clamp(area.left() + 8.0, area.right() - bar_w - 8.0),
                            (sel.top() - gap - bar_h).max(area.top() + 8.0),
                        ),
                        ToolbarPosition::Left => Pos2::new(
                            (sel.left() - gap - bar_w).max(area.left() + 8.0),
                            (sel.center().y - bar_h * 0.5)
                                .clamp(area.top() + 8.0, area.bottom() - bar_h - 8.0),
                        ),
                        ToolbarPosition::Right => Pos2::new(
                            (sel.right() + gap).min(area.right() - bar_w - 8.0),
                            (sel.center().y - bar_h * 0.5)
                                .clamp(area.top() + 8.0, area.bottom() - bar_h - 8.0),
                        ),
                    };
                    egui::Area::new(egui::Id::new("hypershot_onscreen_toolbar"))
                        .fixed_pos(bar_pos)
                        .order(egui::Order::Foreground)
                        .show(ui.ctx(), |ui| {
                            glass_toolbar_frame(&pal).show(ui, |ui| {
                                if horizontal {
                                    ui.set_min_width(bar_w - 16.0);
                                    ui.set_height(TOOLBAR_WIDTH - 16.0);
                                } else {
                                    ui.set_width(TOOLBAR_WIDTH - 16.0);
                                    ui.set_min_height(bar_h - 16.0);
                                }
                                self.draw_toolbar(ui, horizontal);
                            });
                        });
                }
            });

        // Text editor overlay
        if let Some(edit) = self.text_edit.as_ref() {
            if let Some(sel) = self.selection_rect() {
                // Position roughly; full mapping uses last frame geometry via screen rect.
                let _ = sel;
                let screen = ctx.screen_rect();
                let mut buffer = edit.buffer.clone();
                let font_px = self.text_size_for_width();
                let color = self.stroke_color;
                let mut commit = false;
                let mut cancel = false;
                egui::Area::new(egui::Id::new("hypershot_onscreen_text_edit"))
                    .fixed_pos(Pos2::new(screen.center().x - 80.0, screen.center().y))
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        ui.set_min_width(160.0);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut buffer)
                                .hint_text("Text")
                                .font(egui::FontId::proportional(font_px.clamp(12.0, 36.0)))
                                .text_color(color)
                                .frame(true),
                        );
                        resp.request_focus();
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            commit = true;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            cancel = true;
                        }
                    });
                if let Some(edit) = self.text_edit.as_mut() {
                    edit.buffer = buffer;
                }
                if cancel {
                    self.text_edit = None;
                } else if commit {
                    self.commit_text_edit();
                }
            }
        }

        if let Some(until) = self.thickness_hud_until {
            if Instant::now() >= until {
                self.thickness_hud_until = None;
            } else {
                let label = format!("{:.0} px", self.stroke_width);
                egui::Area::new(egui::Id::new("hypershot_onscreen_thickness_hud"))
                    .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 24.0))
                    .order(egui::Order::Foreground)
                    .interactable(false)
                    .show(ctx, |ui| {
                        egui::Frame::new()
                            .fill(Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0xF2))
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::symmetric(12, 7))
                            .show(ui, |ui| {
                                // Shrink-wrap to the label (no min_width) so the pill stays snug.
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&label)
                                            .color(crate::ui::THICKNESS_INK_ON_LIGHT)
                                            .size(13.0)
                                            .strong()
                                            .monospace(),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend)
                                    .selectable(false),
                                );
                            });
                    });
                ctx.request_repaint_after(Duration::from_millis(32));
            }
        }

        let (ctrl_scroll, esc_exit) = ctx.input(|i| {
            let scroll = if i.modifiers.command {
                i.raw_scroll_delta.y
            } else {
                0.0
            };
            (scroll, self.settings.config().esc_to_exit)
        });
        if ctrl_scroll.abs() > 0.0 {
            self.adjust_stroke_width(if ctrl_scroll > 0.0 { 1.0 } else { -1.0 });
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                if self.draft.is_some()
                    || self.drag_start.is_some()
                    || self.selecting
                    || self.text_edit.is_some()
                {
                    self.draft = None;
                    self.drag_start = None;
                    self.selecting = false;
                    self.resize_handle = None;
                    self.text_edit = None;
                } else if esc_exit {
                    self.set_outcome(AnnotateOutcome::Cancelled);
                }
            }
            if i.key_pressed(egui::Key::Enter) {
                if let Err(err) = self.enter_save_and_copy() {
                    eprintln!("enter save/copy failed: {err:#}");
                }
            }
            if i.modifiers.command && i.key_pressed(egui::Key::Z) {
                if i.modifiers.shift {
                    self.redo();
                } else {
                    self.undo();
                }
            }
        });

    }
}

/// Marker beside a PNG meaning “re-open floating preview after on-screen handoff”.
pub fn preview_handoff_marker(png: &Path) -> PathBuf {
    let mut p = png.to_path_buf();
    let name = png
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("capture.png");
    p.set_file_name(format!("{name}.preview-handoff"));
    p
}

pub fn take_preview_handoff(png: &Path) -> bool {
    let marker = preview_handoff_marker(png);
    if marker.is_file() {
        let _ = std::fs::remove_file(&marker);
        true
    } else {
        false
    }
}

fn handle_labeled_points(sel: Rect) -> [(Handle, Pos2); 8] {
    [
        (Handle::Nw, sel.left_top()),
        (Handle::N, Pos2::new(sel.center().x, sel.top())),
        (Handle::Ne, sel.right_top()),
        (Handle::E, Pos2::new(sel.right(), sel.center().y)),
        (Handle::Se, sel.right_bottom()),
        (Handle::S, Pos2::new(sel.center().x, sel.bottom())),
        (Handle::Sw, sel.left_bottom()),
        (Handle::W, Pos2::new(sel.left(), sel.center().y)),
    ]
}

fn near_selection_edge(pos: Pos2, sel: Rect, band: f32) -> bool {
    if !sel.expand(band * 0.35).contains(pos) {
        return false;
    }
    // Inside the outer band of the rect (including just outside the stroke).
    let inset = sel.shrink(band);
    if inset.width() <= 0.0 || inset.height() <= 0.0 {
        return sel.contains(pos);
    }
    !inset.contains(pos)
}

fn cursor_for_handle(h: Handle) -> egui::CursorIcon {
    match h {
        Handle::N | Handle::S => egui::CursorIcon::ResizeVertical,
        Handle::E | Handle::W => egui::CursorIcon::ResizeHorizontal,
        Handle::Nw | Handle::Se => egui::CursorIcon::ResizeNwSe,
        Handle::Ne | Handle::Sw => egui::CursorIcon::ResizeNeSw,
        Handle::Move => egui::CursorIcon::Move,
    }
}

fn pointer_from_response(response: &egui::Response) -> Option<Pos2> {
    response.hover_pos()
}

fn translate_shapes(shapes: &mut [Shape], delta: Vec2) {
    for s in shapes.iter_mut() {
        translate_shape(s, delta);
    }
}

fn translate_shape(shape: &mut Shape, delta: Vec2) {
    match shape {
        Shape::Arrow { from, to, .. }
        | Shape::Highlighter { from, to, .. }
        | Shape::Line { from, to, .. } => {
            *from += delta;
            *to += delta;
        }
        Shape::Rect { a, b, .. } | Shape::Pixelate { a, b } => {
            *a += delta;
            *b += delta;
        }
        Shape::Step { center, .. } | Shape::Magnify { center, .. } => {
            *center += delta;
        }
        Shape::Text { pos, .. } => {
            *pos += delta;
        }
    }
}

fn paint_dim_around(painter: &egui::Painter, outer: Rect, hole: Rect, dim: Color32) {
    let hole = hole.intersect(outer);
    if hole.width() <= 0.0 || hole.height() <= 0.0 {
        painter.rect_filled(outer, 0.0, dim);
        return;
    }
    painter.rect_filled(
        Rect::from_min_max(outer.min, Pos2::new(outer.max.x, hole.min.y)),
        0.0,
        dim,
    );
    painter.rect_filled(
        Rect::from_min_max(Pos2::new(outer.min.x, hole.max.y), outer.max),
        0.0,
        dim,
    );
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(outer.min.x, hole.min.y),
            Pos2::new(hole.min.x, hole.max.y),
        ),
        0.0,
        dim,
    );
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(hole.max.x, hole.min.y),
            Pos2::new(outer.max.x, hole.max.y),
        ),
        0.0,
        dim,
    );
}

fn paint_size_coord_pill(
    painter: &egui::Painter,
    pos: Pos2,
    w: u32,
    h: u32,
    x: i32,
    y: i32,
    pal: &AnnotatePalette,
) {
    let text = format!("{w} x {h}   {x}, {y}");
    let font = egui::FontId::proportional(12.0);
    let galley = painter.layout_no_wrap(text, font, crate::ui::ICON_DARK);
    let pad = Vec2::new(12.0, 6.0);
    let rect = Rect::from_min_size(pos, galley.size() + pad * 2.0 + Vec2::new(18.0, 0.0));
    painter.rect_filled(rect, 14.0, Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0xF5));
    painter.rect_stroke(
        rect,
        14.0,
        Stroke::new(1.0_f32, pal.toolbar_stroke),
        egui::StrokeKind::Inside,
    );
    // Tiny magnifier cue
    let c = Pos2::new(rect.left() + 14.0, rect.center().y);
    painter.circle_stroke(c, 5.0, Stroke::new(1.5_f32, crate::ui::ICON_DARK));
    painter.line_segment(
        [c + Vec2::splat(3.5), c + Vec2::splat(7.0)],
        Stroke::new(1.5_f32, crate::ui::ICON_DARK),
    );
    painter.galley(
        Pos2::new(rect.left() + 26.0, rect.min.y + pad.y),
        galley,
        Color32::PLACEHOLDER,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handoff_marker_sits_beside_png() {
        let p = PathBuf::from("/tmp/hypershot-test.png");
        let m = preview_handoff_marker(&p);
        assert!(m
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with(".preview-handoff"));
    }

    #[test]
    fn edge_band_detects_border_not_interior() {
        let sel = Rect::from_min_size(Pos2::new(100.0, 100.0), Vec2::new(200.0, 120.0));
        assert!(near_selection_edge(Pos2::new(100.0, 160.0), sel, 12.0));
        assert!(near_selection_edge(Pos2::new(200.0, 100.0), sel, 12.0));
        assert!(!near_selection_edge(Pos2::new(200.0, 160.0), sel, 12.0));
    }
}
