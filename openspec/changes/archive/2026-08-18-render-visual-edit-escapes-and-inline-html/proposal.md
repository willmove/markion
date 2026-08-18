# Proposal: render-visual-edit-escapes-and-inline-html

## Why

In Visual Edit, a paragraph containing escaped ASCII punctuation (`\*`, `\.`) or any non-image inline HTML (`<br>`, `<em>…</em>`) collapses wholesale into the gray monospace source island. One two-byte escape anywhere in the paragraph demotes the entire block to raw source, even though Split Preview and Read mode render the same prose normally. This is a whole-block collapse for a per-construct condition and is one of the largest remaining WYSIWYG fidelity gaps for ordinary prose.

## What Changes

- Escaped ASCII punctuation (`\` + ASCII punctuation, e.g. `\*`, `\.`, `\\`) renders as the literal character with the backslash as a hidden one-byte marker. Moving the caret into the construct reveals the complete authored `\X` group through the existing reveal-group mechanism. The whole-block escape collapse (`contains_markdown_escape`) is removed in favor of this byte-exact per-construct handling.
- A narrow, exactly recognized subset of inline HTML renders inside prose blocks instead of forcing a source island:
  - style pairs `<em>`/`<i>`, `<strong>`/`<b>`, `<s>`/`<del>`/`<strike>`, `<code>`, `<mark>`, `<sub>`, `<sup>` map onto the existing `InlineStyle` flags;
  - void `<br>`, `<br/>`, `<br />` render as an authored line break (stacked wrap row).
  - Tags act as hidden markers; entering the construct reveals the complete element source (open tag, content, close tag).
- Anything outside the proven subset — unknown or attributed tags, unpaired/crossing tags, other inline HTML, HTML entities (`&amp;`), or a visible-text reconstruction that does not match the parser byte-for-byte — keeps the existing whole-block conservative source island. Inline `<img>` handling and the existing image-bearing mixed-path rendering are unchanged.
- The Visual Edit support matrix and the affected `markdown-editing` requirements are updated to classify these constructs as rendered-with-progressive-reveal instead of always-conservative.

**Non-goals:** HTML entity references; attributes or non-listed tags; uppercase/unknown tag forms; block-level HTML rendering (already handled by the shared HTML-parts pipeline); any change to Split Preview, Read mode, exporters, or the canonical Markdown source format.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: the *Visual Edit inline formatting fidelity* requirement currently sends every escaped construct to the conservative fallback and offers no inline-HTML rendering; it will additionally require byte-exact rendering of backslash-escaped ASCII punctuation and of the narrow inline-HTML subset listed above, with hidden-marker reveal and unchanged conservative fallback for everything unproven. The support matrix maintained under the *Maintained Visual Edit support classification* requirement gets corresponding row updates (documentation, no requirement-text change).

## Impact

- `src/visual.rs`: `inline_runs` (escape splitting, inline-HTML style stack, `<br>` runs, removal of the whole-block escape collapse), `push_text_runs`/`push_run` byte-proof path, new `VisualRevealKind` variants, reveal-group construction.
- `src/parse.rs`: new GPUI-free narrow recognizer for the supported inline-HTML tag forms (beside `parse_inline_html_image`).
- `src/model.rs`: `VisualRevealKind` additions.
- `src/app/preview.rs`: projection/hit-test handling for the atomic `<br>` run (boundary-only caret resolution); the whole-block source-island gate itself is unchanged.
- Tests: `src/visual.rs` unit tests (escape splitting, HTML subset, unpaired fallback), `src/app/tests.rs` rendered-view tests; `docs/visual-editing-quality.md` matrix rows.
- Invariants preserved: canonical `MarkdownDocument.text` only; no second document parser — the escape/HTML handling is a narrow exact recognizer inside `inline_runs` with source-island fallback; per-document-version caching, identity, and incrementality untouched.
