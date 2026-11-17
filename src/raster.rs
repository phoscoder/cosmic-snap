//! Software rasteriser used to bake annotations into the exported image.
#![allow(unused_imports)]
use crate::shapes::{arrow_segments, ellipse_points, norm_rect, Shape};
use ab_glyph::{point, Font, FontArc, PxScale, ScaleFont};
use eframe::egui::{Color32, Pos2};
use image::{Rgba, RgbaImage};

// ───────────────────────── rasteriser (export) ─────────────────────────

pub(crate) fn blend(img: &mut RgbaImage, x: i32, y: i32, c: Color32, a: f32) {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 || a <= 0.0 {
        return;
    }
    let a = a.min(1.0);
    let p = img.get_pixel_mut(x as u32, y as u32);
    let src = [c.r(), c.g(), c.b()];
    for i in 0..3 {
        p[i] = (p[i] as f32 * (1.0 - a) + src[i] as f32 * a).round() as u8;
    }
    p[3] = 255;
}

pub(crate) fn dist_seg(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let l2 = ab.length_sq();
    if l2 < 1e-6 {
        return (p - a).length();
    }
    let t = (((p - a).x * ab.x + (p - a).y * ab.y) / l2).clamp(0.0, 1.0);
    (p - (a + ab * t)).length()
}

/// Draw round-capped thick segments as one union mask, blended once (so alpha doesn't stack).
pub(crate) fn stroke_segments(img: &mut RgbaImage, segs: &[(Pos2, Pos2)], w: f32, c: Color32, alpha: f32) {
    if segs.is_empty() {
        return;
    }
    let pad = w / 2.0 + 2.0;
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for (a, b) in segs {
        for p in [a, b] {
            x0 = x0.min(p.x);
            y0 = y0.min(p.y);
            x1 = x1.max(p.x);
            y1 = y1.max(p.y);
        }
    }
    let bx0 = ((x0 - pad).floor() as i32).max(0);
    let by0 = ((y0 - pad).floor() as i32).max(0);
    let bx1 = ((x1 + pad).ceil() as i32).min(img.width() as i32 - 1);
    let by1 = ((y1 + pad).ceil() as i32).min(img.height() as i32 - 1);
    if bx1 < bx0 || by1 < by0 {
        return;
    }
    let mw = (bx1 - bx0 + 1) as usize;
    let mh = (by1 - by0 + 1) as usize;
    let mut mask = vec![0f32; mw * mh];
    for (a, b) in segs {
        let sx0 = ((a.x.min(b.x) - pad).floor() as i32).max(bx0);
        let sy0 = ((a.y.min(b.y) - pad).floor() as i32).max(by0);
        let sx1 = ((a.x.max(b.x) + pad).ceil() as i32).min(bx1);
        let sy1 = ((a.y.max(b.y) + pad).ceil() as i32).min(by1);
        for y in sy0..=sy1 {
            for x in sx0..=sx1 {
                let d = dist_seg(Pos2::new(x as f32 + 0.5, y as f32 + 0.5), *a, *b);
                let cov = (w / 2.0 + 0.5 - d).clamp(0.0, 1.0);
                let i = (y - by0) as usize * mw + (x - bx0) as usize;
                if cov > mask[i] {
                    mask[i] = cov;
                }
            }
        }
    }
    for y in 0..mh {
        for x in 0..mw {
            let cov = mask[y * mw + x];
            if cov > 0.0 {
                blend(img, bx0 + x as i32, by0 + y as i32, c, cov * alpha);
            }
        }
    }
}

pub(crate) fn fill_rect(img: &mut RgbaImage, a: Pos2, b: Pos2, c: Color32, alpha: f32) {
    let r = norm_rect(a, b);
    for y in (r.min.y.floor() as i32)..(r.max.y.ceil() as i32) {
        for x in (r.min.x.floor() as i32)..(r.max.x.ceil() as i32) {
            blend(img, x, y, c, alpha);
        }
    }
}

pub(crate) fn fill_circle(img: &mut RgbaImage, c: Pos2, r: f32, col: Color32) {
    for y in ((c.y - r - 1.0).floor() as i32)..=((c.y + r + 1.0).ceil() as i32) {
        for x in ((c.x - r - 1.0).floor() as i32)..=((c.x + r + 1.0).ceil() as i32) {
            let d = (Pos2::new(x as f32 + 0.5, y as f32 + 0.5) - c).length();
            blend(img, x, y, col, (r + 0.5 - d).clamp(0.0, 1.0));
        }
    }
}

pub(crate) fn text_width(font: &FontArc, size: f32, s: &str) -> f32 {
    let sf = font.as_scaled(PxScale::from(size));
    s.chars().map(|ch| sf.h_advance(sf.glyph_id(ch))).sum()
}

pub(crate) fn draw_text(img: &mut RgbaImage, font: &FontArc, p: Pos2, size: f32, s: &str, c: Color32) {
    let scale = PxScale::from(size);
    let sf = font.as_scaled(scale);
    let base = p.y + sf.ascent();
    let mut caret = p.x;
    for ch in s.chars() {
        let id = sf.glyph_id(ch);
        let g = id.with_scale_and_position(scale, point(caret, base));
        caret += sf.h_advance(id);
        if let Some(og) = font.outline_glyph(g) {
            let b = og.px_bounds();
            og.draw(|x, y, cov| {
                blend(img, b.min.x as i32 + x as i32, b.min.y as i32 + y as i32, c, cov);
            });
        }
    }
}

pub(crate) fn raster_shape(img: &mut RgbaImage, s: &Shape, font: Option<&FontArc>) {
    match s {
        Shape::Arrow { a, b, c, w } => stroke_segments(img, &arrow_segments(*a, *b, *w), *w, *c, 1.0),
        Shape::Rect { a, b, c, w } => {
            let r = norm_rect(*a, *b);
            let (p1, p2, p3, p4) = (r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom());
            stroke_segments(img, &[(p1, p2), (p2, p3), (p3, p4), (p4, p1)], *w, *c, 1.0)
        }
        Shape::Ellipse { a, b, c, w } => {
            let pts = ellipse_points(*a, *b);
            let segs: Vec<_> = pts.windows(2).map(|p| (p[0], p[1])).collect();
            stroke_segments(img, &segs, *w, *c, 1.0)
        }
        Shape::Free { pts, c, w } => {
            let segs = poly_segs(pts);
            stroke_segments(img, &segs, *w, *c, 1.0)
        }
        Shape::Highlight { pts, c, w } => {
            let segs = poly_segs(pts);
            stroke_segments(img, &segs, *w, *c, 0.4)
        }
        Shape::Redact { a, b } => fill_rect(img, *a, *b, Color32::BLACK, 1.0),
        Shape::Marker { p, n, c, r } => {
            fill_circle(img, *p, *r, *c);
            if let Some(f) = font {
                let s = n.to_string();
                let size = r * 1.35;
                let w = text_width(f, size, &s);
                let sf = f.as_scaled(PxScale::from(size));
                let h = sf.ascent() - sf.descent();
                draw_text(img, f, Pos2::new(p.x - w / 2.0, p.y - h / 2.0), size, &s, contrast(*c));
            }
        }
        Shape::Text { p, s, c, size } => {
            if let Some(f) = font {
                draw_text(img, f, *p, *size, s, *c);
            }
        }
    }
}

pub(crate) fn poly_segs(pts: &[Pos2]) -> Vec<(Pos2, Pos2)> {
    if pts.len() == 1 {
        return vec![(pts[0], pts[0])];
    }
    pts.windows(2).map(|p| (p[0], p[1])).collect()
}

pub(crate) fn contrast(c: Color32) -> Color32 {
    let l = 0.299 * c.r() as f32 + 0.587 * c.g() as f32 + 0.114 * c.b() as f32;
    if l > 150.0 { Color32::BLACK } else { Color32::WHITE }
}

pub(crate) fn render(base: &RgbaImage, shapes: &[Shape], font: Option<&FontArc>, backdrop: bool) -> RgbaImage {
    let mut img = base.clone();
    for s in shapes {
        raster_shape(&mut img, s, font);
    }
    if !backdrop {
        return img;
    }
    let (w, h) = (img.width(), img.height());
    let pad = ((w.max(h) as f32) * 0.06).clamp(40.0, 120.0) as u32;
    let (cw, ch) = (w + pad * 2, h + pad * 2);
    let mut out = RgbaImage::new(cw, ch);
    let (c1, c2) = ([99.0f32, 102.0, 241.0], [236.0f32, 72.0, 153.0]);
    for y in 0..ch {
        for x in 0..cw {
            let t = (x as f32 / cw as f32 + y as f32 / ch as f32) / 2.0;
            out.put_pixel(
                x,
                y,
                Rgba([
                    (c1[0] + (c2[0] - c1[0]) * t) as u8,
                    (c1[1] + (c2[1] - c1[1]) * t) as u8,
                    (c1[2] + (c2[2] - c1[2]) * t) as u8,
                    255,
                ]),
            );
        }
    }
    let radius = 16.0f32;
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let cx = fx.clamp(radius, w as f32 - radius);
            let cy = fy.clamp(radius, h as f32 - radius);
            let d = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
            let cov = (radius + 0.5 - d).clamp(0.0, 1.0);
            let src = img.get_pixel(x, y);
            let dst = out.get_pixel_mut(x + pad, y + pad);
            for i in 0..3 {
                dst[i] = (dst[i] as f32 * (1.0 - cov) + src[i] as f32 * cov) as u8;
            }
        }
    }
    out
}

