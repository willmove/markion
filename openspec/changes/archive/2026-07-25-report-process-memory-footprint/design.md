## Context

`docs/memory-retention.md` closes with an "Unexplained remainder" section that can only gesture at two suspects — the Layer A fixed baseline, and "GPU atlas / allocator overhead" — because the report has no view of the process itself. Every figure in it is a sum of things Markion chose to count.

That is sufficient when the question is *which Markion cache is large*. It is useless when the question is *why does resident memory not fall after the caches shrink*, because the four candidate answers all look identical from inside the accounting:

```
  observation: RSS stays flat after closing tabs
       │
       ├─ (a) Markion still holds per-tab state   → accounted_total stays high
       ├─ (b) allocator keeps freed pages         → accounted_total falls, RSS does not
       ├─ (c) a transient peak raised the heap    → accounted_total falls, RSS does not,
       │                                             and peak >> both
       └─ (d) GPU atlas grew and cannot shrink    → accounted_total falls, RSS does not
```

Only (a) is distinguishable today. Cases (b), (c), and (d) produce the same report, and they want opposite fixes: (b) argues for a different allocator, (c) argues for changing an allocation pattern, (d) argues for patching or working around upstream GPUI. Case (c) is currently the leading hypothesis for the reported image-heavy workload, and it is the one that a *peak* counter separates cleanly from the others — a peak far above both the current resident size and the accounted total is only explicable by allocations that have already been freed.

The existing report is produced by `MemoryReport` in `src/app/memory.rs`, built from `MemorySite` values with an `Owned` / `Shared` / `External` contribution kind, and rendered by `format_log` into the `markion::memory` tracing target.

## Goals / Non-Goals

**Goals:**

- Make the four cases above distinguishable from a single report.
- Report peak watermarks, which cannot be reconstructed by sampling the current value from a diagnostic action the user triggers after the fact.
- Keep counter availability honest per platform and per counter, matching the report's existing treatment of unenumerable sites.
- Give the harness the same counters so a document profile's transient cost is visible without a window.
- Leave `accounted_total` and every existing site figure byte-for-byte unchanged, so the numbers already recorded in `docs/memory-retention.md` remain comparable.

**Non-Goals:**

- Reducing memory. No cache budget, decode path, concurrency limit, allocator, or atlas behaviour changes here.
- Continuous sampling, time series, or a monitoring UI. The counters are read when a report is produced.
- Resetting or trimming watermarks. Windows `EmptyWorkingSet` and Linux `clear_refs` can lower a peak; using them would destroy the evidence this change exists to collect.
- Attributing process memory to sites. The counters describe the whole process; correlating them with sites is the reader's job, guided by the documented interpretation rules.

## Decisions

### Process counters are a separate report section, not another `MemorySite`

`accounted_total` sums site contributions. A process counter is a measurement of everything, including the sites already summed, so modelling it as a site would either double-count it into the total or require a fourth contribution kind that exists solely to be excluded. Both are worse than a distinct section.

The report therefore gains a footprint section rendered after the site lists and before the unaccounted note, where the reader naturally compares it against the totals printed at the top.

Alternative considered: a `SiteContribution::Process` variant excluded from totals. Rejected — it overloads a type whose entire purpose is "how does this contribute to the total" with a member that does not contribute, and every consumer would need to special-case it.

### Four counters: resident and commit, each current and peak

| Counter | What it answers |
|---|---|
| resident current | What the user sees in a task manager right now |
| resident peak | Did we ever spike? — the decisive figure for case (c) |
| commit current | How much private memory the process holds, independent of OS paging decisions |
| commit peak | Did the spike involve real private allocation, or only page residency? |

Resident size alone is unreliable as a sole signal because the OS can trim a working set under pressure without the process freeing anything, which would read as an improvement that did not happen. Private commit does not move for that reason, so the pair together separate "we released memory" from "the OS reclaimed pages we still own".

### Per-counter availability, because no platform supplies all four

This is not defensive programming for a hypothetical host; it is the actual situation on all three targets:

| Counter | Windows | Linux | macOS |
|---|---|---|---|
| resident current | `WorkingSetSize` | `VmRSS` | `resident_size` |
| resident peak | `PeakWorkingSetSize` | `VmHWM` | `resident_size_max` |
| commit current | `PagefileUsage` | approximated from `VmData`, or unavailable | `phys_footprint` |
| commit peak | `PeakPagefileUsage` | **unavailable** | **unavailable** |

Only Windows — the platform where the problem was reported — supplies the full set. Reporting an unavailable counter as `0` would make a Linux report look like a process that had never allocated anything, which is exactly the class of silent-zero mistake the existing spec already forbids for unenumerable sites. Each counter is therefore individually optional and rendered as unavailable when absent.

The Linux commit-current row stays deliberately soft: `VmData` is an approximation of private commit, not the same quantity, and the design permits either supplying it as an approximation or declaring it unavailable. That choice is better made against a real `/proc/self/status` dump than in advance.

### Sourcing, and the `windows` dependency

A single platform-conditional module exposes one function returning the four optional counters. No trait, no injection: there is exactly one implementation per target and the test surface asserts availability and ordering, not specific values.

- **Windows** — `GetProcessMemoryInfo` fills `PROCESS_MEMORY_COUNTERS` with all four in one call. The `windows` crate is already compiled as a transitive GPUI dependency, so promoting it to a direct dependency with the process-status feature adds no new compilation. This is the only new dependency edge in the change.
- **Linux** — parse `/proc/self/status`. No dependency.
- **macOS** — `task_info` with the basic-info and VM-info flavours. Reachable through the `libc` crate already in the tree.

Alternative considered: the `sysinfo` crate for all three. Rejected — it pulls a broad system-inspection surface (CPU, disks, networks, process enumeration) for four integers, and its process-refresh model does not expose peak commit at all, which is the counter the change exists for.

### The harness records counters per profile and documents their contamination

The harness runs every profile inside one test process. Current counters are meaningful per profile; **peaks are not**, because a peak set by an earlier profile persists for the life of the process and will be reported unchanged by every profile after it.

Rather than pretend otherwise, the harness records the counters per profile and the documentation states the constraint explicitly, the same way the existing dump already warns that "global totals accumulate across profiles in a single test process". A reader comparing peak cost across profiles must run them in separate processes; a reader checking whether one profile spikes should read the peak's rise across the profile boundary.

### Interpretation rules are documentation, not code

The mapping from counter relationships to a diagnosis is a reading guide that will be refined as evidence arrives. Encoding it as an automatic verdict in the report would freeze a judgement that is currently a hypothesis, and would be wrong in the cases nobody has thought of yet. `docs/memory-retention.md` gets the table; the report prints numbers.

The rules to document, given `A = accounted_total` and `B` = the fixed Layer A baseline:

| Observation | Reading |
|---|---|
| `resident current ≈ A + B` and both fall together | Working as intended |
| `A` falls, resident current does not, peak ≈ current | Allocator retention or GPU atlas — cases (b)/(d) |
| `A` falls, resident current does not, peak ≫ current | Transient allocation spike raised the watermark — case (c) |
| `A` does not fall | Markion is still holding state — case (a); fix before looking further |
| commit current falls while resident current does not | OS trimmed the working set; no real change |

### Reading counters must stay side-effect free

The existing spec requires that producing a report changes nothing a later frame or edit would observe, and that two consecutive reports agree. Process counters cannot satisfy the second clause literally — resident size legitimately moves between two reads without Markion doing anything. The requirement is therefore scoped correctly: reading the counters must not perturb Markion state, and the existing "repeated reports agree" guarantee continues to cover the site figures, which is where it was always aimed. Tests assert agreement on sites and availability plus ordering on counters.

## Risks / Trade-offs

**Peak counters never fall, so a single long session eventually reports a peak from an event the user has forgotten** → This is inherent to a watermark and is why the counter is useful. Documented as such; a user chasing a specific spike restarts and reproduces.

**Reporting resident size invites treating it as a scoreboard and optimizing the number rather than the cause** → The interpretation table leads with the relationship between counters, not the absolute value, and the existing `engineering-quality` requirement already forbids gating tests on absolute byte thresholds.

**Promoting `windows` to a direct dependency couples the root crate to a version GPUI also selects** → Both resolve to one version through Cargo's unification; a future GPUI bump that moves the major version would surface as a duplicate-compile warning rather than a break, and the API used is one of the oldest and most stable in the platform surface.

**macOS `phys_footprint` and Windows `PagefileUsage` are not the same quantity** → They are not presented as comparable across platforms. The report labels the platform, and the interpretation rules compare counters *within* one report, never one host's number against another's.

**A platform's parse or call fails at runtime** → Every counter is already optional, so a failure degrades to the unavailable rendering rather than failing the report. The report must never fail to be produced because a counter could not be read.
