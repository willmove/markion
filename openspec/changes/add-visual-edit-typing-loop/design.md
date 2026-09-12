## Context

See proposal.md for motivation. Visual Edit already projects headings, lists, quotes, emphasis, and `:shortcode:` emoji through a source-backed `VisualBlock` stream. Typing still goes through `replace_text_in_range` (`src/app/editor_element.rs`) with no pairing; task-list rows paint `☐`/`☑` as inert text (`src/app/preview.rs` list-item chrome); emoji lookup is a closed `match` in `src/parse.rs` (`emoji_for_shortcode`) with no completer; structural prefixes reveal when the caret is inside or at the end of `VisualBlockPrefix.source_range`.

Invariants that shape the approach: `MarkdownDocument.text` is the only persisted representation; derived preview/visual/outline/stats stay `Arc` caches keyed by document version; showing UI (slash palette, find overlay) today does not bump that version; `crates/*` stay GPUI-free.

```
  keystroke / click
        │
        ▼
  interaction-only? (palette filter, hover, prefix hide after caret left)
        │ yes ──▶ no version bump, no cache rebuild
        ▼ no
  one canonical SourceEdit (pair / toggle [ ]↔[x] / insert :name: / Enter)
        │
        ▼
  existing dirty / undo / autosave / recovery
        │
        ▼
  version++ ──▶ incremental Visual Edit derivation (fallback to full parse)
```

## Goals / Non-Goals

**Goals:**

- Pairing, checkbox toggle, and emoji confirm are ordinary source mutations with one-undo semantics.
- Palette open/filter/dismiss and unfocused prefix hiding are interaction-only.
- Task-checkbox click leaves the WYSIWYG coverage gap class.

**Non-Goals (design-level):**

- A second rich-text document or inferred checkbox/emoji object store.
- Pairing logic inside `crates/*` (keep GPUI-free members unaware of keystrokes).
- Reworking slash-command ownership; emoji reuses its chrome pattern only.

## Decisions

### D1: Auto-pair at the text-input boundary, not in the parser

Intercept collapsed or ranged inserts in the shared source-input path used by the source editor and Visual Edit prose (and Visual Edit table-cell editors), *before* `prepare_range_mutation`. For a single typed opener (`*`, `_`, `` ` ``, `$`, `(`, `[`, `{`, `"`, `'`) with an empty selection, insert opener+closer and place the caret between them. For a non-empty selection, wrap it. If the next source character is already the matching closer, skip over it instead of inserting. Backspace on an empty pair deletes both characters. A preceding `\` suppresses pairing. IME marked ranges skip pairing until commit.

Do **not** pair `=` (assignment/`==` highlight is too ambiguous in prose). Do **not** pair inside fenced-code, block-math, or diagram payload editors.

**Why not a keymap of hidden insert-pair actions?** Platform IME and Visual Edit projection already funnel through one replace helper; a second action table would miss IME commits and cell editors.

**Why not pair in `crates/markdown`?** Pairing is an editor policy over UTF-8 offsets, not a parse concern; putting it in a member crate would either pull GPUI or fork a second mutation API.

### D2: Unfocused structural prefixes hide from caret ownership, not from a second rewrite

Keep the existing prefix-reveal predicate (caret inside prefix, or at prefix end, or empty payload that owns the caret). After Enter (or any caret move) the previous heading/list/quote/task row no longer owns the caret, so its `#` / `-` / `1.` / `>` / `- [ ]` source stays in `MarkdownDocument.text` but is not painted. No extra source edit is required to hide markers.

Additionally, a Visual Edit paragraph whose line is exactly a typed ATX/list/task/quote prefix (with optional title text after ATX) must be reclassified by the *next ordinary derivation* as that block — the existing pulldown-cmark path already does this once the line is a heading/list/quote; tests must lock that the unfocused row then shows the rendered glyph, not the raw marker characters as prose.

### D3: Checkbox click mutates the proven task-marker bytes

The painted `☐`/`☑` column is a hit target only in Visual Edit, and only when the prefix is *not* revealed (while revealed, the user edits `- [ ]` as source). A primary click maps to the owning `VisualBlock`'s task prefix and replaces the checkbox token `[ ]` ↔ `[x]` (`[X]` normalizes to `[x]` on toggle-off) as one checked mutation. The click MUST NOT insert a caret into the item text as a side effect beyond placing the caret in that item so undo selection is defined. Read and Split Preview stay non-editing.

**Why not a hidden Format action only?** The gap is pointer interaction on the glyph; keyboard Format → Task List already exists and must keep wrapping/toggling list type, not this glyph.

### D4: Emoji completer is a sibling of slash state, inserting `:shortcode:`

Track a per-tab `EmojiQuery { start, query, selected }` analogous to `SlashCommandState`. Trigger when the user types `:` at a word boundary (start of line, whitespace, or opening punctuation) outside code/math/diagram payloads and outside an IME composition. Filter the maintained shortcode table by ASCII prefix; render a localized palette (reuse slash-command layout: occluded panel, selected row, click/Enter/Tab confirm, Esc dismiss). Confirm replaces the open `:query` with `:name:` (closing colon included) as one undoable edit. Canonical source stays the shortcode; preview/Visual Edit continue to render the glyph through the existing extended-inline parser.

Expand `emoji_for_shortcode` into a queryable static table (current entries plus a modest common GitHub-name set). No new crate, no network. If slash palette is open (line starts with `/`), slash wins and `:` inside that query does not open emoji.

### D5: Auto-pair preference is a General boolean defaulting on

Persist `markdown_auto_pair` in `config.toml` (bool, default `true`, missing/invalid → true). Preferences → General toggle, included in reset. Pairing checks this flag at insert time; toggling is presentation/policy only (no document version bump).

## Risks / Trade-offs

- [Pairing fights CJK/IME or `$100`] → Skip while a marked range is active; preference off; do not pair `=`.
- [Checkbox click vs caret placement on the same row] → Hit-test the marker column first; a miss falls through to existing caret placement.
- [Emoji `:` vs time `12:30` or URLs] → Require a word-boundary `:` and ASCII shortcode charset; ignore `://`.
- [Prefix hide looks like data loss] → Source still contains markers; focusing the row reveals them (existing progressive-reveal). Tests assert version-stable hide on caret leave.
- [Larger emoji table on the typing path] → Static `&[(&str, &str)]` prefix filter, bounded visible rows; no per-keystroke parse of the whole document.

## Migration Plan

- Absent `markdown_auto_pair` → on (Typora-like). No document migration.
- Rollback: revert the change; documents remain valid Markdown (`:name:` and `[x]` unchanged).

## Open Questions

None that affect the spec or task breakdown. Palette row cap (about 8–12) can be tuned in implementation without changing observable confirm/dismiss behavior.
