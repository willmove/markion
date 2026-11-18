## Why

macOS users who download the ad-hoc-signed DMGs from v0.4.8/v0.4.9 in a browser are still told "Markion is damaged and can't be opened", and they can only launch it after running `xattr -cr` in Terminal. Inspection of the published `Markion_0.4.9_aarch64.dmg` shows the bundle is correctly sealed: the CodeDirectory has the `adhoc` and `runtime` flags, Info.plist and `CodeResources` are bound, all 3,170 code-page hashes match, and all 59 sealed resource hashes match. So the block is not a packaging defect. It is Gatekeeper policy. A quarantined app on current macOS (Sequoia/Tahoe, Apple silicon) that carries no Developer ID and no notarization ticket is rejected, and an ad-hoc signature does not reliably unlock the System Settings **Open Anyway** route. The `improve-macos-distribution` change assumed it would, based on a local `spctl` assessment of a bundle that was not quarantined.

Developer ID signing plus Apple notarization is the only supported way for a downloaded app to open with a plain double-click. The user then sees only the standard "downloaded from the Internet — Open?" confirmation. cargo-packager 0.11.8, which we already use, implements both. The workflow just never supplies the credentials.

## What Changes

- Public `v*` tags and manual signing rehearsals require the complete Apple credential set. A missing credential blocks publication. Pushes to the main branch and pull requests continue to produce ad-hoc CI packages without accessing Apple credentials.
- Verification mounts the final DMG read-only and checks the signed, stapled app inside it, including its architecture. Maintainers can run the same check against the public download.
- Before release publication, launch the app installed from each final signed DMG on a runner with the matching CPU architecture. Both apps must open a visible window.
- cargo-packager uses the Developer ID to sign the app with the hardened runtime and a secure timestamp, then uses the App Store Connect API key to notarize and staple the app. The workflow signs, notarizes and staples the disk image too.
- The macOS signature gate becomes mode-aware:
  - Ad-hoc builds must still pass `codesign --verify --deep --strict` with `Signature=adhoc`.
  - Developer ID builds must show a `Developer ID Application` authority, a team identifier, the hardened runtime and a timestamp on the embedded app.
  - Both the app and DMG must pass `stapler validate`. With Gatekeeper enabled, `spctl` must report `source=Notarized Developer ID`.
- Documentation:
  - A new maintainer guide, `docs/macos-signing.md`, covers enrolling, creating the certificate and API key, setting the secrets, and verifying the result.
  - `docs/release-process.md` covers notarization verification and the release-note wording.
  - The README (English and Simplified Chinese) and FAQ Gatekeeper guidance are corrected. They no longer promise that **Open Anyway** works for ad-hoc builds; they say notarized releases open directly, and keep `xattr -cr` / `curl -LO` as the fallback for non-notarized builds.

Non-goals: Mac App Store distribution, a universal binary, in-app macOS self-update, Windows Authenticode signing, and buying the Apple Developer Program membership (a maintainer action this change only documents).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `release-packaging`: public macOS installers require Developer ID signing and notarization. Development builds retain ad-hoc signing. The macOS packaging gate verifies the final disk image and embedded app. This delta also carries forward the Intel installer and mirror requirements from `improve-macos-distribution` while superseding its public ad-hoc signing requirements.

## Impact

- Code: `.github/workflows/release.yml` (macOS signing configuration, DMG notarization, mode-aware signature verification), new `scripts/configure-macos-signing.sh` and `scripts/notarize-macos-dmg.sh`. `packager.toml` changes only in comments, and the committed identity stays `"-"`.
- Docs: new `docs/macos-signing.md`; `docs/release-process.md`, `docs/faq.md`, `README.md` (English and its Chinese section), `README.zh-CN.md`.
- Secrets: `APPLE_SIGNING_IDENTITY`, `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_API_KEY`, `APPLE_API_ISSUER`, and `APPLE_API_PRIVATE_KEY` are required for public tags and signing rehearsals. Their values stay in GitHub Actions Secrets and the signing runner.
- Invariants: no application code changes. The cached-per-version Markdown state, the editor and the gpui-free member crates are untouched. Windows, Linux, the updater signing flow and the OSS mirror (byte-for-byte copies of the release assets) are unchanged.
