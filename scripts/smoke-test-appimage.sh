#!/usr/bin/env bash
# Start an AppImage the way the AppImage catalog test does (Xvfb + icewm,
# `firejail --noprofile --net=none --appimage`) and check that it shows a
# window that is not blank.
#
# Run as a regular user on a machine with Xvfb, icewm, xdotool, xwininfo,
# ImageMagick, FUSE 2 and a setuid firejail. With MARKION_SMOKE_REQUIRE_NO_VULKAN_DRIVER=1
# the test refuses to run when a system Vulkan driver manifest exists, so a
# pass proves the AppImage works without one.
#
# Usage: smoke-test-appimage.sh <path-to-appimage> <output-dir>
# Writes app.log and screenshot.png to <output-dir>; exits 1 on failure.

set -uo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: $0 <path-to-appimage> <output-dir>" >&2
    exit 1
fi

appimage_src="$(realpath "$1")"
out_dir="$(realpath -m "$2")"
mkdir -p "$out_dir"
display_number="${SMOKE_DISPLAY:-99}"
window_timeout="${SMOKE_WINDOW_TIMEOUT:-30}"
# icewm's own windows are visible too, so look for the application's title.
window_name="${SMOKE_WINDOW_NAME:-^Markion}"

pids=()
cleanup() {
    for pid in "${pids[@]}"; do
        kill "$pid" 2>/dev/null
    done
    sleep 1
    [[ -n "${run_dir:-}" ]] && rm -rf "$run_dir"
}
trap cleanup EXIT

echo "== Vulkan drivers on this system"
icd_manifests="$(find /usr/share/vulkan/icd.d /etc/vulkan/icd.d /usr/local/share/vulkan/icd.d \
    -name '*.json' 2>/dev/null || true)"
echo "${icd_manifests:-none}"
if [[ "${MARKION_SMOKE_REQUIRE_NO_VULKAN_DRIVER:-0}" == 1 && -n "$icd_manifests" ]]; then
    echo "ERROR: a system Vulkan driver is installed; this test must run without one" >&2
    exit 1
fi
for name in VK_DRIVER_FILES VK_ICD_FILENAMES MARKION_SOFTWARE_VULKAN; do
    unset "$name"
done

run_dir="$(mktemp -d)"
cp "$appimage_src" "$run_dir/Markion.AppImage"
chmod +x "$run_dir/Markion.AppImage"
cd "$run_dir"

export DISPLAY=":$display_number"
Xvfb "$DISPLAY" -screen 0 800x600x24 >/dev/null 2>&1 &
pids+=("$!")
sleep 2
mkdir -p "$HOME/.icewm" "$HOME/.local/share/appimagekit"
printf 'ShowTaskBar = 0\nTaskBarAutoHide = 1\n' > "$HOME/.icewm/preferences"
touch "$HOME/.local/share/appimagekit/no_desktopintegration"
icewm >/dev/null 2>&1 &
pids+=("$!")
sleep 1

echo "== Starting $(basename "$appimage_src") with firejail --appimage"
firejail --quiet --noprofile --net=none --appimage ./Markion.AppImage > "$out_dir/app.log" 2>&1 &
app_pid=$!
pids=("$app_pid" "${pids[@]}")

result=1
for _ in $(seq 1 "$window_timeout"); do
    if ! kill -0 "$app_pid" 2>/dev/null; then
        echo "ERROR: the application exited instead of showing a window"
        break
    fi
    if windows="$(timeout 5 xdotool search --onlyvisible --name "$window_name" 2>/dev/null)" && [[ -n "$windows" ]]; then
        result=0
        break
    fi
    sleep 1
done

if [[ $result -eq 0 ]]; then
    sleep 3
    echo "== Visible windows"
    for window in $(xdotool search --onlyvisible --name "$window_name"); do
        echo "  $window: $(xdotool getwindowname "$window")"
    done
    import -window root "$out_dir/screenshot.png"
    # Share of the most common color; the catalog rejects near-uniform screenshots.
    share="$(convert "$out_dir/screenshot.png" -alpha off -depth 8 -format '%c' histogram:info:- \
        | sort -rn | awk -v size="$(identify -format '%w %h' "$out_dir/screenshot.png")" \
            'NR == 1 { top = $1 + 0 } END { split(size, s, " "); printf "%d", 100 * top / (s[1] * s[2]) }')"
    echo "Screenshot: ${share}% one color"
    if [[ "$share" -ge 95 ]]; then
        echo "ERROR: the window appears to be empty"
        result=1
    elif ! kill -0 "$app_pid" 2>/dev/null; then
        echo "ERROR: the application exited after showing a window"
        result=1
    fi
elif kill -0 "$app_pid" 2>/dev/null; then
    echo "ERROR: no window appeared within ${window_timeout} seconds"
    import -window root "$out_dir/screenshot.png" 2>/dev/null
fi

echo "== Application output"
cat "$out_dir/app.log"
if [[ $result -eq 0 ]]; then
    echo "PASS: the AppImage shows a window"
else
    echo "FAIL"
fi
exit "$result"
