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

### Requirement: Data-URI image destinations SHALL be decoded and rendered inline

A Markdown image whose destination is a `data:` URI (`data:<mediatype>[;base64],<data>`) SHALL be rendered in preview, Visual Edit, and raw-HTML `<img>` surfaces by decoding the URI payload entirely in-process — without issuing a network request or reading from disk. Base64-encoded payloads SHALL be decoded to bytes and fed into the same decode pipeline used for local and remote images. Non-base64 (URL-encoded) data-URI payloads SHALL be percent-decoded to bytes and rendered the same way. Decoded data-URI images SHALL be cached and deduplicated under the same bounded preview-image cache as local and remote images, keyed by the full URI, so repeated occurrences do not re-decode.

#### Scenario: Base64 PNG data URI renders

- **WHEN** a document contains an image with destination `data:image/png;base64,<base64-encoded PNG bytes>`
- **THEN** preview, Visual Edit, and raw-HTML surfaces render the decoded PNG
- **AND** no outbound network request is made for the image

#### Scenario: SVG data URI renders as a vector image

- **WHEN** a document contains an image with destination `data:image/svg+xml;base64,<base64-encoded SVG>` or an equivalent non-base64 `data:image/svg+xml,...` URI
- **THEN** the surface renders the SVG through the vector rasterization path
- **AND** the result is presented at the SVG's intrinsic size, subject to the same maximum-edge clamp as file-based SVGs

#### Scenario: Non-base64 data URI is percent-decoded

- **WHEN** a document contains an image with a non-base64 data URI (no `;base64` marker)
- **THEN** the payload is percent-decoded to bytes and decoded as an image of the declared media type

#### Scenario: Repeated data URI deduplicates in cache

- **WHEN** the same data URI appears more than once in a document or across visible documents
- **THEN** the payload is decoded at most once per cache residency
- **AND** each occurrence renders the cached result

#### Scenario: Malformed data URI shows explicit recovery state

- **WHEN** a document contains a data URI that cannot be parsed or whose decoded bytes are not a supported image format
- **THEN** the surface shows an explicit missing-resource placeholder rather than silently dropping the image
- **AND** the placeholder does not mutate the document source

### Requirement: Sync SHALL check note attachment participation without rewriting content
For participating notes, Markion SHALL identify newly referenced local resources absent from the tracked or selected content, including ignored, missing, and out-of-repository resources. Approved new notes and images SHALL follow the persisted scope without repeated selection. An omitted resource SHALL offer explicit include, existing resource-organization, repair, or acknowledged-omission choices as applicable. Acknowledgment SHALL be invalidated when the relevant reference/resource changes. Network URLs and embedded data URIs SHALL retain their existing source semantics; sync SHALL NOT automatically download them, rewrite URLs, or remove unreferenced resources.

#### Scenario: Note and managed image are both approved
- **WHEN** an eligible new note references a new image within approved resource scope
- **THEN** both participate in the normal one-click commit without another file-selection dialog

#### Scenario: New reference points outside the repository
- **WHEN** a participating note newly references an image outside repository scope
- **THEN** the app identifies its lack of portability and offers an explicit organize/copy or acknowledged-omission action without silently changing source

#### Scenario: Referenced attachment is ignored
- **WHEN** a participating note newly references an untracked Git-ignored image
- **THEN** the app explains the omission and requires an explicit choice rather than reporting the note's resources as fully synchronized

### Requirement: Git resource updates SHALL coordinate writes and invalidate affected images
Image imports/replacements SHALL respect repository write admission and conflict ownership. Git changes to resource bytes SHALL invalidate affected preview and image-tab caches even if referring Markdown bytes are unchanged. Unchanged Markdown SHALL retain its document version and derived state.

#### Scenario: Image changes without a Markdown edit
- **WHEN** pull replaces an image used by an open note but the note's source is identical
- **THEN** the displayed image refreshes and the note's Markdown version/caches are not invalidated solely for that resource update

#### Scenario: Image import occurs during repository mutation
- **WHEN** a paste/drop would store a new image while Git owns the repository write barrier
- **THEN** the app does not write the resource or insert a reference to an uncommitted failed import through an uncoordinated path
