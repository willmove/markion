## 1. Shared Extended-Inline Ranges

- [x] 1.1 Add a crate-internal range-aware matcher for valid highlight, superscript, and subscript delimiters, and make Preview's extended parser consume the shared delimiter rules.
- [x] 1.2 Add parser tests covering valid, invalid, nested, UTF-8, and strikethrough-adjacent extended syntax without changing existing Preview output.

## 2. Cached Visual Inline Model

- [x] 2.1 Split exact pulldown text events into extended-style visual runs and typed reveal groups while preserving canonical source ranges.
- [x] 2.2 Accept only properly contained nested reveal candidates, normalize active nested groups to one outermost projection range, and retain conservative fallback for crossing or byte-inexact syntax.
- [x] 2.3 Add pure visual-model tests for combined strong/emphasis, all extended styles, local marker reveal, UTF-8 mappings, ambiguous fallback, and unchanged per-version cache reuse.

## 3. GPUI Rendering Regression

- [x] 3.1 Apply highlight and super/subscript Visual Edit GPUI styles consistently with Preview.
- [x] 3.2 Add a rendered-window regression proving the default Inline formatting paragraph remains visual, styles reach the projection, and caret-only reveal does not mutate document/version state.

## 4. Validation

- [x] 4.1 Run `cargo fmt --check`, focused inline/Visual Edit tests, `cargo test`, and `openspec validate fix-visual-edit-inline-formatting`.
