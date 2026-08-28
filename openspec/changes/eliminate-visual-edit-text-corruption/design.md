## Context

See `proposal.md` — Why. The current canonical mutation choke points are `MarkdownDocument::replace_source_range` and `set_text`, but high-level operation labels are emitted separately by app handlers. The resulting log lines do not share a document-instance identifier or mutation identifier, and the canonical layer does not know the operation's expected document version or source ownership. The instrumentation in `a530a82` can narrow the next incident but cannot reject a stale/wrong-target write or prove which high-level event authorized it.

Visual Edit derives exact source ranges from versioned cached state, while platform input, IME sessions, commands, undo/redo, background reloads, and tab replacement have different lifetimes. `MarkdownDocument.text` is the only canonical value; outline and preview are derived from it and therefore cannot independently detect a canonical text duplication. Any guard must sit before the canonical write and must not add a parse, outline derivation, or text-handle rebuild to render-only interaction.

Current flow and intended boundary:

```text
platform / IME / command / history / reload event
  -> resolve exact document instance + expected version + source ownership
  -> CheckedMutation { origin, target, precondition, edit }
  -> MarkdownDocument::apply_checked_mutation
       reject: text/version/history/caches unchanged + bounded evidence
       accept: one exact replace -> version++ -> existing cache invalidation
  -> MutationReceipt -> selection/history/autosave/recovery coordination
  -> lazy/version-gated preview + outline + visual-block derivation (unchanged)
```

## Goals / Non-Goals

**Goals:**

- Make a canonical write impossible without an attributable operation, exact target document, expected version, and validated edit boundary.
- Turn stale, cross-tab, and source-mismatched events into observable no-ops rather than best-effort edits.
- Preserve enough content-free evidence to identify the first bad operation and replay the same operation class in tests.
- Establish regression evidence before declaring the observed duplication fixed.

**Non-Goals:**

- Proving that every semantically undesirable user command can be recognized from text alone.
- Persisting mutation history, adding a second document model, or using the outline as an integrity oracle.
- Recomputing full-document hashes or Markdown-derived state on every keystroke.
- Folding the separate tail-height/scroll fixes into this change.

## Decisions

### D1 — One checked mutation envelope at the canonical boundary

Introduce a GPUI-free mutation envelope in the root document layer with:

- a session-local `DocumentInstanceId` that changes when a tab's document is replaced;
- an exhaustive `MutationOrigin` for platform text, IME update/commit, structural edit, formatting, exact block/table command, undo, redo, external reload, recovery, and trusted construction/test operations;
- `expected_version` and an edit variant: exact range replacement or explicitly authorized whole-document replacement;
- source-ownership evidence for range edits (the expected bytes or an exact owner/version token). The boundary compares evidence directly but retains only a fingerprint in diagnostics;
- a `MutationReceipt` or typed rejection returned to the caller.

`apply_checked_mutation` validates instance, version, range ordering/bounds/UTF-8 boundaries, and source evidence before writing. Accepted range edits use the declared range without the current silent boundary clamping, preserve prefix/suffix bytes by construction, and advance the version once. Whole-document replacement is limited to named lifecycle origins; ordinary Visual Edit input cannot request it.

The current convenience mutation methods are migrated to construct checked operations or become test/internal helpers so production code cannot bypass the boundary.

*Alternatives:* keep independent debug labels at every call site (insufficient correlation and no prevention); add assertions after arbitrary writes (detects too late and production assertions may be disabled); serialize all editing through a parallel rendered document (violates the canonical-source contract).

### D2 — Validate visual ownership before payload application

Operations derived from Visual Edit carry the document instance/version and, where applicable, the visual block identity plus exact owned source range. The app resolves display/IME ranges to canonical UTF-8 ranges against that same projection and submits the checked operation without later rebasing. If the tab, document version, block identity, or owned range has changed, the operation is rejected and the UI refreshes from current canonical state. IME rejection also terminates or refreshes the stale composition rather than applying its payload elsewhere.

Structural/block/table helpers continue to compute one replacement, but their result becomes a checked edit tied to the source/version used to compute it. Undo/redo use authorized whole-document replacements tied to the current document instance and the selected snapshot. Background reload completion uses the captured document instance, generation/version, dirty precondition, and disk identity before replacement.

*Alternatives:* rebase a stale range by byte delta (ambiguous across block splits, equal headings, and IME updates); trust the active tab index (a slot can contain a different document by callback time); clamp invalid byte offsets (silently changes the requested edit and hides mapping defects).

### D3 — Correlated bounded evidence without document content

Each document keeps a ring of the newest 256 accepted or rejected mutation metadata entries. A monotonically increasing mutation sequence correlates the app event, canonical decision, and receipt. An entry stores document instance, origin, expected/current/before/after versions, declared range, replaced/replacement lengths, small fingerprints of compared/replacement slices, and rejection reason. It never stores source or replacement bytes.

Fingerprints cover only bytes already inspected for the operation; range edits do not hash the whole document. Whole-document lifecycle replacements may fingerprint their payload because they are already O(document size). The ring is emitted at error level on an integrity rejection and remains available to the existing debug log path for an incident report. Memory accounting includes the fixed bound, and cloning for undo does not clone the journal as authored state.

*Alternatives:* log full snapshots (privacy and volume risk); hash the full document before/after every keypress (O(document size) hot-path cost); retain an unbounded event log (memory growth); rely on operation names without a sequence/document ID (ambiguous with multiple tabs and IME).

### D4 — Regression strategy uses an edit oracle, not the outline

The outline is deliberately not the corruption oracle: it is a 1:1 derivation of canonical text and correctly mirrors duplicated headings. Tests instead compare each accepted edit with a reference splice over the prior source and assert that only the declared range changed, the version advanced once, and the receipt matches.

Add deterministic state-machine coverage over a heading-dense fixture modeled on §1.1–§1.9. The operation generator interleaves ordinary input/replacement, selections, IME update/commit/cancel, structural Enter/Backspace, formatting, exact block/table edits, undo/redo, mode/tab switches, stale callbacks, and background-derived/reload completion. A simple reference `String` plus per-tab snapshot stacks is the oracle. Fixed seeds and a printed/minimized operation trace make failures replayable without a new fuzzing dependency.

This suite can be written before the incident root cause is known. It does not replace the incident-specific gate: once logs or the state machine expose the responsible path, first preserve the smallest failing sequence as a named regression, then fix that path. The change remains incomplete if only generic tests and diagnostics pass.

*Alternatives:* wait for another manual reproduction before adding any tests (leaves known invariants uncovered); assert only that outline equals a fresh parse (passes when canonical text is already corrupted); snapshot the final UI (cannot identify the unauthorized write).

### D5 — Root-cause evidence is a completion gate

Implementation proceeds in two safe stages. Stage A lands the checked boundary, path migration, bounded evidence, and generic deterministic tests. Stage B obtains a minimal failing sequence from captured evidence or state-machine exploration, documents the responsible operation and violated precondition, adds the red test, and applies the narrow fix. Tasks for the incident-specific regression and fix cannot be checked merely because the defect did not recur manually.

If Stage A reveals that `a530a82` already captured a reproducible bad operation, use that trace directly. Otherwise the product is hardened against the enumerated stale/wrong-target classes, but the change is not described or archived as elimination of the observed incident until Stage B is satisfied.

*Alternatives:* ship a speculative change to the most suspicious edit helper (can move or mask corruption); call diagnostics alone a fix (does not meet the data-integrity contract).

## Risks / Trade-offs

- [Mutation API migration misses a bypass] → Make the unchecked write primitive private to the canonical implementation and use repository-wide searches/tests to prove every production path supplies an origin.
- [IME produces legitimate rapid version changes and gets rejected] → Track one composition session explicitly; allow only its receipt-version chain, and test CJK/emoji update, commit, cancel, tab switch, and stale callback sequences.
- [Metadata fingerprints are mistaken for security guarantees] → Use direct byte/version validation for acceptance; fingerprints are diagnostic correlation only.
- [The 256-entry journal adds per-tab memory and hot-path work] → Store fixed-size metadata, fingerprint only touched slices, include it in memory accounting, and benchmark a large-document typing loop.
- [Generic hardening does not reproduce the historical path] → Keep the Stage B named-regression gate; report Stage A as hardening/diagnosis rather than a root-cause fix.
- [Rejecting an operation surprises the user] → Preserve text, reset stale composition/control state, refresh from canonical state, and surface a localized integrity warning with log location rather than applying a guessed edit.

## Migration Plan

1. Add the checked mutation types and contract tests while retaining temporary adapters for existing call sites.
2. Migrate platform input/IME, visual commands, structural edits, formatting, block/table operations, history, recovery, and reload; remove production access to unchecked writes.
3. Add the bounded journal, rejection UX, memory accounting, and performance coverage; verify existing per-version caches remain lazy.
4. Run the deterministic state machine, capture and minimize the first historical-class failure, add its named red regression, and implement the path-specific fix.
5. Run `cargo test --workspace`, focused long-sequence tests, and manual IME/multi-tab/reload verification before archiving.

No persisted format changes are involved. Rollback is a code revert; files and undo/recovery formats remain compatible. If the checked boundary itself causes regressions, retain the regression fixtures and evidence format while reverting caller migration as one unit rather than reopening unchecked writes piecemeal.
