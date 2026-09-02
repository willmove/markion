## Why

Unsaved work is still easy to lose. Untitled dirty tabs (the welcome page after typing, or any not-yet-saved draft) are treated as replaceable, so a file-tree click or File → Open silently overwrites them. Close-tab and quit dialogs only offer discard-or-cancel, with no way to save first, and the close-tab copy even reuses the "new document" wording.

## What Changes

- Treat a document tab as replaceable only when it is **not dirty**. Image tabs stay replaceable. A dirty untitled tab is no longer a "placeholder slot" that non-explicit opens may overwrite.
- Route every non-explicit open (file-tree click, drag-and-drop, Open Recent, File → Open) through that predicate: dirty active documents, named or untitled, append a new tab and never prompt or discard. File → Open no longer shows a discard guard before replacing, because a replaceable tab cannot be dirty.
- Replace the close-tab and quit/window-close confirmations with a three-way **Save / Don't Save / Cancel** prompt. Save writes the affected dirty document(s) (Save As for untitled) and then proceeds; Don't Save discards through the existing recovery-cleanup path; Cancel leaves everything open. Menu Quit and the window-close guard share this path, including discarding **all** dirty tabs' recovery snapshots on Don't Save.
- Give close-tab its own localized title and detail instead of reusing the File → New discard copy.

Non-goals: File → New stays a discard-then-replace confirmation; Close Others / Close to the Right stay "keep dirty tabs, then optional discard-all"; file-tree delete of an open dirty file is unchanged; quit does not cycle a per-tab prompt (one aggregate three-button dialog); the open-in-current-tab preference itself is unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: the Multi-document tab model’s replace-eligibility, File → Open dirty handling, close-tab confirmation, and quit/window-close confirmation.
- `image-file-viewing`: File → Open of an image follows the same replace-eligibility rule (no separate dirty-guard-then-replace).

## Impact

- Code: `src/app/state.rs` (`is_safe_to_replace`), `src/app/workspace.rs` (`default_open_intent` comments/callers), `src/app/documents.rs` (File → Open, close tab), `src/app/editing.rs` (`request_quit`, shared unsaved prompt), `src/app/bootstrap.rs` (window-close guard), `src/i18n.rs` (seven-language Save / Don't Save / Cancel and close-tab copy), tests in `src/app/tests.rs`.
- Spec-tree: the unarchived `open-documents-in-current-tab` delta still calls untitled tabs replaceable and keeps a File → Open dirty guard; this change must rewrite that wording so the two cannot archive into contradictory specs.
- Invariants: opening, saving, and closing still go through existing document-version, undo, autosave, and recovery paths. Derived Markdown caches, syntax highlighting, and per-version text handles are not recomputed on a different schedule; a saved-then-closed tab tears down as today’s close path already does.
- Persistence: Don't Save remains an intentional discard that retires recovery snapshots (`reliable-file-persistence`). Save uses the existing `save` / Save As pipeline, including external-change conflict handling; a failed or cancelled save aborts the close or quit.
