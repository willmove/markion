## ADDED Requirements

### Requirement: Parallel-friendly decode concurrency with a safety cap
Markion SHALL allow multiple preview-image fetch/decode tasks to run concurrently so that typical multi-image documents can warm in parallel, while still imposing a finite overall in-flight safety cap so pathological documents cannot stampede the process. When more images need loading than the overall cap allows, excess sources MUST remain in the existing pending presentation state until a slot frees, without blocking the GPUI frame or typing path. The concurrency policy MUST NOT change claim/release semantics or the completed-byte budget, and MUST NOT serialize ordinary warm to a single-digit default as low as two concurrent tasks.

#### Scenario: Typical multi-image document warms in parallel
- **WHEN** a document references several unloaded preview images within the overall in-flight safety cap
- **THEN** Markion may run those fetch/decode tasks concurrently
- **AND** pending presentation is used only for sources that have not yet started or finished

#### Scenario: Pathological many-image document respects the overall cap
- **WHEN** a document references more unloaded preview images than the configured overall concurrency cap
- **THEN** at most that many fetch/decode tasks are in flight at once
- **AND** the remaining sources stay pending until a running task completes

#### Scenario: Completions free slots for remaining pending images
- **WHEN** an in-flight decode completes while other claimed sources are still pending
- **THEN** further decode work for those pending sources can start without a user edit
- **AND** document text and version remain unchanged

### Requirement: Tighter limit only for probed oversized (heavy) decodes
When Markion can cheaply determine that a source's longer edge exceeds the configured display maximum, that decode MAY be classified as heavy and MUST additionally respect a tighter heavy in-flight limit, so several large photographs do not each hold oversized intermediates at once while small images continue in parallel under the overall cap. Sources that are within the display maximum, or that cannot be classified without a full decode, MUST NOT be forced through the heavy limit solely to reduce concurrency.

#### Scenario: Small images are not gated by the heavy limit
- **WHEN** several unloaded preview images are at or below the display maximum edge
- **THEN** they may proceed concurrently up to the overall safety cap
- **AND** they are not blocked waiting on the heavy-slot limit

#### Scenario: Oversized images respect the heavy-slot limit
- **WHEN** more probed oversized images need decoding than the heavy concurrency limit allows
- **THEN** at most that many heavy decodes are in flight at once
- **AND** excess heavy sources remain pending until a heavy slot frees

### Requirement: Oversized decode avoids full-resolution RGBA intermediates
When decoding a non-SVG preview image whose source dimensions exceed the configured display maximum edge, Markion MUST produce the display-sized bitmap without retaining a full-resolution RGBA buffer solely for the purpose of downsampling it afterward. The ready cache entry's longer edge MUST still respect the same display maximum as today, with aspect ratio preserved. Consuming conversions (rather than cloning into a second full-resolution buffer) MUST be preferred when the decoder API allows them.

#### Scenario: Large photograph retains only the display-sized ready raster
- **WHEN** a local or remote preview image decodes from a source whose longer edge exceeds the display maximum
- **THEN** the ready `RenderImage` stored in the cache has a longer edge no greater than that maximum
- **AND** the decode path does not keep a full-resolution RGBA buffer alive after the ready entry is produced

#### Scenario: Already-small images are unchanged
- **WHEN** a preview image's longer edge is already within the display maximum
- **THEN** the ready raster keeps those dimensions
- **AND** presentation matches the existing bounded-cache behaviour

### Requirement: Decode-peak improvement remains observationally checkable
The repository SHALL keep a headless way to observe process footprint around a synthetic large-decode workload so contributors can compare peak resident or commit figures before and after this change. Absolute byte thresholds MUST NOT be merge gates; the probe is informational and MUST use the same honesty rules as process-footprint reporting (peaks are process-lifetime; unavailable counters are not zeros).

#### Scenario: Contributor runs the decode-spike probe
- **WHEN** a contributor runs the decode-spike footprint probe
- **THEN** the output reports process footprint counters around a large transient allocation or decode
- **AND** ordinary CI success does not depend on a fixed peak-byte threshold
