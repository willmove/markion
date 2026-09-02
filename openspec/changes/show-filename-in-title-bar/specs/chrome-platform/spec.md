## ADDED Requirements

### Requirement: Native window title shows the active file name
The editor SHALL present the active tab's file name in the native window title immediately after the Markion brand text that follows the window logo. The caption SHALL be the string `Markion - {tab_title}`, where `{tab_title}` is the same title the document tab strip uses (the path's file name, or `Untitled.md` when the tab has no filesystem path). When the active tab is a document with unsaved changes, the caption SHALL append a space and `*` after the file name. Image tabs SHALL use the image file name and SHALL NOT append a dirty suffix. The editor SHALL update the native window title when the active tab, its title, or its dirty state changes. Applying an unchanged caption SHALL NOT force a redundant platform title update. Computing or applying the window title SHALL NOT reparse Markdown, invalidate per-document-version derived caches, or recompute syntax highlighting.

#### Scenario: Saved document name appears after the brand
- **WHEN** the active tab is a saved Markdown document named `notes.md` with no unsaved changes
- **THEN** the native window title is `Markion - notes.md`

#### Scenario: Untitled document uses the tab title
- **WHEN** the active tab has no filesystem path
- **THEN** the native window title is `Markion - Untitled.md` when the document is clean
- **AND** `Markion - Untitled.md *` when the document is dirty

#### Scenario: Unsaved changes mark the title
- **WHEN** the active document tab has unsaved changes
- **THEN** the native window title ends with the file name, a space, and `*`
- **AND** a successful save that clears dirty state removes that suffix

#### Scenario: Image tabs show the file name without a dirty suffix
- **WHEN** the active tab is a read-only image file named `photo.png`
- **THEN** the native window title is `Markion - photo.png`
- **AND** the caption does not include a `*` suffix

#### Scenario: Switching tabs updates the native title
- **WHEN** the user activates a different tab
- **THEN** the native window title immediately uses that tab's title and dirty state

#### Scenario: Title updates do not disturb derived caches
- **WHEN** the window title is applied because the active tab, file name, or dirty flag changed
- **THEN** preview blocks, outline, stats, line count, syntax highlighting, and cached text handles are not recomputed as a result of the title update

## MODIFIED Requirements

### Requirement: Persistent document context in the status bar
The status bar SHALL retain save-state and transient operation feedback while also presenting a compact persistent context for the active tab. The status-bar feedback region SHALL NOT repeat the Markion brand, the active tab's file name, or the unsaved-changes `*` suffix; those belong to the native window title. The persistent context SHALL include the active document's character count and word count, the one-based line and column of the active caret whenever an editing surface is present, and the current named Git branch when a repository can be resolved from the active document or workspace. Character count SHALL count Unicode scalar values, including whitespace and line breaks, and word count SHALL count contiguous non-whitespace sequences. Every new user-visible label SHALL use the active interface language. Document metrics SHALL reuse per-document-version derived state, and Git discovery or refresh SHALL NOT perform filesystem or process work on the render or typing path.

#### Scenario: Active document metrics are always visible
- **WHEN** an active tab displays a document in any view mode
- **THEN** the persistent status context shows that document's character count and word count
- **AND** editing the document updates both values from the new document version

#### Scenario: Counts have defined Unicode and whitespace semantics
- **WHEN** a document contains non-ASCII text, emoji, whitespace, and line breaks
- **THEN** the character count equals the number of Unicode scalar values in the complete source
- **AND** the word count equals the number of contiguous non-whitespace source sequences

#### Scenario: Editing modes show the active caret position
- **WHEN** the active view mode is Edit, Visual Edit, or Split Preview
- **THEN** the persistent status context shows the active caret's one-based logical line and Unicode-scalar column
- **AND** a non-empty selection reports the position of its active caret end rather than always reporting the selection's lower offset

#### Scenario: Read mode omits caret position
- **WHEN** the active view mode is Read and no editing caret is presented
- **THEN** the persistent status context omits the line-and-column item
- **AND** the character count, word count, and any available Git branch remain visible

#### Scenario: Named Git branch is shown for repository-backed context
- **WHEN** the active saved document belongs to a Git working tree with a named branch
- **THEN** the persistent status context shows that branch name
- **AND** the active document's nearest repository takes precedence over a broader workspace repository

#### Scenario: Unsaved document uses an established workspace
- **WHEN** the active document has no filesystem path but the user has established a workspace inside a Git working tree with a named branch
- **THEN** the persistent status context shows the workspace repository's branch name

#### Scenario: Unavailable branch is omitted without replacing feedback
- **WHEN** neither the active document nor established workspace belongs to a Git repository, Git HEAD is detached, or repository metadata cannot be read
- **THEN** the Git branch item is omitted
- **AND** the status bar continues to show document metrics and existing transient operation feedback without surfacing the lookup failure as an operation error

#### Scenario: Branch context follows repository changes
- **WHEN** the active document or workspace changes, or the repository switches to another named branch while Markion remains open
- **THEN** the persistent status context eventually refreshes to the branch for the current context
- **AND** a stale lookup result from an earlier document or workspace is not displayed

#### Scenario: Switching tabs updates all document context
- **WHEN** the user activates a different tab
- **THEN** counts and caret position immediately describe the newly active tab
- **AND** the Git branch item resolves from the newly active tab or its workspace fallback

#### Scenario: Persistent context coexists with transient feedback
- **WHEN** a save, export, search, formatting, error, or other existing operation updates the transient status message
- **THEN** that feedback remains visible in the status bar alongside the persistent document context
- **AND** the status bar remains a single compact row without overlapping or wrapping its items

#### Scenario: Status feedback omits document identity
- **WHEN** the active tab has a file name and optional unsaved changes
- **THEN** the status-bar feedback region does not contain the Markion brand prefix, that file name, or the unsaved-changes `*` suffix
- **AND** save-state tokens and transient operation messages remain visible in that region

#### Scenario: Context labels follow the active language
- **WHEN** the active interface language changes
- **THEN** the character, word, branch, line, and column labels are rendered through the localization catalog in the selected language
- **AND** document text and the branch name are displayed verbatim rather than translated

#### Scenario: Status rendering preserves typing-path caches
- **WHEN** the document is rendered repeatedly without a text-version change, or only the caret moves
- **THEN** the status bar reuses the cached document metrics for that version
- **AND** caret-only changes do not invalidate Markdown-derived state
- **AND** Git discovery and refresh do not run synchronously during rendering or text input
