## Context

Visual Edit already derives its blocks from parsed preview blocks and retains exact source ranges for editing. Preview and Read mode use the parsed fence body for rendering. Visual Edit currently reconstructs that body separately when scheduling work and when looking up the result.

## Decisions

Keep the parsed body on Visual Edit code blocks for registered diagram languages. The preview and visual models then supply the same body to the cache key, which also includes the backend and theme. Scheduling still sends misses to the background renderer. The diagram payload editor continues to use exact source ranges. This handles Markdown container indentation and CRLF using parser output and keeps normalization within the existing per-version derivation.

Draw both task markers using the same square with a side equal to the rendered font size. Center the square on the first list line and use the existing check icon for the filled state. Keep the existing pointer handler and structural-prefix reveal path.

## Risks / Trade-offs

The visual model retains a parsed string for each diagram fence. Limit that storage to registered diagram languages and include it in the existing memory accounting.
