## MODIFIED Requirements

### Requirement: Images SHALL support exact replacement and practical presentation controls
An exactly mapped inline image SHALL expose source-backed editing for alt text, URL, optional title, width, and alignment. Width SHALL be an integer percent from 10 through 100 inclusive, serialized in the image title metadata as `{width=N align=…}`. The focused Visual Edit chrome SHALL keep 25%, 50%, 75%, and 100% preset buttons and SHALL also expose a drag handle that resizes the image continuously in that range. Pointer-move while dragging SHALL update the on-screen width without mutating document text, dirty state, undo history, document version, or per-version derived caches. Releasing the drag SHALL write the new width as one undoable source mutation through the existing presentation path. Widths within 3 percent of a preset MAY snap to 25, 50, 75, or 100. Replacing a local image SHALL retain authored alt text and presentation metadata unless the user changes them, SHALL store the new bytes through the resource workflow, and SHALL update the image in one undoable source mutation. Presentation metadata SHALL remain valid Markdown and SHALL degrade without destroying the resource URL in other CommonMark consumers. Reference-style images SHALL NOT show width or drag-resize controls.

#### Scenario: Existing image is replaced
- **WHEN** the user chooses Replace on an exactly mapped local image and selects another supported image
- **THEN** the replacement bytes are stored as a managed resource
- **AND** the canonical image source changes once while retaining alt text and presentation settings

#### Scenario: Width and alignment change
- **WHEN** the user selects a supported width preset or left, center, or right alignment
- **THEN** the canonical image source records the setting in valid Markdown
- **AND** preview and Visual Edit apply the new presentation without maintaining a second document model

#### Scenario: Arbitrary width between 10 and 100 round-trips
- **WHEN** an inline image title contains `{width=40 align=center}`
- **THEN** preview and Visual Edit render the image at 40% of the content column
- **AND** confirming a presentation edit preserves `width=40`

#### Scenario: Drag-resize overlays then commits once
- **WHEN** the user drags the resize handle of a focused exactly mapped inline image
- **THEN** the on-screen width follows the pointer without changing document version
- **AND** releasing the drag writes the new `{width=N align=…}` metadata as one undoable mutation

#### Scenario: Reference-style images keep presentation controls hidden
- **WHEN** the focused image is reference-style
- **THEN** width presets and the drag-resize handle are not shown
- **AND** the source is not rewritten into `![alt](url)` form
