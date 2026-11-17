# Markion v0.6.1 native acceptance record

Date/time: 2026-11-18, Asia/Shanghai. Stage: candidate development precheck; final package acceptance remains separate.

## Build and environment identity

- Source baseline: `ade570d41623176314402a5c05d8834e6838e8ce` plus the reviewed block-edit work and 0.6.1 metadata. The [source-check record](../release-v0.6.1-source-checks.json) retains the dirty diff identity, untracked-file manifest identity, tool versions and source verification. Reviewed application commit: `f7cb683`; initial metadata candidate: `c5bbd4f`. Corrected CI/docs candidate: `17db038`; the follow-up changes no application source or resources.
- Executable: copied `target/debug/markion.exe`, SHA-256 `2E80B7B385E56DF0E8F9A463E88D8E5A87004C3E1EBFDFD5927F2B161823C052`; PID 7728. This is not a release installer.
- Resource source: copied repository `assets/`, not a native package. Resource packaging is verified independently by CI; this precheck cannot establish installed-resource lookup.
- OS: Windows 11 Pro 10.0.26300 x86_64, Paper theme, English UI. Real system scale was not independently measured. No IME-candidate, browser-clipboard or Word-viewer test was performed.
- Fresh run: `target/tmp/release-v0.6.1/acceptance-run`; 160 fixture files. Manifest SHA-256 `f2ea48d67f62b383cdc92dce5772fa379cbb495f0dd93dc050ccae3b303723aa`. Separate APPDATA/LOCALAPPDATA under this run; auto-save, startup override and automatic update downloads disabled. The benign spacing change wrote only to the isolated config.

## Actual observations

1. Opened the native File menu and Preferences with pointer input. Appearance showed 100% line spacing; clicking plus showed 105%, reflowed the document and persisted `line_spacing = 105` under the isolated roaming profile. [Screenshot](spacing.png).
2. Switched to Source, clicked the synthetic paragraph and entered ` QA` using the native automation text API. Saved through Ctrl+S. Every UTF-8 byte matched an independently constructed expected file, including the complete unchanged Chinese table. [Saved source](source-saved.png).
3. One Ctrl+Z followed by Ctrl+S restored every original byte. [Restored source](undo-saved.png). The isolated process was closed and its window disappeared; existing user processes and daily configurations were not used.

The text API is not an actual IME composition/candidate exercise. An attempted Ctrl+End produced no observed navigation and was not counted as a passed navigation case; the edit used a directly clicked, visibly focused source caret. These limited steps do not pass the full D01, D09, E01 or P03 variant groups.

## Required matrix status

| Scope | Current result | Reason |
| --- | --- | --- |
| Windows NSIS/ZIP installed-package, real input/IME, table and DPI variants | NOT_RUN | Only the labeled development precheck above was executed; no dedicated installer/upgrade test account |
| macOS arm64 and actual Intel DMG interaction/Gatekeeper/IME | BLOCKED | No dedicated native release-test environment used in this run |
| Linux X11/Wayland DEB/RPM/AppImage interaction, IBus/Fcitx and CSD | BLOCKED | No dedicated native desktop environment used in this run |
| Browser/WeChat/Word, actual DOCX producer corpus and viewer layout | NOT_RUN | Full supported interoperability matrix not executed |
| Previous-version Windows/AppImage signed upgrade, Later/reopen, dirty refusal and failure recovery | NOT_RUN | Requires final signed metadata and dedicated isolated system environments |
| Five-hour wall-clock update soak and full native performance/input matrix | NOT_RUN | Automated deterministic scheduling tests and historical measurements do not establish these results |
| Automated workspace/bundle/examples/OpenSpec | PASS within CLI/GPUI scope | See [source check record](../release-v0.6.1-source-checks.json); 5 ignored tests remain ignored |
| Final packages, signatures, manifests and mirror | NOT_RUN | Await authoritative tag run and asset verification |

The 62-case template is a group inventory, not an expanded executed-variant count. No coverage percentage or full-suite native PASS is claimed. The per-variant matrix was not executed and remains unverified. Existing historical evidence is not attributed to this binary.

## Release-specific decision

Exception `V061-NATIVE-SCOPE`, authorized 2026-11-18 (Asia/Shanghai): **“允许本次例外，披露未验证范围并继续发布”**. The maintainer explicitly accepted the incomplete Windows installed-package/IME/DPI, macOS/Linux interaction, browser/WeChat/Word/producer and dedicated previous-version signed-upgrade matrix for v0.6.1. The [preflight](../release-v0.6.1-preflight.md) records scope and reason. This does not change NOT_RUN/BLOCKED to PASS or authorize an exception for another release. Automated native packaging/resource verification, no-GPU AppImage smoke, updater signing, all assets, hashes and both update channels remain mandatory. The final release notes disclose the unverified matrix.
