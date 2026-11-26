## 1. Regression coverage and LaTeX list serializer

- [x] 1.1 Add Issue #51 export regressions for nested same-kind and mixed-kind lists, level decreases, ordered starts at 6 and 12, separated lists, nested task markers, and rich item text; use `export_to` to produce real `.tex` files and confirm the new assertions fail against the current flat serializer.
- [x] 1.2 Implement the shared recursive list renderer for top-level and blockquote items, preserve resolved ordered indices with explicit labels, and retain existing checkbox/rich-text rendering; verify with the new regression and all existing LaTeX-related root-crate tests.

## 2. Integration verification and specification sync

- [x] 2.1 Run `cargo fmt --all`, `cargo test --workspace`, and `git diff --check`; verify the implementation is limited to the LaTeX serializer, its regression tests, and this change, with no dependency, public API, document-version, or cache behavior changes.
- [x] 2.2 Run `openspec validate fix-latex-list-fidelity`, archive the completed change so the `export` delta syncs into the main specification, and verify the resulting specification and task checklist reflect the implemented behavior.
