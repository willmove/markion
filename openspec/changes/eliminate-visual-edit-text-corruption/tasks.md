## 1. Checked canonical mutation boundary

- [ ] 1.1 Add failing unit tests for exact range replacement, one-version advancement, unchanged prefix/suffix bytes, stale-version rejection, wrong-document rejection, invalid/reversed/non-UTF-8 ranges, and unauthorized whole-document replacement
- [ ] 1.2 Add GPUI-free document instance IDs, exhaustive mutation origins, checked range/whole-document edit types, preconditions, receipts, and typed rejection reasons in the root document layer
- [ ] 1.3 Implement the single checked mutation boundary so validation happens before writes and accepted mutations reuse the existing dirty/version/cache-invalidation paths exactly once
- [ ] 1.4 Migrate construction/test convenience methods onto explicit trusted origins and make unchecked source writes private to the boundary; use a repository-wide call-site audit to prove production code cannot bypass it

## 2. Editing and lifecycle path migration

- [ ] 2.1 Migrate ordinary platform text input and selection replacement to checked operations carrying the exact document instance, version, canonical UTF-8 range, and source evidence used by the projection
- [ ] 2.2 Give IME composition a document-bound version chain and migrate update/commit/cancel paths; stale composition callbacks must reset or refresh composition without mutating current text
- [ ] 2.3 Migrate structural Enter/Backspace, formatting, slash/link/image field edits, and exact block/table/reorder commands so each helper result is one checked range edit tied to the source version and ownership used to compute it
- [ ] 2.4 Migrate undo/redo and recovery application to authorized whole-document operations with exact document-instance/version checks and snapshot selection restoration
- [ ] 2.5 Migrate manual/background reload and tab-document replacement to generation-bound lifecycle operations so a delayed result cannot overwrite intervening edits or a replacement document

## 3. Correlated privacy-preserving evidence

- [ ] 3.1 Add the bounded 256-entry per-document mutation journal with mutation sequence, origin, target instance, expected/current/before/after versions, range/length metadata, touched-slice fingerprints, and rejection reason, storing no authored bytes
- [ ] 3.2 Correlate high-level app events, canonical decisions, and receipts under the same mutation sequence; replace the uncorrelated `a530a82` diagnostic pairs where the checked boundary now supplies stronger evidence
- [ ] 3.3 Emit the bounded journal on integrity rejection and add a localized user warning that preserves the document, resets stale UI/composition state, and identifies the log location without exposing content
- [ ] 3.4 Update document memory accounting and add a large-document typing benchmark/test proving range edits do not hash/parse the whole document or invalidate derived caches more than once

## 4. Deterministic invariant coverage

- [ ] 4.1 Add table-driven mutation-contract tests for every origin, including accepted exact splices and unchanged state for every rejection class
- [ ] 4.2 Add multi-tab tests for delayed platform/command/reload results, tab switching, tab-slot replacement, and undo/redo isolation using document instance IDs rather than active indices
- [ ] 4.3 Build a deterministic state-machine harness over a §1.1–§1.9 heading-dense document with a reference `String`, selection, per-tab history, fixed seeds, and a replayable operation trace
- [ ] 4.4 Cover ordinary input, replacement, structural edits, formatting, block/table commands, IME update/commit/cancel, undo/redo, mode/tab switches, stale callbacks, derived completion, and reload completion in the state machine; compare source and versions after every step
- [ ] 4.5 Verify render-only caret/selection/scroll/hover/control and derived-cache activity creates no mutation entries, changes no version, and preserves existing cached-per-version Markdown invariants

## 5. Historical incident root cause and fix gate

- [ ] 5.1 Run and extend the state machine against the suspected Visual Edit workflows and analyze any available/next incident journal until the first unauthorized duplication operation and responsible path are identified; record the minimal operation trace in implementation notes
- [ ] 5.2 Add a named regression that fails on the pre-fix code with the minimized heading-duplication sequence and asserts the exact authorized final source, not merely outline/full-parse agreement
- [ ] 5.3 Document the violated precondition and implement the narrow root-cause fix in the responsible operation path without weakening the checked boundary or adding per-keystroke Markdown derivation
- [ ] 5.4 Prove the named incident regression across relevant Visual Edit, IME, history, async, and multi-tab variants and confirm the same sequence cannot modify bytes outside its declared edit ranges

## 6. Verification

- [ ] 6.1 Run formatting and the focused mutation, state-machine, IME, multi-tab, reload, history, cache, memory, and performance tests
- [ ] 6.2 Run `cargo test --workspace` and fix all regressions without bypassing rejection checks
- [ ] 6.3 Manually verify normal typing, CJK/emoji IME, structural editing, formatting, undo/redo, tab switching, and external reload; force one stale event and confirm content preservation plus content-free diagnostics
- [ ] 6.4 Run `openspec validate eliminate-visual-edit-text-corruption` and leave tasks 5.1–5.4 unchecked unless the historical incident has a reproduced red test and path-specific fix
