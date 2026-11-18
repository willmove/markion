## Why

The reproduced P1 issues can alter authored list structure, click targets, and imported content. Dark themes also make ordinary Markdown and HTML tables difficult to read.

## What Changes

- Preserve list ownership when moving or changing the nesting of a subtree (#52, #53, #54).
- Retain explicit link destinations and the literal meaning of HTML and DOCX prose (#34, #41, #47).
- Preserve and display rich content and paragraph boundaries in rectangular DOCX table cells. Emit drawings in source order (#36, #38).
- Derive native table surfaces and controls from the active palette (#27).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing` covers list ownership and literal imported text.
- `export` covers explicit link destinations.
- `docx-import` covers ordered drawing and table content fidelity.
- `tables-outline` covers native table palettes and inline images.

## Impact

Source edits remain checked and undoable. List ownership stays with the versioned preview and Visual Edit blocks. Import libraries remain GPUI-free.
