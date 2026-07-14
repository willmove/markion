## Why

Markion has no release pipeline. Today the only way to get the app is to clone the repo and `cargo run`, which excludes non-developer users and every platform except the contributor's host. To distribute installable desktop builds for Windows, macOS, and Linux we need a reproducible build + packaging pipeline — and because Markion depends on `gpui` (Zed's GPU UI framework), which uses a distinct native GPU backend per platform (DirectX/Vulkan/Metal), the binaries can only be produced by building natively on each target OS. This change introduces that pipeline as a GitHub Actions matrix and wires the standard multi-platform packager (`cargo-packager`) to emit NSIS Windows installers, macOS `.app`/`.dmg`, and Linux `.deb`/`.AppImage`.

## What Changes

- **Release workflow (`.github/workflows/release.yml`, new).** A three-OS matrix (`ubuntu-22.04` × `x86_64-unknown-linux-gnu`, `macos-latest` × `aarch64-apple-darwin`, `windows-latest` × `x86_64-pc-windows-msvc`) that, per platform: installs OS-native build deps (Linux only), `cargo build --release --target <t>`, runs `cargo-packager` to produce installers, and uploads them as artifacts. A separate `release` job runs only on `v*` tags, merges all platform artifacts, and attaches them to a GitHub Release with auto-generated notes.
- **Packaging config (`Packager.toml`, new).** Source-of-truth for `cargo-packager`: product name (`Markion`), bundle identifier (`dev.markion.app`), version, category (`Productivity`), icons (`markion.ico`/`.icns`/`.png`), resources, `before_each_package_command`, binaries/out dirs, and per-platform format + option tables (macOS `app`+`dmg`, Linux `deb`+`appimage`, Windows `nsis` with `currentUser` install mode).
- **Cargo metadata (`Cargo.toml`).** Add `description`, `license`, `authors`, `repository`; add a tuned `[profile.release]` (`lto = "thin"`, `codegen-units = 1`, `strip = true`) for a smaller, faster distributable; extend the legacy `[package.metadata.bundle]` `icon` array to all three icon files so a plain `cargo bundle` still works.
- **License (`LICENSE`, new).** MIT, since `Cargo.toml` now declares `license = "MIT"` and packagers expect a license file.
- **`.gitignore`.** Ignore `/dist/` (packager output).
- **Signing: none.** Builds are unsigned (no paid code-signing certificate). macOS Gatekeeper and Windows SmartScreen will warn end users on first launch; users bypass manually. Documented as a known limitation. Non-goals: no code signing, no notarization, no macOS universal binary (Intel users run via Rosetta), no auto-update channel, no cargo-bundle (superseded by cargo-packager), no ARM Windows/Linux targets.

## Capabilities

### New Capabilities
- `release-packaging`: A reproducible GitHub Actions pipeline that produces signed-off, distributable per-platform installers (Windows NSIS `.exe`, macOS `.app`+`.dmg`, Linux `.deb`+`.AppImage`) from the same source on every push and publishes them to a GitHub Release on version tags, using native per-OS builds plus `cargo-packager`.

### Modified Capabilities
<!-- None — this change adds a new release capability; it does not alter existing editor behavior. -->

## Impact

- **New files:** `.github/workflows/release.yml`, `Packager.toml`, `LICENSE`.
- **Edited files:** `Cargo.toml` (metadata + `[profile.release]` + bundle icons), `.gitignore` (`/dist/`).
- **No source-code changes:** `src/` is untouched; no editor behavior, no cached-per-version Markdown invariants affected.
- **CI/infra:** Consumes GitHub Actions minutes (3 OS jobs per run; macOS arm64 runner is the most expensive). Adding `Swatinem/rust-cache@v2` mitigates rebuild cost. The release job requires `contents: write` permission (granted in-workflow).
- **Invariants touched:** None of the project's architectural invariants (cached derived Markdown state, memoized highlighting, bounded file-tree rows) are on this path.
- **Distributable shape:** Each release tag yields up to 5 assets — `Markion-0.1.0-windows-x86_64-NSIS-setup.exe` (naming may vary), `Markion.app` + `Markion-0.1.0.dmg` (macOS arm64), `markion_0.1.0_amd64.deb` + `Markion-0.1.0-x86_64.AppImage` (Linux). Exact filenames follow cargo-packager conventions.
- **Outward-facing:** Pushing a `v*` tag creates a public GitHub Release. Contributors should tag deliberately.
