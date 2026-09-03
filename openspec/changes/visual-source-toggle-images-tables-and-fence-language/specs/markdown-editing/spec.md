## ADDED Requirements

### Requirement: On-demand Markdown image source editing in Visual Edit
Visual Edit SHALL present a block-level Markdown image whose complete authored span can be proven exactly with the same on-demand source affordance used by block math, diagrams, and raw-HTML blocks: collapsed by default (rendered image, read-only caption, and the existing caret-owner presentation controls), with a hover-visible source toggle that expands one monospaced payload editor covering the complete authored span — `![alt](destination "title {width=… align=…}")` — as a single field. Payload edits SHALL use one exact canonical source replacement through the existing source-selection, platform input, IME, history, dirty-state, and multi-tab paths. While the destination cannot be loaded or decoded, the payload editor SHALL remain visible regardless of the toggle state, mirroring the forced-expand rule for invalid math and failed diagrams. Expanding and collapsing SHALL be presentation-only: they MUST NOT mutate document text, dirty state, undo history, document version, or derived Markdown caches.

Reference-style images, multiline or malformed image syntax, and spans whose exact boundaries cannot be proven MUST keep today's presentation without a source toggle, and no image payload range MAY be guessed. Images inline with prose keep their existing caret-proximity reveal behavior and are not affected by this requirement.

#### Scenario: Toggle expands the complete image syntax as one payload

- **WHEN** Visual Edit shows a block-level `![alt](url "Cap {width=50 align=right}")` and the user activates the hover source toggle
- **THEN** a monospaced payload editor shows the exact authored span as one editable field
- **AND** a payload edit applies as one exact canonical source replacement, after which the image, caption, width, and alignment re-derive from the re-parsed source

#### Scenario: Collapse follows the shared affordance rules

- **WHEN** the image payload is expanded and the primary click lands outside the block
- **THEN** the payload collapses back to the rendered image
- **AND** while the caret remains inside the payload the block stays expanded, and clicking the image presentation itself never expands the source

#### Scenario: Unloadable image forces the source editor visible

- **WHEN** the destination of an exactly proven image cannot be loaded or decoded
- **THEN** the payload editor remains visible regardless of the toggle state so the destination can be corrected
- **AND** the load failure does not mutate source, history, or document version

#### Scenario: Ambiguous image spans get no toggle

- **WHEN** an image uses reference syntax, malformed delimiters, or another form whose exact span cannot be proven
- **THEN** Visual Edit keeps today's presentation for that construct without a source toggle
- **AND** no payload range is guessed

#### Scenario: Toggling is presentation-only

- **WHEN** the user expands or collapses an image's source payload
- **THEN** document text, document version, dirty state, and undo history are unchanged
- **AND** derived Markdown caches and image render caches are not invalidated by the toggle itself
