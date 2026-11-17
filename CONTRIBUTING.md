# Contributing to cosmic-snap

Thanks for your interest! Bug reports, ideas and pull requests are all welcome.

## Getting set up

```sh
git clone https://github.com/phoscoder/cosmic-snap.git
cd cosmic-snap
cargo build --release
./target/release/cosnap
```

You need a COSMIC (Wayland) session to run the full capture flow, plus `wl-clipboard`.
`cosnap some-image.png` skips the capture step and is handy for testing the editor.

## Before opening a pull request

1. Keep changes focused: one logical change per pull request, and small atomic commits.
2. Run `cargo fmt` and `cargo clippy` and fix anything they report.
3. Make sure `cargo build --release` succeeds without new warnings.
4. Try the change by hand (capture, select, annotate, copy and save), and say what you tested in the PR.
5. Update the README if you change user-visible behavior or shortcuts.

## Commit messages

Use short, imperative subjects with an optional type prefix, for example:

- `feat: add pixelate mode to redact`
- `fix: keep text inside the selection`
- `docs: clarify install steps`

## Code layout

| File | Purpose |
|---|---|
| `src/shapes.rs` | Annotation shapes, tools and geometry |
| `src/raster.rs` | Software rasteriser that bakes annotations into the exported image |
| `src/system.rs` | Capture, clipboard, save path and font lookup |
| `src/ui.rs` | Palette, vector icons and panel styling |
| `src/app.rs` | Region selection and the in-place editor |

Anything drawn on screen must also be drawn by the rasteriser, or exports will not match the editor.

## Reporting bugs

Open an issue with your COSMIC version, what you did, what you expected and what happened.
For security problems, see [SECURITY.md](SECURITY.md) instead of filing a public issue.

By contributing you agree that your contributions are licensed under the [MIT License](LICENSE).
Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).
