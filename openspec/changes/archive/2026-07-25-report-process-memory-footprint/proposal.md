## Why

The memory report added by `add-memory-diagnostics` accounts only for bytes Markion itself owns. When a user observes that resident memory stays high after closing nearly every tab, the report cannot say which of four very different causes is responsible: Markion still retaining per-tab state, the allocator holding freed pages instead of returning them to the OS, a transient allocation peak having raised the heap high-water mark, or GPU atlas growth. These demand opposite fixes, and `accounted_total` alone cannot separate them.

An image-heavy workload made this concrete. The preview-image decode path expands a photograph to a full-resolution `DynamicImage`, copies it again into full-resolution RGBA, and only then downsamples to the display cap — with one unbounded background task per image. That pattern would produce a large transient peak and a permanently raised heap watermark while the steady-state cache stays small, which matches the reported symptom exactly. It is currently a plausible story with no measurement that can confirm or refute it, because the only counter that would settle the question — peak process memory — is not reported.

## What Changes

- Extend the memory report with OS-level process footprint counters measured alongside the existing per-site accounting: current resident size, peak resident size, current private commit, and peak private commit.
- Report peak counters as first-class values, not derived ones. The peak watermark is the single figure that distinguishes a transient allocation spike from steady-state retention, and it cannot be reconstructed from repeated sampling of the current value.
- Source counters from platform APIs, with each counter individually marked unavailable when the host cannot supply it, rather than being reported as zero or silently omitted — the same honesty rule the existing report applies to externally-owned sites.
- Capture the same counters in the headless attribution harness so a document profile's peak cost can be compared against its steady-state cost without a running window.
- Document the interpretation rules that map counter relationships to a diagnosis, so a future report can be read without re-deriving the reasoning.

Non-goals: this change reduces no memory. It does not touch cache budgets, the image decode path, decode concurrency, the global allocator, or GPU atlas handling. It does not add user-visible chrome or new translated strings. Every fix that the resulting numbers justify is deferred to follow-up changes, exactly as `add-memory-diagnostics` deferred its own findings.

## Capabilities

### New Capabilities

None. The reporting surface this extends already exists.

### Modified Capabilities

- `memory-diagnostics`: the capability today requires attribution of Markion-owned allocation sites only, and explicitly frames anything outside that as unenumerable. It needs to additionally require reporting the process's own OS-level footprint including peak watermarks, require per-counter availability reporting on platforms that cannot supply a given counter, require the harness to capture the same counters, and require that reading these counters remains as side-effect free as the rest of the report.

## Impact

- `src/app/memory.rs` gains a process-counter source and extends `MemoryReport` and its log formatting. The report's existing structure — named sites with byte figures and supporting counts — is preserved; process counters are a distinct section rather than another site, because they measure the whole process and would otherwise be double-counted against `accounted_total`.
- A new platform-conditional module supplies the counters: Windows via the process-memory API already reachable through the `windows` crate present in the dependency tree as a transitive GPUI dependency, Linux via `/proc/self/status`, macOS via the Mach task-info interface. Promoting `windows` to a direct dependency adds no new compilation.
- The harness in the root crate's test surface records counters per profile. Per the existing `engineering-quality` requirement, memory tests must not assert absolute byte thresholds, so assertions are limited to counter availability, the invariant that a peak is never below its corresponding current value, and monotonicity of peaks within a process.
- `docs/memory-retention.md` gains the interpretation rules and a refreshed harness dump. The "Unexplained remainder" section, which currently can only gesture at the baseline and the allocator, becomes answerable.
- No workspace member is affected; the harness must stay in the root crate because it depends on GPUI.
