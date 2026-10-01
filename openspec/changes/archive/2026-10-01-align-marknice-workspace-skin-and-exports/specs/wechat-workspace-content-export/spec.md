## ADDED Requirements

### Requirement: The current themed preview can be printed as PDF
The publishing workspace SHALL provide a user-initiated Save as PDF action that opens the browser print dialog on a hidden print document whose body is a clone of the current sanitized MarkNice preview, including the selected publishing theme, font-size offset, paragraph-spacing offset, and rendered math. The print stylesheet SHALL supply only page-box, margin, image-size, and break hints and SHALL NOT replace those inline theme styles with a generic unthemed article stylesheet. Workspace chrome, the phone bezel, and toolbars SHALL NOT appear in the print document. The action SHALL NOT call Markion's native or Pandoc PDF exporter, SHALL NOT send article HTML to a Node service, and SHALL report that the print dialog was opened rather than that a PDF file was saved. Completing, cancelling, or failing the action SHALL NOT mutate the Markion document.

#### Scenario: PDF print preserves the current MarkNice presentation
- **WHEN** the user changes theme or typography offsets and invokes Save as PDF
- **THEN** the print document contains the current sanitized preview HTML
- **AND** its inline heading, paragraph, and accent styles still reflect the selected theme and offsets
- **AND** those styles are not overwritten by a generic print article stylesheet

#### Scenario: Print dialog is the save mechanism
- **WHEN** the current preview has renderable content and the user invokes Save as PDF
- **THEN** the workspace opens the browser print dialog on the print document
- **AND** the status tells the user to choose Save as PDF in that dialog
- **AND** the workspace does not claim that a file was written to disk

#### Scenario: Empty content does not open print
- **WHEN** the current browser editor has no renderable content and the user invokes Save as PDF
- **THEN** no print dialog is opened
- **AND** the workspace shows a localized empty-content message

#### Scenario: Print output excludes workspace chrome
- **WHEN** Save as PDF runs while phone preview is active
- **THEN** the print document contains the article clone without the phone frame, notch, formatting toolbar, or session disclosure banner

#### Scenario: Browser PDF is distinct from Markion native PDF
- **WHEN** the workspace presents the Save as PDF control or its documentation
- **THEN** the action is identified as printing the MarkNice preview
- **AND** it is not presented as Markion's Export → PDF native or Pandoc writer

#### Scenario: Print does not change the Markion document
- **WHEN** Save as PDF succeeds, the user cancels the print dialog, or print setup fails
- **THEN** the active Markion document remains byte-identical to its prior state
- **AND** its version, selection, dirty state, undo history, and derived-cache identities remain unchanged

### Requirement: Export control labels track the pinned MarkNice editor actions
The workspace export controls SHALL use localized labels equivalent to the pinned MarkNice editor actions for copying Markdown, saving HTML, saving Word, and saving PDF, including accessible names. Existing safety, embedding, and offline-bundle contracts for HTML and DOCX SHALL remain in force.

#### Scenario: HTML and Word actions keep MarkNice wording
- **WHEN** the workspace is shown in Simplified Chinese
- **THEN** the themed HTML download is labeled as saving HTML rather than a generic “download HTML”
- **AND** the browser DOCX action is labeled as saving Word

#### Scenario: PDF joins the same export toolbar
- **WHEN** the preview toolbar is shown
- **THEN** Save as PDF appears with Copy Markdown, Save as HTML, Save as Word, and Copy to WeChat
- **AND** each control has a localized accessible name
