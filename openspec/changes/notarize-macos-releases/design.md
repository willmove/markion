# Design — notarize-macos-releases

## Diagnosis: the shipped signature is valid; the policy rejects it

The published `Markion_0.4.9_aarch64.dmg` was downloaded and its HFS+ payload extracted on Linux (7-Zip). A small Mach-O parser then read the embedded signature of `Markion.app/Contents/MacOS/markion`:

| Check | Result |
| --- | --- |
| CodeDirectory version / flags | `0x20500` / `0x10002` (`adhoc` + `runtime`) |
| Identifier / Team ID | `dev.markion.app` / none |
| Special slot −1 (Info.plist) | bound, hash matches |
| Special slot −3 (CodeResources) | bound, hash matches |
| Code pages | 3,170 checked, 0 mismatches |
| `CodeResources` `files2` | 59 entries checked, 0 mismatches, no unsealed files |

The `improve-macos-distribution` change fixed the v0.4.7 failure (an unsealed bundle with an invalid signature), and that fix is intact in the shipped DMG. The remaining "damaged" verdict is Gatekeeper's response to a *quarantined* app whose signature carries no Developer ID and has no notarization ticket. Current macOS releases on Apple silicon refuse such apps by default, with the "move to Trash" dialog. The previous design's evidence was a `spctl -a` run on a non-quarantined bundle, which does not reproduce the double-click dialog. No free, no-Terminal bypass is reliable: macOS 15 removed Control-click → Open, and the remaining override (allowing apps from "Anywhere") itself needs `sudo spctl`.

The only path that meets "download, open, run" is **Developer ID Application signing + hardened runtime + notarization (+ stapling)**. That needs a paid Apple Developer Program membership, which the maintainer must buy. This change makes the pipeline use it as soon as the credentials exist.

## What cargo-packager 0.11.8 already does (source-verified)

- `[macos] signing-identity` is read only from the config file; there is no CLI or env override. When `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD` are present, `codesign::try_sign` imports the `.p12` into a temporary keychain and signs with `codesign --force -s <identity> --keychain <tmp> --options runtime --timestamp`. Main executables and the bundle get `--options runtime`.
- After signing the `.app`, `notarize_auth()` reads `APPLE_KEYCHAIN_PROFILE`, then `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID`, then `APPLE_API_KEY` / `APPLE_API_ISSUER` / `APPLE_API_KEY_PATH`. With credentials, it zips the bundle with `ditto`, runs `notarytool submit --wait`, fails on any status other than `Accepted` (printing the notary log), and staples the `.app`. Without credentials it only logs a warning and skips.
- The DMG is assembled afterwards from `dist/Markion.app` (now stapled) and signed with the same identity, but it is **not** notarized.
- Pitfall: `APPLE_CERTIFICATE` set to an *empty string* still counts as present (`var_os` returns `Some("")`), and keychain setup then fails. These variables must therefore be absent, not empty, in ad-hoc mode. Writing them to `$GITHUB_ENV` only when they are set keeps them absent; a step-level `env:` mapping from possibly-empty secrets would not.

## Mode selection (`scripts/configure-macos-signing.sh`)

The script runs in both macOS matrix jobs before packaging and selects one of three modes:

| Developer ID secrets | API key secrets | Ref | Mode |
| --- | --- | --- | --- |
| none | none | any | `adhoc` (today's behavior) |
| all 3 | any | branch / PR | `developer-id`, not notarized (no notary round-trip on every push) |
| all 3 | all 3 | `v*` tag or `workflow_dispatch` | `developer-id` + notarize |
| some, not all | — | any | **fail** (misconfiguration) |
| none | some or all | any | **fail** (notarization needs Developer ID) |

In `developer-id` mode the script performs the following steps:

1. It rewrites the `signing-identity` line of the working copy of `packager.toml`. The value is TOML-escaped through `json.dumps`, and the committed file keeps `"-"`.
2. It checks the identity starts with `Developer ID Application:`. A distribution or development certificate cannot be notarized for outside-store distribution.
3. It exports `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD` through `$GITHUB_ENV` using random heredoc delimiters, because base64 `.p12` text may be multi-line.

When notarization is enabled it writes the `.p8` key to `$RUNNER_TEMP/private_keys/AuthKey_<id>.p8` with mode 0600 and exports `APPLE_API_KEY`, `APPLE_API_ISSUER` and `APPLE_API_KEY_PATH`. Only the API-key method is supported: it is Apple's recommended CI credential, it is not tied to a person's Apple ID, and it avoids app-specific passwords. Values never reach the log; GitHub masks secrets, and the script prints only the mode.

The `mode` and `notarize` step outputs drive the later steps. A `v*` tag build that does not notarize emits a `::warning::` saying its macOS DMGs will be blocked after a browser download.

## DMG notarization (`scripts/notarize-macos-dmg.sh`)

When `notarize == true`, this step runs after packaging and before verification:

1. It submits `dist/*.dmg` (exactly one per job) with `notarytool submit --wait --timeout 1h --output-format json` using the same API key.
2. It parses `status` and requires `Accepted`. On any other status it prints the notary log and fails.
3. It staples the DMG with `xcrun stapler staple`.

The `.app` inside the DMG is already stapled, so Gatekeeper can verify it offline. Stapling the DMG as well avoids a separate online check when it is mounted.

## Verification gate (mode-aware)

The gate always requires `codesign --verify --deep --strict dist/Markion.app` to pass. It then reads `codesign -dvv` and applies the checks for the selected mode:

- `adhoc`: requires `Signature=adhoc` (unchanged).
- `developer-id`:
  - requires an `Authority=Developer ID Application:` line;
  - requires a `TeamIdentifier` that is not `not set`;
  - requires `runtime` in the CodeDirectory flags;
  - requires `codesign --verify --strict` on the DMG.
- `notarize`:
  - requires `xcrun stapler validate` to pass on the `.app` and the DMG;
  - requires `spctl --assess --type execute -vv` on the `.app` to report `source=Notarized Developer ID`;
  - requires `spctl --assess --type open --context context:primary-signature -vv` on the DMG to report the same.

A run that should have shipped a notarized DMG but did not therefore fails before any artifact upload.

## Compatibility and risk

- The hardened runtime already ships today (flags `0x10002`), and v0.4.9 runs once its quarantine is cleared, so Developer ID signing adds no new runtime restriction. Markion has no JIT engine, no unsigned plug-in loading and no `DYLD_*` use, so it needs no entitlements file.
- The x86_64 bundle is cross-built and stapled on the arm64 runner. `codesign`, `notarytool`, `stapler` and `spctl` assessments are architecture-independent.
- The OSS mirror copies the GitHub artifacts byte for byte, so mirrored DMGs are the notarized, stapled ones. `sha256sums.txt` is computed after stapling, because stapling happens before the artifact upload.
- A notary outage or a stuck submission fails the tag build (`--timeout 1h`) instead of shipping an un-notarized DMG. The workflow can be re-run on the same tag.
- A certificate expiry or revocation fails `codesign`, which fails the build. Rotation is documented in `docs/macos-signing.md`.
