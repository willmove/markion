## Context

See `proposal.md` for motivation. GPUI asks Markion's source `EntityInputHandler` for a screen-space rectangle and forwards it to the Wayland text-input protocol. Markion currently forms that rectangle directly from two layout points. The Linux path requests a collapsed range at the marked-text start, so both points are identical and the resulting rectangle has zero width and height. Some Wayland input methods treat that as an unspecified cursor rectangle and fall back to or infer placement, while Windows uses a separate native path.

Source layout remains versioned by the existing `SourceLayoutKey`; typewriter mode changes presentation-only spacer and scroll geometry without mutating the document or its derived caches.

Data flow:

`Wayland marked range start -> GPUI bounds_for_range -> UTF-16 to source range -> current wrapped-line points -> normalized caret rectangle -> Wayland cursor rectangle`

## Goals / Non-Goals

**Goals:**

- Return valid source-editor IME geometry with positive caret dimensions.
- Preserve the composition-start anchor across preedit growth and typewriter scrolling.
- Exercise the actual `EntityInputHandler` seam with ordinary and typewriter source layouts.

**Non-Goals:**

- Change IME mutation, marked-range, commit, or undo behavior.
- Add platform detection or a Linux-only application branch.
- Recompute Markdown-derived state or source layouts solely for candidate placement.

## Decisions

1. Normalize source range geometry at Markion's input-handler boundary. A collapsed or cross-row request becomes a caret-sized rectangle at the requested range start; a same-row non-empty request retains its measured width while gaining the current line height. This directly satisfies the platform contract and avoids a Wayland-specific workaround. The alternative of patching the compositor protocol path would leave Markion returning invalid geometry to other platform clients.
2. Use the requested range start as the anchor. GPUI's Wayland backend deliberately asks for the marked start during composition, which is invariant across replacement strings; using the selected caret end would reproduce horizontal drift as preedit length changes.
3. Reuse the current wrapped-line snapshot and element bounds. Candidate queries stay O(current line lookup) and do not parse Markdown, allocate a new document-wide text copy, or invalidate per-version caches.
4. Add a small pure geometry test plus a GPUI source-editor composition test. The pure test locks down non-empty caret dimensions; the GPUI test covers UTF-16 conversion, marked-range replacement, typewriter scroll translation, and stable screen-space origin.

## Risks / Trade-offs

- [Risk] A non-empty multi-row range cannot be represented as one exact rectangle. -> Return the first caret rectangle, matching the `firstRect` semantics and the candidate-placement use case.
- [Risk] Geometry queried before a fresh layout exists can still be unavailable. -> Preserve the existing `None` behavior until a current wrapped layout has been painted; do not fabricate window-origin coordinates.
- [Risk] Source scroll changes legitimately move the anchor. -> Assert stability only after each requested typewriter recenter/render cycle; movement caused by the edited row itself moving remains allowed by the spec.

## Migration Plan

No data migration is required. The change is a local geometry correction and can be rolled back by reverting the helper and its call site.
