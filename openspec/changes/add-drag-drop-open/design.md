## Why a design note

This change is small (one handler, two hook sites), but it touches the render tree and reuses GPUI's external-drag machinery, which behaves differently from the internal `DraggedEditorSplitHandle` drags already in the file. Recording the data flow once prevents confusion during implementation and review.

## How GPUI turns an OS file drag into a drop event

Markion does not run its own drop detection. The platform layer does it:

```
   OS file manager            GPUI platform layer                     App
   ────────────────           ──────────────────────             ──────────────

1. user drags .md ───────▶ WindowsDragDropHandler::DragEnter
   over the window            (IDropTarget_Impl)                active_drag = Some(AnyDrag {
                                emits FileDropEvent::              value: Arc<ExternalPaths>,
                                  Entered { paths }  ─────────▶    view: ExternalPaths view,
                                                                    cursor_offset, ... })
                              renders OS file icons for the
                              dragged files (ExternalPaths:
                              Render => Empty; platform draws them)

2. mouse moves       ───────▶ FileDropEvent::Pending  ─────────▶  MouseMove (no app code)

3. user releases     ───────▶ FileDropEvent::Submit    ─────────▶ MouseUp(left button) on the
                              (cx.activate(true))               element under cursor
                                                                 → dispatches to that element's
                                                                   on_drop listeners whose TypeId
                                                                     matches the active_drag value's TypeId
                                                                   = on_drop::<ExternalPaths>

4. user drags out    ───────▶ FileDropEvent::Exited    ─────────▶ active_drag.take()  (cleared)
   without releasing
```

Source: `gpui-0.2.2/src/window.rs:~3620` (`dispatch_event` `PlatformInput::FileDrop` arm) and `gpui-0.2.2/src/platform/windows/window.rs` (`WindowsDragDropHandler_Impl`, `IDropTarget`).

Key consequence: **Markion never sees `FileDropEvent` directly.** It only needs to register `on_drop::<ExternalPaths>` on the elements that should accept drops. The type parameter `ExternalPaths` is what makes GPUI route the drop to Markion's handler (the listener's `TypeId` must equal `TypeId::of::<ExternalPaths>()` of the active_drag value). The handler receives `&ExternalPaths` and reads `.paths()`. This is exactly how the existing `on_drop::<DraggedEditorSplitHandle>` works - same machinery, different key type.

## The two drop sites

```
main-content-row (src/main.rs:5080)                ← NOT a drop target (keeps the sidebar out of scope)
├── sidebar_view(...)                              ← NOT a drop target (file tree stays click-to-open)
├── [editor/preview divider]                        ← existing DraggedEditorSplitHandle drag (unchanged)
├── editor pane div (src/main.rs:5103)             ← NEW: on_drop::<ExternalPaths>
│     hidden in ViewMode::Read
│   └── editor-scroll (src/main.rs:5123)            (child; inherits drop from parent)
└── preview pane div (src/main.rs:5170)            ← NEW: on_drop::<ExternalPaths>
    └── list(...) (src/main.rs:5189)                 (child; inherits drop from parent)
```

Why two sites, not one on `main-content-row`:

- **Split mode:** both panes are visible. The user should be able to drop on whichever side the cursor is over. A single row-level handler would also fire for drops on the sidebar/file-tree gap, which the user did not ask for and which muddies the "drop onto the editor" mental model.
- **Edit mode:** editor pane visible, preview hidden. The editor-site handler covers it.
- **Read mode:** editor pane hidden (`display: none` via `.hidden()`), preview visible. The preview-site handler covers it.

Both sites call the **same** handler (`handle_external_drop`), so behaviour is identical regardless of which pane received the drop.

## Handler data flow

```
ExternalPaths { paths: [p1.md, p2.png, p3.markdown, dir/] }
        │
        ▼   for each path:
   ┌────────────────────────────────────────────┐
   │ is_markdown_path(p)?  AND  p.is_file()?    │
   └────────────────────────────────────────────┘
        │ no  ────────────────────▶ skip (silent)
        │ yes
        ▼
   open_file_in_new_tab_from_path(p, cx)   ◀── existing (src/main.rs:2473)
        │
        ├── MarkdownDocument::open(p)      (fs::read_to_string, sets path, version)
        ├── self.open_in_new_tab(document) (pushes EditorTab, sets active_tab, refresh_search_matches)
        ├── self.update_workspace_root_from_document(cx)
        └── self.status = tf(Msg::StatusOpened, &[p])   ◀── reused; last call wins
```

No per-document caches are invalidated by this handler beyond what `open_in_new_tab` already does: a freshly-constructed `MarkdownDocument` starts with empty `cached_preview_blocks` / `cached_outline` / `cached_stats` (they are `RefCell::new(None)` in `with_state`), so the first render after the drop computes them lazily as normal. The `version()` is a fresh monotonic counter, so it cannot collide with a stale cache entry from a closed document. **The cached-per-version invariants are preserved unchanged.**

## Why open-as-new-tab, not replace-active

`open_in_new_tab_from_path` is chosen over `replace_active_tab` for the same reason the file tree uses it (`open_tree_file`, `src/main.rs:1368`): replacing the active tab risks discarding unsaved edits in the current document, which would need a dirty-guard prompt. Opening a new tab never loses data, requires no prompt, and matches the file-tree open flow the user already knows.

## Why no new i18n string

The user decided to reuse `StatusOpened` (`"Opened {0}"` / `"已打开 {0}"`). The handler calls `open_file_in_new_tab_from_path` per file, and each call sets `self.status` itself - so after the loop, `status` already holds `tf(Msg::StatusOpened, &[last_opened_path])`. The "0 files opened" case is left as a no-op on the status bar rather than a new "nothing to open" string, per the user's decision. This keeps `ui-i18n` untouched.

## Risk: multi-file drop is one event, not many

GPUI fires `on_drop::<ExternalPaths>` **once** with all dropped paths bundled in a single `ExternalPaths` value - it is not one event per file. The handler must loop over `paths()`, not take only the first. Tasks 1.3 and 4.2 enforce this.

## Risk: case sensitivity of extensions

Windows file systems are case-insensitive, so `NOTE.MD` is a real Markdown file. `is_markdown_path` must compare lower-cased extensions (`md` / `markdown` / `mdown`). The file tree's existing check (`src/storage/file_tree.rs:~287`) should be checked for whether it already lower-cases; if it does, the shared helper from task 1.2 inherits the same behaviour.
