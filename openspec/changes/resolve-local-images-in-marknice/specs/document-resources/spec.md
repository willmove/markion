## ADDED Requirements

### Requirement: Out-of-scope local images can be organized into the asset directory
Markion SHALL provide a user-initiated organize action for a saved document that finds local image references which do not resolve within the document's publishing image scope (the document's own directory tree at any depth and up to one directory level above it), yet point to readable supported image files elsewhere on the local filesystem. The action SHALL present the candidates and require explicit confirmation before any change. On confirmation it SHALL copy each candidate file into the document-associated asset directory using the collision-safe, content-addressed resource naming, rewrite only the image destinations that referenced those files to the new document-relative URLs in one undoable edit, and leave the document dirty without saving it. Candidates that are missing, unsupported, or unreadable SHALL be skipped and reported rather than blocking the others. The action SHALL make no filesystem or document change for an untitled document beyond asking the user to save first, and a non-image link sharing a candidate destination SHALL NOT be rewritten.

#### Scenario: Confirmed organize copies and rewrites in one undo step
- **WHEN** the document references `../../shared/logo.png` and `C:\pictures\banner.png`, both readable supported images, and the user confirms the organize prompt
- **THEN** both files are copied into the document's asset directory with content-addressed names
- **AND** exactly the image destinations referencing them are rewritten to document-relative asset URLs in a single undoable edit
- **AND** the document becomes dirty and is not saved

#### Scenario: Cancel leaves document and filesystem unchanged
- **WHEN** the user cancels the organize prompt
- **THEN** no file is copied into the asset directory
- **AND** the document's text, version, dirty state, and undo history are unchanged

#### Scenario: Identical bytes are reused, not duplicated
- **WHEN** an organized candidate has byte-identical content to an image already stored in the asset directory
- **THEN** no duplicate file is created
- **AND** the reference is rewritten to the existing asset URL

#### Scenario: Unresolvable references are skipped and reported
- **WHEN** the document also references a missing file, an unsupported image type, or a `file:` URL
- **THEN** those references are not offered for organizing and remain untouched
- **AND** the completion status distinguishes them from successfully organized images

#### Scenario: Untitled document requires a durable base path
- **WHEN** the organize action is invoked on an untitled document
- **THEN** the user is asked to save the document first
- **AND** declining leaves the document and filesystem unchanged

#### Scenario: Non-image links sharing a destination are preserved
- **WHEN** a Markdown link `[site](../../shared/logo.png)` appears alongside the image `![icon](../../shared/logo.png)` and the user confirms organizing
- **THEN** only the image destination is rewritten
- **AND** the plain link keeps its authored destination
