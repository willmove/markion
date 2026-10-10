#!/usr/bin/env bash
# Fix AppImage permissions after cargo-packager build.
#
# Problem: cargo-packager 0.11.8 produces AppImages with restrictive
# permissions (AppRun 0744, dirs 0700), which fail the AppImage catalog test
# when the image runs inside a firejail sandbox as a non-owner user.
#
# Solution: Extract the AppImage, normalize all permissions to world-readable
# (dirs 0755, AppRun and the main binary 0755, regular files 0644), then
# repack with appimagetool.
#
# Usage: fix-appimage-permissions.sh <path-to-appimage>

set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <path-to-appimage>" >&2
    exit 1
fi

appimage="$1"
if [[ ! -f "$appimage" ]]; then
    echo "AppImage not found: $appimage" >&2
    exit 1
fi

appimage_abs="$(realpath "$appimage")"
appimage_dir="$(dirname "$appimage_abs")"
appimage_name="$(basename "$appimage_abs")"
temp_extract="$(mktemp -d)"

cleanup() {
    rm -rf "$temp_extract"
}
trap cleanup EXIT

echo "Extracting $appimage_name..."
cd "$temp_extract"
chmod +x "$appimage_abs"
"$appimage_abs" --appimage-extract >/dev/null

extract_dir="$temp_extract/squashfs-root"
if [[ ! -d "$extract_dir" ]]; then
    echo "Extraction failed: squashfs-root not found" >&2
    exit 1
fi

echo "Fixing permissions..."
cd "$extract_dir"

# Fix all directories: 0755 (rwxr-xr-x)
find . -type d -exec chmod 0755 {} +

# Fix AppRun: must be executable by everyone
if [[ -f AppRun ]]; then
    chmod 0755 AppRun
    echo "  - Fixed AppRun: 0755"
fi

# Fix the main binary: 0755
# The binary is typically in usr/bin/ for AppImages built by cargo-packager
if [[ -d usr/bin ]]; then
    find usr/bin -type f -exec chmod 0755 {} +
    echo "  - Fixed binaries in usr/bin/: 0755"
fi

# Fix all regular files: 0644 (rw-r--r--)
# Keep executables (AppRun, binaries in usr/bin/, shell scripts) at 0755
find . -type f ! -name AppRun ! -path "*/usr/bin/*" \
    ! -name "*.sh" -exec chmod 0644 {} +

echo "Repacking with appimagetool..."
# appimagetool expects to be run on the extracted squashfs-root
cd "$temp_extract"

# Set ARCH for appimagetool (it reads this env var)
export ARCH=x86_64

# Repack: use gzip compression (same as cargo-packager default)
# The -n flag skips the desktop integration prompt
appimagetool --comp gzip -n "$extract_dir" "$appimage_abs" >/dev/null 2>&1

echo "AppImage permissions fixed and repacked: $appimage_name"

# Verify the result
echo "Verifying AppRun permissions in repacked AppImage..."
verify_temp="$(mktemp -d)"
cd "$verify_temp"
"$appimage_abs" --appimage-extract >/dev/null 2>&1
apprun_perms="$(stat -c '%a' squashfs-root/AppRun)"
if [[ "$apprun_perms" != "755" ]]; then
    rm -rf "$verify_temp"
    echo "ERROR: AppRun permissions after repack are $apprun_perms, expected 755" >&2
    exit 1
fi
rm -rf "$verify_temp"
echo "Verification passed: AppRun is 0755"
