# Tasks: prefer-appimage-update-on-arch

## 1. Sequencing

- [x] 1.1 Before archiving this change, archive `add-updater-manifest-fallback` — verified archived and synced on 2026-11-18. Rebase the delta on the current requirement, including archived background-update behavior, without dropping existing scenarios.

## 2. Implementation

- [x] 2.1 In `src/app/update.rs`, add a distribution-family helper that classifies Linux from `/etc/os-release` contents (`ID` or any space-separated `ID_LIKE` token equal to `arch` → pacman-based; missing or unparseable → deb-based) and thread it through `browser_download_url` so Linux x86_64 offers `_x86_64.AppImage` on pacman-based systems and `_amd64.deb` otherwise
- [x] 2.2 Add hermetic unit tests over fixture `os-release` contents: `ID=arch`, `ID=omarchy`/`ID_LIKE=arch`, `ID=manjaro` + `ID_LIKE=arch`, `ID=debian`, and a missing file; keep the existing cross-platform mapping tests passing unchanged
- [x] 2.3 Run `cargo test --workspace` and confirm zero failures

## 3. Validation

- [x] 3.1 Run `openspec validate prefer-appimage-update-on-arch`
- [ ] 3.2 On the Omarchy machine, confirm "Check for Updates…" on a newer release offers the AppImage asset (may be verified during the next release's manual checks)

## Release validation repair — 2026-11-18

The v0.6.1 preflight initially failed strict validation because this older MODIFIED block omitted five scenarios added by the archived background-update change. Rebased the block on the current stable requirement while preserving the distribution-family rule and all existing scenarios. `openspec validate prefer-appimage-update-on-arch --strict` passes; no application code or stable specs changed. Native Omarchy verification remains pending.
