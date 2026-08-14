# image-file-viewing

## Purpose

Defines how Markion opens supported local image assets as safe, read-only content alongside editable Markdown and text documents.

## Requirements

### Requirement: Supported local image files SHALL open as read-only content
Markion SHALL recognize local files with the case-insensitive extensions `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`, `.tiff`, and `.svg` as supported image files. File → Open, Open in New Tab, Open Recent, and file-tree opening SHALL route those paths to a read-only image viewer instead of decoding them as UTF-8 text. A successfully routed image path SHALL participate in the normal tab title, current-file marking, recent-file, workspace-root, navigation, and close behavior.

#### Scenario: File Open replaces the active content with an image
- **WHEN** the user selects a supported image through File → Open and the active editable document passes its dirty guard
- **THEN** the active tab is replaced by a read-only image tab for the selected path
- **AND** the image bytes are not interpreted as document text

#### Scenario: New-tab flows open an image tab
- **WHEN** the user opens a supported image from the file tree, Open in New Tab, or Open Recent
- **THEN** Markion appends and activates a read-only image tab, or focuses the existing tab for that path

#### Scenario: Extension matching is case-insensitive
- **WHEN** an image path uses an uppercase or mixed-case supported extension such as `.PNG` or `.JpEg`
- **THEN** Markion recognizes and opens it through the same image-viewing path

#### Scenario: Unsupported file is not treated as an image
- **WHEN** an interactive open request selects a file whose extension is not in the supported image set and which is not a supported Markdown or curated text file
- **THEN** Markion does not create an image tab for that file
- **AND** it reports a localized open failure without disturbing existing tabs

### Requirement: Image tabs SHALL present decoded content without becoming editable documents
An image tab SHALL show a localized loading state while decoding, then display the decoded image centered on a dedicated image surface with its aspect ratio preserved. Images larger than the available content area SHALL be reduced to fit both dimensions, while smaller images SHALL NOT be enlarged above their intrinsic presentation size. Animated formats SHALL be presented as a static decoded frame; animation playback is not required. Image tabs SHALL NOT expose Markdown editing, formatting, save, autosave, recovery, outline, document statistics, view-mode, or export behavior.

#### Scenario: Image loads successfully
- **WHEN** a supported image is readable and decodes successfully
- **THEN** the image is displayed centered with its aspect ratio preserved
- **AND** its rendered size fits the available surface without upscaling above its intrinsic presentation size

#### Scenario: Animated image uses a static presentation
- **WHEN** a supported animated image such as a GIF is opened
- **THEN** Markion displays a decoded static frame without offering playback controls

#### Scenario: Document commands do not mutate an image
- **WHEN** an image tab is active and the user invokes an editing, formatting, save, autosave, recovery, outline, view-mode, or export action
- **THEN** the image file and all open document state remain unchanged
- **AND** Markion does not mark the image tab dirty or create undo or recovery state for it

#### Scenario: Image activation bypasses Markdown derivation
- **WHEN** the user switches from an editable document tab to an image tab and later switches back
- **THEN** Markion does not parse the image as Markdown or invalidate the document's derived caches
- **AND** the document's text, selection, history, scroll positions, dirty state, and cached derived state are preserved

### Requirement: Image loading failures SHALL remain contained and recoverable
If a supported image path cannot be read, decoded, or safely rasterized, its image tab SHALL show a localized unavailable-image state identifying the affected path and failure. The failure SHALL NOT mutate the file, close the tab, replace another tab, or disrupt the application. Closing the failed tab and opening another supported file SHALL continue to work normally.

#### Scenario: Corrupt image shows an error state
- **WHEN** a file has a supported image extension but its bytes cannot be decoded
- **THEN** the image tab shows a localized unavailable-image state with the path and failure detail
- **AND** the tab remains closable and the application remains responsive

#### Scenario: Image disappears before loading completes
- **WHEN** an image path is opened but becomes unreadable or disappears before decoding completes
- **THEN** the image tab shows the same contained unavailable-image state
- **AND** no existing document or image tab is changed
