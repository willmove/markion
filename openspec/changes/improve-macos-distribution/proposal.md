## Why

macOS users who download the published DMG are shown the hard Gatekeeper block "Markion is damaged and can't be opened" with no override button, and the currently documented right-click → Open / "Open Anyway" path does not appear for it. Diagnosis of the published v0.4.7 `.app` found the cause: the executable carries only the linker's ad-hoc signature while the bundle itself is unsealed (`codesign -dv` reports `Sealed Resources=none`, `Info.plist=not bound`), which `spctl` classifies as an invalid signature (`code has no resources but signature indicates they must be present`) — the state that Gatekeeper words as "damaged". Re-sealing the same bundle with `codesign --force --deep --sign -` locally changes the verdict to a plain unsigned-app rejection, for which System Settings → Privacy & Security → **Open Anyway** is available.

Separately, Intel Mac users have no runnable build: the macOS release targets `aarch64-apple-darwin` only, and the README/FAQ claim "Intel Macs run via Rosetta" is wrong (Rosetta 2 exists only on Apple Silicon and translates x86_64 → arm64, never the reverse).

## What Changes

- macOS `.app` bundles (both architectures) are sealed with an ad-hoc code signature during packaging (`[macos] signing-identity = "-"` in `packager.toml`; cargo-packager runs `codesign --force -s - --options runtime --timestamp`). No Apple account or certificate is involved. Gatekeeper still warns, but the wording becomes "can't be checked for malicious software" with a working **Open Anyway** path; the `xattr -cr` fallback stays documented.
- The CI matrix gains a macOS x86_64 job that cross-compiles `x86_64-apple-darwin` on the arm64 `macos-latest` runner (same-OS cross-compilation of the Metal backend is supported) and packages `Markion_<version>_x64.dmg`. A post-packaging step verifies the ad-hoc signature of the produced `.app` so a packaging regression fails the build instead of shipping another "damaged" release.
- The Aliyun OSS mirror uploads/verifies the x64 DMG and lists it in `manifest.json` as `macos-x86_64`.
- The in-app update check maps `("macos", "x86_64")` to `_x64.dmg` so Intel builds deep-link their asset instead of falling back to the Release page.
- README (all seven language files), `docs/faq.md`, and `docs/release-process.md` are updated: the Rosetta claim is corrected, the Intel DMG appears in platform tables and asset checklists, the Gatekeeper guidance describes the ad-hoc-signed behavior ("Open Anyway" works; `xattr -cr` and the quarantine-free `curl -LO` download remain documented alternatives).

Non-goals: Apple Developer ID signing and notarization ($99/year membership), a universal (arm64+x86_64) binary, Mac App Store distribution, and Homebrew casks (Homebrew removed `--no-quarantine` and now requires Gatekeeper-passing casks, so it is no longer a free distribution path).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `release-packaging`: the build matrix gains cross-compiled macOS x86_64; macOS installers are ad-hoc code-signed (still not Developer-ID signed or notarized); the mirror and update-check asset lists gain the x64 DMG; the "Intel via Rosetta" requirement is replaced with a native Intel build.

## Impact

- Code: `.github/workflows/release.yml` (matrix entry, cross-compile build/packaging steps, signature verification, mirror upload/verify/manifest/repair lists), `packager.toml` (`[macos] signing-identity`, comments), `src/app/update.rs` (asset suffix map + tests), `Cargo.toml` only if `cargo check --target x86_64-apple-darwin` surfaces target-conditional issues.
- Docs: `README.md`, `README.zh-CN.md`, `README.zh-TW.md`, `README.ja.md`, `README.fr.md`, `README.de.md`, `README.es.md`, `docs/faq.md`, `docs/release-process.md`.
- Invariants: no Markdown/editor code paths change; the update-check change is confined to the existing asset-suffix map and runs on the existing off-main-thread path. The Windows NSIS signing/mirror flow is untouched.
