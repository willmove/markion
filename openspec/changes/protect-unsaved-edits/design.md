## Context

Non-explicit opens already resolve through `default_open_intent()` (`src/app/workspace.rs`), which replaces the active tab only when `open_in_current_tab` is on **and** `EditorTab::is_safe_to_replace()` is true. That predicate currently treats every untitled document as replaceable (`path().is_none() || !is_dirty()`), so a dirty welcome page or unsaved draft is overwritten by a file-tree click with no prompt. File → Open still wraps replace with `confirm_discard_then`, which for untitled dirty tabs asks to discard rather than opening a new tab.

Close tab and quit already prompt, but only Discard/Cancel (or “Exit Without Saving”/Cancel). Close tab reuses `DialogDiscardNewDetail`. Menu Quit discards only the **active** tab’s recovery snapshot; the window-close guard discards **all**. GPUI prompts are not re-entrant; a four-button external-conflict dialog already exists, and Windows TaskDialog needs distinct `PromptButton` variants (`Ok` / `Other` / `Cancel`) so button IDs do not collide.

## Goals / Non-Goals

**Goals:**

- Never replace a dirty document tab, named or untitled, via a non-explicit open.
- Offer Save / Don’t Save / Cancel on close-tab and on quit/window-close, with Save going through the existing save / Save As pipeline.
- Unify recovery-snapshot cleanup on confirmed Don’t Save (and after a successful save-then-close/quit) across menu Quit and the window-close guard.

**Non-Goals:**

- File → New, Close Others / Close to the Right, and file-tree delete of an open dirty file.
- Per-tab prompt loops on quit; changing `open_in_current_tab`; new recovery format; derived-cache schedule changes.

## Decisions

### D1: Replace-eligibility is “not dirty” (images always)

Change `is_safe_to_replace` to:

```
document tab → !document.is_dirty()
image tab   → true
```

A pristine welcome or empty untitled tab stays replaceable (`from_text` / `new` start with `dirty == false`). The first keystroke marks dirty and subsequent non-explicit opens append. Alternative considered: keep untitled always replaceable as a “scratch slot” — rejected because that is the bug.

`default_open_intent()` is unchanged besides the predicate. File → Open drops `confirm_discard_then`: after D1, `ReplaceActive` already implies the tab is not dirty (or is an image), so the guard is dead. Gestures stay prompt-free.

### D2: One three-button unsaved prompt, unique GPUI button kinds

Shared helper (conceptually `prompt_unsaved_choice`) shows:

| Index | Kind | Role |
| --- | --- | --- |
| 0 | `PromptButton::ok(Save)` | default / Enter |
| 1 | `PromptButton::other(Don't Save)` | discard |
| 2 | `PromptButton::cancel(Cancel)` | Escape |

Do **not** use three `PromptButton::ok` values: on Windows they all map to `IDOK` and the first wins. Close-tab uses close-specific title/detail (document name or “Untitled”). Quit/window-close uses an aggregate title/detail that can mention the dirty-tab count. `File → New` keeps today’s two-button `confirm_discard_then`.

### D3: Save means save the affected dirty tabs, then proceed

**Close one dirty tab.** Capture `TabContextTarget` first.

- **Save:** named → `MarkdownDocument::save()` on that tab (not `force_save`). Untitled → activate that tab and run the existing Save As picker (`prompt_for_save_path` + `save_as`). On success, discard that tab’s recovery (existing save path already does) and run today’s `close_tab_confirmed`. On save failure, external conflict, or Save As cancel: abort close; the tab stays open. External conflict reuses `prompt_external_save_conflict` and does not continue closing.
- **Don’t Save:** today’s `close_tab_confirmed` (recovery delete, last tab becomes a fresh untitled).
- **Cancel:** no mutation.

**Quit / window close** (any dirty document tab). One prompt for the whole set.

- **Save:** walk dirty document tabs in opening order by captured identity. Named tabs save in place without requiring activation. Each untitled tab is activated, then Save As. Stop at the first failure, conflict, or cancelled picker; remaining dirty tabs stay dirty and the app stays open. After every dirty tab is clean, discard leftover recoveries (should be none), then the existing teardown (`allow_close` + quit; menu Quit also `remove_window`).
- **Don’t Save:** `discard_all_tab_recovery_files()` (both Quit and window-close; this fixes menu Quit only clearing the active snapshot), then teardown.
- **Cancel:** clear `confirming_close`, stay open.

Keep `confirming_close` set for the whole Save As chain so a second close event cannot stack prompts (GPUI prompts are not re-entrant). Identity matching already exists on `TabContextTarget` (`recovery_id` distinguishes untitled tabs).

### D4: Data flow / caching

```
non-explicit open
  → default_open_intent()
  → dirty document?  OpenInNewTab (existing append + dormancy on the left tab)
  → clean/image?     ReplaceActive (existing replace; no recovery belonging to a dirty tab)

close / quit
  → if any targeted document is dirty → 3-button prompt
  → Save: existing save / Save As (dirty false, path may change; text_version unchanged on save)
  → Don’t Save / after successful save: existing close or process exit
```

Save does not rewrite document text, so per-version preview/outline/highlight/text-handle caches stay valid until the tab is closed. Opening a diverted new tab uses `editor_tab_for_document` / `open_tab_in_new_tab` as today. No new derived-state invalidation.

### D5: Align the unarchived sibling delta before either archives

`open-documents-in-current-tab` still specifies untitled as replaceable and File → Open as dirty-guard-then-replace. This change restates the same requirement in its own delta **and** rewrites that sibling’s markdown-editing / image-file-viewing wording so archive order cannot resurrect the hole.

## Risks / Trade-offs

- **[Save As during close feels modal and slow]** → accepted; untitled work has no path. Cancelling the picker aborts close rather than discarding.
- **[Quit Save with many untitled tabs stacks pickers]** → accepted vs a per-tab three-button loop; one aggregate choice then sequential pickers only for tabs that need a path.
- **[External conflict mid-Save-all leaves some tabs saved and some dirty]** → abort quit/close and surface the conflict on the failing tab; already-saved tabs stay saved (correct on disk).
- **[Windows button-id collision]** → D2’s Ok / Other / Cancel mapping.
- **[Sibling delta contradiction]** → D5 rewrite task.

## Migration Plan

No config migration. Behavior change for untitled dirty opens (append instead of replace) and for close/quit (Save appears). Rollback is a revert of this change; `open_in_current_tab` is untouched.

## Open Questions

None. Close Others stays two-phase keep-then-discard-all; File → New stays discard-then-replace.
