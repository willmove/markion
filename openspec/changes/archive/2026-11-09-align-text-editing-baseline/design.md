## Context

Keep text navigation and input rules behind GPUI-free seams. `text_util` already owns string/offset helpers, `auto_pair` runs before the canonical mutation, and `block_edit` owns slash recognition. The app should translate those pure decisions into actions without creating another document model.

## Goals / Non-Goals

**Goals:**

- Match established desktop word/mouse/navigation behavior with exact UTF-8 offsets and one predictable undo unit per command.
- Reuse the existing source-backed indentation, clipboard, and context-menu mutation paths.
- Make input conversion context-sensitive rather than adding more unconditional pairing.
- Keep new fixed navigation bindings distinct from customizable menu-action overrides.

**Non-Goals:**

- No full SelectionState or overlay redesign.
- No asynchronous parsing or new dependency; Unicode segmentation already exists.
- No automatic conversion inside code, math, diagram, front-matter, HTML, or active IME fields.

## Decisions

### 1. Pure word and block navigation

Implement Quiet-compatible word rules using `split_word_bound_indices`: previous word selects the last non-whitespace segment start before the caret; next word selects the first non-whitespace segment end after it. Evaluate one logical line at a time to avoid a whole-document scan and to keep CRLF behavior explicit.

New fixed actions cover word move/delete, line-boundary selection, and page movement. Page movement uses the visible line-height where geometry exists and the existing visual-navigation path in Visual Edit. Select All first selects the current paragraph or VisualBlock's exact editable content and selects the whole document only when that local range was already selected.

### 2. One source-selection helper for mouse and Select All

A single helper resolves the current block's exact text range: editable run bounds for ordinary Visual prose, a dedicated editor payload for complex fields, and `paragraph_range_at` in Source. Double/triple click and progressive Select All use the same result; no structural prefix is included when a narrower payload exists.

### 3. Reuse existing mutation and menu infrastructure

Collapsed-list Tab/Shift+Tab is available only when the active block is a list item and the requested transition is structurally legal. It reuses `indent_lines`/`outdent_lines` and the existing one-snapshot history path. Other collapsed text keeps literal four-space insertion.

The editor context menu stores only pointer position and an immutable selection snapshot. Cut/Copy/Paste/Select All dispatch through existing commands, with disabled states derived from current selection/clipboard availability where practical. Menu opening and dismissal do not mutate the document.

### 4. Context-sensitive conversion before mutation

The auto-pair path recognizes the exact `**` + Space state at a legal block start and replaces it with `* ` in the same canonical edit as the typed Space. This preserves existing pairing for actual emphasis while allowing the common bullet prefix.

A separate `input_rules` module recognizes HTTP(S) and `www.` tokens immediately before typed whitespace and serializes one Markdown link. URL conversion runs only in unrestricted prose, never during IME or in dedicated source fields. It returns the same range/replacement/selection shape as auto-pair so no second mutation path is introduced.

Slash recognition finds `/` or `、` after line start or whitespace and requires a whitespace-free query and a non-URL prefix. The existing command edit replaces the complete marker and query.

### 5. Redo aliases remain one action

Install `secondary-y` and `secondary-shift-z` for the same Redo action and show both in documentation. The existing customizable `redo` action retains its override model; fixed navigation actions are not added to the override registry.

## Risks / Trade-offs

- [Word rules differ across scripts] → Test ASCII, CJK, punctuation, emoji, combining characters, and CRLF rather than assuming whitespace-only words.
- [Triple click or Select All includes hidden syntax] → Use exact editable payload ranges and preserve canonical source selection semantics.
- [List indentation is invalid at a boundary] → Refuse unsupported transitions without changing source and retain literal Tab outside lists.
- [URL or bullet conversion is surprising] → Use narrow recognizers, one undoable mutation, and no conversion in restricted fields.
- [Two Redo bindings conflict with prior single-binding spec] → Modify the requirement and every shortcut surface through this change rather than adding a hidden alias.
- [Ctrl+Delete conflicts with file-tree deletion on Windows/Linux] → Move the file-tree action to `secondary-shift-delete` and update reserved bindings, tests, and documentation; keep word deletion on the conventional Ctrl+Delete binding.

## Migration Plan

1. Add pure navigation/input rules and tests.
2. Add actions and Source/Visual integration, then context menu and structural indentation.
3. Add rendered selection/menu tests and shortcut updates.
4. Run complete verification and document any platform-specific key reservation.

## Open Questions

None.
