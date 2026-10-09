# Design — improve-macos-distribution

## Why ad-hoc signing fixes the "damaged" wording

Gatekeeper distinguishes an *invalid* signature from a *valid but untrusted* one. The published v0.4.7 app carries the linker's ad-hoc Mach-O signature on the executable, but the bundle was never sealed as a bundle: `codesign -dv` shows `Sealed Resources=none` and `Info.plist=not bound`, and `spctl -a -t exec` fails with `code has no resources but signature indicates they must be present`. That invalid state is what recent macOS reports as "app is damaged and can't be opened" — a hard block with no override.

Re-sealing the identical bundle with `codesign --force --deep --sign -` (verified locally against the v0.4.7 DMG contents) turns the `spctl` verdict into a plain unsigned-app rejection. For that state Gatekeeper shows "can't be opened because Apple cannot check it for malicious software" and System Settings → Privacy & Security offers **Open Anyway**. Ad-hoc signing requires no Apple account, so it is free; it is not notarization, and the warning never disappears entirely.

cargo-packager 0.11.8 invokes `codesign --force -s <identity> [--options runtime] [--timestamp]` when `[macos] signing-identity` is set (verified in its source: `sign()` builds exactly those args; notarization is skipped with a warning when no Apple credentials exist). Passing `"-"` makes that invocation an ad-hoc seal. `--options runtime --timestamp` were verified compatible with identity `-` on this machine (`codesign` exits 0).

A CI step asserts `codesign --verify --deep --strict dist/Markion.app` and `Signature=adhoc`, so a packaging regression (e.g. a future cargo-packager behavior change) fails the build instead of republishing an unsealed "damaged" app.

## Cross-compiling x86_64 macOS on the arm64 runner

The per-OS GPU-backend constraint in the spec is about cross-OS compilation; compiling `x86_64-apple-darwin` on an arm64 macOS runner is same-OS cross-compilation against the same Metal SDK and is fully supported by rustc and the Xcode CLT. `macos-latest` is arm64; no Intel runner is needed (GitHub is retiring them).

Mechanics, from cargo-packager 0.11.8 source and CLI:

- `cargo packager --target <triple>` sets `config.target_triple`, which drives `target_arch()` → DMG name. For `x86_64` the name is `Markion_<version>_x64.dmg` (`"x86_64" => "x64"` in `dmg/mod.rs`).
- The binary lands in `target/x86_64-apple-darwin/release/markion`, so the x86_64 job passes `--binaries-dir target/x86_64-apple-darwin/release`. `binaries-dir` in `packager.toml` stays `target/release` for all other (native) jobs.
- `before-packaging-command = "cargo build --release"` would otherwise rebuild the *host* arch (and with a copied-binary approach could clobber it). Setting `CARGO_BUILD_TARGET=x86_64-apple-darwin` only for the x86_64 packaging step makes that command rebuild the same cross target — a no-op after the explicit build step. The env var is step-scoped so the MarkNice verification `cargo run` step keeps running natively.
- The arm64 macOS job and the Linux/Windows jobs keep today's commands verbatim; the packaging step is split by condition so only the x86_64 entry takes the new path.

## Mirror and update-check

`Markion_<version>_x64.dmg` is added to the mirror-repair `gh release download` patterns, the explicit OSS upload list, the HTTP-200 verification list, and the `manifest.json` assets map as `macos-x86_64`. The `sha256sum *.dmg` glob already covers it. The in-app updater's suffix map gains `("macos", "x86_64") => "_x64.dmg"`; today that combination falls through to the Release page, so the new mapping is a strict improvement. `update.json` remains Windows-only.

## Local-build caveat

This machine (no full Xcode) needs gpui's `runtime_shaders` feature for local builds; that is an uncommitted local `Cargo.toml` tweak and unrelated to CI, which always has full Xcode. `cargo check --target x86_64-apple-darwin` on this machine gates the "the crate compiles for Intel" claim before the change lands.

## Risks

- Gatekeeper behavior is Apple-controlled and tightens over time (Homebrew already removed `--no-quarantine`; macOS 15 removed right-click Open). Ad-hoc sealing keeps the documented **Open Anyway** path working today; if Apple later hardens further, Developer ID + notarization remains the only full fix (explicit non-goal).
- cargo-packager's signing behavior was verified from its 0.11.8 source, not executed in CI yet; the signature-verification step makes any divergence visible on the first `main` push rather than at release time.
