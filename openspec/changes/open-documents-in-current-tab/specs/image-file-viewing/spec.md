## MODIFIED Requirements

### Requirement: Supported local image files SHALL open as read-only content
Markion SHALL recognize local files with the case-insensitive extensions `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`, `.tiff`, and `.svg` as supported image files. File → Open, Open in New Tab, Open Recent, and file-tree opening SHALL route those paths to a read-only image viewer instead of decoding them as UTF-8 text, regardless of which tab target the open request resolves to. A successfully routed image path SHALL participate in the normal tab title, current-file marking, recent-file, workspace-root, navigation, and close behavior.

#### Scenario: File Open routes an image through the open-target preference
- **WHEN** the user selects a supported image through File → Open and the active editable document passes its dirty guard
- **THEN** with the open-in-current-tab preference on, the active tab is replaced by a read-only image tab for the selected path; with the preference off, a new image tab is appended and activated
- **AND** the image bytes are not interpreted as document text

#### Scenario: Tree, Recent, and explicit new-tab flows open image tabs
- **WHEN** the user opens a supported image from the file tree (plain click), Open Recent, or the file-tree context-menu Open action
- **THEN** Markion opens a read-only image tab in the tab chosen by the default open-target rule — replacing the current tab when that is allowed under the preference, otherwise appending a new tab — or focuses the existing tab for that path
- **AND** Open in New Tab and Ctrl/Cmd+click in the file tree always append a read-only image tab

#### Scenario: Extension matching is case-insensitive
- **WHEN** an image path uses an uppercase or mixed-case supported extension such as `.PNG` or `.JpEg`
- **THEN** Markion recognizes and opens it through the same image-viewing path

#### Scenario: Unsupported file is not treated as an image
- **WHEN** an interactive open request selects a file whose extension is not in the supported image set and which is not a supported Markdown or curated text file
- **THEN** Markion does not create an image tab for that file
- **AND** it reports a localized open failure without disturbing existing tabs
