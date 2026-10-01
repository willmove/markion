## Context

See proposal.md for motivation. The visual inline recognizer and style-depth stack omit underline; the shared model and GPUI style conversion already expose it. The existing stable inline-fidelity spec predates several completed, unarchived HTML changes, so this delta adds focused requirements without reverting those changes.

## Goals / Non-Goals

**Goals:** Reuse exact tag groups, existing link navigation, and underline painting. Exercise the reported TOC in literal-escaped and actual-HTML forms.

**Non-Goals:** Rewrite Markdown to infer intended formatting or introduce a second editable HTML representation.

## Decisions

1. Add underline to the allowlisted tag recognizer and independent HTML style depths. Recognize `u` and `ins`, preserving the existing ignorable-attribute policy. Reuse `InlineStyle.underline`; no special TOC rendering path.
2. Keep parser Text events as text. Escaped `<` must never be reparsed as HTML and escaped stars must never become emphasis. Audit leading escapes at link/format boundaries and complete source reveal.
3. Validate HTML pairs by authored tag name as well as style where necessary; style aliases cannot prove that differently named opening/closing tags form a valid element. Malformed pairs retain source-backed fallback.
4. Data flow: canonical source/version -> cached Markdown blocks -> inline events, exact runs and reveal groups -> existing projection -> GPUI underline. Focus changes only the projection and does not invalidate document caches.

## Risks / Trade-offs

- Escaped source can intentionally resemble markup -> regression tests explicitly distinguish literal and semantic forms.
- Nested HTML and Markdown can overlap -> test full source reveal, byte-safe caret mappings, and adjacent styles.
- Unsupported attributes and arbitrary tags remain visible inert source -> document this boundary; do not discard markup silently.

## Migration Plan

No migration. Source bytes and anchor destinations are unchanged. Reverting the presentation changes restores the former rendering.
