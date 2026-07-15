## Why

The full test suite currently fails because standalone raw HTML blocks can disappear from derived preview blocks and visible whitespace between adjacent HTML inline elements can be lost. This breaks README-style centered logos and language selectors even though raw HTML is valid input for Markion's Markdown preview.

## What Changes

- Preserve standalone raw HTML emitted by `pulldown-cmark` as an HTML preview block, including when parser container state would otherwise cause it to be treated as inline content.
- Preserve a single visible separator for whitespace that occurs between adjacent styled or linked HTML text runs.
- Add regression coverage for raw HTML block ordering and README-style centered image/text rendering.
- Non-goals: implementing a general-purpose browser HTML renderer, changing HTML export semantics, or bypassing the existing per-document-version derived-state cache.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Clarify that raw HTML blocks remain available to the rendered preview and that their visible text spacing and ordering are preserved.

## Impact

- Affected code: Markdown event-to-preview-block construction in `src/lib.rs`, HTML preview decomposition in `src/parse.rs`, and their unit tests.
- No public API or dependency changes are expected.
- The existing invariant that derived Markdown state is cached once per document version remains unchanged.
