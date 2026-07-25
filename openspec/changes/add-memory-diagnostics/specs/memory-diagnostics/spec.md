## ADDED Requirements

### Requirement: Retained-memory accounting by site
Markion SHALL be able to report its retained memory attributed to individual allocation sites rather than as a single total. The report MUST cover every per-tab site (document text, each derived Markdown cache, undo and redo history, retained shaped editor lines, and per-tab layout caches) and every process-global render cache (diagram, math, and syntax-highlight). Each site MUST report both an estimated byte figure and the underlying counts that produced it, so an estimate can be checked by hand. Sites whose storage is owned outside Markion and cannot be enumerated MUST be labelled as externally owned rather than estimated as zero or silently omitted.

#### Scenario: Report attributes bytes to named sites
- **WHEN** a memory report is produced for an application with open tabs
- **THEN** the report lists each per-tab site and each global cache site separately with its own byte figure
- **AND** each figure is accompanied by the count it was derived from

#### Scenario: A site cannot be enumerated
- **WHEN** a retention site's storage is owned by the UI framework and is not readable by Markion
- **THEN** the report names the site and marks it as externally owned
- **AND** the report does not attribute a fabricated byte figure to it

### Requirement: Accounting is observational and side-effect free
Producing a memory report MUST NOT change any state that a subsequent frame or edit would observe. Accounting MUST read derived Markdown caches through their existing storage without invoking the deriving accessors, so an unpopulated cache reports zero instead of being populated in order to be measured. Document versions, cached text handles, cache contents, and selection or scroll state MUST be identical before and after a report. State shared between a document and a tab through a reference-counted handle MUST be counted once, with the sharing tab's handle reported as shared rather than as an independent allocation.

#### Scenario: Reporting an unrendered document
- **WHEN** a memory report is produced for a tab whose document has never been rendered in a mode that derives preview or visual blocks
- **THEN** those derived-cache sites report zero
- **AND** the caches remain unpopulated after the report

#### Scenario: Repeated reports agree
- **WHEN** two memory reports are produced with no intervening edit, view-mode change, or cache activity
- **THEN** both reports contain identical figures for every site

#### Scenario: Blocks shared between a document and its tab
- **WHEN** a tab holds a reference-counted handle to the same derived blocks its document has cached
- **THEN** the blocks are counted once in the total
- **AND** the tab-level handle is reported as a shared reference

### Requirement: Diagnostic report surface
A normally built Markion SHALL expose a developer-facing action that writes the current memory report to the application's diagnostic log, so a user observing high memory can produce evidence without a debugger or a special build. The report body MUST remain diagnostic output routed through the existing logging setup and MUST NOT introduce new translated user-facing strings beyond an existing status message confirming that a report was written.

#### Scenario: User triggers a memory report
- **WHEN** the memory-report action is invoked in a running application
- **THEN** the complete per-site report is written to the diagnostic log
- **AND** the status line confirms the report was written using an existing localized message

### Requirement: Headless attribution harness
The repository SHALL provide a headless harness that constructs a described tab profile from fixture documents and produces the same per-site report as the running application, so a document profile can be attributed to a specific retention site deterministically. The harness MUST support varying both the number of open tabs and the document content profile, covering at minimum plain text, embedded images, diagrams, math, and code blocks. The harness MUST live in the root application crate because it depends on the UI framework, and MUST NOT be added to a workspace member.

#### Scenario: Attributing a document profile
- **WHEN** the harness opens a given number of tabs of a given content profile and produces a report
- **THEN** the report attributes the retained bytes to the same named sites the running application reports

#### Scenario: Distinguishing tab-linear growth from global growth
- **WHEN** the harness opens an additional tab holding the same document
- **THEN** the per-tab total increases
- **AND** the global render-cache total is unchanged

### Requirement: Tab teardown releases per-tab accounting
Closing a tab SHALL return every per-tab accounting site to the total it reported before that tab was opened, so that a retention leak in tab teardown is detectable from the report alone. Global render caches are outside this requirement because they are intentionally shared across tabs and document versions.

#### Scenario: Closing a tab releases its state
- **WHEN** a tab is opened, rendered, and then closed
- **THEN** the per-tab accounting total returns to the value it held before the tab was opened
