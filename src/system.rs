//! Desktop integration: capture, clipboard, save path, fonts.
#![allow(unused_imports)]
use image::{ImageFormat, RgbaImage};
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

// ───────────────────────── system helpers ─────────────────────────

pub(crate) fn png_bytes(img: &RgbaImage) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png).ok();
    buf.into_inner()
}

pub(crate) fn wl_copy(data: &[u8], mime: &str) -> Result<(), String> {
    let mut child = Command::new("wl-copy")
        .args(["--type", mime])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("wl-copy: {e}"))?;
    child
        .stdin
        .take()
        .ok_or("no stdin")?
        .write_all(data)
        .map_err(|e| e.to_string())?;
    child.wait().map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn save_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let dir = PathBuf::from(home).join("Pictures");
    let _ = std::fs::create_dir_all(&dir);
    dir.join(format!("screenshot-{}.png", chrono::Local::now().format("%Y%m%d-%H%M%S")))
}

pub(crate) fn capture() -> Option<PathBuf> {
    // Non-interactive, full-desktop frame; the in-app selector crops it.
    let dir = std::env::temp_dir().join(format!("cosmic-snap-{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    let out = Command::new("cosmic-screenshot")
        .args(["--interactive=false", "--notify=false", "-s"])
        .arg(&dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let line = s.lines().last()?.trim().trim_start_matches("file://").to_string();
    let p = PathBuf::from(line);
    p.exists().then_some(p)
}

pub(crate) fn load_font_bytes() -> Option<Vec<u8>> {
    let out = Command::new("fc-match")
        .args(["-f", "%{file}", "sans-serif:bold"])
        .output()
        .ok()?;
    std::fs::read(String::from_utf8_lossy(&out.stdout).trim()).ok()
}

