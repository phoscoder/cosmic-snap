#!/bin/sh
# Build and install cosmic-snap, its icon and its launcher entry.
# Installs under ~/.local by default; override with PREFIX=/usr/local (may need sudo).
set -e
cd "$(dirname "$0")"
PREFIX="${PREFIX:-$HOME/.local}"

cargo build --release
install -Dm755 target/release/cosmic-snap "$PREFIX/bin/cosmic-snap"
install -Dm644 assets/cosmic-snap.svg "$PREFIX/share/icons/hicolor/scalable/apps/cosmic-snap.svg"
tmp="$(mktemp)"
# Absolute Icon= path: works even when the icon theme cache or index is missing for $PREFIX.
sed -e "s|^Exec=.*|Exec=$PREFIX/bin/cosmic-snap|" \
    -e "s|^Icon=.*|Icon=$PREFIX/share/icons/hicolor/scalable/apps/cosmic-snap.svg|" \
    assets/cosmic-snap.desktop > "$tmp"
install -Dm644 "$tmp" "$PREFIX/share/applications/cosmic-snap.desktop"
rm -f "$tmp"

command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t "$PREFIX/share/icons/hicolor" 2>/dev/null || true
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database -q "$PREFIX/share/applications" 2>/dev/null || true

echo "Installed to $PREFIX (binary, icon and launcher)."
