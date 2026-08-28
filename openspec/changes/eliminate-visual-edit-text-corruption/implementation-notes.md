# Implementation notes — eliminate-visual-edit-text-corruption

## Stage A summary

- **Checked boundary** (`src/lib.rs`): `DocumentInstanceId`, exhaustive
  `MutationOrigin`, `CheckedMutation` (exact range with expected-source
  evidence / authorized whole-document), `MutationReceipt` /
  `MutationRejection`, and the single `apply_checked_mutation` entry that
  validates instance, version, range ordering/bounds/UTF-8 boundaries, and
  source evidence before any write. Accepted edits reuse the existing
  dirty/version/cache-invalidation path exactly once; the unchecked write
  primitive is private to the boundary.
- **Bounded evidence**: 256-entry per-document ring journal (sequence,
  origin, target, expected/current/before/after versions, range/lengths,
  touched-slice fingerprints, rejection reason) — no authored bytes. Emitted
  at error level with the full ring on every rejection. Memory accounting
  includes the site; undo clones do not carry the journal.
- **Path migration**: platform input and IME (document-bound generation
  tracking via `document_input_target` / `ime_input_target`), structural
  Enter/Backspace, formatting, image/link field edits, exact block and table
  commands (`BlockEdit` now carries its source version), undo/redo
  (authorized whole-document restores plus checked diff application),
  background reload (`ExternalCheckRequest` carries instance/version;
  `apply_external_reload_checked` journals rejections), startup open
  (tab-slot generation binding), and rename-driven tab replacement (history
  reset). Repository-wide enforcement lives in
  `production_code_cannot_bypass_checked_mutations`, which fails when any
  production line calls the trusted convenience mutators.
- **UX on rejection**: localized content-free warning
  (`P0Msg::IntegrityMutationRejected`, all 8 locales) that preserves the
  document, resets stale composition/control state, and points at the log.

## Stage B — incident root cause (tasks 5.1–5.4)

**Responsible path:** `DocumentTabState::restore_snapshot`
(`src/app/state.rs`), invoked by undo/redo of `Full` history entries.

**Violated precondition:** a `text_version` value names at most one text per
document — the invariant the tab-level per-version caches rely on by design
(`display_text_cache`, `source_layout_key`, `measured_height_cache`; see the
"versions are globally unique" comment in `src/app/state.rs`).

**Mechanism:** `restore_snapshot` replaced the live document with the
snapshot clone wholesale. Because clones copy `text_version` verbatim, an
undo *reverted* the version counter. After undo + one more edit, the same
version number named two different texts; the per-version caches then served
the first epoch's text/layout while the canonical layer held another, and
any platform input carrying offsets derived from the aliased generation was
reinterpreted against the wrong text — and, pre-boundary, `replace_range`
*clamped* invalid offsets instead of rejecting, silently splicing the stale
payload into the wrong location. That combination is the
unauthorized-duplication class observed as every heading §1.1–§1.9 appearing
twice in canonical memory while the file on disk remained clean.

**Minimal operation trace** (preserved as the named regression
`undo_restore_cannot_reuse_versions_or_alias_display_text` in
`src/app/mutation_tests.rs`):

```text
1. share text           v1 -> display cache holds T1
2. type "X"; share text v2 -> display cache holds T2
3. undo                 [pre-fix: version REVERTS to v1]
4. type "Y"             [pre-fix: v2 again, text T3 != T2]
5. share text           [pre-fix: stale "T2" served for canonical T3]
```

**Red-test verification:** with `restore_snapshot` temporarily reverted to
the wholesale assignment, both the named regression and the deterministic
state machine fail (`undo must advance the version…` /
`tab 0 version went backwards (3 < 5)`); with the fix restored, the full
suite passes.

**Narrow fix (5.3):** route snapshot restoration through an authorized
whole-document checked mutation on the live document (`MutationOrigin::Undo`
/ `Redo`). Instance identity survives, the version only ever advances, and
the restore itself is journaled. No per-keystroke Markdown derivation was
added and the boundary was not weakened.

**Variant proof (5.4):** the deterministic state machine interleaves the
trace with IME update/commit/cancel, explicit-offset input, stale callback
rejection, undo/redo, mode/tab switches, derived completion, and reload
delivery across three fixed seeds, asserting the reference splice, version
monotonicity, one-text-per-version, and a heading-duplication tripwire after
every step; focused tests cover the delayed-reload, tab-isolation, and
tab-slot-replacement variants directly.

**Evidence status (5.1):** no incident journal from the original report was
available, so identification came from the code audit above plus the
deterministic machine rather than replayed logs. The uncorrelated
`a530a82` diagnostic pairs are superseded by the journaled, sequenced
boundary evidence (`markion::mutation` target with sequence/document/
origin/version correlation on both accept and reject).

## Out-of-band verification still owed

Task 6.3 (manual GUI verification of typing, CJK/emoji IME, structural
editing, formatting, undo/redo, tab switching, external reload, and a forced
stale event) requires a human at the window; it is deliberately left
unchecked.
