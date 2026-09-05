## MODIFIED Requirements

### Requirement: Supported local image files SHALL open as read-only content
Markion SHALL recognize local files with the case-insensitive extensions `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`, `.tiff`, and `.svg` as supported image files. File → Open, Open in New Tab, Open Recent, and file-tree opening SHALL route those paths to a read-only image viewer instead of decoding them as UTF-8 text. A successfully routed image path SHALL participate in the normal tab title, current-file marking, recent-file, workspace-root, per-workspace session snapshot, navigation, and close behavior. An in-root image tab SHALL be recorded in that workspace's snapshot and SHALL reopen when that workspace is restored on launch or by an explicit workspace switch, provided the path still exists.

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

#### Scenario: In-root image tabs restore with the workspace
- **WHEN** a workspace snapshot includes a supported image path that still exists and that workspace is restored on launch or by an explicit switch
- **THEN** the image reopens as a read-only image tab in the recorded order
- **AND** its bytes are not interpreted as UTF-8 document text
