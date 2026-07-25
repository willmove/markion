## 1. Platform counter source

- [x] 1.1 Add a platform-conditional module in `src/app/` exposing one function that returns the four footprint counters (resident current/peak, commit current/peak) as individually optional values, with no trait or injection seam.
- [x] 1.2 Implement the Windows source via `GetProcessMemoryInfo`, promoting `windows` to a direct dependency with only the process-status feature; confirm with `cargo tree -d` that no second version is compiled.
- [x] 1.3 Implement the Linux source by parsing `/proc/self/status` (`VmRSS`, `VmHWM`), and decide from a real dump whether to supply commit-current from `VmData` as an approximation or mark it unavailable; mark commit-peak unavailable either way.
- [x] 1.4 Implement the macOS source via `task_info` basic-info and VM-info flavours (`resident_size`, `resident_size_max`, `phys_footprint`); mark commit-peak unavailable.
- [x] 1.5 Ensure every read path degrades a failed query to unavailable and never panics or propagates an error that would prevent a report from being produced.

## 2. Report integration

- [x] 2.1 Extend `MemoryReport` in `src/app/memory.rs` with a footprint section held separately from `tab_sites` and `global_sites`, so `per_tab_total`, `global_total`, and `accounted_total` are arithmetically untouched.
- [x] 2.2 Extend `format_log` to render the footprint section after the site lists and before the unaccounted note, printing each counter's value or an explicit unavailable marker, and labelling the platform the counters came from.
- [x] 2.3 Populate the footprint when building the report, reading counters without touching document versions, cached text handles, derived caches, selection, or scroll state.
- [x] 2.4 Confirm the diagnostic action path adds no new user-facing or translated string; the existing status message continues to confirm the report was written.

## 3. Tests

- [x] 3.1 Unit-test the counter source: available counters are non-zero, a peak is never below its corresponding current value, and unavailable counters render as unavailable rather than zero.
- [x] 3.2 Test that adding the footprint leaves `accounted_total` equal to the sum of site contributions and that no counter appears as a contributing site.
- [x] 3.3 Test that two successive reports agree on every site figure while a difference in current process counters is tolerated, and that neither report lowered a peak.
- [x] 3.4 Test that producing a report including counters leaves an unpopulated derived cache unpopulated and every document version unchanged.
- [x] 3.5 Keep every assertion machine-independent per the existing `engineering-quality` gate — availability, ordering, and monotonicity only, never an absolute byte threshold.

## 4. Harness

- [x] 4.1 Record the footprint counters per profile in `memory_harness_attribution_dump` alongside the existing per-site figures.
- [x] 4.2 Label peak counters in the harness output as process-lifetime figures shared across profiles, so they are not misread as a single profile's cost.

## 5. Documentation

- [x] 5.1 Add the counter table to `docs/memory-retention.md` describing what each counter means, and the per-platform availability matrix showing which counters each target supplies.
- [x] 5.2 Add the interpretation rules mapping counter relationships to a diagnosis — Markion retention, allocator retention or atlas growth, transient allocation spike, and OS working-set trimming — and rewrite the "Unexplained remainder" section to point at those rules instead of gesturing at the baseline.
- [x] 5.3 Refresh the harness dump table with the new counters, noting the measurement platform and the cross-profile peak caveat.
- [x] 5.4 Capture a real before/after reading for the reported image-heavy workload and record whether the peak counter supports or refutes the transient-spike hypothesis, so the follow-up change is chosen on evidence.

## 6. Verification

- [x] 6.1 Run `cargo fmt --check` and `cargo clippy` clean for the new module on the host platform.
- [x] 6.2 Run `cargo test --workspace`.
- [x] 6.3 Run `openspec validate report-process-memory-footprint`.
