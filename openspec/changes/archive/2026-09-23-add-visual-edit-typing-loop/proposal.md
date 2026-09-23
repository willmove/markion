## Why

Visual Edit already renders headings, lists, quotes, emphasis, and emoji shortcodes, but the *typing* loop still feels like a preview you can poke rather than Typora-style live Markdown: delimiters are not auto-paired, leaving a heading or list by pressing Enter does not reliably hide that row's block markers, task-list checkboxes are inert glyphs (roadmap gap 7), and typing `:` never offers shortcode completion. Closing this loop is the highest-leverage way to make Visual Edit feel like writing, not like editing source through a projection.

## What Changes

- Auto-pair Markdown and matching delimiters while typing on an editing surface (`*`, `_`, `` ` ``, `$`, `=`, brackets/quotes): insert the closer after the caret, wrap a non-empty selection, skip the closer when it is already the next character, and delete the empty pair on Backspace. A persisted General preference (`markdown_auto_pair`, default on) can disable it.
- After Enter (or any caret move that leaves a heading, list, task, or quote row), hide that row's structural prefixes immediately and show the rendered marker; a just-typed ATX / list / task / quote prefix at line start must reclassify as that block so Enter does not leave `#` / `-` as visible prose.
- Clicking the Visual Edit task-list checkbox glyph toggles `[ ]` / `[x]` as one exact source mutation. Close the WYSIWYG coverage-roadmap gap for task-list checkbox click.
- Typing `:` at a word boundary in Visual Edit (and the source editor) opens a slash-palette-like shortcode completer against the maintained emoji table; confirming inserts the canonical `:shortcode:` as one undoable edit.

### Non-goals

YAML front matter, indented/unclosed code, definition lists, malformed tables, reference-style images, math-failure islands, Typora `[TOC]`, emoji *character* insertion (source stays `:name:`), cloud emoji packs, auto-pair inside fenced-code / math / diagram payload editors, Read/Split Preview checkbox editing, and guessing rendered-tree mutations.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Auto-pair, hide block markers when a Visual Edit row loses the caret, clickable task-list checkboxes, emoji `:` completer; close the task-checkbox roadmap gap.
- `chrome-platform`: Persist and expose the Markdown auto-pair preference on Preferences → General.
- `ui-i18n`: Localize the auto-pair preference, emoji palette chrome, and empty-match copy.

## Impact

- **Code:** Visual Edit / source text input (`src/app/editor_element.rs`, `src/app/editing.rs`, `src/app/preview.rs` list-item chrome); prefix-reveal gating; `src/parse.rs` `emoji_for_shortcode` (query API, possibly a larger static table); slash-palette-style overlay in `src/app/root_view.rs`; preferences in `config.toml` / General tab; `docs/visual-editing-quality.md` matrix/roadmap; `src/i18n.rs`.
- **Invariants:** Every mutation is one canonical `MarkdownDocument.text` edit through the existing dirty/undo/autosave/recovery path. Showing, filtering, or dismissing auto-pair / emoji UI MUST NOT increment document version or invalidate per-version derived caches (`Arc` preview/visual/outline/stats). Syntax-highlight memoization and cached text handles stay intact. `crates/*` stay GPUI-free.
- **Tests:** Pure pairing/skip/unwrap and shortcode-filter tests; Visual Edit GPUI tests for Enter-then-hidden prefixes, checkbox toggle + undo, emoji confirm/cancel/IME; preference round-trip; i18n exhaustiveness.
