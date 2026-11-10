#!/usr/bin/env bash
# Removes the Wayland libraries that get packed into the AppImage, then
# repacks it in place.
#
# Why: the AppImage is built on Ubuntu 22.04 and picks up its old
# libwayland-*.so files, but the graphics driver (Mesa's libEGL) always comes
# from the user's own system. On up-to-date systems (Arch, CachyOS, Fedora...)
# the new Mesa next to the old libwayland fails with
#   "Could not create default EGL display: EGL_BAD_PARAMETER. Aborting..."
# and the window stays blank. Every Linux desktop already has libwayland
# (GTK 3 needs it), so the system's own copy is used instead.
#
# Usage: scripts/fix-appimage.sh path/to/App.AppImage
set -euo pipefail

appimage=$(realpath "$1")
tool="${LINUXDEPLOY_PLUGIN_APPIMAGE:-$HOME/.cache/tauri/linuxdeploy-plugin-appimage.AppImage}"
if [ ! -x "$tool" ]; then
  echo "AppImage packing tool not found at $tool (it is downloaded by 'pnpm tauri build --bundles appimage')." >&2
  exit 1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"

# No FUSE needed for either step (GitHub runners have none).
export APPIMAGE_EXTRACT_AND_RUN=1
"$appimage" --appimage-extract >/dev/null

removed=$(find squashfs-root -name 'libwayland-*.so*' -print -delete)
if [ -z "$removed" ]; then
  echo "No bundled libwayland found; leaving $appimage as it is."
  exit 0
fi
echo "Removed from the AppImage:"
echo "$removed" | sed 's/^/  /'

ARCH=x86_64 LDAI_OUTPUT=fixed.AppImage OUTPUT=fixed.AppImage "$tool" --appdir squashfs-root >/dev/null

# Check the result before replacing the original.
rm -rf squashfs-root
./fixed.AppImage --appimage-extract >/dev/null 2>&1 || true
[ -x squashfs-root/AppRun ] || { echo "Repacked AppImage could not be opened." >&2; exit 1; }
if find squashfs-root -name 'libwayland-*.so*' | grep -q .; then
  echo "Repacked AppImage still contains libwayland; something went wrong." >&2
  exit 1
fi
mv fixed.AppImage "$appimage"
chmod +x "$appimage"
echo "Fixed: $appimage"
