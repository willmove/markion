## ADDED Requirements

### Requirement: Baseline text navigation and selection
The editor SHALL provide Unicode-aware previous/next-word movement and deletion, Shift+Home/End selection, PageUp/PageDown navigation, and progressive Select All from the current exact text block to the whole document. Word rules SHALL ignore whitespace-only segments, preserve valid UTF-8 boundaries, and avoid scanning unrelated document lines. These commands SHALL use the existing canonical selection and mutation paths, with one undoable mutation for word deletion.

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

### Requirement: Mouse block selection and ordinary editor context menu
Source SHALL support double-click word selection and triple-click current-block selection, and Visual Edit SHALL support triple-click current-block selection. The editor SHALL provide an ordinary context menu offering Cut, Copy, Paste, and Select All, using existing command paths and appropriate availability for the current selection. Opening or dismissing the menu SHALL NOT mutate document text, version, history, selection, or derived caches.

#### Scenario: Double and triple click
- **WHEN** the user double-clicks Source text or triple-clicks a Source or Visual text block
- **THEN** the word or exact editable block content is selected through the canonical selection path

#### Scenario: Context menu editing
- **WHEN** the user opens the editor context menu and invokes an enabled editing action
- **THEN** the existing Cut, Copy, Paste, or Select All command runs with its established source and undo behavior
- **AND** unrelated menu lifecycle changes leave the document unchanged

### Requirement: Structural collapsed-list indentation
In Visual Edit, a collapsed selection in a list item SHALL use Tab/Shift+Tab for exact source-backed indentation/outdentation when the transition is structurally legal. Unsupported transitions SHALL leave source unchanged. Collapsed text outside a list SHALL retain literal four-space insertion where already supported.

#### Scenario: Legal list indentation is one mutation
- **WHEN** a collapsed-list item can legally indent or outdent and the user invokes the corresponding action
- **THEN** the existing line-indentation helper applies one exact source mutation and selection update
- **AND** one Undo restores the prior list source

#### Scenario: Illegal transition is inert
- **WHEN** a list item has no legal target for the requested transition
- **THEN** no source or selection mutation occurs

### Requirement: Context-sensitive inline input conversion
Auto-pairing SHALL repair the exact auto-paired `**` + Space sequence at a legal unordered-list start into `* ` in the same canonical edit as the typed Space. Immediately after an HTTP(S) or `www.` token, typed whitespace SHALL serialize one Markdown link with a normalized destination. Conversion SHALL NOT run during IME composition or inside code, math, diagram, front-matter, HTML, or other restricted source fields.

#### Scenario: Bullet marker after auto-pair
- **WHEN** auto-pairing produced `**` at a legal block start and the user types Space between the markers
- **THEN** the source becomes `* ` with the caret after the Space
- **AND** the complete change is one undoable canonical edit

#### Scenario: URL becomes a link
- **WHEN** the user types whitespace after a recognized URL token in unrestricted prose
- **THEN** the token becomes `[token](destination)`, with `https://` prepended to a `www.` destination
- **AND** the caret moves after the link and the change uses one undoable source mutation

#### Scenario: Restricted context stays literal
- **WHEN** the same input occurs during IME composition or in a restricted source field
- **THEN** no pairing repair or URL conversion occurs

### Requirement: Slash trigger recognizes ASCII and Chinese markers
Slash-command recognition SHALL accept `/` or `、` after line start or whitespace, require a whitespace-free query, and reject URL-like or otherwise non-command prefixes. Confirmation SHALL replace the complete marker and query through the existing exact block edit path.

#### Scenario: Chinese marker opens an equivalent query
- **WHEN** the user types `、hea` after line start or whitespace in Visual Edit
- **THEN** the same filtered heading commands and stale-target validation apply as for `/hea`

#### Scenario: URL is not a slash query
- **WHEN** the caret follows a URL-like prefix containing `://`
- **THEN** no slash query is created
