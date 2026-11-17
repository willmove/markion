# Markion v0.6.1 release preflight

Prepared 2026-11-17; verified 2026-11-18 (Asia/Shanghai). Previous stable tag: `v0.6.0`.

## Scope and source identity

- Starting source: `ade570d41623176314402a5c05d8834e6838e8ce` on main, synchronized with origin/main. The latest main packaging run [44099650255](https://github.com/willmove/markion/actions/runs/44099650255) succeeded.
- Reviewed post-v0.6.0 work: eight-language README/Korean UI, configurable document spacing, maintained release acceptance suite and periodic background updates. Existing completed [compact Insert](../../openspec/changes/compact-context-insert-submenu/tasks.md) and [content-preserving insertion](../../openspec/changes/preserve-content-in-block-insertions/tasks.md) changes are included after review and regression checks.
- Documentation/version execution: [release change](../../openspec/changes/refresh-readmes-and-release-v0-6-1/tasks.md). No public tag is moved or recreated.
- No manual migration: Markdown remains canonical; new preference fields use backward-compatible defaults. The updater trust key is unchanged. Automatic update downloads default to enabled; installation still requires confirmation and clean documents.
- Historical screenshots remain labeled v0.6.0 and [performance measurements](../performance.md) remain dated 2026-11-11; neither is represented as fresh v0.6.1 evidence.

## Source verification

Executed on Windows 11 Pro 10.0.26300 x86_64 with PowerShell 7.6.9, Rust/Cargo 1.98.1 and OpenSpec 1.13.0. Raw logs remain in `target/tmp/release-v0.6.1/`; their hashes and exact totals are retained in the [source check record](release-v0.6.1-source-checks.json).

| Check | Result |
| --- | --- |
| Workspace version metadata | All 10 packages resolve to 0.6.1 |
| cargo check --workspace | Passed |
| cargo test --workspace | 2,238 passed, 0 failed, 5 existing ignored across 51 result groups |
| Formatting / Clippy | Passed; existing Clippy/linker warnings and proc-macro-error2 future-compatibility notice retained |
| Pinned MarkNice bundle / examples | 23 files / 3,140,387 bytes verified; 18 Markdown examples, index and local references passed |
| Acceptance fixture integrity | 62 cases / 14 documentation pages checked; fresh 160-file run prepared and baseline/working hashes verified |
| Strict OpenSpec / README link checks / git diff --check | Final 96 items passed / 0 failed; final documentation scan: 13 documents / 387 local links passed; diff check passed |

The initial quality script passed formatting, Clippy, every workspace suite and bundle verification, then failed strict validation on the older `prefer-appimage-update-on-arch` delta. Its MODIFIED requirement omitted five newly archived background-update scenarios. Rebased that active delta on the current requirement while retaining its distribution mapping and all existing scenarios; no application or stable-spec edit was needed. The focused validation and final full strict validation then passed. The initial failure is retained in the hashed quality log; no test failure was hidden or converted to a pass.

CI parity: the old workflow pinned OpenSpec 1.5.0, which rejects pure-doc skip_specs changes supported by local 1.13.0. Reproduced that incompatibility with the exact 1.5.0 CLI; aligned the workflow pin to published 1.13.0. The corrected [remote quality run 44122459579](https://github.com/willmove/markion/actions/runs/44122459579) passed before tagging: 51 groups, 2,238 passed / 0 failed / 5 ignored, formatting, Clippy, bundle and all 96 OpenSpec items. README AppImage links and chmod instructions use the actual lowercase `markion_0.6.1_x86_64.AppImage` asset name.

The five ignored tests remain three optional external export-engine integrations and two external-network tests. They are not passing results. Deterministic scheduling, staging integrity, Later/reopen and dirty-install guards are covered by automated tests; a five-hour soak and actual signed upgrades are not.

## Windows development-binary precheck

Used a copied debug executable and resources under a fresh isolated fixture run, separate APPDATA/LOCALAPPDATA, synthetic text, English/Paper, auto-save and automatic update downloads disabled. Executable SHA-256: `2E80B7B385E56DF0E8F9A463E88D8E5A87004C3E1EBFDFD5927F2B161823C052`. Fixture manifest SHA-256: `f2ea48d67f62b383cdc92dce5772fa379cbb495f0dd93dc050ccae3b303723aa`.

- Actual pointer actions opened Appearance and changed line spacing from 100% to 105%; the document reflowed and the isolated config persisted `line_spacing = 105`.
- Actual UI Source editing added ` QA` to the synthetic paragraph, then Save produced the independently expected complete UTF-8 file, including an unchanged Chinese table. One Undo and Save restored every baseline byte.
- Evidence: [spacing](release-v0.6.1/spacing.png), [saved source](release-v0.6.1/source-saved.png), [restored source](release-v0.6.1/undo-saved.png). This is a development precheck, not final installed-package, IME-candidate or clipboard interoperability acceptance. Only the test instance was closed.

## Native acceptance and release decision

The [maintained acceptance suite](../testing/README.md) requires platform/package variants and actual input/browser/viewer evidence. Unit/GPUI tests and native CI packaging do not establish those results. Previous v0.6.0 exceptions do not apply to this version.

The [native acceptance record](release-v0.6.1/native-acceptance.md) identifies the limited development precheck, environment, binary/resource boundaries and the outstanding matrix.

The full current-candidate/final-package native matrix has not been executed: the final ZIP Source save/Undo subset below does not cover NSIS installation, actual IME/table/DPI variants, macOS arm64/Intel interaction, Linux X11/Wayland editing/input, browser/WeChat/Word and actual DOCX producers, or dedicated clean previous-version Windows/AppImage upgrades. Remaining variants stay NOT_RUN/BLOCKED, not PASS. Existing change-specific Windows evidence is linked above and is not relabeled as final installed-package acceptance.

On 2026-11-18 (Asia/Shanghai), after reviewing the v0.6.1 candidate and the explicit incomplete scope, the maintainer authorized: **“允许本次例外，披露未验证范围并继续发布”** (“Allow this release-specific exception, disclose the unverified scope and continue publication”). Exception `V061-NATIVE-SCOPE` covers the incomplete full Windows installed-package/IME/DPI matrix, macOS/Linux native interaction, supported browser/WeChat/Word and actual-producer interoperability, and previous-version signed Windows/AppImage upgrade/soak/failure variants identified above and in the native report. The Windows development precheck remains a limited precheck; unexecuted cases remain NOT_RUN/BLOCKED, never PASS. The reason is that this run did not execute the exhaustive dedicated native/platform/interoperability/upgrade environments; the maintainer explicitly accepts that limitation for v0.6.1 only. The bilingual release notes disclose the gap. All automated source checks, native packaging/resource verification, no-GPU AppImage smoke, updater signing, assets, manifests and OSS verification remain mandatory.

## Publication checklist

Candidate packaging on `17db038`: [44122459859](https://github.com/willmove/markion/actions/runs/44122459859) succeeded on all four native builds, extracted-resource verification and the no-GPU AppImage smoke test. Branch-only signing/publication/mirror jobs were correctly skipped. The final freeze commit changes release documentation/evidence only; application source and resources match the verified candidate.

- [x] Source checks and reviewed documentation complete.
- [x] Release-specific native acceptance readiness or explicitly authorized exception recorded.
- [x] Dedicated Release Markion v0.6.1 commit and annotated tag pushed.
- [x] All four native builds, resource verification, AppImage smoke, signing, publication and mirror jobs succeed.
- [x] Seven packages, both signatures, channel-specific update.json, versions, hashes and public mirror reachability verified.
- [x] Actual macOS signing mode and final bilingual notes verified.
- [x] Latest stable non-draft metadata and final Git synchronization verified.

## Final publication evidence

- Published [Markion v0.6.1](https://github.com/willmove/markion/releases/tag/v0.6.1) at **2026-11-18 01:08:11 Asia/Shanghai** as the latest stable, non-draft Release, titled `Markion v0.6.1`, with curated English-first/Simplified-Chinese notes and the full v0.6.0 comparison.
- Immutable annotated tag object `7f997b30976482b5ac144a6cdde4d322208657d8` points to dedicated release commit `86de371421be1a772d48bcfa5ce9c547c88fe161`. No public tag was moved or recreated. The final commit's [quality run 44126537705](https://github.com/willmove/markion/actions/runs/44126537705) independently passed all 51 suites (2,238 / 0 / 5 ignored), formatting, Clippy, bundle and 96 strict OpenSpec items.
- Authoritative [tag run 44126538925](https://github.com/willmove/markion/actions/runs/44126538925), **attempt 2**, passed all eight jobs: four native builds, no-GPU AppImage smoke, updater signing/metadata, GitHub publication and OSS mirror. Attempt 1's two macOS jobs never acquired a runner and timed out after 15 minutes; GitHub explicitly reported arm64 capacity constraints. The [original diagnostics](release-v0.6.1/attempt-1-runner-capacity.json) are retained. Retrying failed jobs preserved successful Windows/Linux results and the same tag. Redundant branch packaging on the identical final source was intentionally canceled; the earlier candidate packaging and both candidate/final quality runs passed.
- [Machine-readable asset verification](release-v0.6.1-assets.json): all seven platform packages, both updater signatures and `update.json` are attached. All 13 public mirror objects returned HTTP 200. Every mirrored package was streamed and hashed; size and SHA-256 match GitHub asset digests and `sha256sums.txt`. Both update manifests declare 0.6.1, name their own distribution host, match in all other fields and carry signatures byte-identical to their channel's `.sig` files. Mirror `manifest.json` and `packager.toml` also declare 0.6.1.
- Both macOS job logs explicitly report **`adhoc; notarize: false`**. README and release-note first-launch instructions retain `xattr -cr /Applications/Markion.app`. Updater Minisign authentication is not Authenticode or Apple notarization.
- The actual no-GPU AppImage [screenshot](release-v0.6.1/appimage-smoke.png) was downloaded and inspected: the main window and Markdown content are visibly rendered. This establishes the CI smoke scope, not the full Linux interaction matrix.

## Final Windows portable package verification

The final ZIP's SHA-256 is `cfe10af0f20097dd4b06372f258d09d30a241a8443a690edcb0f36693bdde00e`. The extracted executable reports 0.6.1 and hashes to `3ea9f6a774207e781ad8a08209b46835f4ca747a5bf99b76b7c4fa80468502c6`.

The initial local packaged-example helper rejected `01-getting-started.md`: the current local/Git blob uses LF (3,946 bytes), while the Windows CI checkout and ZIP use CRLF (4,005 bytes). This was a checkout-reference difference, not changed prose. Preserved the failed check and then compared **all 95 resource/license files** strictly against `git -c core.autocrlf=true cat-file --filters v0.6.1:<path>`, with unchanged tagged attributes; every byte matched, with no missing or extra payload files. The pinned MarkNice tree's explicit `eol=lf` remained byte-identical to canonical Git blobs. [Per-file identities and initial-check explanation](release-v0.6.1/final-package-resources.json). The packaged bundle independently verified 23 files / 3,140,387 bytes; examples comprise 18 Markdown files and one PNG.

A fresh isolated run launched the extracted final ZIP with its own packaged assets and APPDATA/LOCALAPPDATA, without installing over the daily app. Actual Source typing and Save produced the complete independently expected **83-byte** file, including an unchanged Chinese table; one Undo and Save restored the complete **80-byte** baseline. The test process was closed. [Final identity and byte checks](release-v0.6.1/final-native-smoke.json), [window](release-v0.6.1/final-portable-window.png), [saved source](release-v0.6.1/final-source-saved.png), [restored source](release-v0.6.1/final-source-restored.png). This passes that portable-package subset only; the remaining native matrix stays explicitly unverified under `V061-NATIVE-SCOPE`.

Post-publication records also align the P06 CSV template with its already updated case: default background download/staging is allowed, while installation requires explicit confirmation and a final dirty-document check. No application or packaged resource bytes changed after tagging. Subsequent commits contain verification/documentation only; main is synchronized with origin and the release tag remains on the dedicated release commit.
