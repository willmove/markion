## ADDED Requirements

### Requirement: Image render cache identity is content-exact and repaint-bounded
Every image source presented by preview or Visual Edit SHALL carry a cache identity that distinguishes different normalized local or remote locations and different complete data-URI bytes. For a data URI, identity derivation SHALL consume the complete URI during document-version derivation and SHALL NOT sample only selected regions. Repaint, cache lookup, claim reconciliation, and source-toggle presentation MUST reuse a constant-size identity without rescanning or cloning the complete data URI. Identity derivation SHALL remain part of the existing per-document-version derived-state lifecycle.

#### Scenario: Large data URIs differ outside previous samples
- **WHEN** two valid data-URI images have the same byte length and identical head, middle, and tail regions but differ elsewhere
- **THEN** preview and Visual Edit assign different render-cache identities
- **AND** each surface displays the raster produced from its own complete source

#### Scenario: Editing an unsampled payload byte invalidates the raster
- **WHEN** an edit changes bytes of a large data URI without changing its length or the former sampled regions
- **THEN** the next document version carries a different image identity
- **AND** the old ready or failed cache result is not reused for the edited image

#### Scenario: Repainting does not scan the payload
- **WHEN** unrelated interaction state causes repeated repaints of a document containing a multi-megabyte data-URI image
- **THEN** deterministic counters show no complete-payload hash or clone in repaint, claim reconciliation, or cache lookup
- **AND** the precomputed identity is reused from version-cached derived state

### Requirement: Extended inline candidates preserve source provenance across parser events
Markion SHALL recognize supported `~subscript~` syntax even when `pulldown-cmark` divides one authored candidate across adjacent text events, including when the closing delimiter is at the end of a paragraph. Reconstructing an extended-inline candidate MUST use original source provenance, MUST NOT cross a non-text semantic event, and MUST preserve escaped single-tilde text and GFM `~~strikethrough~~` semantics.

#### Scenario: Trailing subscript uses the default parser
- **WHEN** the default parser reads `H~2~` at the end of a paragraph with strikethrough enabled
- **THEN** the document AST contains text `H` followed by a subscript containing `2`
- **AND** preview and export consumers receive the same subscript structure

#### Scenario: Strikethrough remains owned by GFM parsing
- **WHEN** the default parser reads `~~strike~~` or text containing a closing double-tilde delimiter
- **THEN** the input retains its existing GFM strikethrough interpretation
- **AND** no part of the double-tilde construct is emitted as a subscript

#### Scenario: Escaped tilde remains literal
- **WHEN** an authored backslash escapes a single tilde that would otherwise resemble a subscript delimiter
- **THEN** the parser does not reconstruct that escaped marker into a subscript
- **AND** the resulting visible text matches the existing escape semantics

### Requirement: Invalid text offsets are contained at UTF-8 boundaries
Editor operations that consume stale, out-of-range, reversed, or mid-codepoint caret and selection offsets SHALL clamp or collapse them to valid UTF-8 boundaries before slicing canonical text. This behavior is defensive invariant containment and SHALL NOT imply support for otherwise invalid or unparsed Markdown constructs.

#### Scenario: Invalid selection reaches a text consumer
- **WHEN** copy, cut, find-prefill, link editing, or navigation receives a selection containing an invalid UTF-8 boundary
- **THEN** the operation does not panic or slice at an invalid boundary
- **AND** an unprovable selection collapses without mutating document text

