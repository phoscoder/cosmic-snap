//! The editor application: region selection phase and in-place annotation phase.
#![allow(unused_imports)]
use crate::raster::{contrast, render};
use crate::shapes::{arrow_segments, ellipse_points, Shape, Tool};
use crate::system::{png_bytes, save_path, wl_copy};
use crate::ui::{dark_frame, icon_btn, sep, Icon, ACCENT, PALETTE, PANEL};
use ab_glyph::FontArc;
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use image::RgbaImage;
use std::process::Command;

pub(crate) enum Phase {
    Select { start: Option<Pos2>, cur: Option<Pos2> },
    Edit,
}

pub(crate) struct App {
    full: RgbaImage,
    tex: egui::TextureHandle,
    base: RgbaImage,
    sel: Rect, // selection, in full-image pixels
    shapes: Vec<Shape>,
    redo: Vec<Shape>,
    draft: Option<Shape>,
    tool: Tool,
    color: Color32,
    width: f32,
    marker_n: u32,
    text_edit: Option<(Pos2, String, bool)>,
    backdrop: bool,
    font: Option<FontArc>,
    status: String,
    phase: Phase,
    color_open: bool,
    help: bool,
}

impl App {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>, full: RgbaImage, font_bytes: Option<Vec<u8>>, select: bool) -> Self {
        let mut font = None;
        if let Some(bytes) = font_bytes {
            if let Ok(f) = FontArc::try_from_vec(bytes.clone()) {
                font = Some(f);
                let mut defs = egui::FontDefinitions::default();
                defs.font_data.insert("snap".into(), egui::FontData::from_owned(bytes));
                defs.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "snap".into());
                cc.egui_ctx.set_fonts(defs);
            }
        }
        let (w, h) = (full.width(), full.height());
        let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], full.as_raw());
        let tex = cc.egui_ctx.load_texture("shot", ci, egui::TextureOptions::LINEAR);
        Self {
            base: full.clone(),
            full,
            tex,
            sel: Rect::from_min_size(Pos2::ZERO, Vec2::new(w as f32, h as f32)),
            shapes: vec![],
            redo: vec![],
            draft: None,
            tool: Tool::Arrow,
            color: PALETTE[0],
            width: 4.0,
            marker_n: 1,
            text_edit: None,
            backdrop: false,
            font,
            status: if select { "Drag to select an area · click for full screen".into() } else { "Area selected · pick a tool".into() },
            phase: if select { Phase::Select { start: None, cur: None } } else { Phase::Edit },
            color_open: false,
            help: true,
        }
    }

    /// Where the full frame sits on screen, and its screen-pixels-per-image-pixel scale.
    fn geometry(&self, ctx: &egui::Context) -> (Rect, f32) {
        let avail = ctx.screen_rect();
        let (iw, ih) = (self.full.width() as f32, self.full.height() as f32);
        let s = (avail.width() / iw).min(avail.height() / ih);
        (Rect::from_center_size(avail.center(), Vec2::new(iw * s, ih * s)), s)
    }

    fn select_ui(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            std::process::exit(0);
        }
        let (fit, s) = self.geometry(ctx);
        let (iw, ih) = (self.full.width() as f32, self.full.height() as f32);
        let mut finish: Option<Rect> = None;
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            finish = Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(iw, ih)));
        }
        egui::CentralPanel::default().frame(egui::Frame::none().fill(Color32::BLACK)).show(ctx, |ui| {
            let resp = ui.allocate_rect(fit, Sense::click_and_drag());
            ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
            let painter = ui.painter_at(fit);
            painter.image(self.tex.id(), fit, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
            let to_img = |p: Pos2| Pos2::new(((p.x - fit.min.x) / s).clamp(0.0, iw), ((p.y - fit.min.y) / s).clamp(0.0, ih));
            let dim = Color32::from_black_alpha(150);
            let hover = ctx.input(|i| i.pointer.hover_pos());

            let Phase::Select { start, cur } = &mut self.phase else { return };
            if resp.drag_started() {
                *start = resp.interact_pointer_pos();
            }
            if resp.dragged() {
                *cur = resp.interact_pointer_pos();
            }
            let sel = match (*start, *cur) {
                (Some(a), Some(b)) => Some(Rect::from_two_pos(a, b).intersect(fit)),
                _ => None,
            };
            if resp.drag_stopped() {
                if let Some(r) = sel {
                    let (a, b) = (to_img(r.min), to_img(r.max));
                    if (b.x - a.x) > 4.0 && (b.y - a.y) > 4.0 {
                        finish = Some(Rect::from_min_max(a, b));
                    }
                }
                *start = None;
                *cur = None;
                if finish.is_none() {
                    finish = Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(iw, ih)));
                }
            } else if resp.clicked() {
                finish = Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(iw, ih)));
            }

            let pill = |painter: &egui::Painter, at: Pos2, text: String| {
                let g = painter.layout_no_wrap(text, FontId::proportional(13.0), Color32::from_gray(235));
                let bg = Rect::from_min_size(at, g.size() + Vec2::new(16.0, 8.0));
                painter.rect_filled(bg, 7.0, Color32::from_black_alpha(215));
                painter.galley(bg.min + Vec2::new(8.0, 4.0), g, Color32::WHITE);
            };
            match sel {
                Some(r) => {
                    painter.rect_filled(Rect::from_min_max(fit.min, Pos2::new(fit.max.x, r.min.y)), 0.0, dim);
                    painter.rect_filled(Rect::from_min_max(Pos2::new(fit.min.x, r.max.y), fit.max), 0.0, dim);
                    painter.rect_filled(Rect::from_min_max(Pos2::new(fit.min.x, r.min.y), Pos2::new(r.min.x, r.max.y)), 0.0, dim);
                    painter.rect_filled(Rect::from_min_max(Pos2::new(r.max.x, r.min.y), Pos2::new(fit.max.x, r.max.y)), 0.0, dim);
                    painter.rect_stroke(r, 0.0, Stroke::new(1.5_f32, Color32::WHITE));
                    let (a, b) = (to_img(r.min), to_img(r.max));
                    let label = format!("{} × {}", (b.x - a.x).round() as i32, (b.y - a.y).round() as i32);
                    pill(&painter, Pos2::new(r.min.x, (r.min.y - 30.0).max(fit.min.y + 4.0)), label);
                }
                None => {
                    painter.rect_filled(fit, 0.0, dim);
                    if let Some(h) = hover {
                        let guide = Stroke::new(1.0_f32, Color32::from_white_alpha(110));
                        painter.line_segment([Pos2::new(fit.min.x, h.y), Pos2::new(fit.max.x, h.y)], guide);
                        painter.line_segment([Pos2::new(h.x, fit.min.y), Pos2::new(h.x, fit.max.y)], guide);
                        let p = to_img(h);
                        pill(&painter, h + Vec2::new(14.0, 14.0), format!("{}, {}", p.x as i32, p.y as i32));
                    }
                    let msg = "Drag to select an area · Click for full screen · Esc to cancel";
                    let g = painter.layout_no_wrap(msg.into(), FontId::proportional(14.0), Color32::from_gray(235));
                    let at = Pos2::new(fit.center().x - g.size().x / 2.0 - 10.0, fit.max.y - 46.0);
                    pill(&painter, at, msg.into());
                }
            }
        });
        if let Some(r) = finish {
            self.finish_select(r);
        }
    }

    fn finish_select(&mut self, r: Rect) {
        let (fw, fh) = (self.full.width(), self.full.height());
        let x = (r.min.x.round() as u32).min(fw - 1);
        let y = (r.min.y.round() as u32).min(fh - 1);
        let w = (r.width().round() as u32).clamp(1, fw - x);
        let h = (r.height().round() as u32).clamp(1, fh - y);
        self.base = image::imageops::crop_imm(&self.full, x, y, w, h).to_image();
        self.sel = Rect::from_min_size(Pos2::new(x as f32, y as f32), Vec2::new(w as f32, h as f32));
        self.phase = Phase::Edit;
        self.status = "Area selected · pick a tool and draw".into();
    }

    fn output(&self) -> RgbaImage {
        render(&self.base, &self.shapes, self.font.as_ref(), self.backdrop)
    }

    fn do_copy(&mut self) -> bool {
        let bytes = png_bytes(&self.output());
        match wl_copy(&bytes, "image/png") {
            Ok(_) => {
                self.status = "Copied to clipboard".into();
                true
            }
            Err(e) => {
                self.status = format!("Copy failed: {e}");
                false
            }
        }
    }

    fn do_save(&mut self) {
        let p = save_path();
        self.status = match self.output().save(&p) {
            Ok(_) => format!("Saved {}", p.display()),
            Err(e) => format!("Save failed: {e}"),
        };
    }

    fn do_ocr(&mut self) {
        let tmp = std::env::temp_dir().join("cosnap-ocr.png");
        if self.base.save(&tmp).is_err() {
            self.status = "OCR: could not write temp file".into();
            return;
        }
        match Command::new("tesseract").arg(&tmp).arg("stdout").output() {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let _ = wl_copy(text.as_bytes(), "text/plain");
                self.status = format!("OCR: copied {} characters", text.chars().count());
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                let line = err.lines().find(|l| !l.trim().is_empty()).unwrap_or("unknown error");
                self.status = format!("OCR failed: {line}");
            }
            Err(_) => self.status = "OCR failed: tesseract not found".into(),
        }
    }

    fn undo(&mut self) {
        if let Some(s) = self.shapes.pop() {
            if matches!(s, Shape::Marker { .. }) {
                self.marker_n = self.marker_n.saturating_sub(1).max(1);
            }
            self.redo.push(s);
        }
    }

    fn redo(&mut self) {
        if let Some(s) = self.redo.pop() {
            if matches!(s, Shape::Marker { .. }) {
                self.marker_n += 1;
            }
            self.shapes.push(s);
        }
    }

    fn push(&mut self, s: Shape) {
        if matches!(s, Shape::Marker { .. }) {
            self.marker_n += 1;
        }
        self.shapes.push(s);
        self.redo.clear();
    }

    fn make_draft(&self, p: Pos2) -> Option<Shape> {
        let (c, w) = (self.color, self.width);
        Some(match self.tool {
            Tool::Arrow => Shape::Arrow { a: p, b: p, c, w },
            Tool::Rect => Shape::Rect { a: p, b: p, c, w },
            Tool::Ellipse => Shape::Ellipse { a: p, b: p, c, w },
            Tool::Free => Shape::Free { pts: vec![p], c, w },
            Tool::Highlight => Shape::Highlight { pts: vec![p], c, w: w * 4.0 + 8.0 },
            Tool::Redact => Shape::Redact { a: p, b: p },
            Tool::Marker | Tool::Text => return None,
        })
    }

    fn paint_shape(&self, painter: &egui::Painter, s: &Shape, origin: Pos2, k: f32) {
        let tp = |p: Pos2| origin + p.to_vec2() * k;
        match s {
            Shape::Arrow { a, b, c, w } => {
                for (p, q) in arrow_segments(*a, *b, *w) {
                    painter.line_segment([tp(p), tp(q)], Stroke::new(w * k, *c));
                }
            }
            Shape::Rect { a, b, c, w } => {
                painter.rect_stroke(Rect::from_two_pos(tp(*a), tp(*b)), 0.0, Stroke::new(w * k, *c));
            }
            Shape::Ellipse { a, b, c, w } => {
                let pts: Vec<_> = ellipse_points(*a, *b).into_iter().map(tp).collect();
                painter.add(egui::Shape::line(pts, Stroke::new(w * k, *c)));
            }
            Shape::Free { pts, c, w } => {
                let pts: Vec<_> = pts.iter().map(|p| tp(*p)).collect();
                if pts.len() > 1 {
                    painter.add(egui::Shape::line(pts, Stroke::new(w * k, *c)));
                }
            }
            Shape::Highlight { pts, c, w } => {
                let col = Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 102);
                let pts: Vec<_> = pts.iter().map(|p| tp(*p)).collect();
                if pts.len() > 1 {
                    painter.add(egui::Shape::line(pts, Stroke::new(w * k, col)));
                }
            }
            Shape::Redact { a, b } => {
                painter.rect_filled(Rect::from_two_pos(tp(*a), tp(*b)), 0.0, Color32::BLACK);
            }
            Shape::Marker { p, n, c, r } => {
                painter.circle_filled(tp(*p), r * k, *c);
                painter.text(tp(*p), Align2::CENTER_CENTER, n.to_string(), FontId::proportional(r * 1.35 * k), contrast(*c));
            }
            Shape::Text { p, s, c, size } => {
                painter.text(tp(*p), Align2::LEFT_TOP, s, FontId::proportional(*size * k), *c);
            }
        }
    }

    fn edit_ui(&mut self, ctx: &egui::Context) {
        let (fit, s) = self.geometry(ctx);
        let sel_s = Rect::from_min_size(fit.min + self.sel.min.to_vec2() * s, self.sel.size() * s);
        let (cw, ch) = (self.base.width() as f32, self.base.height() as f32);
        let editing = self.text_edit.is_some();
        let screen = ctx.screen_rect();

        // ── keyboard ──
        let (mut copy, mut save, mut undo, mut redo, mut ocr) = (false, false, false, false, false);
        if !editing {
            ctx.input(|i| {
                let cmd = i.modifiers.command;
                if i.events.iter().any(|e| matches!(e, egui::Event::Copy)) { copy = true; }
                if cmd && i.key_pressed(egui::Key::Z) { if i.modifiers.shift { redo = true } else { undo = true } }
                if cmd && i.key_pressed(egui::Key::Y) { redo = true; }
                if cmd && i.key_pressed(egui::Key::S) { save = true; }
                if i.key_pressed(egui::Key::Escape) { std::process::exit(0); }
                if !cmd {
                    for (k, t) in [
                        (egui::Key::A, Tool::Arrow), (egui::Key::R, Tool::Rect), (egui::Key::E, Tool::Ellipse),
                        (egui::Key::F, Tool::Free), (egui::Key::H, Tool::Highlight), (egui::Key::D, Tool::Redact),
                        (egui::Key::C, Tool::Marker), (egui::Key::T, Tool::Text),
                    ] {
                        if i.key_pressed(k) { self.tool = t; }
                    }
                    for (n, k) in [
                        egui::Key::Num1, egui::Key::Num2, egui::Key::Num3, egui::Key::Num4,
                        egui::Key::Num5, egui::Key::Num6, egui::Key::Num7, egui::Key::Num8,
                    ].iter().enumerate() {
                        if i.key_pressed(*k) { self.color = PALETTE[n]; }
                    }
                    if i.key_pressed(egui::Key::O) { ocr = true; }
                    if i.key_pressed(egui::Key::B) { self.backdrop = !self.backdrop; }
                    if i.key_pressed(egui::Key::F1) { self.help = !self.help; }
                }
            });
        }

        // ── canvas ──
        egui::CentralPanel::default().frame(egui::Frame::none().fill(Color32::BLACK)).show(ctx, |ui| {
            let full_painter = ui.painter_at(fit);
            full_painter.image(self.tex.id(), fit, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
            let dim = Color32::from_black_alpha(150);
            full_painter.rect_filled(Rect::from_min_max(fit.min, Pos2::new(fit.max.x, sel_s.min.y)), 0.0, dim);
            full_painter.rect_filled(Rect::from_min_max(Pos2::new(fit.min.x, sel_s.max.y), fit.max), 0.0, dim);
            full_painter.rect_filled(Rect::from_min_max(Pos2::new(fit.min.x, sel_s.min.y), Pos2::new(sel_s.min.x, sel_s.max.y)), 0.0, dim);
            full_painter.rect_filled(Rect::from_min_max(Pos2::new(sel_s.max.x, sel_s.min.y), Pos2::new(fit.max.x, sel_s.max.y)), 0.0, dim);

            let resp = ui.allocate_rect(sel_s, Sense::click_and_drag());
            ctx.set_cursor_icon(if resp.hovered() {
                if self.tool == Tool::Text { egui::CursorIcon::Text } else { egui::CursorIcon::Crosshair }
            } else {
                egui::CursorIcon::Default
            });
            if resp.hovered() {
                let d = ctx.input(|i| i.raw_scroll_delta.y);
                if d != 0.0 {
                    self.width = (self.width + d.signum()).clamp(1.0, 24.0);
                    self.status = format!("Size {}", self.width as i32);
                }
            }
            let to_crop = |p: Pos2| Pos2::new(((p.x - sel_s.min.x) / s).clamp(0.0, cw), ((p.y - sel_s.min.y) / s).clamp(0.0, ch));

            if !editing {
                if resp.drag_started() {
                    if let Some(p) = resp.interact_pointer_pos() {
                        self.draft = self.make_draft(to_crop(p));
                    }
                }
                if resp.dragged() {
                    if let (Some(p), Some(d)) = (resp.interact_pointer_pos(), self.draft.as_mut()) {
                        let p = to_crop(p);
                        match d {
                            Shape::Arrow { b, .. } | Shape::Rect { b, .. } | Shape::Ellipse { b, .. } | Shape::Redact { b, .. } => *b = p,
                            Shape::Free { pts, .. } | Shape::Highlight { pts, .. } => {
                                if pts.last().map_or(true, |l| (*l - p).length() > 1.0) {
                                    pts.push(p);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                if resp.drag_stopped() {
                    if let Some(d) = self.draft.take() {
                        self.push(d);
                    }
                }
                if resp.clicked() {
                    self.color_open = false;
                    if let Some(p) = resp.interact_pointer_pos() {
                        let p = to_crop(p);
                        match self.tool {
                            Tool::Marker => {
                                let (c, r, n) = (self.color, 12.0 + self.width * 1.5, self.marker_n);
                                self.push(Shape::Marker { p, n, c, r });
                            }
                            Tool::Text => self.text_edit = Some((p, String::new(), true)),
                            _ => {}
                        }
                    }
                }
            }

            let cp = ui.painter_at(sel_s);
            for sh in &self.shapes {
                self.paint_shape(&cp, sh, sel_s.min, s);
            }
            if let Some(d) = &self.draft {
                self.paint_shape(&cp, d, sel_s.min, s);
            }
            ui.painter().rect_stroke(sel_s, 0.0, Stroke::new(1.0_f32, Color32::from_white_alpha(70)));
        });

        // ── floating toolbar ──
        let below = sel_s.max.y + 70.0 < screen.max.y;
        let above = sel_s.min.y - 70.0 > screen.min.y;
        let cx = sel_s.center().x.clamp(screen.min.x + 340.0, (screen.max.x - 340.0).max(screen.min.x + 340.0));
        let (pos, pivot) = if below {
            (Pos2::new(cx, sel_s.max.y + 10.0), Align2::CENTER_TOP)
        } else if above {
            (Pos2::new(cx, sel_s.min.y - 10.0), Align2::CENTER_BOTTOM)
        } else {
            (Pos2::new(cx, sel_s.max.y - 56.0), Align2::CENTER_TOP)
        };
        let cur_color = self.color;
        let tb = egui::Area::new(egui::Id::new("toolbar"))
            .order(egui::Order::Foreground)
            .pivot(pivot)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                dark_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::splat(3.0);
                        for (t, icon, tip) in [
                            (Tool::Arrow, Icon::Arrow, "Arrow (A)"),
                            (Tool::Rect, Icon::Rect, "Rectangle (R)"),
                            (Tool::Ellipse, Icon::Ellipse, "Ellipse (E)"),
                            (Tool::Free, Icon::Pen, "Freehand (F)"),
                            (Tool::Highlight, Icon::Highlight, "Highlighter (H)"),
                            (Tool::Redact, Icon::Redact, "Redact (D)"),
                            (Tool::Marker, Icon::Marker, "Numbered marker (C)"),
                            (Tool::Text, Icon::Text, "Text (T)"),
                        ] {
                            if icon_btn(ui, icon, self.tool == t, tip, cur_color).clicked() {
                                self.tool = t;
                            }
                        }
                        sep(ui);
                        if icon_btn(ui, Icon::Color, self.color_open, "Color (1–8)", cur_color).clicked() {
                            self.color_open = !self.color_open;
                        }
                        sep(ui);
                        if icon_btn(ui, Icon::Ocr, false, "Copy text from image (O)", cur_color).clicked() { ocr = true; }
                        if icon_btn(ui, Icon::Backdrop, self.backdrop, "Backdrop (B)", cur_color).clicked() { self.backdrop = !self.backdrop; }
                        if icon_btn(ui, Icon::Undo, false, "Undo (Ctrl+Z)", cur_color).clicked() { undo = true; }
                        sep(ui);
                        if icon_btn(ui, Icon::Copy, false, "Copy (Ctrl+C)", cur_color).clicked() { copy = true; }
                        if icon_btn(ui, Icon::Save, false, "Save (Ctrl+S)", cur_color).clicked() { save = true; }
                        if icon_btn(ui, Icon::Close, false, "Close (Esc)", cur_color).clicked() { std::process::exit(0); }
                    });
                });
            });
        let tbr = tb.response.rect;

        if self.color_open {
            let (cpos, cpivot) = if below || !above {
                (Pos2::new(tbr.center().x, tbr.max.y + 8.0), Align2::CENTER_TOP)
            } else {
                (Pos2::new(tbr.center().x, tbr.min.y - 8.0), Align2::CENTER_BOTTOM)
            };
            egui::Area::new(egui::Id::new("colors")).order(egui::Order::Foreground).pivot(cpivot).fixed_pos(cpos).show(ctx, |ui| {
                dark_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::splat(7.0);
                        for c in PALETTE {
                            let (r, resp) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
                            ui.painter().circle_filled(r.center(), 10.0, c);
                            ui.painter().circle_stroke(r.center(), 10.0, Stroke::new(1.0_f32, Color32::from_gray(90)));
                            if self.color == c {
                                ui.painter().circle_stroke(r.center(), 12.5, Stroke::new(2.0_f32, Color32::WHITE));
                            }
                            if resp.clicked() {
                                self.color = c;
                                self.color_open = false;
                            }
                        }
                        egui::color_picker::color_edit_button_srgba(ui, &mut self.color, egui::color_picker::Alpha::Opaque);
                    });
                });
            });
        }

        // ── status pill ──
        egui::Area::new(egui::Id::new("status")).order(egui::Order::Foreground).anchor(Align2::CENTER_BOTTOM, [0.0, -16.0]).show(ctx, |ui| {
            egui::Frame::none().fill(PANEL).rounding(8.0).inner_margin(egui::Margin::symmetric(12.0, 6.0)).show(ui, |ui| {
                ui.label(egui::RichText::new(&self.status).size(13.0).color(Color32::from_gray(225)));
            });
        });

        // ── cheat sheet ──
        if self.help {
            egui::Area::new(egui::Id::new("help")).order(egui::Order::Foreground).anchor(Align2::RIGHT_TOP, [-16.0, 16.0]).show(ctx, |ui| {
                egui::Frame::none().fill(Color32::from_rgba_unmultiplied(28, 28, 32, 215)).rounding(8.0).inner_margin(egui::Margin::same(10.0)).show(ui, |ui| {
                    egui::Grid::new("helpgrid").num_columns(4).spacing([14.0, 3.0]).show(ui, |ui| {
                        let rows = [
                            ("A", "Arrow", "Wheel", "Tool size"),
                            ("R", "Rectangle", "O", "Copy text (OCR)"),
                            ("E", "Ellipse", "B", "Backdrop"),
                            ("F", "Freehand", "Ctrl+Z", "Undo"),
                            ("H", "Highlighter", "Ctrl+C", "Copy + close"),
                            ("D", "Redact", "Ctrl+S", "Save"),
                            ("C", "Marker", "Esc", "Close"),
                            ("T", "Text", "F1", "Hide this"),
                            ("1–8", "Color", "", ""),
                        ];
                        for (a, b, c, d) in rows {
                            for (i, t) in [a, b, c, d].iter().enumerate() {
                                let col = if i % 2 == 0 { Color32::from_gray(140) } else { Color32::from_gray(225) };
                                ui.label(egui::RichText::new(*t).size(12.0).color(col));
                            }
                            ui.end_row();
                        }
                    });
                });
            });
        }

        // ── inline text editor ──
        if let Some((p, mut text, first)) = self.text_edit.take() {
            let size = 14.0 + self.width * 4.0;
            let pos = sel_s.min + p.to_vec2() * s;
            let (mut keep, mut commit) = (true, false);
            egui::Area::new(egui::Id::new("text_edit")).fixed_pos(pos).order(egui::Order::Tooltip).show(ctx, |ui| {
                let te = egui::TextEdit::singleline(&mut text)
                    .font(FontId::proportional(size * s))
                    .text_color(self.color)
                    .desired_width(280.0);
                let r = ui.add(te);
                if first {
                    r.request_focus();
                }
                let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
                if r.lost_focus() {
                    keep = false;
                    commit = !esc;
                }
            });
            if keep {
                self.text_edit = Some((p, text, false));
            } else if commit && !text.trim().is_empty() {
                let (c, size) = (self.color, size);
                self.push(Shape::Text { p, s: text, c, size });
            }
        }

        if undo { self.undo(); }
        if redo { self.redo(); }
        if copy && self.do_copy() {
            // copy hands off to wl-copy, which keeps serving the clipboard after we exit
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if save { self.do_save(); }
        if ocr { self.do_ocr(); }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if matches!(self.phase, Phase::Select { .. }) {
            self.select_ui(ctx);
        } else {
            self.edit_ui(ctx);
        }
    }
}

