## ADDED Requirements

### Requirement: Process-level footprint reporting
A memory report SHALL include the operating system's own measurement of the process's memory footprint alongside the per-site accounting, covering resident size and private commit, each reported both as its current value and as its peak value over the life of the process. These figures MUST be presented as a distinct section rather than as allocation sites, and MUST NOT contribute to the accounted total, because they measure the whole process including every site already summed. The presence of the footprint section MUST NOT change any existing site figure or the accounted total.

#### Scenario: Report includes the process footprint
- **WHEN** a memory report is produced
- **THEN** the report contains a process-footprint section separate from the per-tab and global site lists
- **AND** that section reports resident size and private commit, each as a current value and a peak value

#### Scenario: Footprint does not disturb the accounted total
- **WHEN** a memory report containing process counters is produced for a given application state
- **THEN** the accounted total equals the sum of the site contributions alone
- **AND** no process counter appears as a contributing site

### Requirement: Per-counter availability is reported honestly
No supported platform supplies every footprint counter, so each counter MUST be reported individually as either a measured value or as unavailable. A counter the host cannot supply, or that fails to be read at runtime, MUST be marked unavailable rather than reported as zero or silently omitted, matching the treatment the report already gives to externally owned sites. A counter that cannot be read MUST NOT prevent the rest of the report from being produced.

#### Scenario: Platform cannot supply a counter
- **WHEN** a report is produced on a platform whose interfaces expose no peak private-commit figure
- **THEN** that counter is named in the footprint section and marked unavailable
- **AND** the remaining counters report their measured values

#### Scenario: Reading a counter fails
- **WHEN** the platform query for a footprint counter fails while a report is being produced
- **THEN** the affected counter is marked unavailable
- **AND** the report is still produced with its complete site accounting

### Requirement: Peak watermarks are preserved, not reset
Peak counters SHALL report the highest value reached over the life of the process, because a peak is the only evidence that distinguishes a transient allocation spike from steady-state retention and cannot be reconstructed by sampling the current value after the fact. Markion MUST NOT invoke any platform facility that trims a working set or clears a peak watermark, as doing so would destroy that evidence. A reported peak MUST NOT be lower than the current value of the same counter.

#### Scenario: Peak bounds the current value
- **WHEN** a report is produced and both a counter's current value and its peak are available
- **THEN** the peak is greater than or equal to the current value

#### Scenario: Producing a report leaves watermarks intact
- **WHEN** two reports are produced in succession
- **THEN** neither report has lowered a peak counter
- **AND** the second report's peak for a counter is greater than or equal to the first report's peak for that counter

### Requirement: Harness captures the process footprint per profile
The headless attribution harness SHALL record the same footprint counters for each document profile it runs, so a profile's transient cost is observable without a running window. Because the harness runs every profile inside one process, a peak set by an earlier profile persists into every later profile; the harness output and the maintained memory document MUST state this constraint so that peak figures are not read as per-profile costs.

#### Scenario: Harness records counters alongside site attribution
- **WHEN** the harness runs a document profile and emits its report
- **THEN** the output includes the process-footprint counters for that profile in addition to the per-site attribution

#### Scenario: Peaks are qualified across profiles
- **WHEN** the harness runs several profiles in one process
- **THEN** its recorded output identifies the peak counters as process-lifetime figures shared across profiles rather than as the cost of an individual profile

### Requirement: Footprint reading does not perturb Markion state
Reading the process footprint MUST NOT change any state a subsequent frame or edit would observe: document versions, cached text handles, derived cache contents, selection, and scroll state MUST be identical before and after a report, and no derived cache may be populated in order to be measured. The existing guarantee that two consecutive reports agree continues to apply to every site figure; it does not apply to process counters, whose current values may legitimately differ between two reads because the operating system's measurement moves independently of Markion.

#### Scenario: Report with counters leaves the application unchanged
- **WHEN** a report including process counters is produced
- **THEN** document versions, derived cache contents, selection, and scroll state are unchanged
- **AND** no previously unpopulated derived cache has been populated

#### Scenario: Repeatability is scoped to site figures
- **WHEN** two reports are produced with no intervening edit, view-mode change, or cache activity
- **THEN** both reports contain identical figures for every site
- **AND** a difference between the two reports' current process counters is not treated as a failure
