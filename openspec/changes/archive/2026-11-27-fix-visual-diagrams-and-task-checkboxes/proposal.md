## Why

Visual Edit can remain on the Mermaid loading placeholder after rendering finishes because it looks up the raw editor payload under a different cache key. Checked and unchecked task markers can have visibly different sizes because they use different Unicode glyphs.

## What Changes

- Use the same parsed diagram body for rendering and cache lookup in Visual Edit, Split Preview, and Read mode.
- Give checked and unchecked Visual Edit task markers the same square dimensions and align them with the first line of the item.

## Capabilities

### New Capabilities

### Modified Capabilities

- `diagram-rendering` specifies that all three rendered modes share the rendered fence result across line-ending and indentation variants of the fence.
- `markdown-editing` specifies consistent checked and unchecked checkbox geometry in Visual Edit.

## Impact

Visual Edit derives the diagram body once with the per-version Markdown model. Background rendering, cache limits, and source-backed editing continue through their existing paths.

Non-goals include export changes and Preview or Read task-list styling.
