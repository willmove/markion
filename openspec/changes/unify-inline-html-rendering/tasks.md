## 1. Shared semantics

- [x] 1.1 Implement the shared element classifier and nested style/link state; verify tag aliases, inert attributes, bounded colors, inheritance and malformed inputs with unit tests.
- [x] 1.2 Use shared semantics in mixed Markdown, HTML blocks and HTML cells; verify context parity and adjacent-cell isolation.
- [x] 1.3 Extend exact Visual Edit groups to color/wrapper/anchor forms, including linked images; verify source reveal, literal escapes, navigation, UTF-8 and undo/cache invariants.

## 2. Rendered geometry

- [x] 2.1 Preserve leading, repeated and trailing br elements through parsing and line layout; verify break counts and empty-row heights in both modes.
- [x] 2.2 Render super/subscript with smaller glyphs and shifted baselines in preview and Visual Edit, including tables/links; verify scaled layout, pointer/selection and IME geometry.

## 3. Completion

- [x] 3.1 Update coverage documentation and a representative HTML regression fixture; verify it covers every repaired family and accurately states remaining unsupported browser features.
- [x] 3.2 Run parser/projection and GPUI tests, root tests and applicable workspace tests, formatting and strict OpenSpec validation; record results.
