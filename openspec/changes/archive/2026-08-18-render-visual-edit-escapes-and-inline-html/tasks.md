## 1. Recognizers and model foundations

- [x] 1.1 Add `parse_inline_html_style_tag` to `src/parse.rs` beside `parse_inline_html_image`: exact, unattributed, case-insensitive recognition of opening/closing `em|i`, `strong|b`, `s|del|strike`, `code`, `mark`, `sub`, `sup` and void `<br>`/`<br/>`/`<br />`, each style tag mapped to its `InlineStyle` flag; everything else returns `None`. Cover with unit tests (accepted forms, attributes/unknown/malformed rejection).
- [x] 1.2 Add `Escape` and `InlineHtml` variants to `VisualRevealKind` (`src/model.rs`) and audit every `match` over the enum (`src/visual.rs`, `src/app/preview.rs`) so the new kinds flow through generic reveal/gray-source handling with no `include_end` special case.

## 2. Escaped punctuation rendering

- [x] 2.1 Implement escape splitting in `push_text_runs` (`src/visual.rs`): when the event slice differs from the parser's visible text, reconstruct the visible text by removing backslashes before ASCII punctuation; on exact equality, emit identity runs for plain segments (each still routed through `extended_inline_matches`), a one-byte content run per escaped character, and `Escape` reveal candidates for each `\X`; on mismatch keep the conservative fallback (entities etc.). Delete the whole-block `contains_markdown_escape` collapse in `inline_runs`.
- [x] 2.2 Unit tests in `src/visual.rs`: paragraph with `\*`/`\.` renders non-conservative with hidden `\` marker and a `\X` reveal group; `\\` form; escape inside strong and inside `==highlight==`; unmatched reconstruction (e.g. `&amp;`) stays conservative; rewrite `escaped_inline_syntax_uses_conservative_fallback` to assert the new rendering and reveal behavior.

## 3. Inline HTML rendering

- [x] 3.1 Handle supported tags in `inline_runs`' `Event::Html | Event::InlineHtml` arm: maintain a strict style stack (open pushes the flag and records the open-tag range; close must match the top of stack), register one `InlineHtml` reveal candidate per pair spanning open tag start to close tag end, and leave tag bytes uncovered so `marker_ranges()` hides them; styling composes with Markdown emphasis, code, math, and images between the tags.
- [x] 3.2 Push `<br>` variants as atomic line-break runs (`visible_text: "\n"`, `content_range` = tag bytes, non-conservative) with an `InlineHtml` reveal group over the tag, bypassing `push_run`'s identity checks the same way math/image runs do.
- [x] 3.3 Implement the "one bad tag spoils the block" rule: rejected tags, closes without opens, and opens left unclosed at event-loop end set the existing `contains_non_image_html` flag, demote runs emitted inside a failed element to `conservative_fallback` by range containment, and preserve the mixed `<img>` exemption rendering. Unit tests: `<em>em</em>`, `<strong>`+`<b>` composition, `<br>` run shape, unknown/attributed/unpaired tags stay whole-block conservative, `<a><img></a>` keeps image runs, `<br>` + `<img>` mixed block renders both; rewrite `visual_edit_marks_non_image_inline_html_for_conservative_runs` accordingly.

## 4. Projection and view verification

- [x] 4.1 Audit `VisualProjectionSegment` consumers (hit testing, selection mapping, keyboard navigation, IME bounds) for per-char identity assumptions against the one-char-to-tag-bytes `<br>` segment; fix any that interpolate inside a segment and add tests for click, arrow traversal, and selection across a `<br>` boundary resolving to the tag's safe source boundaries.
- [x] 4.2 GPUI view tests in `src/app/tests.rs`: an unfocused paragraph containing `\*`/`\.` or `<em>`/`<br>` renders styled prose with no source-island box; caret placement reveals the complete `\X` / element group and leaving restores rendering without a version bump; extend the wrapped-badges mixed-path test with unpaired-tag cases.

## 5. Documentation and validation

- [x] 5.1 Update the support matrix in `docs/visual-editing-quality.md`: move escaped ASCII punctuation and the supported inline-HTML subset into the progressive-reveal row, narrow the "other inline HTML" fallback trigger to the unsupported subset, and state the byte-proof gates.
- [x] 5.2 Run the quality gate (`pwsh ./scripts/check-quality.ps1`) and `openspec validate render-visual-edit-escapes-and-inline-html`; fix any regressions, then confirm the spec scenarios map to the tests added in 2–4.
