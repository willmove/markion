## Why

The defect-audit branch fixes several real crash and performance problems, but review found that its bounded data-URI cache key can deterministically alias distinct images and that its YAML/front-matter hardening still emits invalid or lossy YAML for common container and multiline values. These merge blockers need a coherent correctness contract and regression evidence before the branch can safely reach `main`.

## What Changes

- Replace sampled large-data-URI cache identity with a collision-resistant identity derived from the complete URI once per document version, while keeping repaint work bounded and preserving the pending-payload lifetime and raster-memory limits.
- Make Markdown front-matter rendering structurally valid for scalar, sequence, and mapping values of every size, including single-element containers, and preserve parsed values across render/parse round trips.
- Make DOCX/PDF pandoc front-matter title overrides use the same YAML-safe scalar serialization, including newlines, carriage returns, document-marker text, control characters, quotes, and backslashes.
- Complete end-to-end trailing subscript recognition in the normal parser configuration without reclassifying GFM strikethrough.
- Retain the UTF-8 caret/selection clamps as defense in depth, but describe and test them as malformed/stale-range containment rather than claiming an unproven ordinary Callout crash path.
- Add deterministic regression tests for image identity, YAML round trips, export title overrides, extended-inline event boundaries, and safe invalid-range handling.

Non-goals: redesigning the image cache capacity policy, changing Visual Edit source-elision UX, replacing `pulldown-cmark`, implementing a general YAML editor, or addressing the separate full-document Visual Edit parsing and synchronous-save performance backlog.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Require distinct complete image sources to retain distinct render-cache identity with bounded repaint work, require trailing subscript syntax to survive parser event boundaries, and define invalid UTF-8 selection containment as defensive behavior.
- `export`: Require rendered front matter and pandoc title overrides to remain valid YAML and preserve scalar/container values across round trips.
- `engineering-quality`: Require deterministic collision fixtures and semantic parse/round-trip assertions for cache identities and generated YAML rather than string-shape assertions alone.

## Impact

- Root application: `src/app/preview_image.rs`, image derivation/model fields, cache accounting, and related GPUI/pure tests.
- Markdown crate: `crates/markdown/src/renderer.rs`, parser event handling around extended inline syntax, and round-trip tests.
- Export crate: shared YAML scalar/front-matter construction used by DOCX and PDF pandoc inputs, plus export tests.
- OpenSpec: coordinates with `fix-visual-data-uri-source-toggle-freeze`; it preserves that change's bounded per-frame work requirement while superseding its implementation-level assumption that `PreviewImageKey` remains unchanged.
- Architecture invariants: derived identity remains cached per document version; repaint paths do not hash or clone multi-megabyte authored payloads; workspace member crates remain GPUI-free.
