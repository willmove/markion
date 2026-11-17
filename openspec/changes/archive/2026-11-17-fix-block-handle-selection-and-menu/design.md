## Context

See `proposal.md` for the requested behavior. Investigation found two distinct paths:

- `select_source_blocks` constructs a versioned `BlockSelection`; `source_block_unit` includes separator whitespace up to the next block's source start. Its range is half-open and already excludes the next heading marker.
- `build_visual_projection_with_marked_range` activates a reveal when either selection endpoint belongs to a marker range. A non-empty selection ending at the following heading's prefix start therefore reveals that prefix, even though none of its bytes are selected. The separately supplied cursor is the same exclusive ending boundary and can also activate it.

A temporary diagnostic outside the repository, linked against an existing local Markion library build, confirmed this for `Paragraph\n\n## Heading\n`, CRLF prose followed by a Chinese H3, and H2 followed by H3. The selected byte slices contain only the first source unit; each following heading nevertheless projects its visible `## ` or `### ` prefix. No application source was changed for this investigation. New regressions must reproduce this against the current source during implementation.

The grip currently selects on mouse-down and begins a drag on movement. Existing right-click/keyboard block menus already provide anchored root overlays, one-level submenus, viewport constraints, stale-target checking and canonical edits. General preferences already support immediate toggles, TOML persistence and reset. Shared menu files currently contain uncommitted work for `compact-slash-command-panel`.

## Goals / Non-Goals

**Goals:** isolate source ownership from projection activation; share commands between the existing context menu and a compact grip presentation; keep click, Shift-click and drag unambiguous; make the automatic popup optional without disabling block selection.

**Non-Goals:** replacing the source-unit model, reparsing Markdown on selection/menu changes, adding document color/alignment metadata, or introducing another writable representation of a document.

## Decisions

### 1. Respect the exclusive end in projection, without shortening block selections

For a collapsed selection, preserve the existing caret behavior at marker starts and ends, including empty headings/list items and IME composition. For a non-empty selection, determine marker activation using actual source intersection rather than treating its excluded end as an insertion caret in the next block. The separately supplied cursor must not reintroduce that activation when it equals the non-empty selection's excluded end. Marked ranges retain their independent intersection behavior.

Keep `BlockSelection::range` and the complete source-unit operation authoritative. Trimming `#` characters or subtracting one byte from selection ranges would corrupt structural copy/delete/move, UTF-8 safety and separators, so it is not the fix. Test headings, lists, quote markers, formatted inline groups, reversed selections and real collapsed carets before broadening any shared endpoint rule.

### 2. Open only after a completed ordinary grip click

Retain mouse-down selection behavior needed to extend a range and preserve a selected multi-block range before dragging. A completed unmodified click validates the captured target, selects its single complete source unit, and opens the popup if enabled. Shift-click only extends selection. Once the pointer interaction becomes a drag, suppress/dismiss any pending click popup; do not open it on mouse-down. Use the framework's completed-click/drag distinction and explicit gesture state only if required.

Capture a versioned `BlockTarget`, revalidate it on completion, and avoid relying on a stale virtual-list index. Preserve multi-block drag semantics. A stale or unsupported target yields no guessed operation. Escape and outside dismissal preserve the block selection established by the initiating click.

### 3. Reuse menu state, command validation and root overlay hosting

Record the invocation style so the grip can render a compact popup while explicit right-click and keyboard context menus retain their existing selection-preserving behavior. Share command items and activation functions rather than duplicating mutation logic.

The grip popup uses a Text/H1-H6 button row (wrapping if needed), a second compact group for bulleted/numbered/task lists, quote, code, divider and table, then separated Copy/Cut/Duplicate/Move/Delete/Add Below actions. Buttons use existing embedded vector icons where available, active-theme colors, localized tooltips and a current-type indicator. Disabled transforms stay visibly unavailable when exact source ownership cannot be proven. Keyboard navigation follows a stable logical item order; Enter invokes the selected command and Escape dismisses it. The existing viewport-constrained overlay hosts the popup above following rows/media.

Copy and Cut target the clicked complete source unit, publish portable visible text plus HTML using the structured-copy path, and preserve authored Markdown through existing explicit Markdown-copy behavior. Cut then removes exactly that unit with one checked edit and one undo entry; failed or stale operations leave source unchanged. Add Below inserts one editable blank paragraph after that unit (including a list subtree or quote group), preserving LF/CRLF and following block markers, and moves the caret there. Reuse source-unit boundaries and the canonical transaction path rather than synthesizing offsets from displayed text.

### 4. Add one persistent, application-wide preference

Use `block_handle_menu: bool`, default `true`, in `AppPreferences`, the TOML adapter, application initialization/save/reset and a localized General-tab control. Missing or invalid values fall back to `true` without discarding unrelated valid preferences. Turning it off closes a grip-invoked popup immediately, retains the structured selection, and leaves explicit context-menu actions usable. No native menu or shortcut is needed for this preference.

### 5. Preserve cache and version boundaries

```text
grip gesture --> validate target --> select canonical unit
                                  --> optional menu state --> root overlay
menu command --> validate current target --> checked source transaction
                                          --> one version / undo / cache refresh
preference toggle --> persist UI setting --> notify / dismiss grip popup
selection render --> cached VisualBlock + corrected projection activation
```

Selection, popup lifecycle and preference toggles do not mutate document text or version, invalidate shared per-version Markdown derivations, discard highlight memoization, or recreate cached source text handles. Actual edit commands continue through the existing versioned mutation pipeline. Keep all GPUI types in the root crate.

## Risks / Trade-offs

- [A global endpoint change could hide legitimate syntax during editing] -> Cover collapsed carets, non-empty/reversed selections, IME, inline atoms and empty structural rows separately; make the narrowest rule consistent with half-open source ownership.
- [Opening a popup interferes with block dragging or range extension] -> Defer to completed click and test movement past the drag threshold, Shift-click, cancellation and multi-block drag.
- [Popup commands act on a previous selection or stale row] -> Keep a versioned identity/range target and resolve complete units immediately before each operation.
- [A compact palette overflows near edges] -> Reuse overlay clamping and scrolling; test all commands and tooltips in narrow/short windows and light/dark themes.
- [Concurrent slash-palette changes share files and icons] -> Inspect the current diff before edits and reuse its command icons/tooltips where compatible; do not revert or silently include unrelated work.
- [Reference screenshot includes commands without an established Markdown representation] -> Limit this change to supported block structures and source-safe editing actions; arbitrary color and alignment require a separate design.

## Migration Plan

No document migration is required. Existing configuration without `block_handle_menu` enables the new click popup. Users can disable it in General preferences to retain selection-only clicks. Reset restores enabled. Older builds ignore the extra TOML key; documents remain ordinary Markdown. Validate the completed change before any later archive; stable specs are unchanged during planning and implementation.

## Implementation and verification notes

- Non-empty selection endpoints are gated by actual range intersection; the excluded end is not treated as an insertion caret. Fully enclosed inline groups retain their existing styled projection; structural prefixes that actually intersect the selection remain source-backed and editable.
- Completed-click dispatch clears its pending click when a drag begins, so no additional persistent gesture state is required. Grip events validate a captured target rather than indexing a possibly stale virtual row.
- The compact popup shares menu command state and a logical keyboard order. A menu-owned scroll handle maps logical commands to their rendered rows (including compact transform groups and separators), keeping keyboard selection visible without moving the document. A generic window autoscroll request did not scroll ordinary menu divs and was replaced after the narrow-window regression exposed it.
- Native Windows verification uses isolated APPDATA/LOCALAPPDATA and disposable fixtures. The available native APIs do not support a held modifier during a click; Shift-click uses GPUI mouse-event regression coverage, while ordinary click, preference persistence, explicit right-click, drag, Cut/undo and Add Below have OS-input checks. This changes the evidence method, not the specified Shift-click behavior.
- A pre-existing sidebar layout test used a hard-coded Unix path separator in a file-row selector. The fixture selector now derives from its PathBuf so the full root suite can run on Windows.

## Follow-up: Preserve block boundaries during reordering

The user requested fixing the terminal-separator issue recorded by the first native verification. `move_source_units` currently inserts two newlines only when its left piece does not end in LF. A single terminal LF/CRLF therefore joins two formerly independent paragraphs into one Markdown paragraph. Both movement directions and a contiguous block selection reach this helper.

Pass the document's existing shared VisualBlock slice into the move helper. At the new join, require a blank line between independent source units, adding only missing line endings and preserving existing whitespace and LF/CRLF. Preserve the one-line separation of sibling list items belonging to the same original list container (same level, ordered/bullet delimiter, no intervening parent or non-list block). Resolve a list tail through its complete subtree so nested code and child items do not lose their parent identity. Independent quote groups and boundaries between containers and unrelated prose receive blank-line separation. Do not infer list ownership from a final text line or reparse the document to plan the move.

The returned replacement still covers only the old/new interval, and the caret includes any separator inserted before the moved range. Canonical edit/undo validation, moved bytes and per-version caches remain unchanged. Library tests inspect reparsed paragraph/container structure as well as exact source; desktop tests cover drag, structured movement and exact one-step undo. A Windows OS drag/save/undo check will close the prior recorded follow-up.
