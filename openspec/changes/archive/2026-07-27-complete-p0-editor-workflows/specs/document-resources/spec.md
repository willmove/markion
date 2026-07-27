## ADDED Requirements

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
An exactly mapped inline image SHALL expose source-backed editing for alt text, URL, optional title, width preset, and alignment. Replacing a local image SHALL retain authored alt text and presentation metadata unless the user changes them, SHALL store the new bytes through the resource workflow, and SHALL update the image in one undoable source mutation. Presentation metadata SHALL remain valid Markdown and SHALL degrade without destroying the resource URL in other CommonMark consumers.

#### Scenario: Existing image is replaced
- **WHEN** the user chooses Replace on an exactly mapped local image and selects another supported image
- **THEN** the replacement bytes are stored as a managed resource
- **AND** the canonical image source changes once while retaining alt text and presentation settings

#### Scenario: Width and alignment change
- **WHEN** the user selects a supported width preset or left, center, or right alignment
- **THEN** the canonical image source records the setting in valid Markdown
- **AND** preview and Visual Edit apply the new presentation without maintaining a second document model

### Requirement: Missing image resources SHALL be explicit and recoverable
When a local image URL cannot be resolved or decoded, preview and Visual Edit SHALL show a visible missing-resource state containing the alt text or resource URL and SHALL offer an edit or replacement affordance when the source range is exact. Missing-resource presentation SHALL NOT mutate the document.

#### Scenario: Referenced file is missing
- **WHEN** a document references a local image that does not exist
- **THEN** the rendered surface shows an explicit missing-resource placeholder rather than silent blank space
- **AND** focusing or dismissing that placeholder does not change document version, dirty state, or undo history

