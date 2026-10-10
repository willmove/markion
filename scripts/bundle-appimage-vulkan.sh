#!/usr/bin/env bash
# Add the software Vulkan stack built by build-lavapipe.sh to an AppImage.
#
# The stack lands in <AppDir>/usr/lib/markion-vulkan/, where Markion looks for
# it (relative to usr/bin/markion) when the host Vulkan stack cannot start the
# renderer. Run this before fix-appimage-permissions.sh, which normalizes the
# permissions of the repacked image, and before the AppImage is signed.
#
# Usage: bundle-appimage-vulkan.sh <path-to-appimage> <stack-dir>

set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: $0 <path-to-appimage> <stack-dir>" >&2
    exit 1
fi

appimage_abs="$(realpath "$1")"
stack_dir="$(realpath "$2")"
for required in libvulkan_lvp.so libvulkan.so.1 lvp_icd.x86_64.json VERSIONS; do
    if [[ ! -f "$stack_dir/$required" ]]; then
        echo "Software Vulkan stack is incomplete: $stack_dir/$required is missing" >&2
        exit 1
    fi
done

temp_extract="$(mktemp -d)"
cleanup() {
    rm -rf "$temp_extract"
}
trap cleanup EXIT

echo "Extracting $(basename "$appimage_abs")..."
cd "$temp_extract"
chmod +x "$appimage_abs"
"$appimage_abs" --appimage-extract >/dev/null
extract_dir="$temp_extract/squashfs-root"
if [[ ! -x "$extract_dir/usr/bin/markion" ]]; then
    echo "Extraction failed: usr/bin/markion not found" >&2
    exit 1
fi

target="$extract_dir/usr/lib/markion-vulkan"
rm -rf "$target"
mkdir -p "$target"
cp -R "$stack_dir/." "$target/"
echo "Bundled software Vulkan stack ($(tr '\n' ' ' < "$target/VERSIONS")):"
du -sh "$target"

echo "Repacking with appimagetool..."
export ARCH=x86_64
appimagetool --comp gzip -n "$extract_dir" "$appimage_abs" >/dev/null 2>&1
echo "AppImage repacked: $(du -h "$appimage_abs" | cut -f1) $(basename "$appimage_abs")"
