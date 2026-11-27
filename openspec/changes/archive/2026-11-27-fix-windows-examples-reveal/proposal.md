## Why

On Windows, Help → Examples and Templates finds the bundled index, but the Explorer request can fail with an invalid-argument error. The status bar currently displays the menu label even when the request fails.

## What Changes

- Reveal the bundled index using Markion's system file manager launcher on Windows, with Explorer-compatible path separators and quoting.
- Report launcher errors in the status bar using the existing localized file manager message.
- Verify that Explorer selects the index from a resource path containing spaces and non-ASCII characters.

## Capabilities

### Modified Capabilities

- `bundled-examples` requires localized feedback when Windows cannot launch the file manager and covers resource paths with spaces and non-ASCII characters.

## Impact

The Help action and Windows file manager launcher change. Both menus share the action, and the workspace, open documents, and cached Markdown state are preserved.

Non-goals include changes to the example collection and the package layout.
