## 1. Regression coverage and serializer fix

- [x] 1.1 Add Issue #39 regression tests for bold, italic, bold-italic, one-sided and two-sided edge whitespace, whitespace-only runs, interior whitespace, and the existing bold compatibility-space rule; verify the exact Markdown and CommonMark emphasis events and confirm the new test fails on the unfixed serializer.
- [x] 1.2 Update run flushing to emit raw leading/trailing whitespace outside the complete wrapper stack while passing whitespace-only text through unchanged and retaining raw-text boundary tracking; verify with `cargo test -p markion-docx-import issue_39` and the existing DOCX import suite.

## 2. End-to-end verification

- [x] 2.1 Re-run the real `python-docx` reproduction through `import_docx` and CommonMark HTML rendering; verify it produces `Before **focus** after` and `<strong>focus</strong>` without literal asterisks.
- [x] 2.2 Run `cargo test --workspace` and verify the final diff is limited to the OpenSpec change, DOCX serializer, and regression tests, with no dependency, public API, or cached-document-state changes.
