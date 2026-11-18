#!/usr/bin/env bash
# Prepare Developer ID signing and notarization for a macOS release job.
# Usage: configure-macos-signing.sh [packager.toml]

set -euo pipefail

config="${1:-packager.toml}"
github_env="${GITHUB_ENV:?GITHUB_ENV must be set}"
runner_temp="${RUNNER_TEMP:?RUNNER_TEMP must be set}"

secret_names=(
    APPLE_SIGNING_IDENTITY APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD
    APPLE_API_KEY APPLE_API_ISSUER APPLE_API_PRIVATE_KEY
)
missing=()
for name in "${secret_names[@]}"; do
    if [[ -z "${!name:-}" ]]; then
        missing+=("$name")
    fi
done
if (( ${#missing[@]} > 0 )); then
    echo "::error::Missing Apple release signing secrets: ${missing[*]}. See docs/macos-signing.md."
    exit 1
fi
if [[ "$APPLE_SIGNING_IDENTITY" != "Developer ID Application: "* ]]; then
    echo "::error::APPLE_SIGNING_IDENTITY must name a Developer ID Application certificate."
    exit 1
fi
if [[ ! "$APPLE_API_KEY" =~ ^[A-Za-z0-9]+$ ]]; then
    echo "::error::APPLE_API_KEY must be the alphanumeric App Store Connect key ID."
    exit 1
fi

python3 - "$config" <<'PY'
import json
import os
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
identity = json.dumps(os.environ["APPLE_SIGNING_IDENTITY"], ensure_ascii=False)
text, count = re.subn(
    r'(?m)^signing-identity[ \t]*=[ \t]*"-"[ \t]*$',
    lambda _: f"signing-identity = {identity}",
    path.read_text(encoding="utf-8"),
)
if count != 1:
    sys.exit(f"expected one ad-hoc signing-identity line in {path}, found {count}")
path.write_text(text, encoding="utf-8")
PY

# Keep the private key in a restricted directory outside the packaging tree.
key_dir="$(mktemp -d "${runner_temp}/markion-notary.XXXXXXXX")"
key_path="${key_dir}/AuthKey_${APPLE_API_KEY}.p8"
(umask 077 && printf '%s\n' "$APPLE_API_PRIVATE_KEY" > "$key_path")

# GitHub's multiline syntax preserves wrapped base64 and special characters.
export_env() {
    local name="$1" value="$2" delimiter
    delimiter="EOF_$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')"
    printf '%s<<%s\n%s\n%s\n' "$name" "$delimiter" "$value" "$delimiter" >> "$github_env"
}

export_env APPLE_CERTIFICATE "$APPLE_CERTIFICATE"
export_env APPLE_CERTIFICATE_PASSWORD "$APPLE_CERTIFICATE_PASSWORD"
export_env APPLE_API_KEY "$APPLE_API_KEY"
export_env APPLE_API_ISSUER "$APPLE_API_ISSUER"
export_env APPLE_API_KEY_PATH "$key_path"
echo "macOS signing mode: developer-id; notarize: true"
