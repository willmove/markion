## ADDED Requirements

### Requirement: DOCX imports SHALL persist images as portable managed resources
After a native DOCX import has a selected new Markdown destination, its supported embedded images SHALL be persisted under the destination's derived `<stem>.assets` directory using safe collision-resistant filenames and forward-slash document-relative URLs. The import SHALL preserve separate Markdown occurrences and alt text while deduplicating identical image bytes. Resources SHALL be staged before publication and SHALL NOT require data URIs, browser blob URLs, temporary paths, absolute filesystem paths, or externally hosted images in the saved Markdown. Ordinary clipboard/drop resource behavior SHALL remain unchanged.

#### Scenario: Image-rich document is saved and reopened
- **WHEN** a DOCX with repeated embedded images is imported into a destination whose parent path contains spaces and Chinese characters
- **THEN** its Markdown references safe relative managed-resource URLs and deduplicates identical bytes
- **AND** closing and reopening the output preserves image availability and alt text

#### Scenario: Text-only document is imported
- **WHEN** the prepared import contains no supported image resources
- **THEN** saving produces the Markdown file without creating an empty asset directory

### Requirement: Import publication SHALL never overwrite existing destinations
Native DOCX import SHALL require an absent target `.md` and absent derived asset directory. The commit operation SHALL use create-only/no-clobber publication and containment/identity validation so a file, directory, symbolic link, or junction introduced after the picker check cannot be overwritten or redirect writes outside the validated destination. Assets SHALL be completely published before the complete Markdown file; Markdown publication SHALL be the commit point. The system SHALL NOT claim a single atomic replacement of both sibling paths. A collision SHALL request a different destination and preserve the conflicting content.

#### Scenario: A destination already exists
- **WHEN** the selected Markdown file or derived asset directory already exists
- **THEN** the importer refuses that destination and offers selection of a different name
- **AND** it does not merge into or overwrite either existing path

#### Scenario: A collision appears during staging
- **WHEN** another actor creates or substitutes a destination after initial validation
- **THEN** commit detects the conflict or fails through no-clobber publication
- **AND** the unrelated file/directory/link and its contents remain unchanged

#### Scenario: Markdown becomes visible after its images
- **WHEN** an image-containing import commits successfully
- **THEN** its complete image resources are available before the complete `.md` becomes visible
- **AND** the committed document never references uncommitted temporary resource paths

### Requirement: Failed imports SHALL clean up only import-owned artifacts
Before the Markdown commit point, handled errors and cancellation SHALL remove transaction-owned staging and published resources whose identities still match the transaction, leaving existing documents, source Word files, and unrelated resources unchanged. Cleanup failures SHALL report retained paths without falsely claiming a clean cancellation. After commit, files SHALL remain user-owned even if opening their tab fails. Unexpected process/OS termination before commit SHALL NOT expose a partial Markdown file; possible staging or asset remnants SHALL be documented rather than treated as evidence of a completed import.

#### Scenario: Markdown publication fails after image preparation
- **WHEN** image staging/publication succeeds but writing or publishing the Markdown fails before commit
- **THEN** the importer cleans up its own staging and image outputs while preserving unrelated files
- **AND** it reports failure without creating a document tab

#### Scenario: Cleanup cannot remove an owned temporary file
- **WHEN** rollback encounters a filesystem error or a changed file identity
- **THEN** the application reports the retained path and does not delete a substituted unrelated file
- **AND** it does not report the import as successful or fully cleaned up

#### Scenario: Process stops before Markdown publication
- **WHEN** the process terminates after publishing assets but before the Markdown commit point
- **THEN** there is no partial imported Markdown document or overwritten existing document
- **AND** any remaining staging/assets are not represented as a successful imported tab
