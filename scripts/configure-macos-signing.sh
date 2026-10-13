#!/usr/bin/env bash
# Select the macOS code-signing mode for a release-workflow job.
#
# Modes:
#   adhoc         No Apple secrets: keep packager.toml's `signing-identity = "-"`.
#                 The bundle is sealed but not notarized, so browser downloads
#                 are rejected by Gatekeeper on current macOS.
#   developer-id  APPLE_SIGNING_IDENTITY + APPLE_CERTIFICATE (base64 .p12) +
#                 APPLE_CERTIFICATE_PASSWORD: rewrite the working copy of
#                 packager.toml to that identity and hand the certificate to
#                 cargo-packager, which signs with the hardened runtime.
#   + notarize    Additionally APPLE_API_KEY + APPLE_API_ISSUER +
#                 APPLE_API_PRIVATE_KEY (App Store Connect API key, .p8 text),
#                 on v* tag or workflow_dispatch builds only: cargo-packager
#                 notarizes and staples the .app; scripts/notarize-macos-dmg.sh
#                 then notarizes and staples the DMG.
#
# Partial credential sets are an error, never a silent ad-hoc fallback.
# Exported variables go through $GITHUB_ENV only when set: cargo-packager
# treats an empty APPLE_CERTIFICATE as present and its keychain setup fails.
# Writes `mode` and `notarize` to $GITHUB_OUTPUT. See docs/macos-signing.md.
#
# Usage: configure-macos-signing.sh [packager.toml]

set -euo pipefail

config="${1:-packager.toml}"
github_env="${GITHUB_ENV:?GITHUB_ENV must be set}"
github_output="${GITHUB_OUTPUT:?GITHUB_OUTPUT must be set}"
runner_temp="${RUNNER_TEMP:?RUNNER_TEMP must be set}"

# True only for refs that publish (v* tags) or are run on purpose.
notarize_allowed=false
if [[ "${GITHUB_REF:-}" == refs/tags/v* || "${GITHUB_EVENT_NAME:-}" == "workflow_dispatch" ]]; then
    notarize_allowed=true
fi

# Echo the names of unset variables among the arguments.
missing() {
    local name
    for name in "$@"; do
        [[ -n "${!name:-}" ]] || printf '%s ' "$name"
    done
}

# Append NAME=VALUE to $GITHUB_ENV, using a heredoc delimiter so multi-line
# values (base64 wrapped at 64 columns) survive intact.
export_env() {
    local name="$1" value="$2" delimiter
    delimiter="EOF_$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')"
    {
        printf '%s<<%s\n' "$name" "$delimiter"
        printf '%s\n' "$value"
        printf '%s\n' "$delimiter"
    } >>"$github_env"
}

identity_vars=(APPLE_SIGNING_IDENTITY APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD)
notary_vars=(APPLE_API_KEY APPLE_API_ISSUER APPLE_API_PRIVATE_KEY)
identity_missing="$(missing "${identity_vars[@]}")"
notary_missing="$(missing "${notary_vars[@]}")"

mode=adhoc
notarize=false

if [[ "$identity_missing" != "${identity_vars[*]} " ]]; then
    if [[ -n "$identity_missing" ]]; then
        echo "::error::Incomplete Developer ID secrets; missing: ${identity_missing}(set all of ${identity_vars[*]} or none)"
        exit 1
    fi
    if [[ "$APPLE_SIGNING_IDENTITY" != "Developer ID Application:"* ]]; then
        echo "::error::APPLE_SIGNING_IDENTITY must be a \"Developer ID Application: <Name> (<TEAMID>)\" identity; other certificate types cannot be notarized for distribution outside the Mac App Store"
        exit 1
    fi
    mode=developer-id
elif [[ "$notary_missing" != "${notary_vars[*]} " ]]; then
    echo "::error::Notarization secrets are set but the Developer ID secrets (${identity_vars[*]}) are not; notarization requires a Developer ID signature"
    exit 1
fi

if [[ "$mode" == developer-id ]]; then
    python3 - "$config" <<'PY'
import json, os, re, sys

path = sys.argv[1]
with open(path, encoding="utf-8") as f:
    text = f.read()
# A JSON string is a valid TOML basic string for any identity text.
line = "signing-identity = " + json.dumps(os.environ["APPLE_SIGNING_IDENTITY"])
text, count = re.subn(r'(?m)^signing-identity[ \t]*=[ \t]*"-"[ \t]*$', lambda _: line, text)
if count != 1:
    sys.exit(f"expected exactly one ad-hoc signing-identity line in {path}, found {count}")
with open(path, "w", encoding="utf-8") as f:
    f.write(text)
PY
    export_env APPLE_CERTIFICATE "$APPLE_CERTIFICATE"
    export_env APPLE_CERTIFICATE_PASSWORD "$APPLE_CERTIFICATE_PASSWORD"

    if [[ -n "$notary_missing" && "$notary_missing" != "${notary_vars[*]} " ]]; then
        echo "::error::Incomplete notarization secrets; missing: ${notary_missing}(set all of ${notary_vars[*]} or none)"
        exit 1
    fi
    if [[ -z "$notary_missing" && "$notarize_allowed" == true ]]; then
        if [[ ! "$APPLE_API_KEY" =~ ^[A-Za-z0-9]+$ ]]; then
            echo "::error::APPLE_API_KEY must be the App Store Connect key ID (alphanumeric), not the .p8 contents"
            exit 1
        fi
        key_dir="${runner_temp}/private_keys"
        key_path="${key_dir}/AuthKey_${APPLE_API_KEY}.p8"
        mkdir -p "$key_dir"
        (umask 077 && printf '%s\n' "$APPLE_API_PRIVATE_KEY" >"$key_path")
        export_env APPLE_API_KEY "$APPLE_API_KEY"
        export_env APPLE_API_ISSUER "$APPLE_API_ISSUER"
        export_env APPLE_API_KEY_PATH "$key_path"
        notarize=true
    fi
fi

echo "macOS signing mode: ${mode}; notarize: ${notarize}"
if [[ "${GITHUB_REF:-}" == refs/tags/v* && "$notarize" != true ]]; then
    echo "::warning::This tagged release's macOS DMGs are NOT notarized (mode: ${mode}). Browser downloads will be blocked by Gatekeeper; configure the Apple secrets described in docs/macos-signing.md."
fi

{
    echo "mode=${mode}"
    echo "notarize=${notarize}"
} >>"$github_output"
