## ADDED Requirements

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
