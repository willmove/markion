## Why

macOS users who download the ad-hoc-signed DMGs from v0.4.8/v0.4.9 in a browser are still told "Markion is damaged and can't be opened", and they can only launch it after running `xattr -cr` in Terminal. Inspection of the published `Markion_0.4.9_aarch64.dmg` shows the bundle is correctly sealed: the CodeDirectory has the `adhoc` and `runtime` flags, Info.plist and `CodeResources` are bound, all 3,170 code-page hashes match, and all 59 sealed resource hashes match. So the block is not a packaging defect. It is Gatekeeper policy. A quarantined app on current macOS (Sequoia/Tahoe, Apple silicon) that carries no Developer ID and no notarization ticket is rejected, and an ad-hoc signature does not reliably unlock the System Settings **Open Anyway** route. The `improve-macos-distribution` change assumed it would, based on a local `spctl` assessment of a bundle that was not quarantined.

Developer ID signing plus Apple notarization is the only supported way for a downloaded app to open with a plain double-click. The user then sees only the standard "downloaded from the Internet — Open?" confirmation. cargo-packager 0.11.8, which we already use, implements both. The workflow just never supplies the credentials.

## What Changes

- The release workflow gains an optional **Developer ID** signing mode for both macOS jobs. When the repository secrets `APPLE_SIGNING_IDENTITY`, `APPLE_CERTIFICATE` (base64 `.p12`) and `APPLE_CERTIFICATE_PASSWORD` are set, `packager.toml`'s ad-hoc `signing-identity = "-"` is overridden at build time. cargo-packager then signs the `.app` with the hardened runtime and a secure timestamp, and signs the DMG.
- On `v*` tag builds and manual `workflow_dispatch` builds, when App Store Connect API key secrets (`APPLE_API_KEY`, `APPLE_API_ISSUER`, `APPLE_API_PRIVATE_KEY`) are also set, the `.app` is notarized and stapled by cargo-packager. The DMG is then notarized and stapled by a new workflow step, so both the disk image and the app carry offline-verifiable tickets.
- The macOS signature gate becomes mode-aware:
  - Ad-hoc builds must still pass `codesign --verify --deep --strict` with `Signature=adhoc`.
  - Developer ID builds must show a `Developer ID Application` authority, a team identifier and the hardened runtime.
  - Notarized builds must pass `stapler validate` for the `.app` and the DMG, and `spctl` must report `source=Notarized Developer ID`.
- Partial secret sets fail the job instead of silently degrading. A tag build that falls back to ad-hoc emits a workflow warning that the macOS DMGs will be blocked after a browser download.
- Without any Apple secrets, behavior is unchanged: the free ad-hoc seal, the same artifacts and the same names. Fork pull requests, which get no secrets, keep working.
- Documentation:
  - A new maintainer guide, `docs/macos-signing.md`, covers enrolling, creating the certificate and API key, setting the secrets, and verifying the result.
  - `docs/release-process.md` covers notarization verification and the release-note wording.
  - The README (English and Simplified Chinese) and FAQ Gatekeeper guidance are corrected. They no longer promise that **Open Anyway** works for ad-hoc builds; they say notarized releases open directly, and keep `xattr -cr` / `curl -LO` as the fallback for non-notarized builds.

Non-goals: Mac App Store distribution, a universal binary, in-app macOS self-update, Windows Authenticode signing, and buying the Apple Developer Program membership (a maintainer action this change only documents).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `release-packaging`: macOS installers move from "always ad-hoc" to "Developer ID signed and notarized when credentials are configured, ad-hoc otherwise". The macOS packaging gate verifies the configured mode. The documentation requirement drops the claim that ad-hoc builds are always overridable through Open Anyway. This delta builds on the requirement as renamed by `improve-macos-distribution`, which must be archived first.

## Impact

- Code: `.github/workflows/release.yml` (macOS signing configuration, DMG notarization, mode-aware signature verification), new `scripts/configure-macos-signing.sh` and `scripts/notarize-macos-dmg.sh`. `packager.toml` changes only in comments, and the committed identity stays `"-"`.
- Docs: new `docs/macos-signing.md`; `docs/release-process.md`, `docs/faq.md`, `README.md` (English and its Chinese section), `README.zh-CN.md`.
- Secrets: new optional repository secrets `APPLE_SIGNING_IDENTITY`, `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_API_KEY`, `APPLE_API_ISSUER`, `APPLE_API_PRIVATE_KEY`. None appear in the repository or logs.
- Invariants: no application code changes. The cached-per-version Markdown state, the editor and the gpui-free member crates are untouched. Windows, Linux, the updater signing flow and the OSS mirror (byte-for-byte copies of the release assets) are unchanged.
