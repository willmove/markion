## Purpose

Defines how an authenticated local MarkNice publishing session lets users recover its exact Markdown source and download safe, portable themed HTML and browser-generated DOCX artifacts without changing the Markion document.

## ADDED Requirements

### Requirement: The current session Markdown can be copied exactly
The publishing workspace SHALL provide a user-initiated Copy Markdown action that writes the exact current browser editor value to the clipboard as `text/plain`, including session-local edits and authored whitespace, without substituting the original launch snapshot. The workspace SHALL report success only after the clipboard operation succeeds and SHALL provide localized, actionable feedback when the content is empty or browser clipboard access fails.

#### Scenario: Session edits are copied
- **WHEN** the user changes the Markdown in the browser workspace and invokes Copy Markdown
- **THEN** the clipboard plain-text value is byte-for-byte equivalent to the current browser editor value when encoded as UTF-8
- **AND** it is not replaced by the Markdown captured when the session launched

#### Scenario: Clipboard access is denied
- **WHEN** the browser denies both the preferred plain-text clipboard operation and its supported fallback
- **THEN** the workspace does not report copy success
- **AND** it tells the user to allow clipboard access or manually copy from the editor

#### Scenario: Empty Markdown is not reported as copied
- **WHEN** the current browser editor contains no Markdown and the user invokes Copy Markdown
- **THEN** the workspace does not report a successful copy
- **AND** it shows a localized empty-content message

### Requirement: The current browser Markdown can be formatted with MarkNice controls
The authenticated workspace SHALL provide a P0 MarkNice-style formatting toolbar for H1, H2, H3, bold, italic, underline, ordered list, unordered list, inline code, link, quote, fenced code block, image syntax, and table insertion. It SHALL also provide Ctrl/Cmd+B, Ctrl/Cmd+I, Ctrl/Cmd+U, and Ctrl/Cmd+K while the Markdown textarea is focused.

#### Scenario: A toolbar command formats the current selection
- **WHEN** a user selects Markdown text and invokes a supported toolbar command
- **THEN** the workspace SHALL apply the corresponding MarkNice wrapper or line prefix to that exact selection
- **AND** it SHALL restore textarea focus and a useful resulting selection or caret
- **AND** it SHALL render the updated current Markdown in the preview

#### Scenario: A toolbar command has no active selection
- **WHEN** a user invokes a supported formatting command with an empty selection
- **THEN** the workspace SHALL insert a localized placeholder or structural template appropriate to that command
- **AND** it SHALL select the editable placeholder or place the caret at the next useful edit position

#### Scenario: Existing formatting is toggled
- **WHEN** the selected text or selected lines already have the command's supported wrapper or prefix
- **THEN** invoking that command again SHALL remove that formatting where the pinned MarkNice behavior defines a toggle
- **AND** it SHALL preserve the underlying Markdown text

#### Scenario: A supported keyboard shortcut is pressed
- **WHEN** the Markdown textarea is focused and the user presses Ctrl or Cmd with B, I, U, or K without Alt
- **THEN** the workspace SHALL invoke the matching formatting command
- **AND** it SHALL prevent the browser default only for a handled shortcut

#### Scenario: Formatting remains browser-session local
- **WHEN** a toolbar or shortcut command changes the current workspace Markdown
- **THEN** the change SHALL remain in the browser-owned session and its preview
- **AND** it SHALL NOT mutate Markion's GPUI document, document version, tab state, or saved file

#### Scenario: Image formatting is requested
- **WHEN** the user invokes the image command
- **THEN** the workspace SHALL insert Markdown image syntax
- **AND** it SHALL NOT upload, import, or transmit a local image

#### Scenario: The formatting toolbar is localized and narrow-screen accessible
- **WHEN** the workspace is shown in any declared locale or at the narrow layout breakpoint
- **THEN** every formatting action SHALL have a localized title and accessible name
- **AND** the toolbar SHALL remain reachable through horizontal scrolling without covering the Markdown editor

### Requirement: The current MarkNice presentation can be downloaded as portable HTML
The publishing workspace SHALL provide a user-initiated themed HTML download generated from the current browser-session Markdown and the currently selected MarkNice theme, font-size offset, and paragraph-spacing offset. The downloaded UTF-8 `.html` file SHALL preserve the sanitized rendered article and rendered math without requiring Markion, the loopback service, the hosted MarkNice site, a CDN, or another application asset after download. Authored remote HTTP(S) resources MAY remain remote and SHALL be distinguishable from embedded content.

#### Scenario: Current session presentation is downloaded
- **WHEN** the user edits the browser Markdown, changes its publishing presentation, and invokes Download themed HTML
- **THEN** the browser starts a `.html` download whose article content reflects the current session editor value
- **AND** opening that file preserves the selected MarkNice theme, typography offsets, structural formatting, and rendered math

#### Scenario: HTML opens after the publishing session ends
- **WHEN** a themed HTML artifact containing no authored remote dependency is downloaded and the Markion process and publishing session are then closed
- **THEN** the downloaded article remains readable and styled when opened without network access
- **AND** it requests no asset from the former loopback origin, a CDN, or the hosted MarkNice site

#### Scenario: Managed local images are embedded
- **WHEN** the current preview contains a managed local image that the authenticated session can read safely
- **THEN** the themed HTML download embeds an export-safe representation of that image
- **AND** the artifact contains no loopback, blob, or filesystem URL for that image

#### Scenario: Empty content does not create an HTML artifact
- **WHEN** the current browser editor has no renderable content and the user invokes Download themed HTML
- **THEN** no download is started
- **AND** the workspace shows a localized empty-content message

### Requirement: The current MarkNice presentation can be exported to DOCX in the browser
The publishing workspace SHALL provide a user-initiated browser-side DOCX action that converts the current sanitized MarkNice presentation into a `.docx` download without submitting article content to Markion, a Node service, a CDN, or a hosted conversion API. The result SHALL preserve the pinned MarkNice browser Word-export compatibility for representative headings, paragraphs, inline emphasis, lists, blockquotes, tables, code, links, rendered math, and supported images, and SHALL be identified as a browser-generated MarkNice DOCX rather than Markion's native or Pandoc DOCX export.

#### Scenario: DOCX is generated from current session state
- **WHEN** the user changes the browser Markdown or publishing presentation and invokes Download DOCX
- **THEN** the browser creates and starts downloading a structurally valid `.docx` package from that current state
- **AND** no document-conversion request containing article content leaves the browser tab

#### Scenario: Browser DOCX preserves representative content
- **WHEN** the compatibility corpus containing headings, inline formatting, nested lists, blockquotes, tables, code, links, math, and supported images is exported
- **THEN** the downloaded document opens in the documented target Microsoft Word versions
- **AND** its content and presentation match the maintained normalized browser-DOCX expectations within the documented compatibility envelope

#### Scenario: Managed local images are embedded in DOCX
- **WHEN** a safely readable managed local image appears in the current preview
- **THEN** the browser-generated DOCX embeds the image bytes or an equivalent package-contained representation
- **AND** the DOCX contains no loopback, blob, or filesystem reference to that image

#### Scenario: DOCX generation fails
- **WHEN** the browser runtime cannot produce a valid DOCX blob or initiate its download
- **THEN** the workspace does not report that the file was saved
- **AND** it shows a localized actionable failure while keeping the current Markdown and presentation state intact

### Requirement: Downloaded artifacts enforce a safe resource boundary
Before themed HTML or DOCX generation, the workspace SHALL prepare a sanitized export clone that removes executable authored content and unsafe URL schemes. A downloaded artifact SHALL NOT contain a session claim, session bearer token, Markion loopback URL, blob URL, filesystem path, event handler, executable script, embedded frame or plugin, form submission, or `javascript:` navigation. Safely readable managed resources SHALL be embedded without widening the immutable launch-time resource allowlist; missing, expired, unsupported, or unresolved protected resources SHALL use a visible non-sensitive fallback and SHALL be reported to the user.

#### Scenario: Authored active content is inert in downloaded HTML
- **WHEN** the Markdown contains script elements, event-handler attributes, embedded frames or plugins, forms, or unsafe navigation URLs
- **THEN** the downloaded HTML contains no executable form of that content
- **AND** opening the artifact does not execute authored script or submit authored data

#### Scenario: Session secrets and local references do not escape
- **WHEN** either download action completes for a session containing managed local resources
- **THEN** inspection of the artifact finds no claim capability, bearer token, loopback URL, blob URL, filesystem path, or unresolved protected-resource identifier

#### Scenario: Protected resource cannot be embedded
- **WHEN** an allowed image is missing, becomes unsafe, exceeds an export limit, or cannot be read before export
- **THEN** the artifact replaces it with a visible alt-text or resource-name fallback that reveals no absolute path
- **AND** the workspace reports the fallback count instead of claiming complete image fidelity

#### Scenario: Authored remote image remains explicit
- **WHEN** the article contains an HTTP or HTTPS image that is not a managed local resource
- **THEN** the export MAY retain that authored remote reference without proxying or silently claiming it is embedded
- **AND** the workspace feedback or documentation identifies that the artifact may require the remote host

### Requirement: Content exports remain offline-capable, localized, and isolated from Markion state
Every application script, style, font, and conversion runtime required by Copy Markdown, themed HTML download, and browser DOCX export SHALL be served from the verified checked-in publishing bundle. New third-party runtime files SHALL be pinned and represented in the bundle manifest and applicable license notices. Invoking, completing, cancelling, or failing any content-export action SHALL NOT mutate or save the Markion document, increment its version, change its selection or dirty state, alter undo history, or invalidate its shared derived caches.

#### Scenario: Exports work without external application dependencies
- **WHEN** the workspace is loaded with external networking unavailable and the article has no authored remote dependency
- **THEN** Copy Markdown and both download actions remain available from local bundled assets
- **AND** no runtime script, style, font, or converter is fetched remotely

#### Scenario: Export runtime is release-verifiable
- **WHEN** the source-tree or packaged workspace bundle is verified
- **THEN** every content-export runtime file is listed with matching identity and provenance
- **AND** missing, unlicensed, remotely referenced, or digest-mismatched files fail verification

#### Scenario: Export actions do not change the Markion document
- **WHEN** any content-export action succeeds, fails, or is cancelled
- **THEN** the active Markion document remains byte-identical to its state before the action
- **AND** its version, selection, dirty state, undo history, and existing derived-cache identities remain unchanged

#### Scenario: Export feedback uses the active workspace locale
- **WHEN** the workspace shows an export label, progress message, warning, success, compatibility notice, or failure
- **THEN** it uses the locale supplied for that publishing session with the documented fallback locale
