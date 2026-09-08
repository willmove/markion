## Context

The supplied recording shows a small editor window, a short document, Chinese input, and repeated line insertion. Existing tests mostly inspect centering only after a bounded refinement loop. Investigation must reproduce intermediate displacement through the real editing and layout path before selecting a fix.

Data flow: input / IME / newline -> document version -> cached layout and derived blocks -> viewport measurement -> caret placement -> typewriter recenter -> next frame. Scroll-only updates must not invalidate the document version, derived Arcs, highlighting, or text handle.

## Goals / Non-Goals

Goals: stable small-window typing, consistent blank-line ownership after every terminal Enter, current caret geometry, bounded convergence, and preserved manual scrolling.

Non-goals: scroll animation, new settings, changes to canonical Markdown bytes, or unrelated work in the dirty checkout.

## Decisions

- Start with short-document GPUI tests covering individual input and newline frames. Include composition and wrapped content if needed to isolate the recorded trigger.
- Compare intermediate caret and content positions, not just settled positions. A delayed return to center can still be a visible regression.
- Keep the fix at the existing scrolling/layout boundary identified by the failing test. Do not mask competing scroll updates with a larger tolerance or additional retry frames.
- Preserve source, Visual Edit, and Split behavior and current per-version caches.

### Confirmed cause and correction

The first minimized failing sequence is an empty Visual Edit document at 640 x 400, committed Chinese text, Enter, then another committed word. A single App update containing explicit `Window::draw` calls exposes the first painted frame. The old path paints the newly measured caret 24 logical pixels below center and corrects it on the following render. The list offset remains 12 pixels during the failing transition, ruling out splice reset as the cause of that case. The first text in an empty document also initially appears near the viewport top because the list has no previous viewport from which to size its leading space.

The follow-up report isolates a second defect. For `Body\n`, pulldown-cmark includes the newline in the paragraph source range, but the paragraph has no inline run for it. Visual Edit therefore hides that byte as a marker and maps the source caret back to the end of `Body`. For `Body\n\n`, the second newline is a coverage gap and becomes a `Whitespace` row. Repeated Enter consequently alternates between two different row structures. The failing regression observes this directly on press one: the only block is a paragraph at `0..5`, with `4..5` hidden as a marker.

Terminal unquoted paragraph and heading ranges now release their final LF or CRLF when the remaining source is whitespace through document end. The existing coverage pass collects that line ending together with any later terminal whitespace into one `Whitespace` block. `Body\n`, `Body\n\n`, and later Enter states now share the same paragraph range and grow the same source-backed tail row. Canonical text and parser output remain unchanged; only the derived Visual Edit ownership changes.

`VisualListElement` delegates to the existing GPUI List and publishes current viewport bounds and boundary space before List measures children. `VisualEditableText` resolves identical caret geometry in prepaint and paint. During an active current typewriter request, prepaint requests a viewport-height rectangle centered on the measured caret. GPUI List's existing bounded autoscroll retry places that row before painting the frame. The viewport is published separately to avoid borrowing ListState while GPUI holds its mutable borrow, and to avoid using an inner code/table clip as the document viewport.

No extra document version, parse, cache invalidation, animation, or unbounded retry is introduced. Existing bounded reconciliation still expires the request so manual scrolling remains independent of idle redraws. Tail ownership is derived once per document version alongside the existing visual-block cache.

## Risks / Trade-offs

- Headless GPUI uses different font metrics from the recorded Linux desktop -> assert relative geometry and exercise several small sizes.
- Independent uncommitted changes share app files -> make narrow edits and preserve those changes.
- Terminal block partitioning affects visual row count -> retain contiguous byte coverage and verify paragraph projection, CRLF, block identity, and existing Visual Edit behavior.

## Migration Plan

No persisted migration. Revert the scoped scroll fix and its tests to roll back.

## Open Questions

Native Linux replay of the supplied recording remains a manual visual check; automated reproduction uses the real GPUI layout/paint path with the test platform on Windows.
