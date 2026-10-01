## Context

`visual_block_editor` gave a `CodeBlock` a `Code` editor only when `fenced_payload_ranges` found both an opening backtick or tilde fence and a valid closing line. Otherwise the block fell back to a `Code` source island. pulldown-cmark reports an indented block's range starting after the first line's indentation, so later body lines carry their indentation inside the range (for example `code1\n\n    code2\n`). Unclosed fences run to the end of the container.

## Goals / Non-Goals

**Goals:** close both roadmap gaps using the existing `Code` editor, field-edit path, and projection machinery, with no parallel model.

**Non-Goals:** see proposal.

## Decisions

### 1. Record the parser's block kind
`PreviewBlock::CodeBlock` gains `fenced: bool`, taken from `CodeBlockKind` in the same parse pass. Detecting the kind from text is ambiguous: an indented block whose first line is ```` ``` ```` looks like a fence, and inside list items the container indentation cannot be told apart from code indentation without the parser.

### 2. Indented code: a distinct field kind
`VisualEditorFieldKind::IndentedCodePayload` covers the whole block range. The `Code` editor gets empty opening, closing, and info ranges at the block edges, and `info_range: None`. The field kind carries all behavior differences, so exhaustive matches surface every site:
- Projection: `indented_code_display` keeps the first line whole and drops one tab or up to four leading spaces from each later line. Dropped bytes belong to no segment, so they are hidden and the caret skips them.
- Highlighting and copy use the same dedented text, keeping highlight ranges aligned with the projection.
- Replacement sanitizing turns every `\n` (CRLF and CR normalized) into `\n` + indentation unit (tab if any body line starts with a tab, else four spaces). Enter takes the ordinary payload path (`\n`) and gets indentation from the same sanitizer, so Enter and paste cannot diverge.
- Auto-pair is disabled, like fenced payloads. The language chip is never shown. `block_body` dedents before block transforms.

*Alternative:* store dedented text and rewrite the whole block on each edit. Rejected: it breaks the one-exact-range-per-edit invariant and per-keystroke cost.

### 3. Unclosed fences: payload to the block end
`fenced_payload_ranges` gains `allow_unclosed`. Only the code-block path sets it. When no closing line is found, the payload runs from after the opening line to the block end, and the closing fence is an empty range there. Fenced math keeps the strict behavior, since its delimiter handling differs. Malformed closing lines (extra text after the fence) are not closing lines in CommonMark, so they are payload.

### 4. Caches
The `fenced` flag lives in the per-version preview-block cache, and visual blocks derive from it as before. Dedenting and sanitizing run on demand for the focused field or the rendered block. Nothing recomputes per keystroke beyond the existing single-block path.

## Risks / Trade-offs

- [Mixed tab and space indentation inside one block] → The display drops one tab or up to four spaces per line, which is exactly CommonMark's code indentation. Insertion uses tab if any line uses a tab. Mixed blocks stay byte-exact, and only new lines follow the chosen unit.
- [Unclosed fence in a quote or list shows container prefixes in its payload] → Unchanged from closed fences in the same containers. This is a non-goal here.
