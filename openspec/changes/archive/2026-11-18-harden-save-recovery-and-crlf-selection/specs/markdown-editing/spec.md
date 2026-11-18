## MODIFIED Requirements

### Requirement: Baseline text navigation and selection
The editor SHALL provide Unicode-aware previous/next-word movement and deletion, Shift+Home/End selection, PageUp/PageDown navigation, and progressive Select All from the current exact text block to the whole document. Word rules SHALL ignore whitespace-only segments, preserve valid UTF-8 boundaries, and avoid scanning unrelated document lines. These commands SHALL use the existing canonical selection and mutation paths, with one undoable mutation for word deletion. Source paragraph selection SHALL recognize blank separators with LF and CRLF while preserving the original source offsets.

#### Scenario: Word movement and deletion
- **WHEN** the user invokes previous/next-word movement or deletion in editable text
- **THEN** the caret or deleted range follows Unicode word boundaries while ignoring whitespace-only segments
- **AND** ASCII, CJK, punctuation, emoji, combining characters, and CRLF remain valid UTF-8 without corruption
- **AND** one Undo restores a word deletion and its selection

#### Scenario: Progressive Select All
- **WHEN** the user invokes Select All without already selecting the current block's exact editable content
- **THEN** that content is selected without including a structural prefix when a narrower payload range exists
- **WHEN** the user invokes Select All again with that exact content selected
- **THEN** the complete document is selected

#### Scenario: Source paragraph selection in a CRLF document
- **WHEN** the user invokes Select All from a collapsed caret inside one of several CRLF-separated paragraphs
- **THEN** the first invocation selects only the containing paragraph
- **AND** replacement edits only that paragraph and Undo restores the original bytes
