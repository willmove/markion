## 1. Documentation

- [x] 1.1 Refresh and synchronize the primary bilingual READMEs, localized current-status summaries, and documented limitations/packages.
- [x] 1.2 Add a dated project-status index and reconcile Visual Edit coverage and historical P0/P1 progress links with implemented work and evidence.
- [x] 1.3 Write complete English-first/Simplified-Chinese v0.5.3 release notes from v0.5.2 through HEAD, with truthful compatibility and verification status.

## 2. Release preparation

- [x] 2.1 Synchronize Cargo workspace/root, packager, and Cargo-generated lockfile versions to 0.5.3; verify all workspace metadata.
- [x] 2.2 Pass workspace tests, bundled-workspace verification, documentation/link checks, diff checks, and OpenSpec validation; investigate current CI failures and record unresolved manual checks.

## 3. Publication

- [ ] 3.1 Commit documentation and create the dedicated release commit plus annotated v0.5.3 tag; push using an authenticated publishing environment.
- [ ] 3.2 Verify all tag workflow jobs, platform assets, updater signatures/manifests, OSS mirror, curated release metadata, and final repository synchronization before reporting completion.

2026-11-13 follow-up: authenticated GitHub CLI is available. The maintainer authorized testing all available paths and skipping genuinely unavailable manual environments for v0.5.3; the exact scope is recorded in the release preflight. Local full suite reran successfully after correcting the Windows short-path assertion; browser self-test, actual Markdown copy and HTML/DOCX download checks pass.
