## Why

Visual Edit exposes `<u>` tags in imported table-of-contents links because its inline HTML recognizer omits underline even though the model and renderer already support it. Related escape and HTML nesting cases need regression coverage to keep literal markup distinct from formatting and preserve exact edits.

## What Changes

- Render paired inline `<u>` and `<ins>` as underline, including supported ignorable attributes and composition with Markdown links and formatting.
- Audit and correct closely related escape and HTML pair mapping defects uncovered by regression tests.
- Preserve literal escaped markup, full source reveal, UTF-8 mappings, and per-version caches.

Non-goals: browser CSS, arbitrary HTML DOM editing, automatic removal of authored backslashes, and changes to imported source or anchor destinations.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Underlined inline HTML and exact literal/reveal behavior in mixed Markdown prose.

## Impact

`src/parse.rs`, `src/visual.rs`, related regression tests, and the visual editing quality matrix. Existing `InlineStyle.underline` and GPUI styling are reused; derived state remains cached per document version. No dependencies or persistence changes.
