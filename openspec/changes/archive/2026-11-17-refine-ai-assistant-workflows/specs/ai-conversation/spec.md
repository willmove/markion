## ADDED Requirements

### Requirement: The conversation layout SHALL prioritize messages and retain the composer
The panel SHALL provide a fixed header, a message region that uses available height and a bounded composer that remains reachable while reading messages or reviews. Writing actions SHALL be disclosed contextually rather than occupying the full ordinary chat surface. File permissions and operations SHALL be associated with an explicit workspace mode. Icon actions SHALL have localized tooltips and keyboard activation. Sending SHALL be visibly unavailable for empty input or running work; running work SHALL have a reachable Stop action.

#### Scenario: Read a long response
- **WHEN** a user scrolls a long response
- **THEN** message scrolling does not move the composer out of view and resizing the window changes the space available for messages

### Requirement: Multiline AI input SHALL wrap while preserving input semantics
Multiline input SHALL wrap to available width and maintain correct source positions for selection, vertical movement and IME candidate placement across CJK, emoji and explicit newlines. The caret SHALL remain visible while editing. Wrapping SHALL NOT change source text, undo history or document state.

#### Scenario: Type a long Chinese prompt
- **WHEN** a prompt exceeds the available input width without an explicit newline
- **THEN** it wraps visibly, preserves its exact text and supports selection and IME composition on the wrapped rows

#### Scenario: Type without an IME
- **WHEN** the focused composer or AI settings field receives ordinary printable keyboard input
- **THEN** letters, shifted characters, punctuation and spaces reach that input without changing the document or triggering document formatting actions

### Requirement: The panel SHALL distinguish pending context and prior sources
Pending attachments SHALL identify their document/file and source range and be individually removable. Previously used sources SHALL be distinguishable from attachments for the next request. History status SHALL reflect whether local persistence is enabled. Conversation deletion SHALL require confirmation. Workspace mode SHALL explain missing tool verification or folder authorization before sending.

#### Scenario: Follow up after sending an attachment
- **WHEN** an attachment is consumed by a request and the user starts a follow-up
- **THEN** the panel distinguishes earlier sources from new attachments instead of implying that the conversation has never used document content

#### Scenario: Repair a local input-budget failure
- **WHEN** an attachment or prepared request exceeds the saved profile's input-byte budget
- **THEN** feedback identifies the local input limit, names the rejected attachment when known, exposes the configured byte budget and offers the existing advanced limits settings
- **AND** the user may increase that finite budget or explicitly reduce the selected scope without silent truncation or automatic transmission
