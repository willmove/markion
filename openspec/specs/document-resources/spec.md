# document-resources Specification

## Purpose
TBD - created by archiving change complete-p0-editor-workflows. Update Purpose after archive.
## Requirements
### Requirement: Local images SHALL be imported as portable document resources
When a named Markdown document receives supported image bytes from the clipboard or supported image files from an OS drop, Markion SHALL store the bytes in a document-associated asset directory, SHALL use a collision-safe filename, and SHALL insert ordinary Markdown image syntax with a safe document-relative URL. The URL SHALL use forward slashes and SHALL NOT escape the asset directory through traversal. An untitled document SHALL be saved before a resource is imported.

#### Scenario: Clipboard image is imported
- **WHEN** the user pastes an image while an editable document surface is focused
- **THEN** the image bytes are persisted under the document's asset directory
- **AND** one undoable Markdown image reference is inserted at the current selection

#### Scenario: Dropped images are imported in order
- **WHEN** one or more supported local image files are dropped onto an editable pane
- **THEN** each image is copied or reused under the document's asset directory
- **AND** corresponding relative Markdown image references are inserted in drop order

#### Scenario: Unsafe source filename is normalized
- **WHEN** an imported image name contains whitespace, Markdown delimiters, path separators, or traversal segments
- **THEN** the stored filename and relative Markdown URL remain within the asset directory and parse as one image destination

#### Scenario: Untitled document requires a durable base path
- **WHEN** the user pastes or drops an image into an untitled document
- **THEN** Markion requests a Markdown save location before storing the resource
- **AND** canceling the request leaves the document and filesystem unchanged

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

### Requirement: Missing image resources SHALL be explicit and recoverable
When a local image URL cannot be resolved or decoded, preview and Visual Edit SHALL show a visible missing-resource state containing the alt text or resource URL and SHALL offer an edit or replacement affordance when the source range is exact. Missing-resource presentation SHALL NOT mutate the document.

#### Scenario: Referenced file is missing
- **WHEN** a document references a local image that does not exist
- **THEN** the rendered surface shows an explicit missing-resource placeholder rather than silent blank space
- **AND** focusing or dismissing that placeholder does not change document version, dirty state, or undo history

