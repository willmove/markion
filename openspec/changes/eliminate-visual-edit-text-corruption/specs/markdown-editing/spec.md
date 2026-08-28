## ADDED Requirements

### Requirement: Canonical Markdown mutation integrity and provenance
`MarkdownDocument.text` SHALL remain the sole canonical editable document value. Every accepted user or system mutation SHALL target one document instance and one current document version, SHALL declare whether it is an exact UTF-8 range replacement or an authorized whole-document lifecycle replacement, and SHALL be attributable to its originating editing operation. An exact range replacement SHALL change only the declared range and SHALL preserve every byte outside that range. A stale, cross-document, out-of-bounds, non-UTF-8-boundary, source-mismatched, or otherwise ambiguous mutation SHALL be rejected before changing text, version, dirty state, selection, history, recovery state, or derived caches.

Accepted and rejected mutations SHALL produce bounded, privacy-preserving diagnostic evidence sufficient to correlate the high-level operation with the canonical mutation and its before/after versions, ranges, and lengths. This evidence SHALL NOT contain authored document text and SHALL NOT require Markdown parsing or derived-state recomputation. An accepted mutation SHALL invalidate derived Markdown state only through the resulting document-version change, preserving the existing per-version cache contract.

#### Scenario: Exact Visual Edit replacement preserves unrelated source
- **WHEN** Visual Edit accepts platform text input, IME input, a structural edit, formatting, or a block or table command for an exact current source range
- **THEN** the canonical source after the mutation equals the source before the mutation with exactly that range replaced
- **AND** every source byte before and after the declared range remains byte-identical
- **AND** the operation advances the document version exactly once through the existing dirty-state, history, autosave, and recovery paths

#### Scenario: Stale visual projection cannot mutate current text
- **WHEN** a Visual Edit operation was derived from an earlier document version or from a visual block that no longer owns the declared source range
- **THEN** the operation is rejected without rebasing or guessing a new range
- **AND** canonical source, version, dirty state, selection, history, recovery state, and derived caches remain unchanged

#### Scenario: Mutation cannot cross document tabs
- **WHEN** a delayed input, composition update, command, or reload result targets a document instance that is no longer the active tab or has been replaced
- **THEN** the operation is applied only if that exact document instance and expected version are still valid
- **AND** it cannot mutate the document currently occupying another tab or a replacement document in the same tab slot

#### Scenario: Invalid range is rejected instead of silently clamped
- **WHEN** a mutation declares a range that is out of bounds, reversed, not on UTF-8 boundaries, or inconsistent with the source evidence from which the operation was derived
- **THEN** the mutation is rejected before any document state changes
- **AND** diagnostic evidence identifies the origin, target document, expected/current versions, and invalid range without recording authored text

#### Scenario: Undo and redo remain exact whole-document operations
- **WHEN** Undo or Redo restores a document snapshot
- **THEN** the replacement is explicitly attributed to that history operation and targets the exact current document instance and version
- **AND** the restored source and selection equal the corresponding snapshot
- **AND** one history action advances the document version exactly once without attaching the snapshot to another tab

#### Scenario: External reload cannot overwrite intervening edits
- **WHEN** a background disk read completes after the target document has changed, become dirty, been replaced, or moved out of the expected document generation
- **THEN** the reload result does not replace the newer canonical in-memory text
- **AND** the rejected lifecycle replacement is attributable without logging either the disk or in-memory document content

#### Scenario: Mutation evidence is bounded and content-free
- **WHEN** a document receives more mutations than the diagnostic retention bound
- **THEN** only the newest bounded sequence of mutation metadata is retained
- **AND** each retained entry contains operation identity, document identity, versions, ranges, lengths, and content fingerprints at most
- **AND** no authored source, replacement payload, clipboard content, or IME composition text is written to diagnostics

#### Scenario: Render-only interaction remains mutation-free
- **WHEN** the user moves the caret, changes selection, scrolls, hovers, opens a visual control, or causes derived Markdown to finish without accepting a text operation
- **THEN** no canonical mutation record is created and the document version remains unchanged
- **AND** preview blocks, outline, stats, syntax highlighting, visual blocks, and cached text handles continue to follow their existing per-document-version cache rules
