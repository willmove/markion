## Why

On main at `6e3fdcc`, an escaped pipe after consecutive backslashes can make a Visual Edit cell target a different column and make Format Table add a column (#30). Format Table can also turn a pipe-containing paragraph immediately before a table into its header (#31).

## What Changes

- Use the rendered GFM table's source ownership for lookup and commands that mutate a table.
- Apply one GFM pipe-boundary rule to source cell ranges, caret column lookup, and table rewrites.
- Add regression coverage for exact cell edits and Format Table, including adjacent prose, Unicode, and original line endings.

Non-goals are changes to raw HTML tables, column-width policy, and Markdown export formats.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing` specifies that table editing uses parser-owned table ranges and semantically correct cell delimiters.

## Impact

The change touches Markdown table parsing, editing lookup, and source formatting. Editing reuses per-version derived table blocks and the existing checked mutation and undo paths.
