//! Toolbar widgets: palette, vector icons and dark panel styling.
#![allow(unused_imports)]
use crate::shapes::rot;
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

pub(crate) const PALETTE: [Color32; 8] = [
    Color32::from_rgb(255, 55, 95),
    Color32::from_rgb(255, 159, 10),
    Color32::from_rgb(255, 214, 10),
    Color32::from_rgb(48, 209, 88),
    Color32::from_rgb(10, 132, 255),
    Color32::from_rgb(191, 90, 242),
    Color32::from_rgb(0, 0, 0),
    Color32::from_rgb(255, 255, 255),
];
pub(crate) const ACCENT: Color32 = Color32::from_rgb(10, 132, 255);
pub(crate) const PANEL: Color32 = Color32::from_rgb(28, 28, 32);

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Icon {
    Arrow, Rect, Ellipse, Pen, Highlight, Redact, Marker, Text, Color, Undo, Copy, Save, Ocr, Backdrop, Close,
}

pub(crate) fn draw_icon(p: &egui::Painter, icon: Icon, c: Pos2, col: Color32, cur: Color32) {
    let st = Stroke::new(1.8_f32, col);
    let v = |x: f32, y: f32| c + Vec2::new(x, y);
    match icon {
        Icon::Arrow => {
            p.line_segment([v(-7.0, 7.0), v(7.0, -7.0)], st);
            p.line_segment([v(7.0, -7.0), v(0.5, -7.0)], st);
            p.line_segment([v(7.0, -7.0), v(7.0, -0.5)], st);
        }
        Icon::Rect => {
            p.rect_stroke(Rect::from_center_size(c, Vec2::new(17.0, 13.0)), 2.5, st);
        }
        Icon::Ellipse => {
            p.circle_stroke(c, 8.0, st);
        }
        Icon::Pen => {
            let pts = vec![v(-8.0, 5.0), v(-4.5, -4.0), v(-1.0, 4.0), v(2.5, -4.0), v(5.0, 1.0), v(8.0, -2.0)];
            p.add(egui::Shape::line(pts, st));
        }
        Icon::Highlight => {
            // chisel-tip highlighter pen, angled, with a translucent swipe under the tip
            let tr = |x: f32, y: f32| c + Vec2::new(1.5, -1.5) + rot(Vec2::new(x, y), 0.785);
            let body = vec![tr(-3.5, -8.0), tr(3.5, -8.0), tr(3.5, 2.5), tr(1.2, 7.5), tr(-1.2, 7.5), tr(-3.5, 2.5)];
            p.add(egui::Shape::closed_line(body, st));
            p.line_segment([tr(-3.5, 2.5), tr(3.5, 2.5)], Stroke::new(1.4_f32, col));
            p.line_segment(
                [v(-9.0, 9.0), v(-2.0, 9.0)],
                Stroke::new(3.0_f32, Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 120)),
            );
        }
        Icon::Redact => {
            // pixelated/mosaic block: outlined square with a checker fill
            let r = Rect::from_center_size(c, Vec2::splat(17.0));
            p.rect_stroke(r, 2.5, st);
            let cell = 17.0 / 4.0;
            for iy in 0..4 {
                for ix in 0..4 {
                    if (ix + iy) % 2 == 0 {
                        let min = r.min + Vec2::new(ix as f32 * cell, iy as f32 * cell);
                        p.rect_filled(Rect::from_min_size(min, Vec2::splat(cell)).shrink(0.6), 0.0, col);
                    }
                }
            }
        }
        Icon::Marker => {
            // circle with a numeral "1" drawn from strokes (font independent)
            p.circle_stroke(c, 9.0, st);
            let n = Stroke::new(1.8_f32, col);
            p.line_segment([v(0.8, -4.5), v(0.8, 4.5)], n);
            p.line_segment([v(0.8, -4.5), v(-2.2, -2.3)], n);
            p.line_segment([v(-2.0, 4.5), v(3.6, 4.5)], n);
        }
        Icon::Text => {
            p.text(c, Align2::CENTER_CENTER, "T", FontId::proportional(19.0), col);
        }
        Icon::Color => {
            p.circle_filled(c, 7.5, cur);
            p.circle_stroke(c, 9.0, Stroke::new(1.5_f32, Color32::from_white_alpha(200)));
        }
        Icon::Undo => {
            let pts: Vec<Pos2> = (0..=10)
                .map(|i| {
                    let t = (-150.0_f32 + i as f32 * 21.0).to_radians();
                    v(0.0, 1.0) + Vec2::new(t.cos(), t.sin()) * 7.0
                })
                .collect();
            let start = pts[0];
            p.add(egui::Shape::line(pts, st));
            p.line_segment([start, start + Vec2::new(0.0, -6.0)], st);
            p.line_segment([start, start + Vec2::new(6.0, 0.0)], st);
        }
        Icon::Copy => {
            p.rect_stroke(Rect::from_center_size(v(-2.0, -2.0), Vec2::splat(11.0)), 2.0, st);
            p.rect_stroke(Rect::from_center_size(v(2.5, 2.5), Vec2::splat(11.0)), 2.0, st);
        }
        Icon::Save => {
            p.line_segment([v(0.0, -8.0), v(0.0, 3.0)], st);
            p.line_segment([v(0.0, 3.0), v(-4.0, -1.0)], st);
            p.line_segment([v(0.0, 3.0), v(4.0, -1.0)], st);
            p.add(egui::Shape::line(vec![v(-8.0, 3.0), v(-8.0, 8.0), v(8.0, 8.0), v(8.0, 3.0)], st));
        }
        Icon::Ocr => {
            for (sx, sy) in [(-1.0_f32, -1.0_f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let corner = v(8.0 * sx, 8.0 * sy);
                p.line_segment([corner, corner + Vec2::new(-4.0 * sx, 0.0)], st);
                p.line_segment([corner, corner + Vec2::new(0.0, -4.0 * sy)], st);
            }
            p.line_segment([v(-4.0, 0.0), v(4.0, 0.0)], st);
        }
        Icon::Backdrop => {
            p.rect_stroke(Rect::from_center_size(c, Vec2::new(18.0, 15.0)), 3.0, st);
            p.rect_stroke(Rect::from_center_size(c, Vec2::new(9.0, 7.0)), 1.5, st);
        }
        Icon::Close => {
            p.line_segment([v(-6.0, -6.0), v(6.0, 6.0)], st);
            p.line_segment([v(-6.0, 6.0), v(6.0, -6.0)], st);
        }
    }
}

pub(crate) fn icon_btn(ui: &mut egui::Ui, icon: Icon, active: bool, tip: &str, cur: Color32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::click());
    let p = ui.painter();
    if active {
        p.rect_filled(rect, 8.0, ACCENT);
    } else if resp.hovered() {
        p.rect_filled(rect, 8.0, Color32::from_white_alpha(28));
    }
    let col = if active || resp.hovered() { Color32::WHITE } else { Color32::from_gray(205) };
    draw_icon(p, icon, rect.center(), col, cur);
    resp.on_hover_text(tip)
}

pub(crate) fn dark_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(PANEL)
        .rounding(10.0)
        .inner_margin(egui::Margin::same(6.0))
        .stroke(Stroke::new(1.0_f32, Color32::from_gray(58)))
        .shadow(egui::epaint::Shadow {
            offset: Vec2::new(0.0, 6.0),
            blur: 18.0,
            spread: 0.0,
            color: Color32::from_black_alpha(130),
        })
}

pub(crate) fn sep(ui: &mut egui::Ui) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(9.0, 34.0), Sense::hover());
    ui.painter().line_segment(
        [r.center_top() + Vec2::new(0.0, 8.0), r.center_bottom() - Vec2::new(0.0, 8.0)],
        Stroke::new(1.0_f32, Color32::from_gray(70)),
    );
}

