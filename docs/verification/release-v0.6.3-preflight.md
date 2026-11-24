# Markion v0.6.3 release verification

**Status: CANDIDATE READY UNDER DISCLOSED EXCEPTION V063-EX-01 — final tag and publication verification remain pending.**

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
- Final `cargo test --workspace` on the release-only diff: PASS; the desktop suite reported 897 passed, 0 failed, 2 ignored, and all other workspace suites completed with zero failures. Existing ignored/network-dependent checks were not relabeled.

The previous [main packaging run](https://github.com/willmove/markion/actions/runs/45317341636) passed Windows x64, macOS arm64/x64, Linux x86_64 and AppImage smoke. It predates pacman support and is baseline evidence only, not candidate acceptance.

## Candidate and native acceptance

The first [non-tag candidate workflow](https://github.com/willmove/markion/actions/runs/45325857255) built Windows and both macOS packages successfully, and Arch `makepkg` plus `pacman -U` succeeded with 117 files and zero missing files. It then failed because the gate assumed a specific desktop filename instead of locating the installed launcher. That failure is retained here; the corrected gate requires exactly one desktop file and verifies its `Exec=...markion` command.

The [corrected candidate run](https://github.com/willmove/markion/actions/runs/45327814800) at `07db678091d7f567f0a8b1982250610afd2e3697`: **PASS**. Windows x64, macOS arm64/x64 and Ubuntu x86_64 packaging passed, as did the no-GPU AppImage smoke. Arch `makepkg` and `pacman -U` passed with 117 files and zero missing files; the installed launcher was `/usr/share/applications/markion.desktop`. DEB, RPM, AppImage and pacman each passed the complete examples/MarkNice packaged-resource checks. The first `gh run download` attempt timed out after 30 minutes without leaving a usable archive. A subsequent HTTP Range reader verified the artifact's ZIP central directory and selectively downloaded only the pacman member, avoiding an hour-long full-artifact transfer. The ZIP contains DEB, RPM, AppImage and pacman. The pacman member passed ZIP CRC/size verification and local tar/Zstandard inspection. Candidate package SHA-256: `728aa2b4e8af6a0bc17bd98de280d0b2fe1ad778a7b4c3fa6857c253c956728b`; packaged `usr/bin/markion` SHA-256: `74297971f9362fde19649569e650beb0c8c170d654fa7e5286e131d56c825717`; packaged MarkNice manifest SHA-256: `e45d0d19db043a43f05e02a5a7577211cc2e5fa3e977a90d2b1349806d156f42`. Metadata, all declared dependencies, the desktop launcher and required resources passed. Final tagged-package checks and previous-published-version signed upgrade checks remain PENDING.

The only local WSL distribution is `JoyWord-R7-Ubuntu`; no native Arch/Omarchy desktop environment is available to this run. Real Omarchy desktop interaction, IME, browser/Word/WeChat interoperability and other manual matrix rows are NOT_RUN unless separately recorded below. Arch container installation is CLI evidence only and MUST NOT fill those rows. ### Release-specific exception V063-EX-01

On 2026-11-24, after reviewing the candidate evidence and the explicit BLOCKED/NOT_RUN list, the maintainer authorized a one-time exception for v0.6.3 with this exact statement:

> 我授权 v0.6.3 对上述未完成验收项使用一次性例外，并继续发布。

The exception covers the disclosed unrun native Arch/Omarchy desktop launch/data-retention and offline copy/edit interactions, the real Omarchy update dialog, real IME and browser/Word/WeChat interoperability, and previous-published-version Windows/AppImage signed upgrades. It does not relabel any result as PASS and does not waive final asset existence, signature, manifest, mirror, or workflow verification. It is not reusable by another release.

The isolated run report (`target/tmp/release-qa/run-001/release-report.md`) and expanded `results.csv` record the pacman-specific candidate scope: four CLI_CHECK rows PASS (Arch install, packaged resources, hermetic update mapping, candidate four-package integrity), three OS_INPUT rows BLOCKED (native launch/data retention, offline copy/edit, real Omarchy update UI), and final-tag P07 NOT_RUN. These artifacts are retained locally and are not committed. The candidate may proceed to tagging under V063-EX-01; final native smoke, asset, signature and mirror verification remain mandatory before completion.

## Final publication checklist

- [x] Complete workspace and release-only diff validation; all 10 workspace packages resolve to 0.6.3 and no target tag/Release collision exists.
- [x] Successful non-tag native candidate matrix with all four Linux package formats.
- [x] Applicable candidate evidence and explicit v0.6.3-specific maintainer exception V063-EX-01 with disclosed scope; final P01/P02/P06/P07 results remain pending and are not relabeled.
- [ ] Successful tag workflow, including both notarized macOS installed-app launches, AppImage smoke, updater signing and publication.
- [ ] Eight GitHub packages, NSIS/AppImage signatures, `update.json`, and curated bilingual notes verified.
- [ ] Eight OSS package mirrors, both signatures, both manifests, `packager.toml` and `sha256sums.txt` verified with matching versions, URLs and signatures.
