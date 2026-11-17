//! Annotation shapes, tools and shared geometry helpers.
#![allow(unused_imports)]
use eframe::egui::{Color32, Pos2, Rect, Vec2};

// ───────────────────────── shapes ─────────────────────────

#[derive(Clone)]
pub(crate) enum Shape {
    Arrow { a: Pos2, b: Pos2, c: Color32, w: f32 },
    Rect { a: Pos2, b: Pos2, c: Color32, w: f32 },
    Ellipse { a: Pos2, b: Pos2, c: Color32, w: f32 },
    Free { pts: Vec<Pos2>, c: Color32, w: f32 },
    Highlight { pts: Vec<Pos2>, c: Color32, w: f32 },
    Redact { a: Pos2, b: Pos2 },
    Marker { p: Pos2, n: u32, c: Color32, r: f32 },
    Text { p: Pos2, s: String, c: Color32, size: f32 },
}

impl Shape {
    /// Move the whole shape by `d` (used to keep annotations anchored when the region changes).
    pub(crate) fn translate(&mut self, d: Vec2) {
        match self {
            Shape::Arrow { a, b, .. } | Shape::Rect { a, b, .. } | Shape::Ellipse { a, b, .. } | Shape::Redact { a, b } => {
                *a += d;
                *b += d;
            }
            Shape::Free { pts, .. } | Shape::Highlight { pts, .. } => pts.iter_mut().for_each(|p| *p += d),
            Shape::Marker { p, .. } | Shape::Text { p, .. } => *p += d,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Tool {
    Arrow,
    Rect,
    Ellipse,
    Free,
    Highlight,
    Redact,
    Marker,
    Text,
    Resize,
}


pub(crate) fn norm_rect(a: Pos2, b: Pos2) -> Rect {
    Rect::from_two_pos(a, b)
}

pub(crate) fn rot(v: Vec2, ang: f32) -> Vec2 {
    let (s, c) = ang.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

pub(crate) fn arrow_segments(a: Pos2, b: Pos2, w: f32) -> Vec<(Pos2, Pos2)> {
    let d = b - a;
    if d.length() < 1.0 {
        return vec![(a, b)];
    }
    let dir = d.normalized();
    let len = (w * 4.0).max(16.0).min(d.length() * 0.6);
    let h1 = b - rot(dir, 0.45) * len;
    let h2 = b - rot(dir, -0.45) * len;
    vec![(a, b), (b, h1), (b, h2)]
}

pub(crate) fn ellipse_points(a: Pos2, b: Pos2) -> Vec<Pos2> {
    let r = norm_rect(a, b);
    let c = r.center();
    let (rx, ry) = (r.width() / 2.0, r.height() / 2.0);
    let n = 96;
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            Pos2::new(c.x + rx * t.cos(), c.y + ry * t.sin())
        })
        .collect()
}

