## ADDED Requirements

### Requirement: Visual Edit payload editors are display-bounded per frame
Every Visual Edit surface that reveals authored source for editing (image source payloads, fenced code payloads, math and diagram payloads, raw-HTML payloads) SHALL keep its per-frame work bounded by the length of the displayed text rather than the length of the authored span: building the display projection, submitting text for layout, and constructing caret-navigation snapshots MUST NOT perform per-character position queries against the text layout, and an elided payload MUST produce a constant-bounded display text. Collapsed source affordances MUST NOT clone the authored span or re-derive span-length-dependent cache keys on frames where the payload is not shown. These bounds SHALL be gated with deterministic counters (query counts, display lengths) in the existing test harness rather than wall-clock thresholds.

#### Scenario: Expanding a multi-megabyte data-URI image source

- **WHEN** a test document contains a block-level image with a multi-megabyte base64 data-URI destination and the source affordance is expanded
- **THEN** deterministic counters show the display text length is bounded by the elision policy, not the authored span length
- **AND** navigation snapshot construction issues no per-character layout position queries
- **AND** the frame completes through the ordinary test paint path without a timeout-dependent assertion

#### Scenario: Navigating a large fenced code payload

- **WHEN** a fenced code payload far larger than one screen is focused and the caret moves line to line
- **THEN** deterministic counters show snapshot construction cost proportional to the wrapped-line count, not to characters times lines
- **AND** caret movement resolves against the snapshot with correct source offsets

#### Scenario: Collapsed source affordance skips span work

- **WHEN** a document contains a data-URI image whose source affordance is collapsed and unrelated state changes trigger repaints
- **THEN** counters show no per-frame clone of the authored span and no span-length-dependent key derivation for the collapsed block
