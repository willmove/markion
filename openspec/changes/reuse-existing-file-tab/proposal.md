## Why

Opening the same Markdown file repeatedly creates duplicate tabs that hold redundant document state, increasing memory use and making tab navigation noisier. Reusing an existing tab for an already-open file keeps the session compact without changing the multi-tab editing model.

## What Changes

- Detect when a file being opened already matches a document path in an existing tab.
- Focus the existing tab and show its current content/state instead of creating or replacing another tab.
- Apply the behavior to file-tree opens, Open in New Tab, File->Open, and external drag/drop opens that load Markdown files by path.
- Non-goals: this does not merge unsaved untitled documents, deduplicate recovered documents without paths, or add persistent cross-session tabs.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: multi-document tabs reuse already-open file tabs instead of opening duplicate tabs for the same file path.

## Impact

- Affected code: `src/main.rs` tab/file-open flows and related tab tests.
- No new dependencies or storage migrations.
- Preserves the per-tab cached derived Markdown state; focusing an existing tab must not reparse or reset that tab.
