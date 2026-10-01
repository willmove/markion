## ADDED Requirements

### Requirement: The publishing editor chrome tracks the pinned MarkNice editor skin
The local publishing workspace SHALL present its Markdown editor and WeChat preview using an editor skin derived from the pinned MarkNice editor section: dual rounded cards, traffic-light panel headers, the MarkNice CSS-variable palette including indigo accent `#6366f1`, Inter plus PingFang SC and Microsoft YaHei chrome fonts, an SF Mono / Fira Code / Menlo / Consolas editor stack, SVG formatting-toolbar icons in pill buttons, grouped font-size and paragraph-spacing steppers, SVG desktop/phone preview toggles, and a phone preview framed at 375px CSS pixels with a notch. The skin SHALL remain as close as the local workspace allows to that editor section. The workspace SHALL keep its session-local and privacy disclosures and SHALL NOT restore MarkNice marketing chrome (site navbar, hero, features, guide, or footer). Dark appearance SHALL use the same `data-mode='dark'` token set as the pinned MarkNice editor.

#### Scenario: Editor cards and typography match the MarkNice editor section
- **WHEN** the authenticated workspace shell is shown
- **THEN** the Markdown input and WeChat preview occupy separate rounded cards with traffic-light headers
- **AND** chrome fonts include Inter, PingFang SC, and Microsoft YaHei rather than a Segoe-only stack
- **AND** the Markdown textarea uses a monospace stack that includes SF Mono or Fira Code before Consolas

#### Scenario: Formatting and preview controls use MarkNice icon chrome
- **WHEN** the formatting toolbar and preview toolbar are shown
- **THEN** formatting actions other than heading labels use SVG stroke icons in pill buttons rather than Unicode glyphs
- **AND** font-size and paragraph-spacing offsets are grouped stepper controls
- **AND** desktop and phone modes are SVG toggles that share a `.mode-btn` active-state contract so only one mode appears selected

#### Scenario: Phone preview uses a device frame
- **WHEN** the user selects phone preview
- **THEN** the preview article is shown inside a 375px-wide framed phone chrome with a notch
- **AND** the frame is not merely a max-width constraint on an otherwise undressed preview card

#### Scenario: Marketing chrome stays omitted
- **WHEN** the workspace loads
- **THEN** it does not render the MarkNice site navbar, hero, features grid, guide link farm, or footer
- **AND** the session-local editing disclosure remains visible

### Requirement: Word import replaces only the browser-session Markdown
The publishing workspace SHALL provide a user-initiated Import Word action that reads a `.docx` file entirely in the browser with the pinned JSZip and MarkNice Word-import runtime, converts it to Markdown, and replaces the current workspace textarea and preview. The action SHALL NOT send the file or resulting Markdown to Markion, SHALL NOT mutate or save the Markion document, and SHALL NOT increment its version or invalidate derived caches. After a successful import, the workspace SHALL tell the user that the result is session-local and that bringing it into Markion requires Copy Markdown or another explicit save of the session Markdown. Application scripts required for import SHALL load from the verified local bundle with no CDN.

#### Scenario: A Word document becomes session Markdown
- **WHEN** the user selects a supported `.docx` file in the workspace Import Word control
- **THEN** the workspace textarea is replaced with the converted Markdown
- **AND** the preview rerenders from that session Markdown
- **AND** the Markion document text, dirty state, version, selection, undo history, and derived-cache identities remain unchanged

#### Scenario: Successful import discloses the recovery path
- **WHEN** Word import completes
- **THEN** the workspace shows a localized success status
- **AND** that status states that the import is session-local and that the user must copy or otherwise save the session Markdown to keep it in Markion

#### Scenario: Invalid or oversized Word input fails closed
- **WHEN** the selected file is not a readable `.docx`, exceeds the documented import size bound, or the parser throws
- **THEN** the workspace does not replace the current Markdown with a partial conversion
- **AND** it shows a localized actionable error
- **AND** the Markion document remains unchanged

#### Scenario: Word import works offline
- **WHEN** the workspace is used with external networking unavailable
- **THEN** Import Word still runs from bundled JSZip and the Word-import runtime
- **AND** no parser, zipper, or conversion script is fetched from a remote host

#### Scenario: Word-embedded images stay in the session as data URIs
- **WHEN** the imported document contains a supported embedded image
- **THEN** the session Markdown may reference that image as a data URI
- **AND** the preview does not expose a filesystem path for that image
- **AND** the image is not treated as a managed loopback resource that must be omitted solely because it originated from Word
