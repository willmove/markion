## Why

Opening a valid Markdown document can terminate Markion during its first Visual Edit render when a top-level list item contains a blockquote that itself contains a list. The parser assigns the outer list draft to the currently open quote, producing a reversed visual source range; slicing that range panics inside the GPUI window callback and Windows reports the abort as `0xC0000409`.

## What Changes

- Preserve the destination container of every in-progress list-item draft so entering or leaving a nested blockquote cannot reroute an already-authored outer item.
- Keep list-item nested-block truncation state with the item that owns it, preventing state from leaking between parent, child, quoted, and top-level items.
- Require parser and Visual Edit derivation to produce ordered, in-bounds, UTF-8-safe source ranges for list/blockquote/list combinations.
- Add a Visual Edit range-validation backstop that degrades an invalid derived leaf to source-backed fallback coverage instead of indexing an invalid Rust string range.
- Add regressions for the minimal list → blockquote → list topology, CRLF and UTF-8 content, sibling/continuation variants, parser ownership, and visual coverage.

Non-goals: redesigning the streaming parser as a general recursive Markdown DOM, changing Markdown semantics or persisted files, expanding the Visual Edit coverage matrix, or changing the existing per-document-version derived-state caches.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: nested container combinations must retain their authored ownership and yield safe, monotonic source ranges without crashing preview or Visual Edit rendering.
- `engineering-quality`: parser ownership and Visual Edit range invariants gain executable regression evidence for cross-container nesting and a non-panicking invalid-range fallback.

## Impact

- `src/parse.rs`: list-item draft metadata and flush routing helper.
- `src/lib.rs`: `pulldown-cmark` item-event routing, nested-block truncation ownership, and parser tests.
- `src/visual.rs`: source-range validation/fallback and pure projection tests.
- Existing `Arc`-shared, per-document-version preview and Visual Edit caches remain unchanged; no GPUI dependency is introduced into workspace member crates.
