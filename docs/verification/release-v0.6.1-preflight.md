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
| Strict OpenSpec / README link checks / git diff --check | Final 96 items passed / 0 failed; 12 documents / 359 local links passed; diff check passed |

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

The full current-candidate/final-package native matrix has not been executed: Windows NSIS/ZIP actual save, IME and table workflows at real DPI variants; macOS arm64/Intel; Linux X11/Wayland DEB/RPM/AppImage interaction and IME; browser/WeChat/Word interoperability and actual DOCX producers; and dedicated clean previous-version Windows/AppImage upgrades. These remain NOT_RUN/BLOCKED, not PASS. Existing change-specific Windows evidence is linked above and is not relabeled as installed-package acceptance.

On 2026-11-18 (Asia/Shanghai), after reviewing the v0.6.1 candidate and the explicit incomplete scope, the maintainer authorized: **“允许本次例外，披露未验证范围并继续发布”** (“Allow this release-specific exception, disclose the unverified scope and continue publication”). Exception `V061-NATIVE-SCOPE` covers the incomplete full Windows installed-package/IME/DPI matrix, macOS/Linux native interaction, supported browser/WeChat/Word and actual-producer interoperability, and previous-version signed Windows/AppImage upgrade/soak/failure variants identified above and in the native report. The Windows development precheck remains a limited precheck; unexecuted cases remain NOT_RUN/BLOCKED, never PASS. The reason is that this run did not execute the exhaustive dedicated native/platform/interoperability/upgrade environments; the maintainer explicitly accepts that limitation for v0.6.1 only. The bilingual release notes disclose the gap. All automated source checks, native packaging/resource verification, no-GPU AppImage smoke, updater signing, assets, manifests and OSS verification remain mandatory.

## Publication checklist

Candidate packaging on `17db038`: [44122459859](https://github.com/willmove/markion/actions/runs/44122459859) succeeded on all four native builds, extracted-resource verification and the no-GPU AppImage smoke test. Branch-only signing/publication/mirror jobs were correctly skipped. The final freeze commit changes release documentation/evidence only; application source and resources match the verified candidate.

- [x] Source checks and reviewed documentation complete.
- [x] Release-specific native acceptance readiness or explicitly authorized exception recorded.
- [ ] Dedicated Release Markion v0.6.1 commit and annotated tag pushed.
- [ ] All four native builds, resource verification, AppImage smoke, signing, publication and mirror jobs succeed.
- [ ] Seven packages, both signatures, channel-specific update.json, versions, hashes and public mirror reachability verified.
- [ ] Actual macOS signing mode and final bilingual notes verified.
- [ ] Latest stable non-draft metadata and final Git synchronization verified.
