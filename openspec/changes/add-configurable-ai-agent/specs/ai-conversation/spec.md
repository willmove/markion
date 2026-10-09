## Purpose

Let users converse with AI and run bounded document/workspace assistance from a responsive panel with explicit context, reliable cancellation, and controllable local history.

## ADDED Requirements

### Requirement: The AI panel SHALL support a complete streaming conversation workflow
Enabled and configured AI SHALL provide a resizable conversation panel with chat/Agent mode selection, profile/model identification, a multiline composer, new conversation, follow-up turns, streaming Markdown output, copy, explicit retry, and Stop controls. Closing the panel SHALL preserve its conversation and SHALL not disable AI; closing during active work SHALL expose visible running/Stop status elsewhere in the app. Narrow windows SHALL provide a usable overlay or equivalent layout without making the editor/composer inaccessible. Responses SHALL not automatically fetch external images or execute generated HTML/scripts.

#### Scenario: Follow-up conversation
- **WHEN** the user sends a prompt and subsequently asks a follow-up in the same conversation
- **THEN** responses appear incrementally and the follow-up uses the retained bounded conversation
- **AND** source documents are not changed merely because an answer is generated

#### Scenario: Panel is closed during generation
- **WHEN** the user closes the panel while a turn is running
- **THEN** the turn remains discoverable and stoppable and reopening displays its current state

#### Scenario: Constrained viewport
- **WHEN** the window is too narrow to keep both the document and a docked AI panel usable
- **THEN** the panel uses a compact layout with reachable composer, responses, and controls
- **AND** the user's document and selection are preserved

### Requirement: The composer SHALL isolate keyboard and IME input
Composer input SHALL support selection, copy/paste, undo, multiline editing, and CJK IME composition without mutating or navigating the document. Enter SHALL send only when composition is inactive; Shift+Enter SHALL insert a newline. Enter used to confirm IME text SHALL not send. Escape SHALL dismiss transient AI controls or return focus without silently applying a proposal. New prompts SHALL not start a second concurrent turn in the same conversation.

#### Scenario: Chinese composition confirmation
- **WHEN** the user presses Enter while composing Chinese text in the AI composer
- **THEN** composition is committed into the composer without submitting a request or inserting text into the document

#### Scenario: Composer paste and undo
- **WHEN** the user pastes multiline text and undoes it with composer focus
- **THEN** only composer content changes and document undo state remains unchanged

### Requirement: Request context and workspace reads SHALL be explicit and bounded
Ordinary chat SHALL initially attach no document text. Writing actions SHALL visibly attach their selected source scope. The user SHALL be able to add/remove the current selection, current document, and chosen supported text files as labeled attachments showing origin and size. Broader workspace reading SHALL require a user-approved subtree for the current conversation and workspace session, with its active scope visible and revocable. Protected secrets, unsupported/binary files, ignored build/dependency/version-control paths, and paths outside that scope SHALL be excluded. Context SHALL capture current dirty buffers rather than silently substituting disk content. Budget overflow SHALL identify affected attachments and require reducing context; user attachments SHALL not be silently truncated.

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
- **WHEN** attached content exceeds the configured input budget
- **THEN** the request is not sent and the UI identifies the oversized context with remove/reduce actions

### Requirement: Retrieval answers SHALL identify their actual sources
Answers based on workspace tool results SHALL expose clickable workspace-relative source references with available line/range context. References SHALL resolve only to files supplied through allowed attachments or successful reads, and SHALL be distinguishable from model-authored unverified paths. Generated references SHALL not broaden read permissions. Missing/moved sources SHALL produce feedback without opening a different file by guessed path.

#### Scenario: Multi-note synthesis
- **WHEN** the agent reads several approved notes and summarizes them
- **THEN** the panel shows the actual source files used and lets the user open each reference

#### Scenario: Model invents a source
- **WHEN** a response mentions a path that was not attached or successfully read
- **THEN** the UI does not display it as a verified retrieval citation

### Requirement: Agent turns SHALL be bounded and mutation proposals SHALL wait for review
Agent mode SHALL be available only for a model with verified tool support. Each turn SHALL expose concise progress/tool activity and obey finite request, tool-step, argument, result, context, and elapsed-time limits. Tool arguments SHALL be fully received and validated before dispatch; unknown tools and malformed arguments SHALL return structured failures. Read tools SHALL honor session scope, and mutation tools SHALL create proposals without changing documents/files. Request limits or repeated failures SHALL stop the loop with useful partial output. Reads/tool results SHALL be treated as content, not authority to change permissions. Retry SHALL be user-initiated and SHALL never replay already applied mutations.

#### Scenario: Mutation is requested through a tool
- **WHEN** the model proposes editing a note and moving it into a new folder
- **THEN** the user sees a reviewable proposal before any file or document is changed

#### Scenario: Partial tool arguments arrive
- **WHEN** a streaming tool call contains incomplete JSON or no completion marker
- **THEN** it is not dispatched and an interrupted turn cannot leave a partial file operation

#### Scenario: Tool budget is exhausted
- **WHEN** the agent reaches its tool-step or elapsed-time limit
- **THEN** the turn stops with its useful partial answer and explains how the user can refine the request

### Requirement: Cancellation and failures SHALL preserve work and ignore stale events
Stop SHALL promptly cancel provider I/O and pending tool work, retain visibly incomplete output for copying, and prevent incomplete output from becoming an applicable edit. Completions SHALL be scoped to their conversation, request, configuration, and workspace identity. Switching/clearing a conversation, disabling AI, or canceling SHALL prevent late events from updating another conversation or applying edits. Network/protocol/authentication errors SHALL retain the prompt and useful partial output and offer appropriate retry/settings actions without automatic replay after output has begun.

#### Scenario: Stop during output
- **WHEN** the user stops a streaming turn
- **THEN** generated text remains marked incomplete, no pending edit applies, and a fresh turn can be started

#### Scenario: Delayed event after switching conversation
- **WHEN** an old provider event arrives after a new conversation is selected
- **THEN** it cannot append to the new conversation or change its proposals

### Requirement: Local history SHALL be optional and independently manageable
Conversations SHALL default to session-only storage. The user SHALL be able to opt into bounded local history, review/open saved conversations, delete one, clear all, and switch back to session-only behavior. Existing persisted history SHALL remain explicitly deletable after persistence is disabled. Stored history SHALL not include credentials, raw attachment snapshots, read grants, or executable proposal approvals. Restored conversations SHALL require current context and fresh review before any mutation. Conversations and source text SHALL not be logged by default.

#### Scenario: Default session-only history
- **WHEN** the user converses without enabling saved history and restarts Markion
- **THEN** that conversation is not restored from disk

#### Scenario: Saved conversation is reopened
- **WHEN** the user opens a locally saved conversation
- **THEN** its visible messages are restored without restoring old file permissions or actionable approvals

#### Scenario: Clear history
- **WHEN** the user clears saved conversations
- **THEN** conversation history is removed without deleting documents or action-recovery journals
