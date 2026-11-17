## MODIFIED Requirements

### Requirement: Request context and workspace reads SHALL be explicit and bounded
Ordinary chat SHALL initially attach no document text. Writing actions SHALL visibly attach their selected source scope. The user SHALL be able to add or remove the current selection, current document, and chosen supported text files as labeled attachments showing origin and size. Broader workspace reading SHALL require a user-approved subtree for the current conversation and workspace session, with its active scope visible and revocable. Protected secrets, unsupported or binary files, ignored build, dependency, and version-control paths, and paths outside that scope SHALL be excluded. Context SHALL capture current dirty buffers rather than silently substituting disk content. The current user turn and its attachments SHALL NOT be silently truncated. When that current turn fits the configured context budget but older complete turns do not, the request SHALL omit the oldest complete turns until it fits and SHALL show how many earlier turns were omitted. Tool-call transcripts that cannot be split without separating a call from its result SHALL NOT be partially rewritten; if such a transcript still exceeds the budget, the request SHALL NOT be sent. Budget overflow of the current turn or an attachment SHALL identify the affected material and require reducing context.

#### Scenario: Selection-only request
- **WHEN** the user attaches a selected paragraph and sends a request
- **THEN** only that selected source and explicit conversation context are sent
- **AND** other document paragraphs and unrelated files are not automatically attached

#### Scenario: Workspace read scope
- **WHEN** the user approves reading the `notes/research` subtree for a conversation
- **THEN** the agent can read allowed text files in that subtree without prompting for each file
- **AND** reading another subtree requires a new explicit scope grant

#### Scenario: Scope is revoked or workspace changes
- **WHEN** the user revokes read permission or changes the workspace
- **THEN** active retrieval stops and old grants and pending proposals cannot authorize work in the new scope

#### Scenario: Context exceeds budget
- **WHEN** attached content or the current turn exceeds the configured input budget
- **THEN** the request is not sent and the UI identifies the oversized context with remove or reduce actions
- **AND** the attachment text is not shortened

#### Scenario: Older turns are omitted to fit
- **WHEN** the current turn fits the context budget and earlier complete turns push the prepared request over that budget
- **THEN** the oldest complete turns are omitted until the request fits
- **AND** the panel states that earlier turns were omitted
- **AND** the current turn is sent intact

### Requirement: The panel SHALL distinguish pending context and prior sources
Pending attachments SHALL identify their document or file and source range and be individually removable. Previously used sources SHALL be distinguishable from attachments for the next request. History status SHALL reflect whether local persistence is enabled. Conversation deletion SHALL require confirmation. Workspace mode SHALL explain missing tool verification or folder authorization before sending.

#### Scenario: Follow up after sending an attachment
- **WHEN** an attachment is consumed by a request and the user starts a follow-up
- **THEN** the panel distinguishes earlier sources from new attachments instead of implying that the conversation has never used document content

#### Scenario: Repair a local input-budget failure
- **WHEN** an attachment or the current turn exceeds the saved profile's input-byte budget
- **THEN** feedback identifies the local input limit, names the rejected attachment when known, exposes the configured budget, and offers the existing advanced limits settings
- **AND** the user may increase that finite budget or explicitly reduce the selected scope without silent truncation or automatic transmission

## ADDED Requirements

### Requirement: The composer SHALL accept input up to the context budget
The composer SHALL accept text up to the active profile's context budget and SHALL refuse additional text beyond that budget. A refused paste or insertion SHALL leave the previous composer text in place and SHALL show the existing context-budget feedback. The refusal SHALL NOT change document text, selection, dirty state, or undo history.

#### Scenario: Paste within the budget
- **WHEN** the user pastes composer text that fits the active profile's context budget
- **THEN** the composer contains the pasted text

#### Scenario: Paste beyond the budget
- **WHEN** the user pastes composer text larger than the active profile's context budget
- **THEN** the previous composer text remains and the panel explains the budget limit
- **AND** the document is unchanged
