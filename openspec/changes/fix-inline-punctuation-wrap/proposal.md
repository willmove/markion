## Why

In rendered Markdown, closing punctuation can wrap away from the text it follows. A paragraph containing inline math makes this especially visible in Chinese because the mixed layout wraps a punctuation mark together with the long run of prose that follows it.

## What Changes

- Use text-based wrap opportunities across style boundaries in mixed prose, so closing punctuation stays with the preceding text while following CJK prose can use the remaining line width.
- Respect prohibited breaks before closing punctuation in shaped prose as well.
- Preserve authored breaks, inline atoms, and the existing source-backed selection and caret mapping.

Non-goals include changing Markdown parsing, document caches, and inline atom rendering.

## Capabilities

### New Capabilities

### Modified Capabilities

- `document-typography` gains a requirement for wrapping punctuation in rendered prose.

## Impact

Visual Edit, Split Preview, and Read share the change in the rendered text layout. The per-document-version derived Markdown caches remain reusable.
