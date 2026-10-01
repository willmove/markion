## Context

`push_text_runs` (`src/visual.rs:1826-1863`) classifies each prose `Event::Text`: when the authored slice equals the parser's visible text, it emits identity runs; when it differs, it tries the backslash-escape projection (`escape_matches` at `:1999` + `push_escaped_text_runs` at `:1885`, shipped by `2026-08-18-render-visual-edit-escapes-and-inline-html`); anything else gets `push_run(..., force_fallback = true)`, which makes the whole paragraph a **permanent** source island via the `always_source` gate (`src/app/preview.rs:2838-2842`).

pulldown-cmark decodes HTML entity references inside `Event::Text` (full HTML5 semantics). Visual Edit already disables smart punctuation for its projection (`src/parse.rs:1750-1753`), so escapes and entities are the only two non-identity Text transformations left. Entities are therefore the exact same problem escapes were, with one extra twist: the decoded character is generally **not a byte substring** of the authored token (`&#39;` contains no `'` byte), while `push_run` (`:2112-2143`) proves runs by substring lookup.

`src/parse.rs:1471-1487` has an entity decoder, but it serves the HTML attribute pipeline: its table is tiny and lossy (`nbsp` → `' '` instead of U+00A0), so it cannot be reused for a projection that must reconstruct the parser's visible text byte-for-byte.

Reveal groups (`VisualRevealKind`, `src/model.rs:723`) carry the progressive-reveal contract; each kind has a shape validator arm in `reveal_candidate_is_exact` (`src/visual.rs:2250+`), and invalid/crossing candidates invalidate the block back to the conservative path (`:2202-2205`).

## Goals / Non-Goals

**Goals:**

- Paragraphs containing entity references in the proven set render as normal prose; no permanent island.
- Each entity token is a progressive-reveal group: hidden `&…;` by default, complete authored token revealed on caret entry, hidden again on exit, no document-version change.
- Entities compose with Markdown formatting, escapes, and the existing reveal machinery; unprovable forms keep today's conservative behavior.
- Byte-exact proof: accept a projection only when the decoded reconstruction reproduces the parser's visible text exactly.

**Non-Goals:**

- Entities inside table cells (keep flattened text — Read-mode parity), link destinations/titles, inline code (already raw), or HTML blocks.
- Multi-codepoint named entities (`&NotEqualTilde;` → 2 chars), the full HTML5 ~2200-entry table, or a new dependency. The table is the extension point.
- Touching the attribute-facing decoder in `parse.rs`, parser options, the document model, persistence, or exporters.

## Decisions

### Decision 1: Reconstruction-proof prover, mirroring `escape_matches`

**Choice:** Add `decoded_text_matches(event_source, visible) -> Option<Vec<DecodedSpan>>` that walks the authored slice once: a `\X` escape contributes `X`; an `&name;`/`&#NN;`/`&#xHH;` token that the exact table decodes contributes its decoded character; every other byte contributes itself. The projection is accepted **iff** the reconstruction equals the parser's visible text; otherwise return `None` and keep the existing force-fallback. One pass covers escapes and entities together, so mixed events (`a \* b &amp; c`) render instead of falling back.

**Why:** Identical proof discipline to escapes: no guessed attribution, any parser divergence (unknown name, weird numeric form) degrades to the conservative path rather than mis-mapping bytes.

**Alternatives considered:**
- *Entities-only prober tried after escapes fail.* Rejected — mixed events still fall back while sharing the same code shape.
- *Decode-only forward projection without reconstruction proof.* Rejected — could mis-attribute visible text to the wrong bytes, violating the source-range invariants.

### Decision 2: Generalize the escape run-splitter, don't duplicate it

**Choice:** `push_escaped_text_runs` becomes `push_decoded_text_runs(runs, candidates, …, spans: &[DecodedSpan])` where `DecodedSpan { range, kind: Escape | Entity }`. Boundary splitting, extended-marker conflict checks, style composition, and per-span reveal candidates keep their current logic; `Escape` spans emit runs exactly as today (byte-substring content), `Entity` spans emit runs as Decision 3 describes. Escape-only events hit the same function with no entity spans — behavior-identical for the existing escape tests.

**Why:** The two constructs differ only in how visible text maps to bytes; segmenting is shared. Avoids a second near-identical splitter with its own conflict rules to keep in sync.

### Decision 3: Entity runs carry the token as their canonical content range

**Choice:** An entity run's `visible_text` is the decoded character, while `source_range` **and** `content_range` are the complete authored token (e.g. `&#39;`), with an `Entity` reveal candidate over the same range. The run bypasses `push_run`'s substring proof via a small `push_decoded_run` helper (or a `proven` flag) because the prover already attests the mapping; `conservative_fallback` stays `false`.

**Why:** The decoded character has no byte range of its own — the token *is* its authored form. Reveal-group containment in `build_reveal_groups` (`content_ranges` ⊆ candidate range) holds trivially (equal ranges), and crossing/ambiguity checks keep their existing invalidation behavior.

**Alternatives considered:**
- *`content_range` = empty or a synthetic byte range.* Rejected — breaks the "contained, disjoint, round-trips to the authored slice" invariants and confuses caret mapping.
- *Treat the whole token as a hidden marker with no run.* Rejected — then nothing renders the decoded character.

### Decision 4: Exact single-codepoint decode table, separate from `parse.rs`

**Choice:** A GPUI-free `decode_entity_token(&str) -> Option<char>` beside the visual projection: complete numeric references (`&#NN;`, `&#xHH;` with case-insensitive `x`; `None` for surrogates/out-of-range — those degrade to fallback since pulldown emits U+FFFD), plus a curated single-codepoint named set matching HTML5 byte-for-byte (`nbsp` → U+00A0, not space): core `amp lt gt quot apos`, typographic `hellip mdash ndash lsquo rsquo ldquo rdquo laquo raquo`, symbols `copy reg trade bull dagger deg sect para middot plusmn times divide`, currency `euro pound yen cent`, accented Latin (`eacute egrave agrave ccedil uuml auml ouig…` as maintained). `parse.rs` keeps its attribute decoder untouched.

**Why:** The projection must match pulldown exactly or fall back; a wrong table entry would silently break reconstruction (safe) or, worse, wrongly succeed only if the entry accidentally matched — curation plus unit tests pinned against pulldown output prevent both. Numeric completeness covers the majority of real-world entity use.

**Alternatives considered:**
- *Reuse `parse.rs::decode_html_entity`.* Rejected — lossy `nbsp`, and its role is HTML attribute display.
- *Full HTML5 table via a new dependency.* Deferred — the table is a data-only extension point; no redesign needed to widen it later.

### Decision 5: New `VisualRevealKind::Entity` with a shape validator arm

**Choice:** Add `Entity` to `VisualRevealKind` (`src/model.rs`). Its `reveal_candidate_is_exact` arm validates shape only: starts with `&`, ends with `;`, inner body alphanumeric/`#`, and `decode_entity_token` returns `Some`. The view layer handles the new kind wherever `Escape` is handled (source shown on caret entry, atom rendered when unfocused); exhaustive `match` arms in `preview.rs` make the integration compiler-driven.

**Why:** Same pattern as `Escape`/`InlineHtml`; validators never re-derive the reconstruction (the prover already did).

## Risks / Trade-offs

- **Table drift vs pulldown's HTML5 decoding.** A wrong entry can only make reconstruction fail → conservative island, never a wrong rendering. Unit tests pin each entry to the character pulldown produces.
- **Invalid numeric references (`&#0;`, surrogates).** pulldown emits U+FFFD; our table returns `None` → fallback. Conservative, documented as part of the residual gap.
- **Residual gap stays visible.** Paragraphs with unproven entities still become islands; the roadmap tracks "entity references outside the proven decode table" as a secondary gap so nobody mistakes it for done.
- **Content-range semantics extension.** First run kind whose visible text is not a byte substring; contained in existing invariants (token is the authored slice), covered by new round-trip tests plus the existing differential/property suites.
- **Cost.** The prover runs only on non-identity Text slices — the same rare path escapes already use; no hot-path cost.

## Migration Plan

Pure presentation change; no persistence, settings, or network impact. One merge: model variant, prover, splitter generalization, validator arm, view handling, tests, docs. Rollback is `git revert`.

## Open Questions

- **Table breadth.** Ship the curated ~40-entry named set or widen to the full single-codepoint HTML5 list upfront (data-only, no code change)? **Default:** curated set now; widen in a follow-up if real documents hit the fallback.
- **`&#x` uppercase.** Keep pulldown's case-insensitive hex handling. **Default:** yes, both `#x` and `#X`.
