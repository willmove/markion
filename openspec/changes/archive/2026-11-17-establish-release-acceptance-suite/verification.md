# Verification — 2026-11-17

Scope: delivery of the release acceptance documentation, materials and record tooling. No new native acceptance campaign or release publication was performed.

## Delivered contract

- `docs/testing/README.md`, separate human/Codex guides, eight detailed domain files with 62 stable case groups, and platform/package/variant rules.
- Repository-owned synthetic source fixtures plus a Python 3.10+ standard-library utility. A prepared run contains 160 input files, independent baselines, profile configuration, local Git remote/two clones, report and blank result templates.
- Release report, 62 NOT_RUN result placeholders and real-producer DOCX corpus registration template. Operators must expand variants; placeholders are not complete coverage.
- Canonical release process links candidate acceptance before tagging and actual final-tag package/upgrade verification before declaring completion. Existing manual/signature/mirror gates remain.

## Checks performed

- `python docs/testing/fixtures/prepare.py check`: passed; 62 case IDs, required procedure fields, blank result coverage and 14 documentation pages checked for local links.
- Repeated fresh preparations: passed. Current identical fixture manifests SHA-256: `f2ea48d67f62b383cdc92dce5772fa379cbb495f0dd93dc050ccae3b303723aa`. Both generated Git baselines have identical revisions and clean local clones.
- Baseline/working identity verification, refusal to replace existing run, tampered baseline rejection, compare equal/unequal/missing exit codes 0/1/2: passed. Existing/partial runs were retained.
- Exact UTF-8 100/300/600/1024 KiB inputs, LF/CRLF/no-EOF relationships, known search counts, deterministic DOCX ZIP/XML, PNG CRC/decompression, and uploader success/failure/multiple-URL/timeout stub paths: passed.
- Generated synthetic DOCX was fed to the existing built `markion-docx-import` library through a temporary Rust harness: accepted with one table, expected Unicode/bold, no content-loss flag; invalid ZIP rejected. This is headless semantic evidence, not Word-viewer or native picker qualification.
- A first extended validation driver exposed legacy Windows subprocess diagnostic encoding; tool stdout/stderr now use UTF-8. The corrected validation driver passed, including Chinese Windows errors. No native application behavior changed.
- Strict whole-project OpenSpec validation before archival: 92 passed, 0 failed; existing informational long-requirement notices retained. Focused change validation passed.
- Added-text trailing-whitespace scan and tracked scoped `git diff --check`: passed. Local links were updated after the separate menu change was archived during this work.

Raw disposable evidence: `target/tmp/release-qa/suite-validation-005/evidence/suite-tooling-validation.json`, `validate_suite.py`, `synthetic-import.md`, and `docx-check.txt`. These generated files are not release qualification or checked-in corpus material; they can be removed by normal target cleanup.

## Archive completion

All six implementation tasks completed. The three added `engineering-quality` requirements and their six scenarios were synced inline while retaining all eleven existing requirements; strict stable-spec validation passed (36 items). The change was moved with its `.openspec.yaml` to `2026-11-17-establish-release-acceptance-suite`. Post-archive full strict validation passed: 91 items, 0 failures. The suite/link check and scoped whitespace check passed again. No other change was archived by this work.

## Explicit limits

No Rust application code, dependencies or CI behavior changed. The full Rust workspace suite was not rerun for this documentation/tooling-only change. No OS input, system IME, actual default-browser/WeChat/Word matrix, package install, signed update, macOS/Linux native session or paid provider was exercised. All native case result templates remain NOT_RUN. Real Office-producer corpus rows remain unqualified until supplied and tested. User documents, installed applications, credentials and existing desktop processes were not used.
