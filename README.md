# Cosmic Snap

> *Perfectly balanced, as all screenshots should be.* 🫰

A small screenshot annotator for the [COSMIC](https://system76.com/cosmic) desktop, written in Rust.
Freeze the screen, drag a region, and annotate it in place with a floating toolbar.

The binary is called `cosmic-snap`.

## Features

- Frozen-screen region selector with crosshair guides and a live size readout (click for full screen)
- In-place editing: the selection stays bright, the rest of the screen is dimmed
- Tools: arrow, rectangle, ellipse, freehand pen, highlighter, redact, numbered markers, text
- Color palette, custom color picker, mouse-wheel tool size, undo/redo
- OCR: copy all text in the selection to the clipboard (Tesseract)
- Optional backdrop: gradient padding with rounded corners, applied on export
- Copy to the clipboard (closes the app) or save to `~/Pictures`

## Requirements

- COSMIC desktop (Wayland)
- `cosmic-screenshot` (ships with COSMIC) for the screen capture
- `wl-clipboard` for copying
- `fontconfig` (`fc-match`) to find a UI/annotation font
- Optional, for OCR: `tesseract` and a language pack such as `tesseract-data-eng`
- Rust toolchain to build (install with [rustup](https://rustup.rs))

On Arch-based systems:

```sh
sudo pacman -S wl-clipboard fontconfig tesseract tesseract-data-eng
```

## Build and install

```sh
git clone https://github.com/phoscoder/cosmic-snap.git
cd cosmic-snap
cargo build --release
install -Dm755 target/release/cosmic-snap ~/.local/bin/cosmic-snap
```

## Usage

Run `cosmic-snap` to capture the screen and select a region, or `cosmic-snap image.png` to annotate an existing image.

Bind it to a key in **COSMIC Settings → Keyboard → Keyboard Shortcuts → Custom Shortcuts**, using the full
path to the binary (for example `/home/you/.local/bin/cosmic-snap`) and a combo such as `Super+Shift+S`.

### Shortcuts

| Key | Action | Key | Action |
|---|---|---|---|
| `A` | Arrow | `Wheel` | Tool size |
| `R` | Rectangle | `O` | Copy text (OCR) |
| `E` | Ellipse | `B` | Toggle backdrop |
| `F` | Freehand | `Ctrl+Z` | Undo |
| `H` | Highlighter | `Ctrl+Shift+Z` | Redo |
| `D` | Redact | `Ctrl+C` | Copy and close |
| `C` | Numbered marker | `Ctrl+S` | Save |
| `T` | Text | `Esc` | Close |
| `1`–`8` | Color | `F1` | Hide the cheat sheet |

## Limitations

- Redact draws a solid black box; it does not blur or pixelate.
- No window-click selection, pins, crop or scroll capture yet.
- The region selector is a fullscreen window, not a layer-shell surface.

## License

[MIT](LICENSE)
