## Purpose

Let the agent retrieve approved workspace text and propose practical multi-file organization, while ensuring review, filesystem containment, dirty-buffer safety, and recoverable application.

## ADDED Requirements

### Requirement: Workspace tools SHALL retrieve bounded permitted text
Agent tools SHALL list supported entries, search filenames/text, and read bounded supported text ranges within approved scope. Results SHALL identify relative paths and available line ranges, paginate large listings/searches, and prefer current open-buffer content. Protected secrets, ignored directories, unsupported/binary files, and unapproved files SHALL not be read or sent. Retrieval SHALL run asynchronously and SHALL not introduce automatic whole-workspace indexing or affect document caches.

#### Scenario: Dirty note is read
- **WHEN** the agent reads an approved note that is already open with unsaved text
- **THEN** the result reflects the current buffer and identifies its source/version
- **AND** the note is not automatically saved

#### Scenario: Large search result
- **WHEN** a permitted search matches more files than fit within the result budget
- **THEN** the tool returns a bounded page and continuation information rather than loading every match into model context

### Requirement: Mutating tools SHALL produce concrete editable review plans
The agent SHALL be able to propose creating Markdown/text notes, editing supported text files, creating folders, and renaming/moving permitted supported files. Proposed paths SHALL be workspace-relative. Review SHALL show each affected path, operation, textual before/after changes where applicable, dependencies, and any Markdown link consequences. The user SHALL be able to select operations, revise proposed names/destinations, and accept or reject the concrete selected plan. Editing a plan SHALL revalidate it and invalidate previous approval; dependent operations SHALL not remain selected without their prerequisites. No proposal SHALL execute without this review action.

#### Scenario: Organize several notes
- **WHEN** the user asks to organize approved notes into topic folders
- **THEN** the agent proposes folder creations and individual moves/renames in one review surface
- **AND** the user can deselect or rename items before applying the selected plan

#### Scenario: Rename affects relative links
- **WHEN** a proposed note move would change relative Markdown links in permitted text files
- **THEN** review includes explicitly proposed link edits or states which links cannot be checked outside scope
- **AND** unseen files are not scanned or rewritten without permission

### Requirement: AI file scope SHALL enforce actual filesystem containment
Reads and mutations SHALL reject absolute paths, parent traversal, disallowed files, ambiguous Windows names, and aliases escaping the workspace/read grant. Containment SHALL account for symlinks, junctions/reparse points, case-insensitive aliases, alternate data streams, and real existing ancestors of new destinations. Existing target collisions and unsafe directory cycles SHALL be rejected without overwrite. Checks SHALL be repeated immediately before execution, and ambiguous path identity SHALL fail closed. Existing folder moves SHALL be excluded unless their entire affected subtree is supported, permitted, and reviewed.

#### Scenario: Junction escapes the workspace
- **WHEN** a proposed read or write traverses a workspace junction pointing outside approved scope
- **THEN** the operation is rejected without reading or changing the outside target

#### Scenario: Collision or unsafe filename
- **WHEN** an operation would overwrite an existing file or uses a reserved/ambiguous Windows filename or alternate data stream
- **THEN** it is rejected with actionable path feedback and existing bytes remain unchanged

### Requirement: Agent tools SHALL expose only supported application operations
The agent SHALL not execute shell/system commands, external plugins, web/OS automation, Git synchronization, publishing, arbitrary URL fetches, or permanent deletion. It SHALL describe an unsupported request accurately and offer supported alternatives, such as proposing a folder organization plan. Model-generated text, file content, and restored conversations SHALL not authorize additional tools or wider paths. Protected secret files SHALL not become readable through an explicit attachment or model request.

#### Scenario: Note contains instructions to execute commands
- **WHEN** a retrieved note asks the agent to run a command or read credentials
- **THEN** no such operation executes and the note is treated as task content

### Requirement: Application SHALL preserve open documents and coordinate writers
Before applying, every selected operation SHALL revalidate workspace identity, captured document versions and disk identities, current permissions, and Git writer admission for all affected sources/destinations. Dirty open text edits SHALL operate on checked in-memory buffers with undo/recovery, not overwrite disk underneath them. Dirty open files SHALL require save-first before moves/renames; there SHALL be no implicit save/discard. Clean open tabs SHALL remap on accepted moves/renames. Autosave, external reloads, and Git worktree changes SHALL not race AI file operations. Conflicts SHALL preserve both current content and the unapplied proposal with refresh/regenerate guidance.

#### Scenario: Rename targets a dirty open file
- **WHEN** a reviewed plan includes moving a file with unsaved buffer changes
- **THEN** applying that operation is blocked with Save First feedback
- **AND** neither buffer content nor file path is silently changed

#### Scenario: Disk changed after review
- **WHEN** a closed text file changes externally after its edit is proposed
- **THEN** the stale edit is rejected without overwriting the new disk bytes

#### Scenario: Git owns the repository writer boundary
- **WHEN** Git is mutating a repository containing an AI source or destination
- **THEN** the AI operation waits at a supported safe boundary or reports that it cannot start
- **AND** it does not bypass repository write admission

### Requirement: File application SHALL be journaled and report partial outcomes accurately
Before the first persistent change, application SHALL durably prepare an operation journal with affected paths, original identities and bytes where needed, intended changes, and dependency order. Text replacement SHALL be atomic and failure-safe. Execution SHALL record each completed operation durably, stop dependent work after failure/cancellation, and distinguish applied, failed, skipped, and unapplied operations. Journals/backups SHALL be outside the user workspace and SHALL not include credentials. Application SHALL not claim cross-file atomicity. If journal preparation fails, no persistent operation SHALL start.

#### Scenario: Journal cannot be written
- **WHEN** the recovery journal or required backup cannot be durably prepared
- **THEN** no persistent operation begins and the review reports the preparation failure

#### Scenario: Failure after one successful move
- **WHEN** one selected move succeeds and the following operation fails
- **THEN** the first operation is reported as applied and recoverable, the failure is identified, and dependent operations remain unapplied
- **AND** retry does not repeat the completed move

### Requirement: Applied file operations SHALL be safely restorable across restarts
The UI SHALL offer Restore for applied file plans and surface interrupted journals on startup. Restoration SHALL compare current affected buffer/disk identities with the recorded post-operation state and SHALL refuse to overwrite subsequent user/external edits. Journal recovery SHALL reconcile ambiguous completion through actual filesystem identities before offering resume/restore. In-memory text edits SHALL use document Undo; persistent operations SHALL use the reviewed journal restore flow. Deleting conversation history or resetting preferences SHALL not delete unresolved action recovery. Explicitly retiring a recovery entry SHALL describe what recovery data will be removed.

#### Scenario: Restore an unchanged applied move
- **WHEN** the user restores a completed move whose source/destination identities still match its journal
- **THEN** the file returns to its original location and clean tabs/file-tree state are updated

#### Scenario: Subsequent edit prevents restore
- **WHEN** a moved or edited file has changed after AI application
- **THEN** Restore preserves that newer content and reports the conflict instead of replacing it with the backup

#### Scenario: Interrupted application survives restart
- **WHEN** the app stops between a filesystem mutation and completion recording
- **THEN** the next startup surfaces the journal and reconciles actual paths/content before recovery
- **AND** it does not blindly replay the operation
