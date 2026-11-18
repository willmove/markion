#!/usr/bin/env bash
# Verify a signed release DMG and the application users will copy out of it.
# Usage: verify-macos-release.sh <path-to-dmg> <arm64|x86_64>

set -euo pipefail

if [[ $# -ne 2 || ( "${2:-}" != arm64 && "${2:-}" != x86_64 ) ]]; then
    echo "Usage: $0 <path-to-dmg> <arm64|x86_64>" >&2
    exit 1
fi
dmg="$1"
expected_arch="$2"

codesign --verify --strict --verbose=2 "$dmg"
image_signature="$(codesign -dvv "$dmg" 2>&1)"
grep -q '^Authority=Developer ID Application:' <<< "$image_signature" || {
    echo "DMG must be signed with Developer ID Application: $dmg" >&2
    exit 1
}
xcrun stapler validate "$dmg"

mount_dir="$(mktemp -d "${TMPDIR:-/tmp}/markion-release.XXXXXXXX")"
mounted=false
cleanup() {
    if [[ "$mounted" == true ]]; then
        hdiutil detach -quiet "$mount_dir" || return
    fi
    rmdir "$mount_dir"
}
trap cleanup EXIT
hdiutil attach -readonly -nobrowse -quiet -mountpoint "$mount_dir" "$dmg"
mounted=true
app="$mount_dir/Markion.app"

codesign --verify --deep --strict --verbose=2 "$app"
app_signature="$(codesign -dvv "$app" 2>&1)"
printf '%s\n' "$app_signature"
grep -q '^Authority=Developer ID Application:' <<< "$app_signature"
grep -Eq '^TeamIdentifier=[A-Z0-9]{10}$' <<< "$app_signature"
grep -Eq '^CodeDirectory .*flags=0x[0-9a-f]+\([^)]*runtime' <<< "$app_signature"
grep -q '^Timestamp=' <<< "$app_signature"
xcrun stapler validate "$app"

arch="$(lipo -archs "$app/Contents/MacOS/markion")"
if [[ "$arch" != "$expected_arch" ]]; then
    echo "Expected a $expected_arch executable in $dmg, found $arch" >&2
    exit 1
fi

if [[ "$(spctl --status 2>&1)" == 'assessments disabled' ]]; then
    echo "Gatekeeper assessments are disabled on this Mac. Signatures and tickets passed validation."
else
    app_assessment="$(spctl --assess --type execute -vv "$app" 2>&1)" || {
        echo "$app_assessment" >&2
        exit 1
    }
    dmg_assessment="$(spctl --assess --type open --context context:primary-signature -vv "$dmg" 2>&1)" || {
        echo "$dmg_assessment" >&2
        exit 1
    }
    printf '%s\n%s\n' "$app_assessment" "$dmg_assessment"
    grep -q '^source=Notarized Developer ID' <<< "$app_assessment"
    grep -q '^source=Notarized Developer ID' <<< "$dmg_assessment"
fi

echo "Verified notarized Markion DMG and embedded $expected_arch app: $dmg"
