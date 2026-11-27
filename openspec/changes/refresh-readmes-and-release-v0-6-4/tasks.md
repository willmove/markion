## 1. Release documentation

- [x] 1.1 Review all post-v0.6.3 commits and completed changes; update eight README editions and linked guidance for v0.6.4, preserving historical evidence labels. Verify equivalent release facts and local links.
- [x] 1.2 Write concise English-first/Chinese release notes and a preflight record linked to current evidence; explicitly record unavailable acceptance and obtain any required release-specific scope decision before tagging.

## 2. Candidate verification

- [x] 2.1 Restore the formatting gate for the six drifted source files; confirm the diff is formatting-only.
- [x] 2.2 Synchronize Cargo workspace/root, packager and lockfile versions to 0.6.4; verify cargo metadata and cargo check --workspace.
- [x] 2.3 Run the repository quality gate (formatting, Clippy, workspace tests, MarkNice bundle, strict OpenSpec validation), the examples and acceptance-fixture checks, and diff checks; retain logs and exact results in the preflight record. Commit the reviewed formatting change separately from the dedicated release commit.
- [x] 2.4 Run the applicable candidate acceptance on the Windows candidate package and record the results; keep missing environments NOT_RUN/BLOCKED.

## 3. Publication and final verification

- [ ] 3.1 Push the reviewed main branch and a new annotated v0.6.4 tag on the dedicated Release Markion v0.6.4 commit after acceptance readiness; monitor every native build, smoke, signing, publication and mirror job to success.
- [ ] 3.2 Publish final bilingual notes; verify eight packages, both signatures, update.json, manifest hosts/versions/signatures, mirror hashes/reachability, actual macOS signing mode and stable non-draft metadata. Retain final evidence and verify Git synchronization.
- [ ] 3.3 Update project status and linked documentation with the published results.
