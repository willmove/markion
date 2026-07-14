# Implementation Plan: Cross-platform release packaging

## Overview

This change adds a release pipeline, not editor features. Four new/edited artifacts are the deliverable: `Packager.toml` (packaging source-of-truth), `.github/workflows/release.yml` (3-OS build matrix + release job), `Cargo.toml` metadata + release profile, and `LICENSE`. The first run that actually produces binaries happens on GitHub after the user pushes — these tasks prepare a config that compiles and validates locally where possible. No `src/` code is touched.

## Tasks

- [x] 1. Package metadata + release profile (`Cargo.toml`)
  - [x] 1.1 Add `description`, `license = "MIT"`, `authors = ["Willmove"]`, `repository = "https://github.com/willmove/markion"` to `[package]`.
  - [x] 1.2 Add `[profile.release]` with `lto = "thin"`, `codegen-units = 1`, `strip = true`. Leave panic behavior at default `unwind` so backtraces/GPUI panic hooks still work.
  - [x] 1.3 Extend `[package.metadata.bundle]` `icon` to `["assets/markion.icns", "assets/markion.ico", "assets/markion.png"]` and add `category`/`short_description`/`long_description` so a plain `cargo bundle` still works locally.
  - [x] 1.4 `cargo build` passes with the new profile (verified locally).
  - [x] _Requirements: release-packaging (release profile produces optimized binary)_

- [x] 2. LICENSE (`LICENSE`)
  - [x] 2.1 Add MIT LICENSE (copyright 2026 Willmove), since `Cargo.toml` declares `license = "MIT"` and packagers bundle the project root.
  - [x] _Requirements: release-packaging (license file present)_

- [x] 3. Packaging config (`Packager.toml`)
  - [x] 3.1 Create `Packager.toml` (top-level keys, no `[packager]` wrapper — verified against the crate's `Config` serde schema) with `product-name = "Markion"`, `identifier = "dev.markion.app"`, `version`, `category = "Productivity"`, `icons` (all three icon files), `resources = ["assets"]`, `before-packaging-command`, `binaries-dir = "target/release"`, `binaries = [{ path = "markion", main = true }]`, `out-dir = "dist"`.
  - [x] 3.2 Top-level `formats` array (default; overridden by CI `--formats` per OS) plus the per-platform option tables that actually exist: `[macos] minimum-system-version`, `[linux] generate-desktop-entry`, `[deb] depends`, top-level `[windows]` (`digest-algorithm`, `allow-downgrades`), and top-level `[nsis]` (`install-mode = "currentUser"`, `languages`, `display-language-selector`). Note: `[nsis]`/`[deb]` are top-level tables, NOT nested under `[windows]`/`[linux]`.
  - [x] _Requirements: release-packaging (packager config covers all three platforms)_

- [x] 4. Release workflow (`.github/workflows/release.yml`)
  - [x] 4.1 `build` matrix job: 3 entries (ubuntu-22.04/x86_64-unknown-linux-gnu, macos-latest/aarch64-apple-darwin, windows-latest/x86_64-pc-windows-msvc), each with a per-OS `formats` list and unique `artifact_name`.
  - [x] 4.2 Linux step installs gpui's native apt deps (clang/cmake/pkg-config/libwayland-dev/libxkbcommon-dev/libxcb*-dev/libvulkan-dev/libfontconfig1-dev/libglib2.0-dev/libssl-dev/libasound2-dev) from Zed's script/linux. macOS/Windows runners need no extra deps.
  - [x] 4.3 Each job: checkout → `dtolnay/rust-toolchain@stable` (with target) → `Swatinem/rust-cache@v2` → (Linux) apt install → `cargo build --release` (bare, no `--target`, so binary lands in `target/release/` matching `binaries-dir`) → install cargo-packager (`taiki-e/install-action@v2`) → `cargo packager --release --formats <formats>` → `upload-artifact@v4`.
  - [x] 4.4 `release` job: `needs: build`, `if: startsWith(github.ref,'refs/tags/v')`, `permissions: contents: write`, downloads all artifacts (`merge-multiple: true`) and creates a Release via `softprops/action-gh-release@v2` with `generate_release_notes: true`.
  - [x] 4.5 Triggers: `push: branches:[main]` + `tags:['v*']` + `pull_request` + `workflow_dispatch`; `concurrency` cancels superseded runs.
  - [x] _Requirements: release-packaging (native build per OS; installers published on tags)_

- [x] 5. Ignore packager output (`.gitignore`)
  - [x] 5.1 Add `/dist/` so cargo-packager output is never committed.
  - [x] _Requirements: release-packaging (build outputs excluded from VCS)_

- [x] 6. Verify and finalize
  - [x] 6.1 `cargo build` (verifies Cargo.toml release profile changes compile locally) — passes.
  - [x] 6.2 `openspec validate cross-platform-release-packaging` passes. All YAML/TOML config files validated syntactically (release.yml, Packager.toml, Cargo.toml).
  - [ ] 6.3 Manual step (deferred to user — outward-facing): push to GitHub; confirm the 3 `build` jobs go green and produce artifacts. Then `git tag v0.1.0 && git push --tags` to publish the first Release.
