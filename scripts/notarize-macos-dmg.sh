#!/usr/bin/env bash
# Notarize and staple a Developer ID-signed DMG.
#
# cargo-packager notarizes and staples the .app before building the DMG, but
# only signs the DMG itself. Notarizing the disk image too lets Gatekeeper
# verify it offline when mounted. Uses the App Store Connect API key exported
# by scripts/configure-macos-signing.sh (APPLE_API_KEY, APPLE_API_ISSUER,
# APPLE_API_KEY_PATH). Fails unless Apple returns "Accepted".
#
# Usage: notarize-macos-dmg.sh <path-to-dmg>

set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <path-to-dmg>" >&2
    exit 1
fi

dmg="$1"
if [[ ! -f "$dmg" ]]; then
    echo "DMG not found: $dmg" >&2
    exit 1
fi

auth=(
    --key "${APPLE_API_KEY_PATH:?APPLE_API_KEY_PATH must be set}"
    --key-id "${APPLE_API_KEY:?APPLE_API_KEY must be set}"
    --issuer "${APPLE_API_ISSUER:?APPLE_API_ISSUER must be set}"
)

echo "Submitting $(basename "$dmg") for notarization..."
# --wait returns once Apple has a verdict; a non-Accepted status is checked
# below because notarytool may still exit 0 for an Invalid submission.
result="$(xcrun notarytool submit "$dmg" "${auth[@]}" --wait --timeout 1h --output-format json)"
echo "$result"

read -r submission_id status < <(python3 -c '
import json, sys
data = json.load(sys.stdin)
print(data.get("id", ""), data.get("status", ""))
' <<<"$result")

if [[ "$status" != "Accepted" ]]; then
    echo "::error::Notarization of $(basename "$dmg") finished with status '${status:-unknown}'"
    if [[ -n "$submission_id" ]]; then
        xcrun notarytool log "$submission_id" "${auth[@]}" || true
    fi
    exit 1
fi

xcrun stapler staple -v "$dmg"
