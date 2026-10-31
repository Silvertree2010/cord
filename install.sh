#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
SRC="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

BIN="$PREFIX/bin/cord"
SHARE="$PREFIX/share/cord"
DESK="$PREFIX/share/applications/cord.desktop"
ICON="$PREFIX/share/icons/hicolor/scalable/apps/cord.svg"

have() { command -v "$1" >/dev/null 2>&1; }

check_deps() {
  local missing=()
  have cargo || missing+=("rust")
  have cmake || missing+=("cmake")
  have clang || missing+=("clang")
  have qmake6 || missing+=("qt6-base" "qt6-declarative" "qt6-multimedia")
  have pkg-config || missing+=("pkgconf")
  if have pkg-config; then
    pkg-config --exists gstreamer-1.0 || missing+=("gstreamer")
    pkg-config --exists gstreamer-base-1.0 || missing+=("gst-plugins-base")
    pkg-config --exists opus || missing+=("opus")
    pkg-config --exists libpipewire-0.3 || missing+=("pipewire")
    pkg-config --exists libsecret-1 || missing+=("libsecret")
  fi
  if have gst-inspect-1.0; then
    gst-inspect-1.0 pipewiresrc >/dev/null 2>&1 || missing+=("gst-plugin-pipewire")
    gst-inspect-1.0 videoconvert >/dev/null 2>&1 || missing+=("gst-plugins-base")
    gst-inspect-1.0 matroskamux >/dev/null 2>&1 || missing+=("gst-plugins-good")
    gst-inspect-1.0 h264parse >/dev/null 2>&1 || missing+=("gst-plugins-bad")
  else
    missing+=("gstreamer" "gst-plugins-good" "gst-plugins-bad")
  fi
  have wl-copy || have xclip || missing+=("wl-clipboard")
  if [ "${#missing[@]}" -gt 0 ]; then
    echo "Missing dependencies, Arch packages:"
    printf '%s\n' "${missing[@]}" | sort -u | sed 's/^/  /'
    echo "Optional: playerctl (media controls), gst-plugin-va or nvidia drivers (hardware encode)"
    return 1
  fi
}

install_all() {
  check_deps || echo "Continuing, the build may fail." >&2
  (cd "$SRC" && cargo build --release -p cord)
  install -Dm755 "$SRC/target/release/cord" "$BIN"
  rm -rf "$SHARE/qml"
  mkdir -p "$SHARE"
  cp -r "$SRC/crates/app/qml" "$SHARE/qml"
  install -Dm644 "$SRC/packaging/cord.desktop" "$DESK"
  install -Dm644 "$SRC/packaging/cord.svg" "$ICON"
  echo "Installed to $PREFIX"
  case ":$PATH:" in
    *":$PREFIX/bin:"*) ;;
    *) echo "Note: $PREFIX/bin is not in PATH" ;;
  esac
}

uninstall_all() {
  rm -f "$BIN" "$DESK" "$ICON"
  rm -rf "$SHARE/qml"
  rmdir "$SHARE" 2>/dev/null || true
  echo "Removed cord from $PREFIX"
}

case "${1:-}" in
  --uninstall) uninstall_all ;;
  "") install_all ;;
  *) echo "usage: PREFIX=<dir> $0 [--uninstall]" >&2; exit 2 ;;
esac
