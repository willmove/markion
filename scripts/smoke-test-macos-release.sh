#!/usr/bin/env bash
# Check a signed DMG on the architecture it targets, then launch its app.
# Usage: smoke-test-macos-release.sh <path-to-dmg> <arm64|x86_64> <evidence-dir>

set -euo pipefail

if [[ $# -ne 3 || ( "${2:-}" != arm64 && "${2:-}" != x86_64 ) ]]; then
    echo "Usage: $0 <path-to-dmg> <arm64|x86_64> <evidence-dir>" >&2
    exit 1
fi

dmg="$1"
arch="$2"
evidence="$3"
if [[ "$(uname -m)" != "$arch" ]]; then
    echo "This launch check requires a native $arch runner." >&2
    exit 1
fi
mkdir -p "$evidence"
sw_vers | tee "$evidence/macos.txt"
shasum -a 256 "$dmg" | tee "$evidence/sha256.txt"
bash "$(dirname "$0")/verify-macos-release.sh" "$dmg" "$arch" 2>&1 | tee "$evidence/signature.txt"

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/markion-launch.XXXXXXXX")"
mount_dir="$work_dir/mounted"
mkdir "$mount_dir"
mounted=false
cleanup() {
    if [[ "$mounted" == true ]]; then
        hdiutil detach -quiet "$mount_dir"
    fi
}
trap cleanup EXIT
hdiutil attach -readonly -nobrowse -quiet -mountpoint "$mount_dir" "$dmg"
mounted=true
app="$work_dir/Markion.app"
ditto "$mount_dir/Markion.app" "$app"
hdiutil detach -quiet "$mount_dir"
mounted=false
syspolicy_check distribution "$app" 2>&1 | tee "$evidence/distribution.txt"

/usr/bin/open -n \
    --env "XDG_CONFIG_HOME=$work_dir/config" \
    --env "XDG_CACHE_HOME=$work_dir/cache" \
    --stdout "$evidence/stdout.txt" --stderr "$evidence/stderr.txt" \
    "$app"

swift - "$app" <<'SWIFT' | tee "$evidence/launch.txt"
import AppKit
import CoreGraphics

let installedPath = URL(fileURLWithPath: CommandLine.arguments[1]).resolvingSymlinksInPath().path
for _ in 0..<40 {
    let apps = NSRunningApplication.runningApplications(withBundleIdentifier: "dev.markion.app")
    let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID)
        as? [[String: Any]] ?? []
    if let app = apps.first(where: { $0.bundleURL?.resolvingSymlinksInPath().path == installedPath }),
       windows.contains(where: {
           $0[kCGWindowOwnerPID as String] as? pid_t == app.processIdentifier
               && $0[kCGWindowLayer as String] as? Int == 0
       }) {
        print("The installed Markion app opened a visible window (PID \(app.processIdentifier)).")
        exit(0)
    }
    Thread.sleep(forTimeInterval: 1)
}
fputs("No visible window from the installed Markion app.\n", stderr)
exit(1)
SWIFT
screencapture -x "$evidence/window.png" || echo "Could not capture the runner display."
