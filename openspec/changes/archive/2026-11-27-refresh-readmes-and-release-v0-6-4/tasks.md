## 1. Release documentation

- [x] 1.1 Review all post-v0.6.3 commits and completed changes; update eight README editions and linked guidance for v0.6.4, preserving historical evidence labels. Verify equivalent release facts and local links.
- [x] 1.2 Write concise English-first/Chinese release notes and a preflight record linked to current evidence; explicitly record unavailable acceptance and obtain any required release-specific scope decision before tagging.

## 2. Candidate verification

- [x] 2.1 Restore the formatting gate for the six drifted source files; confirm the diff is formatting-only.
- [x] 2.2 Synchronize Cargo workspace/root, packager and lockfile versions to 0.6.4; verify cargo metadata and cargo check --workspace.
- [x] 2.3 Run the repository quality gate (formatting, Clippy, workspace tests, MarkNice bundle, strict OpenSpec validation), the examples and acceptance-fixture checks, and diff checks; retain logs and exact results in the preflight record. Commit the reviewed formatting change separately from the dedicated release commit.
- [x] 2.4 Run the applicable candidate acceptance on the Windows candidate package and record the results; keep missing environments NOT_RUN/BLOCKED.

## 3. Publication and final verification

- [x] 3.1 Push the reviewed main branch and a new annotated v0.6.4 tag on the dedicated Release Markion v0.6.4 commit after acceptance readiness; monitor every native build, smoke, signing, publication and mirror job to success.
- [x] 3.2 Publish final bilingual notes; verify eight packages, both signatures, update.json, manifest hosts/versions/signatures, mirror hashes/reachability, actual macOS signing mode and stable non-draft metadata. Retain final evidence and verify Git synchronization.
- [x] 3.3 Update project status and linked documentation with the published results.

## Final verification — 2026-11-27

Published v0.6.4 at 4df26771b0337a402e5508ab719d44db9dacf7f7 with annotated tag 14d6f5d4. Final quality: 2,384 passed / 0 failed / 5 ignored across 52 test binaries, 106 strict OpenSpec items. Tag run 45948581120: all ten jobs passed, including both notarized macOS installed-app launches, the no-GPU AppImage smoke, updater signing, publication and OSS mirroring. Eleven Release assets, both `.sig` files and both `update.json` manifests verified with identical signatures; all eight OSS digests equal the GitHub assets; both DMG hashes match their launch jobs with stapled notarization. The final Windows portable (46a45a63…) passed the isolated launch and native smoke (reported version 0.6.4). The formatting-gate repair and the preflight-found D064-01 list-duplicate regression fix shipped in this release; unrun native scope stays disclosed under V064-EX-01. See docs/verification/release-v0.6.4-preflight.md.
