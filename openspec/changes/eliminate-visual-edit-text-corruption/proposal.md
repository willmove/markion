## Why

One observed Visual Edit session duplicated every heading from §1.1 through §1.9 in the canonical in-memory Markdown while the on-disk file remained clean. Because the outline is derived from that canonical text, this is a data-integrity incident rather than a rendering defect; commit `a530a82` added mutation diagnostics but did not identify or remove the corrupting write path.

## What Changes

- Make every canonical document mutation attributable to one typed operation carrying document identity, origin, expected document version, exact source range, and privacy-preserving before/after evidence.
- Route Visual Edit text input, IME, structural edits, formatting, block/table commands, undo/redo, and reload through the same checked mutation boundary; reject stale, out-of-range, or source-mismatched operations before they can modify `MarkdownDocument.text`.
- Retain a bounded per-document mutation journal that can deterministically reconstruct the operation sequence without logging authored content, and emit it when a mutation is rejected or an integrity assertion fails.
- Add deterministic replay, state-machine, and mutation-contract tests over heading-dense documents. The change cannot be declared fixed until the responsible path has a failing regression test and the implementation removes that failure.
- Isolate this P0 integrity work from the tail-height and scroll-fidelity fixes in `fix-visual-edit-tail-fidelity`; its existing diagnostics remain input evidence, not the root-cause fix.

**Non-goals:** no outline UI redesign, no Visual Edit rendering or tail-whitespace changes, no document-content logging, no full Markdown reparse on each keystroke, and no persisted-file or undo-snapshot format migration.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: strengthen the canonical-source and source-backed Visual Edit contracts so every accepted mutation is version-checked, range-exact, attributable, and regression-verifiable; stale or ambiguous visual operations must leave the document unchanged.

## Impact

- `src/lib.rs` and document mutation types: canonical checked-mutation boundary, provenance, and bounded journal.
- `src/app/editor_element.rs`, `src/app/editing.rs`, `src/app/application.rs`, and `src/app/documents.rs`: typed origins and expected-version/range evidence for platform input, IME, commands, history, and reload.
- `src/source_mapped.rs`, Visual Edit projection/edit types, and app state: stale projection rejection and deterministic operation replay.
- Tests in the root crate: localized-splice contracts, stale-event rejection, multi-tab/IME/undo state machines, and an incident-specific regression once the corrupting path is reproduced.
- Existing per-version `Arc`-shared Markdown caches, memoized highlighting, and cached text handles remain version-driven; the journal records mutations without deriving Markdown or adding work to render-only interactions.
