## 1. Canonical Release Runbook

- [x] 1.1 Create `docs/release-process.md` with prerequisites, version selection defaults, repository preflight checks, and version-collision checks.
- [x] 1.2 Document synchronized metadata updates, workspace validation, the release commit and annotated tag conventions, and the push sequence.
- [x] 1.3 Document tag-workflow monitoring, curated release-note content and language defaults, required asset checks, final repository verification, and non-destructive failure handling.

## 2. Persistent Agent Guidance

- [x] 2.1 Update `AGENTS.md` with the mandatory default release behavior and a link to the canonical runbook.

## 3. Verification

- [x] 3.1 Verify documented commands and repository paths, confirm the runbook covers every new release-packaging scenario, and run `openspec validate standardize-github-release-process`.
