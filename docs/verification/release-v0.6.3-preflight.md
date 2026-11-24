# Markion v0.6.3 release verification

**Status: PREPARATION — not yet a release-readiness claim.**

The comparison base is tag `v0.6.2`. The latest published GitHub Release before this work is `v0.6.1`: the [v0.6.2 tag run](https://github.com/willmove/markion/actions/runs/44379616857) failed when Apple notarization lost its network route, so no v0.6.2 Release was published. The public tag has been preserved.

## Release scope

- Native Arch/Omarchy-compatible Linux x86_64 pacman packaging, an Arch installation/resource gate, GitHub/OSS distribution, and distribution-family browser-download selection.
- Fenced code-block formatting with built-in JSON and configured external formatters, optional format-on-paste, localized controls, and undo/source-fidelity protections.
- Math rendering and pointer/source fidelity in native GFM table cells.
- Theme-consistent table interactions and collapsible source editors.

Existing documents, preferences and workspace formats require no migration. External formatters are not bundled. Pacman packages use manual installation and are not added to the signed NSIS/AppImage updater manifest.

## Source and version identity

- Candidate implementation commit: `ab6e121dc32968f9e05309a53adbcfdd8ec76dca` (`feat: add Arch pacman release package`). Its CI packages intentionally exercise the packaging path with the commit's inherited `0.6.2` metadata; they are not final v0.6.3 assets.
- The subsequent dedicated release commit carries the synchronized root, workspace, packager and lockfile versions at `0.6.3`.
- Intended asset: `markion-0.6.3-1-x86_64.pkg.tar.zst`.

## Automated evidence

- `bash -n scripts/build-linux-arch.sh`: PASS.
- Focused update tests: 22 passed, 0 failed, 1 ignored network test (`cargo test app::update::tests`). Arch, Omarchy, Manjaro, EndeavourOS, Debian, missing os-release, AppImage-on-Arch and missing-pacman-asset mapping branches are covered hermetically.
- `cargo check --workspace`: PASS; Cargo metadata reports Markion-owned packages at `0.6.3`.
- Pacman download/install guidance: PASS for all eight maintained READMEs; every table links the exact v0.6.3 pacman asset and includes its `sudo pacman -U` command.
- `scripts/verify-examples.ps1`: PASS (18 Markdown examples, index coverage and local references).
- `python docs/testing/fixtures/prepare.py check`: PASS (62 cases, blank result rows and 14 documentation pages; explicitly no native tests).
- Isolated fixture preparation: PASS; `target/tmp/release-qa/run-001` contains 160 source/working files, manifest SHA-256 `f2ea48d67f62b383cdc92dce5772fa379cbb495f0dd93dc050ccae3b303723aa`. Native acceptance remains NOT_RUN.
- PowerShell parser check for `scripts/verify-packaged-workspace.ps1`: PASS. `release.yml` YAML parse: PASS.
- The workflow's actual embedded manifest and SHA-256 generators: PASS in temporary local fixtures for a new release with pacman, a historical mirror repair without pacman, and mandatory failure when a tag lacks pacman (using GitHub Actions' `bash -eo pipefail` semantics). `update.json` remains limited to `windows-x86_64`/NSIS and `linux-x86_64`/AppImage.
- `scripts/check-quality.ps1` with PowerShell 7.7.0: PASS on 2026-11-24 — formatting, workspace Clippy, complete workspace tests, pinned MarkNice verification, and strict OpenSpec validation (99 items passed, 0 failed). Existing Clippy warnings remain non-fatal; no unrelated warning cleanup was included.

The previous [main packaging run](https://github.com/willmove/markion/actions/runs/45317341636) passed Windows x64, macOS arm64/x64, Linux x86_64 and AppImage smoke. It predates pacman support and is baseline evidence only, not candidate acceptance.

## Candidate and native acceptance

The first [non-tag candidate workflow](https://github.com/willmove/markion/actions/runs/45325857255) built Windows and both macOS packages successfully, and Arch `makepkg` plus `pacman -U` succeeded with 117 files and zero missing files. It then failed because the gate assumed a specific desktop filename instead of locating the installed launcher. The gate now requires exactly one desktop file and verifies its `Exec=...markion` command; the corrected candidate run is PENDING. Pacman artifact download and complete packaged-resource verification have not yet passed. Final tagged-package checks and previous-published-version signed upgrade checks are also PENDING.

Real Omarchy desktop interaction, IME, browser/Word/WeChat interoperability and other manual matrix rows are NOT_RUN unless separately recorded below. Arch container installation is CLI evidence only and MUST NOT fill those rows. No release-specific maintainer exception for incomplete required acceptance has been recorded.

## Final publication checklist

- [ ] Complete workspace and release-only diff validation.
- [ ] Successful non-tag native candidate matrix with all four Linux package formats.
- [ ] Applicable candidate and final P01/P02/P06/P07 evidence, or an explicit v0.6.3-specific maintainer exception with disclosed scope.
- [ ] Successful tag workflow, including both notarized macOS installed-app launches, AppImage smoke, updater signing and publication.
- [ ] Eight GitHub packages, NSIS/AppImage signatures, `update.json`, and curated bilingual notes verified.
- [ ] Eight OSS package mirrors, both signatures, both manifests, `packager.toml` and `sha256sums.txt` verified with matching versions, URLs and signatures.
