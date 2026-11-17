// cosmic-snap — a small screenshot annotator for COSMIC (Wayland).
// Capture the screen, drag-select a region, annotate it in place, copy/save.

mod app;
mod notify;
mod raster;
mod shapes;
mod system;
mod ui;

use eframe::egui;
use std::path::PathBuf;

fn main() {
    let arg = std::env::args().nth(1);
    let select = arg.is_none();
    let path = match arg {
        Some(p) => PathBuf::from(p),
        None => match system::capture() {
            Some(p) => p,
            None => {
                eprintln!("cosmic-snap: capture failed");
                notify::error("Capture failed", "cosmic-screenshot did not return an image");
                std::process::exit(1);
            }
        },
    };
    let base = match image::open(&path) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            eprintln!("cosmic-snap: cannot open {}: {e}", path.display());
            notify::error("Could not open image", &format!("{}: {e}", path.display()));
            std::process::exit(1);
        }
    };
    if select {
        let _ = std::fs::remove_file(&path);
        if let Some(d) = path.parent() {
            let _ = std::fs::remove_dir(d);
        }
    }
    let font_bytes = system::load_font_bytes();
    let vb = egui::ViewportBuilder::default()
        .with_app_id("cosmic-snap")
        .with_title("Cosmic Snap")
        .with_fullscreen(true)
        .with_decorations(false);
    let opts = eframe::NativeOptions { viewport: vb, ..Default::default() };
    eframe::run_native(
        "cosmic-snap",
        opts,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, base, font_bytes, select)))),
    )
    .unwrap();
}

