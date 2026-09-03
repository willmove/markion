## MODIFIED Requirements

### Requirement: On-demand Markdown image source editing in Visual Edit
Visual Edit SHALL present a block-level Markdown image whose complete authored span can be proven exactly with the same on-demand source affordance used by block math, diagrams, and raw-HTML blocks: collapsed by default (rendered image, read-only caption, and the existing caret-owner presentation controls), with a hover-visible source toggle that expands one monospaced payload editor covering the complete authored span — `![alt](destination "title {width=… align=…}")` — as a single field rendered above the image so the authored span sits directly under the toggle the user clicked. Payload edits SHALL use one exact canonical source replacement through the existing source-selection, platform input, IME, history, dirty-state, and multi-tab paths. While the destination cannot be loaded or decoded, the payload editor SHALL remain visible regardless of the toggle state, mirroring the forced-expand rule for invalid math and failed diagrams. Expanding and collapsing SHALL be presentation-only: they MUST NOT mutate document text, dirty state, undo history, document version, or derived Markdown caches.

When the destination is a data URI, or the complete authored span exceeds an elision threshold (64 KiB), the expanded payload editor SHALL elide the opaque payload instead of rendering it verbatim: the editable structural parts (label, `![`, `](`, the `data:` scheme, media type and `;base64,` marker, closing `)` and any title or presentation suffix — or a bounded verbatim head of a non-data-URI destination) SHALL remain visible and editable, while the elided bytes SHALL present as exactly one atomic summary token showing a human-readable size label framed by ellipsis marks (e.g. `…4.2 MB…`) and rendered with visually distinct styling (background tint and dimmed text) so it can never be mistaken for authored bytes. The token SHALL behave as one atomic unit for editing: the caret snaps to the token's boundaries and SHALL NOT rest inside it, double-clicking selects the whole token, and any edit whose selection intersects the token replaces the entire elided byte range through one exact canonical source replacement (single Undo restores it). Expanding an elided source SHALL stay responsive. The raw elided bytes SHALL remain fully visible and editable in the source-text editing modes (Edit and Split source pane), which are unaffected by elision.

Reference-style images, multiline or malformed image syntax, and spans whose exact boundaries cannot be proven MUST keep today's presentation without a source toggle, and no image payload range MAY be guessed. Images inline with prose keep their existing caret-proximity reveal behavior and are not affected by this requirement.

#### Scenario: Toggle expands the complete image syntax as one payload

- **WHEN** Visual Edit shows a block-level `![alt](url "Cap {width=50 align=right}")` whose span is below the elision threshold and the user activates the hover source toggle
- **THEN** a monospaced payload editor shows the exact authored span as one editable field above the image
- **AND** a payload edit applies as one exact canonical source replacement, after which the image, caption, width, and alignment re-derive from the re-parsed source

#### Scenario: Data-URI payload expands to a visually distinct summary token

- **WHEN** Visual Edit shows a block-level `![icon](data:image/png;base64,AAAA…)` and the user activates the hover source toggle
- **THEN** the payload editor shows the label and the `data:image/png;base64,` prefix verbatim, followed by one summary token framed by ellipsis marks with a size label, instead of the base64 bytes
- **AND** the token is rendered with distinct styling (background tint and dimmed text) so it is clearly not authored source text
- **AND** the expansion completes without freezing the UI, whatever the payload size

#### Scenario: Token edits replace the whole payload atomically

- **WHEN** the token is selected (e.g. by double-click) and the user types or pastes a replacement destination
- **THEN** one exact canonical source replacement swaps the entire elided byte range for the typed text
- **AND** a single Undo restores the previous bytes, and the caret never rests at a position inside the elided range

#### Scenario: Oversized non-data-URI spans elide the same way

- **WHEN** a proven image span exceeds the 64 KiB threshold with a non-data-URI destination
- **THEN** the expanded payload editor keeps a bounded verbatim head of the destination and elides the remainder behind the same atomic summary token
- **AND** edits of the visible head apply as exact replacements that do not disturb the elided bytes

#### Scenario: Collapse follows the shared affordance rules

- **WHEN** the image payload is expanded and the primary click lands outside the block
- **THEN** the payload collapses back to the rendered image
- **AND** while the caret remains inside the payload the block stays expanded, and clicking the image presentation itself never expands the source

#### Scenario: Unloadable image forces the source editor visible

- **WHEN** the destination of an exactly proven image cannot be loaded or decoded
- **THEN** the payload editor remains visible regardless of the toggle state so the destination can be corrected
- **AND** a data-URI destination presents its elided token form while forced visible
- **AND** the load failure does not mutate source, history, or document version

#### Scenario: Ambiguous image spans get no toggle

- **WHEN** an image uses reference syntax, malformed delimiters, or another form whose exact span cannot be proven
- **THEN** Visual Edit keeps today's presentation for that construct without a source toggle
- **AND** no payload range is guessed

#### Scenario: Toggling is presentation-only

- **WHEN** the user expands or collapses an image's source payload
- **THEN** document text, document version, dirty state, and undo history are unchanged
- **AND** derived Markdown caches and image render caches are not invalidated by the toggle itself

#### Scenario: Raw elided bytes stay editable in source modes

- **WHEN** the user switches the document to Edit mode or the Split source pane
- **THEN** the complete image span including every elided byte is shown and editable verbatim
- **AND** edits made there re-derive normally in Visual Edit
