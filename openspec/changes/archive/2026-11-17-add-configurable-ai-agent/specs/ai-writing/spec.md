## Purpose

Provide practical AI writing transformations within the editor, with explicit source scope, reviewable results, exact application targets, and existing undo and version safety.

## ADDED Requirements

### Requirement: Writing assistance SHALL offer useful presets and custom instructions
AI writing SHALL offer drafting, continuing, polishing, spelling/grammar correction, shortening, expansion, tone adjustment, translation, summary, outline extraction, task extraction, and custom instructions. The user SHALL be able to specify target language/tone and persistent writing guidance. Selection-based transformations SHALL capture the current selection; an empty selection SHALL offer an explicit current-document scope or drafting input rather than silently sending the entire document. Continue SHALL expose its preceding-context scope. Generated output SHALL preserve Markdown structures when the request is a Markdown transformation and SHALL not translate user content based only on interface language.

#### Scenario: Polish a selected paragraph
- **WHEN** the user selects a paragraph and chooses Polish
- **THEN** the request clearly identifies the selected text as its scope and returns a proposed replacement
- **AND** the remainder of the document is not implicitly sent or modified

#### Scenario: Translation target differs from UI language
- **WHEN** the interface is Chinese and the user explicitly requests translation into English
- **THEN** the result uses English while the UI remains Chinese

#### Scenario: Empty selection
- **WHEN** the user invokes a selection transformation without selecting text
- **THEN** the UI offers a clearly labeled scope choice or drafting input before sending content

### Requirement: Writing entry points SHALL work across supported editor modes
Writing assistance SHALL be reachable from the AI panel and contextual source/Visual Edit/Read selection actions when AI is enabled. Source-backed selections SHALL use exact canonical source boundaries, including CJK/emoji and CRLF content. A rendered selection that cannot be mapped to an exact source range SHALL remain usable as analysis context but SHALL not offer unsafe replacement. Read mode and image tabs SHALL allow copy or new-note results but SHALL not apply an inline document mutation.

#### Scenario: Visual Edit selection
- **WHEN** the user asks to improve a source-mapped Visual Edit selection containing formatted text and emoji
- **THEN** the proposal targets the exact canonical source range and preserves surrounding source

#### Scenario: Read-only surface
- **WHEN** writing assistance is invoked from Read mode or an image tab
- **THEN** replacement and caret insertion are unavailable while copy/new-note actions remain usable

### Requirement: Results SHALL be reviewed and support explicit destinations
Completed writing results SHALL show the generated Markdown, the source scope, and a before/after change preview where a replacement is possible. The user SHALL be able to copy, replace the captured selection/scope, insert at the captured caret, create a new unsaved note, discard, or regenerate with revised instructions as applicable. Generation and dismissal SHALL not change document dirty state or undo history. Incomplete/stopped/failed output SHALL be copyable but not applicable. No apply destination SHALL be inferred from the currently active tab when the result was generated for another document.

#### Scenario: Result is discarded
- **WHEN** a completed rewrite is dismissed without application
- **THEN** the document text, dirty state, selection, and undo history are unchanged

#### Scenario: New note destination
- **WHEN** the user chooses Create New Note for a generated outline
- **THEN** the result opens in a new dirty untitled note without overwriting an existing file

### Requirement: Accepted writing edits SHALL use checked atomic undo boundaries
An accepted replacement/insertion SHALL validate the captured document identity, version, source bytes, and UTF-8-safe target before applying one canonical source mutation. Application SHALL terminate any prior typing group, create one undo entry, update selection deliberately, and use existing dirty/recovery/autosave behavior. The user SHALL be able to undo and redo the complete AI edit with exact source/selection restoration. Generation and token deltas SHALL not increment document versions or invalidate document caches; only accepted text changes SHALL refresh the affected version-derived state.

#### Scenario: Undo an accepted rewrite
- **WHEN** the user accepts a rewrite after ordinary typing and invokes Undo
- **THEN** one Undo restores the source and selection preceding the AI change
- **AND** the earlier typing remains a separate undo group

#### Scenario: Stream without application
- **WHEN** many AI deltas arrive while the user is viewing a document
- **THEN** its text version, undo state, derived Markdown cache, highlighting cache, and cached text handle remain unchanged

### Requirement: Stale writing targets SHALL never be overwritten
If the captured source changes, the document closes/reopens, its path/workspace changes, or a conflicting Git operation occurs, applying the result SHALL fail without modifying content. Moving only the live caret SHALL not retarget a captured insertion. The UI SHALL offer regenerate with current context, copy, or create a new note; it SHALL not automatically rebase or apply to a different document. Whole-document transformations SHALL obey the same validation as selected-range edits.

#### Scenario: User edits during generation
- **WHEN** the document version changes before a completed rewrite is accepted
- **THEN** the old replacement is rejected and the current source is preserved
- **AND** the user can regenerate, copy, or create a new note from the result

#### Scenario: Caret moves without an edit
- **WHEN** a continuation was generated for one caret and the user moves the caret before choosing Insert
- **THEN** insertion uses the visibly identified captured position or requires an explicit newly reviewed destination
- **AND** it does not silently insert at the new caret
