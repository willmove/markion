# Verification

## Scope

Shared inline HTML style/link parsing, exact Visual Edit reveal, linked images,
HTML cell isolation, explicit break preservation and rendered script geometry.
The manual fixture is `docs/fixtures/inline-html-parity.md`.

## Repository-wide specification baseline

`openspec validate unify-inline-html-rendering --strict` passes.
`openspec validate --all --strict --no-interactive` reports 98 passed and 21
failed. These failures are in existing, unchanged specs/changes (including
`document-typography`, `diagram-rendering`, `document-resources`,
`reliable-file-persistence`, `improve-visual-edit-html-rendering` and
`use-icons-in-file-tree`). They include missing scenarios in old MODIFIED
requirements and existing spec-format problems. No stable spec or unrelated
change was edited to suppress them.

## Compatibility boundary

Colors are hex/rgb; supported CSS declarations are color, bold font weight,
italic font style and underline/line-through text decoration. Browser CSS,
class stylesheets and active attributes remain unsupported. Literal escaped,
entity-encoded and code-span tags are preserved. GFM cells retain their
whole-cell source reveal and alt-text image display contracts.

## Automated results

- `cargo test --workspace`: 43 suites, 1672 passed, 0 failed, 5 ignored.
- Final `cargo test -p markion` after the HTML-only layout dispatch refinement:
  595 library tests and 593 application tests passed; 3 existing conditional
  tests ignored. Doc-tests passed.
- New GPUI checks passed for script size/baseline bounds in Read and Visual Edit,
  GFM and HTML tables; pointer entry; CJK/emoji IME geometry; exact undo; and
  leading/repeated/trailing break row heights.
- Existing search regression exposed eager subtraction in disjoint fragment
  ranges; lazy intersection conversion fixes the overflow and the search test
  now passes.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `cargo clippy --workspace`: passed with warnings (including argument-count
  suggestions in the rendering/parser plumbing); warnings are not denied by
  the repository gate.
- Both current HTML changes pass individual strict OpenSpec validation.
- Pinned MarkNice bundle verification: passed, 23 files (3,140,387 bytes).
