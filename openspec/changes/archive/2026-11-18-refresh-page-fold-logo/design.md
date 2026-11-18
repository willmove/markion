## Context

The approved A concept is a raster reference. README graphics use the canonical SVG, while `openspec/scripts/generate-icons.mjs` rasterizes that same SVG for packaging and the Windows executable resource.

## Goals / Non-Goals

Trace the approved silhouette closely enough to preserve the curved page edges, clipped corner, and open interior. Keep the artwork editable and retain the existing export sizes.

## Decisions

Use one rounded rectangle and one filled Bézier path on the existing square viewBox. Sample solid teal and off-white fills from the reference and align the outer tile to the center of the canvas. An embedded bitmap would carry the reference's pixel artifacts into the master. Manual curves keep both colors and every contour editable.

Generate the PNG, ICO, and ICNS together with the existing script. That keeps framing, transparency, and platform size families consistent across consumers.

## Risks / Trade-offs

- The clipped corner and page curves become subtler at 16 px. Inspect the exported ICO at 16, 24, and 32 px and confirm the two stems and center of the M stay distinct.
- Production consumers share four tracked files. Regenerate and verify them as one set so packaging cannot pick up a mismatched export.

## Migration Plan

Replace the four files at their existing paths. A rollback restores the SVG and all three exports together.
