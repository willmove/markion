## 1. Release documentation

- [x] 1.1 Review all post-v0.6.0 commits and pending completed block-edit changes; update eight README editions and linked guidance for v0.6.1, preserving historical evidence labels. Verify equivalent release facts and local links.
- [ ] 1.2 Write concise English-first/Chinese release notes and a preflight record linked to current evidence; explicitly record unavailable acceptance and obtain any required release-specific scope decision before tagging.

## 2. Candidate verification

- [x] 2.1 Synchronize Cargo workspace/root, packager and lockfile versions to 0.6.1; verify cargo metadata and cargo check --workspace.
- [x] 2.2 Run cargo test --workspace, formatting/Clippy, MarkNice bundle, examples, acceptance fixtures, strict OpenSpec validation and diff checks; retain logs and exact results in the preflight record. Commit reviewed application changes separately from the dedicated release commit.
- [ ] 2.3 Align the CI OpenSpec pin to verified 1.13.0 for skip_specs support and correct README AppImage asset casing; verify all seven asset names against release.yml, strict validation and the remote quality job.

## 3. Publication and final verification

- [ ] 3.1 Push the reviewed main branch and a new annotated v0.6.1 tag on the dedicated Release Markion v0.6.1 commit after acceptance readiness; monitor every native build, smoke, signing, publication and mirror job to success.
- [ ] 3.2 Publish final bilingual notes; verify seven packages, both signatures, update.json, manifest hosts/versions/signatures, mirror hashes/reachability, actual macOS signing mode and stable non-draft metadata. Retain final evidence and verify Git synchronization.
