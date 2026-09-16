## 1. Regression coverage and repair

- [x] 1.1 Add reported TOC and adjacent HTML/escape regression cases; verify unfocused text, styles, destinations, full source reveal, UTF-8 mappings, and cache reuse.
- [x] 1.2 Support underlined inline HTML through the existing style pipeline and verify parser/projection tests, including attributed, nested, heading, list, quote, and table contexts.
- [x] 1.3 Correct proven adjacent escape or mismatched HTML pairing defects and verify malformed/unsupported markup retains source fidelity.

## 2. Integration verification

- [x] 2.1 Update the visual editing quality matrix with supported forms and remaining limitations; verify the documentation matches tests.
- [x] 2.2 Run relevant root-package tests, formatting checks, and OpenSpec validation; record outcomes and any environment limitations.

## Verification results

- `cargo test --lib inline_html_ -- --nocapture`: the initial regression run reproduced three failures (TOC, underline composition, mismatched aliases); the repaired run passed all 17 selected tests.
- `cargo test`: root library, application, and doc-test suites passed, including the added shared-HTML insertion and GPUI paint-highlight tests. Existing ignored external/tool-dependent tests remained ignored.
- `rustfmt --check --edition 2024 src/parse.rs src/visual.rs src/app/tests.rs`: passed.
- `git diff --check`: passed.
- `openspec validate fix-visual-inline-html-underline --strict`: passed.
- No manual desktop visual inspection was performed; rendering was verified through projection and GPUI highlight tests.