## 1. Model + decode table

- [x] 1.1 Add `VisualRevealKind::Entity` to `src/model.rs` with a doc comment mirroring `Escape` (one complete authored `&…;` token rendered as its decoded character; token bytes hidden until reveal).
- [x] 1.2 Add GPUI-free `decode_entity_token(&str) -> Option<char>` beside the visual projection: complete numeric references (`&#NN;`, `&#xHH;`/`&#XHH;`; `None` for surrogates/out-of-range), curated single-codepoint named table matching pulldown-cmark byte-for-byte (`nbsp` → U+00A0). Do **not** touch `src/parse.rs::decode_html_entity`.
- [x] 1.3 Unit-test every table entry against the character pulldown-cmark's `Event::Text` produces for the same reference (property-style loop over the table).

## 2. Reconstruction prover + run splitting

- [x] 2.1 Add `decoded_text_matches(event_source, visible) -> Option<Vec<DecodedSpan>>` (one pass: `\X` escapes + decodable `&…;` tokens + literal bytes; accept iff reconstruction equals `visible`), mirroring `escape_matches` proof discipline.
- [x] 2.2 Generalize `push_escaped_text_runs` into `push_decoded_text_runs(spans: &[DecodedSpan])`: shared boundary/extended-marker/style logic; `Escape` spans keep byte-exact today's behavior; `Entity` spans emit a run whose `visible_text` is the decoded char and whose `source_range`/`content_range` is the complete token, plus an `Entity` reveal candidate (bypass `push_run` substring proof — the prover attests the mapping).
- [x] 2.3 Rewire `push_text_runs`: identity → `decoded_text_matches` projection → existing `force_fallback` (unchanged for unproven slices).
- [x] 2.4 Add the `Entity` arm to `reveal_candidate_is_exact` (shape validation: `&`…`;`, alnum/`#` body, table-decodable).

## 3. View layer

- [x] 3.1 Handle `VisualRevealKind::Entity` wherever `Escape` is handled in `src/app/preview.rs`: token hidden in normal flow, complete authored token revealed on caret/selection entry, hidden again on exit without a version change; unfocused run renders the decoded character.
- [x] 3.2 Confirm pointer hit-testing, selection, copy, and IME operate on the entity run exactly as on escape runs (atom when unfocused, token bytes when revealed).

## 4. Tests

- [x] 4.1 Update `entity_references_stay_conservative` (`src/visual.rs` tests): covered references (`&amp;`, `&#39;`, `&#x2014;`, `&nbsp;`) now render with no conservative run and no source island; keep a variant asserting the conservative fallback for an unproven form (e.g. `&NotEqualTilde;`, unmaintained name).
- [x] 4.2 Add unit tests: reveal-candidate kind/range for entity tokens; composition (`**a &amp; \* b**` reveals one containing group); mixed escape+entity in one event; invalid numeric forms stay conservative; UTF-8/CRLF boundaries; no document-version change on caret-only reveal.
- [x] 4.3 Add rendered GPUI tests mirroring the escape suite: unfocused paragraph renders prose (no island box), focused reveals the token, blur restores, pointer/keyboard resolution limited to token boundaries, undo/redo unaffected.
- [x] 4.4 Run `cargo test --workspace`; confirm the differential/property suites (incremental vs full derivation) pass with entity-containing fixtures.

## 5. Docs + roadmap bookkeeping

- [x] 5.1 `docs/visual-editing-quality.md`: move decoded entities into the inline-formatting matrix row (progressive-reveal class), remove roadmap row 1, renumber remaining priorities (front matter → 1, indented code → 2), add "entity references outside the proven decode table" as a secondary gap row.
- [x] 5.2 Update `README.md` / `README.zh-CN.md`: remove "decoded HTML entities" from the gap wording (keep unproven-form nuance out of READMEs; the matrix carries it).
- [x] 5.3 Confirm the delta specs match final behavior: fidelity scenarios (render/reveal/compose/unproven), classification matrix wording, roadmap gap removal.

## 6. Validation

- [x] 6.1 `openspec validate fix-visual-edit-entity-projection` and `openspec doctor` pass.
- [x] 6.2 Quality gate: `pwsh ./scripts/check-quality.ps1` (formatting, `cargo test --workspace`, strict OpenSpec validation) passes.
