## Why

The `WYSIWYG coverage roadmap` requirement lists indented code blocks as the primary open gap and unclosed or malformed fenced code as a secondary gap. Both still render in Visual Edit as raw-source islands: the fence scanner needs a backtick or tilde fence and a valid closing line. Editing such a block means editing undecorated source, and an unclosed fence that the user is still typing jumps between an island and a code block as soon as the closing fence appears. This change closes both gaps, as the roadmap requires.

## What Changes

- Indented code blocks render as highlighted code blocks with a payload editor. Each line's four-column indentation stays in the source but is hidden in the display. Enter and multi-line paste re-add the block's indentation unit (tab or four spaces), so new lines stay inside the block. Indented code has no editable language label, and transforming it into another block type drops the indentation.
- Unclosed or malformed fenced code blocks keep their payload editor and language label. The payload runs to the end of the block.
- The parser's code-block kind (fenced or indented) is recorded on the preview block, so an indented block whose first line looks like a fence is never mistaken for an unclosed fence.
- The coverage matrix and roadmap move both constructs out of the gap class. Multiline or otherwise unprovable images become the primary open gap.

Non-goals: converting indented code to fenced code automatically; hiding the container prefixes (`> `, list indentation) inside fenced payloads in quotes or lists, which is unchanged; any Split Preview or Read change (both already render these blocks as code).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: adds "Visual Edit SHALL edit indented and unclosed code blocks as code", and updates "Maintained Visual Edit support classification" and "WYSIWYG coverage roadmap" to move indented code and unclosed or malformed fences out of the gap class.

## Impact

- Code: `src/model.rs` (`PreviewBlock::CodeBlock.fenced`, `VisualEditorFieldKind::IndentedCodePayload`), `src/lib.rs` / `src/parse.rs` (record the block kind; sanitize indented-code replacements), `src/visual.rs` (indented-code editor, unclosed-fence payload, dedent helpers), `src/block_edit.rs` (dedented transform body), `src/auto_pair.rs`, `src/app/preview.rs` (dedented projection and highlighting, no language chip), `src/app/editing.rs` (Enter), `docs/visual-editing-quality.md` (matrix and roadmap), and tests.
- Invariants: every edit remains one source mutation through the existing field-edit, history, and dirty path. The fenced/indented flag is computed in the same parse pass and cached with the preview blocks. Display dedenting happens at render time from the cached field range.
