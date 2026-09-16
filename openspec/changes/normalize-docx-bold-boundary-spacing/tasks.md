## 1. Semantic boundary serialization

- [x] 1.1 Add paragraph-local typed boundary state to the GUI-free DOCX inline serializer, using the first unescaped source character and resolved direct/inherited bold state; verify focused fixtures produce `**粗体** 文字`, and equivalent Latin/digit cases contain exactly one U+0020.
- [x] 1.2 Carry the same state through empty runs, accepted revisions, `smartTag`, `sdt`, simple fields, and hyperlink labels while clearing it for images, equations, explicit breaks, and block boundaries; verify exact fixtures cannot leak spacing across paragraphs, table cells, footnote blocks, or structural inline content.
- [x] 1.3 Emit compatibility spacing before the following semantic item so bold combined with italic/strike/underline/superscript/subscript closes completely first; verify exact output and parser events keep the combined span intact and the following word-like content outside bold.

## 2. Compatibility and regression coverage

- [x] 2.1 Add negative fixtures for existing spaces, tabs, explicit line breaks, paragraph ends, ASCII/CJK punctuation, Markdown delimiters, symbols, combining marks, and adjacent runs that remain bold; verify none receive a synthetic space and punctuation consumes the pending boundary.
- [x] 2.2 Update intentional existing golden outputs and add `pulldown-cmark` assertions for CJK, Latin, numeric, wrapper, hyperlink, inherited-style, and combined-style cases; verify authored literal asterisks and unrelated Markdown remain byte-for-byte unchanged.
- [x] 2.3 Run `cargo fmt --check`, `cargo test -p markion-docx-import`, and `cargo test --workspace`, record any unrelated pre-existing environment failures, then run `openspec validate normalize-docx-bold-boundary-spacing` and verify the base-change archive-order note remains accurate without editing stable specs directly.
