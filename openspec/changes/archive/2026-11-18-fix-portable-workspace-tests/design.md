## Context

GPUI resolves the primary shortcut modifier per platform. Its simulated input tests run with an application keymap and must exercise the same dispatch path as the editor. AI workspace grants intentionally reject paths with symlink ancestors; temporary fixtures must pass canonical workspace paths.

PDF shaping resolves a fontdb face and produces glyph IDs specific to that face. The PDF backend has stricter font requirements than the shaper. Swapping only the font at emission would make those glyph IDs invalid.

## Goals / Non-Goals

- Exercise platform-correct shortcuts and retain assertions on document isolation.
- Handle rejected PDF faces by shaping with a compatible fallback.
- Leave production authorization, shortcut defaults, and Markdown caching unchanged.

## Decisions

- Simulate registered shortcuts in GPUI tests, and keep printable input and editor-command checks separate.
- Canonicalize test-owned fixture paths at the filesystem authorization boundary. Keep rejection tests using the deliberately invalid paths.
- Keep a PDF-only shaping cache based on the same discovered fonts as raster export. When the PDF backend rejects a selected face, remove it from the PDF database and restart layout with fresh shaping state. A retry removes one face, so it cannot loop on a rejected font. Keep compatible system faces and the bundled fallback available.

## Risks / Trade-offs

- Installed fonts vary by host. Add a synthetic rejected face and exercise the real PDF export path as well.
