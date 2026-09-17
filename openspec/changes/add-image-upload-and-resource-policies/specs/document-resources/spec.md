## MODIFIED Requirements

### Requirement: Local images SHALL be imported as portable document resources
When a Markdown document receives supported clipboard image bytes or supported local image files through insertion, paste, or OS drop, Markion SHALL use the applicable configured image policy. Defaults SHALL preserve local import into the document-associated asset directory. Local import SHALL use a collision-safe filename and a safe document-relative URL with forward slashes, and SHALL NOT escape the configured asset directory through traversal. Local-file input SHALL additionally support keeping its original reference or uploading; clipboard image bytes SHALL support local storage or uploading. An untitled document SHALL be saved before publishing a document-relative resource or resolving a relative source path. Operations needing neither SHALL be allowed without saving. Inputs SHALL remain associated with the initiating document throughout any save prompt.

#### Scenario: Clipboard image is imported
- **WHEN** the user pastes an image while an editable document surface is focused and the clipboard policy is the default local storage
- **THEN** the image bytes are persisted under the configured document asset directory
- **AND** one undoable Markdown image reference is inserted at the original tracked selection

#### Scenario: Dropped images are imported in order
- **WHEN** supported local image files are dropped onto an editable pane with the default copy policy
- **THEN** each image is copied or reused under the document's configured asset directory
- **AND** corresponding relative Markdown image references are inserted in drop order

#### Scenario: Unsafe source filename is normalized
- **WHEN** an imported image name contains whitespace, Markdown delimiters, path separators, or traversal segments
- **THEN** the stored filename and relative Markdown URL remain within the asset directory and parse as one image destination

#### Scenario: Untitled document requires a durable base path
- **WHEN** a requested image operation needs a document-relative resource directory or relative input path in an untitled document
- **THEN** Markion requests a Markdown save location before publishing that resource
- **AND** canceling leaves the document and its resource directories unchanged

#### Scenario: Upload-only insertion into an untitled document
- **WHEN** the user uploads captured clipboard bytes or an absolute local file into an untitled document
- **THEN** Markion can insert the returned remote URL without requiring a Markdown save location

### Requirement: Images SHALL support exact replacement and practical presentation controls
An exactly mapped inline image SHALL expose source-backed editing for alt text, URL, optional title, width preset, and alignment. Replacing an image with a supported local file SHALL process that file through the configured local-image policy, retain authored alt text and presentation metadata unless the user changes them, and update the image through an undoable source mutation only after the original target is verified. Presentation metadata SHALL remain valid Markdown and SHALL degrade without destroying the resource URL in other CommonMark consumers.

#### Scenario: Existing image is replaced
- **WHEN** the user chooses Replace on an exactly mapped local image, selects another supported image, and uses the default copy policy
- **THEN** the replacement bytes are stored as a managed resource
- **AND** the canonical image source changes once while retaining alt text and presentation settings

#### Scenario: Replacement uses the selected upload policy
- **WHEN** local-image policy is upload and a replacement upload succeeds while its target is still valid
- **THEN** the original image destination is replaced with the returned URL
- **AND** its alt text, title, and presentation settings remain intact

#### Scenario: Width and alignment change
- **WHEN** the user selects a supported width preset or left, center, or right alignment
- **THEN** the canonical image source records the setting in valid Markdown
- **AND** preview and Visual Edit apply the new presentation without maintaining a second document model

### Requirement: Missing image resources SHALL be explicit and recoverable
When a local or remote image cannot be loaded or decoded, preview and Visual Edit SHALL show a concise localized failure message and a readable resource filename when available. Long filenames SHALL be bounded and ellipsized within the document column; raw system errors, full filesystem paths, signed queries, and embedded payloads SHALL NOT be rendered in the placeholder. Visual Edit SHALL retain source editing and replacement affordances for exact image ranges. Missing-resource presentation SHALL NOT mutate the document.

#### Scenario: Referenced file is missing
- **WHEN** a document references a local image that does not exist
- **THEN** the rendered surface shows an explicit missing-resource placeholder rather than silent blank space
- **AND** focusing or dismissing that placeholder does not change document version, dirty state, or undo history

#### Scenario: A failed image has a long encoded destination
- **WHEN** an image fails to load and its destination or system error is long
- **THEN** the placeholder remains inside the document column, including narrow windows
- **AND** it shows a concise localized message and decoded filename without a raw error dump
- **AND** the authored source and undo history remain unchanged

## ADDED Requirements

### Requirement: Image insertion policies SHALL be consistent across explicit entry points
Markion SHALL provide local-file policies of keep reference, copy to resources, and upload; clipboard-byte policies of save to resources and upload; and remote-image policies of keep URL, download to resources, and download then upload. Defaults SHALL be copy, save, and keep respectively. The policy SHALL apply to explicit file/URL insertion, image paste/drop, replacement, and eligible image references within newly pasted Markdown or converted rich text. Only the inserted fragment SHALL be selected by an automatic paste operation. Bare pasted URLs SHALL remain ordinary text unless the user explicitly inserts them as images. Existing embedded `data:` references SHALL remain as authored on paste and SHALL be eligible for explicit manual processing. Opening, rendering, ordinary typing, saving, undo/redo, DOCX import, and export SHALL NOT trigger this automatic pipeline.

#### Scenario: Pasted remote images are localized
- **WHEN** remote-image policy is download and pasted content contains HTTP/HTTPS image references
- **THEN** Markion inserts the content and processes the image references in that fragment
- **AND** successful downloads replace only those destinations with resource-relative URLs while failed ones keep their original URLs

#### Scenario: Local references are kept
- **WHEN** local-file policy is keep reference and a file is inserted
- **THEN** the inserted image references that existing file using a valid relative path when representable from a named document, otherwise a valid absolute path
- **AND** Markion does not copy or upload it

#### Scenario: A remote image is rehosted
- **WHEN** remote-image policy is download then upload and an HTTP/HTTPS image is explicitly inserted
- **THEN** its downloaded image bytes are passed to the selected uploader
- **AND** the final reference uses the validated upload URL only after success

#### Scenario: Opening an old document has no transfer side effects
- **WHEN** the user opens or previews a document after changing image policies
- **THEN** existing image references and files remain unchanged
- **AND** no image upload or resource-localization operation starts automatically

### Requirement: Resource directories SHALL be configurable relative to the document
Local resource publication SHALL support a template relative to the Markdown document directory, defaulting to `{document}.assets`, with `{document}` expanding to a safe document filename stem. `assets`, `./assets`, and nested templates such as `assets/{document}` SHALL be supported. The resolved destination SHALL be a non-empty descendant directory within the document directory; absolute paths, parent traversal, unsupported template variables, and symlink/junction escapes SHALL be rejected. URLs SHALL encode the full relative path safely. Equal content SHALL be reusable in the target directory; name collisions SHALL never overwrite different content. Changing the template or renaming/saving a document elsewhere SHALL NOT automatically relocate resources or rewrite historical references.

#### Scenario: Shared assets subdirectory
- **WHEN** `notes/topic.md` imports an image with template `./assets`
- **THEN** the resource is published beneath `notes/assets/`
- **AND** the inserted destination resolves from `topic.md` to that actual file

#### Scenario: Nested document-specific directory
- **WHEN** the template is `assets/{document}` and the named document is `topic.md`
- **THEN** resources are stored beneath `assets/topic/` relative to the document directory
- **AND** the inserted URL includes both directory segments

#### Scenario: Encoded Unicode resources remain readable
- **WHEN** a document with a Chinese filename imports an image into a Unicode or space-containing resource directory
- **THEN** preview and Visual Edit decode the authored local URL exactly once and read the actual published file
- **AND** existing encoded references work without rewriting the document, while literal percent signs remain literal filename characters

#### Scenario: Invalid or escaping destination
- **WHEN** the configured destination is invalid or an existing directory component redirects outside the permitted document directory
- **THEN** the operation fails before publication and reference mutation
- **AND** Markion identifies the directory setting that needs correction

#### Scenario: Existing files are preserved
- **WHEN** an import would reuse a filename occupied by different bytes
- **THEN** Markion chooses a collision-safe filename instead of overwriting that file
- **AND** importing identical content again into the same destination reuses the existing content where safely identifiable

### Requirement: Existing images SHALL support single-image and current-document actions
Markion SHALL provide Upload image and Save image to resource directory actions for a source-mapped image, plus current-document actions Upload local images, Download remote images, and Organize local images. The Format menu SHALL group image insertion and current-document commands beneath one Images submenu. A selected visual image's right-click menu SHALL provide width, alignment, source editing, replacement, save-local, and upload commands together with applicable block duplicate, move, and delete commands; these controls SHALL NOT occupy a persistent inline toolbar beneath the image. Upload local images SHALL include local files and supported embedded data images but skip already remote URLs. Download remote images SHALL include HTTP/HTTPS sources. Organize local images SHALL include local references outside the currently configured target directory and embedded data images. Upload image on a remote image SHALL explicitly download then upload it. Save image SHALL copy, download, or decode according to source kind. Batch actions SHALL show a review of eligible occurrences, unique inputs, already-satisfied entries, missing resources, and unsupported/ambiguous references before transferring or publishing files; canceling that review SHALL have no transfer or publication side effects.

#### Scenario: Existing document is uploaded
- **WHEN** a document contains repeated local images, an embedded data image, and existing remote URLs and the user confirms Upload local images
- **THEN** each unique eligible captured input is uploaded once per operation
- **AND** all selected occurrences of each successful input are updated while existing remote URLs are unchanged

#### Scenario: Existing network images are downloaded
- **WHEN** the user confirms Download remote images for the current document
- **THEN** successful downloads are published to the configured resource directory and their image destinations become relative URLs
- **AND** failed destinations remain unchanged and are reported individually

#### Scenario: Resource layout is changed explicitly
- **WHEN** the user switches from `{document}.assets` to `assets/{document}` and confirms Organize local images
- **THEN** eligible current-document resources are copied or reused in the new directory and their references updated
- **AND** old files remain available for other documents and undo

#### Scenario: Only one repeated occurrence is selected
- **WHEN** the user invokes a single-image action on one of several images with the same destination
- **THEN** only that image occurrence is replaced
- **AND** other images and ordinary links using the same destination stay unchanged

#### Scenario: A visual image is selected
- **WHEN** the user right-clicks a selected source-mapped image in Visual Edit
- **THEN** its context menu offers width, alignment, edit source, replace, save locally, and upload actions
- **AND** the image does not render a separate inline action strip in the document flow
- **AND** applicable duplicate, move, and delete block actions remain available

### Requirement: Image transformations SHALL preserve unrelated source syntax
Manual image actions SHALL support inline Markdown images, resolved reference-style Markdown images, and source-mapped raw HTML `img` elements with a `src` attribute, including images in tables and nested blocks. Only intended image destinations SHALL change; alt text, titles, presentation, surrounding text, ordinary hyperlinks, code examples, and front matter SHALL remain intact. When a shared reference definition also serves other occurrences or ordinary links, the selected image uses SHALL be converted to equivalent inline images rather than modifying those unrelated uses. Unsupported schemes, `srcset`-only images, and ambiguous source locations SHALL be skipped with a reason before transferring their contents.

#### Scenario: Shared reference definition also serves a hyperlink
- **WHEN** an uploaded reference-style image shares its definition with a normal hyperlink
- **THEN** the selected image becomes an equivalent inline image with its new URL and original description/title
- **AND** the hyperlink and shared definition remain unchanged

#### Scenario: Raw HTML retains attributes
- **WHEN** an eligible HTML image with width, height, style, and alt attributes is localized
- **THEN** only its `src` value changes with correct HTML attribute escaping
- **AND** the other attributes remain authored as before

#### Scenario: Image-looking text is not a resource
- **WHEN** code blocks, inline code, front matter, or ordinary links contain image-looking URLs
- **THEN** a document image action does not select or rewrite them

### Requirement: Background image results SHALL target the originating document and image
Pending image operations SHALL retain the originating document identity, resource base, selected source occurrences/insertion range, and captured policy. Unrelated edits SHALL be allowed while targets are tracked. A target edited, deleted, undone, ambiguously relocated, reloaded, or moved to a different resource base SHALL not receive an automatic completion. Switching tabs SHALL neither retarget a job nor activate its originating tab. Closing the originating document SHALL invalidate its automatic application. Successful unapplied outputs SHALL remain available in the operation result for explicit recovery rather than being written to another location.

#### Scenario: Typing before an existing image while it uploads
- **WHEN** the user makes an unrelated edit before an uploading image
- **THEN** the operation follows that unchanged image to its updated source location
- **AND** a successful result updates the image without losing the intervening text

#### Scenario: User deletes or undoes the target
- **WHEN** an image or pending insertion is removed, changed, or invalidated by undo before completion
- **THEN** completion does not recreate or overwrite it
- **AND** any successful URL/file is reported as unapplied

#### Scenario: Another tab becomes active
- **WHEN** the user switches tabs while an upload is running
- **THEN** the result can update only the original still-valid document
- **AND** the active tab and its selection remain unchanged

### Requirement: Resource operations SHALL preserve recoverability and undo semantics
A completed manual replacement batch SHALL apply all successful still-valid target changes as one undoable document mutation. Failed or stale targets SHALL keep their source unchanged and appear in the result summary. New file/clipboard insertion SHALL commit one ordered source edit for successful items; failed captured inputs SHALL remain recoverable. A pasted text fragment SHALL remain one paste edit, followed by at most one separate undoable automatic image-replacement edit per operation so intervening typing is never merged into it. Cancellation SHALL prevent future automatic commits and retain known results for explicit use. Unresolved captured inputs and successful unapplied outputs SHALL remain recoverable after restart without automatically resuming transfers or editing reopened documents. Undo SHALL restore source references without deleting local or remote resources, and redo SHALL reuse recorded destinations without repeating transfers. Local publication and source commits SHALL respect Git write/conflict ownership. Progress alone SHALL NOT alter document versions or derived Markdown caches.

#### Scenario: Partial batch success
- **WHEN** two images succeed and a third fails
- **THEN** the two successful destinations are applied together in one undoable edit
- **AND** the third remains unchanged with an actionable failure reason

#### Scenario: Automatic upload of a pasted image fails
- **WHEN** captured clipboard image bytes cannot be uploaded
- **THEN** Markion retains a recoverable pending input with Retry, Save locally, and Discard actions
- **AND** it does not insert a nonexistent URL or silently change the selected policy

#### Scenario: Undo and redo do not transfer again
- **WHEN** the user undoes and redoes a completed resource replacement
- **THEN** source destinations restore through the normal undo history
- **AND** no local resource or cloud object is deleted and no upload/download is repeated

#### Scenario: Restart retains unresolved work
- **WHEN** the app restarts with a captured failed input or known successful unapplied output
- **THEN** the user can recover that input or result through the image operation UI
- **AND** no upload resumes and no reopened document is changed automatically

#### Scenario: Git owns the destination
- **WHEN** Git temporarily owns the repository write barrier or conflict ownership blocks the target
- **THEN** image operations defer or report the block without publishing files or committing source through an uncoordinated path
- **AND** any later application revalidates the document base and target
