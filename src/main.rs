// cosnap — a small screenshot annotator for COSMIC (Wayland).
// Capture the screen, drag-select a region, annotate it in place, copy/save.
// Design inspired by Omasnap (MIT, Tobi Lütke).

mod app;
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
                eprintln!("cosnap: capture failed");
                std::process::exit(1);
            }
        },
    };
    let base = match image::open(&path) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            eprintln!("cosnap: cannot open {}: {e}", path.display());
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
        .with_app_id("cosnap")
        .with_title("Cosnap")
        .with_fullscreen(true)
        .with_decorations(false);
    let opts = eframe::NativeOptions { viewport: vb, ..Default::default() };
    eframe::run_native(
        "cosnap",
        opts,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, base, font_bytes, select)))),
    )
    .unwrap();
}

