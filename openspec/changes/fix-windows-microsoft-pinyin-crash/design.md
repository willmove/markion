## Context

See `proposal.md` for motivation and `specs/markdown-editing/spec.md` for the observable contract.

Markion currently consumes GPUI 0.2.2 from crates.io. Its packaged Windows event source matches the pre-fix GPUI source from immediately before upstream Zed commit `2ead8c42fb6792095d7cb02f7b89e467421dc8a0`: `VK_PROCESSKEY` is unwrapped with `ImmGetVirtualKey`, key handling conditionally invokes `TranslateMessage`, and IME composition enters Markion through `EntityInputHandler` callbacks. Windows Error Reporting records the observed failures as repeatable fatal exits in `ucrtbase.dll`, while direct handler tests pass because they bypass rendering after the native callback.

Reproducing the post-backport failure with stderr attached captured the actual panic at `gpui/src/platform/windows/direct_write.rs`: a generated font-run boundary ended at byte 246, inside the three-byte UTF-8 encoding of `现`. The Win32 window procedure cannot unwind that panic and terminates with fast-fail `0xc0000409`. Visual Edit caused the invalid run by collecting ordered inline highlights and then appending an earlier IME marked-text underline after later bold runs. GPUI 0.2.2's `StyledText::compute_runs` assumes ranges are sorted and non-overlapping, so the out-of-order overlay turned individually valid ranges into invalid cumulative `TextRun` lengths.

The reported compile failure is independent but also reproducible. `libgit2-sys` enables cc-rs's `parallel` feature for the shared build dependency, causing unrelated native crates (`onig_sys`, `libz-sys`, and `ring`) to invoke several MSVC 14.44 compiler processes concurrently. Serializing those compilations reduced contention but did not eliminate C1056. The remaining common factor was cc-rs's undocumented `-Brepro` flag: Microsoft `cl.exe` 19.44 could not rewrite the deterministic timestamp field even for a single object, while the same command succeeded without `-Brepro`.

The editing data flow remains:

```text
Win32 keyboard/IME messages
  -> GPUI Windows platform routing
  -> EntityInputHandler UTF-16 ranges and composition text
  -> validated canonical UTF-8 source range
  -> MarkdownDocument edit + per-version cache invalidation
  -> Visual Edit projection/marked-range layout
  -> candidate-window bounds returned to GPUI
```

No derived Markdown state is added or recomputed outside the existing document-version invalidation path. The text handle, syntax-highlight memoization, and shared `Arc` caches keep their current lifetime rules.

## Goals / Non-Goals

**Goals:**

- Backport the upstream GPUI Windows keyboard/IME fix without adopting unrelated post-0.2.2 framework changes.
- Prevent malformed or stale platform ranges from panicking in Markion's native callback.
- Preserve composition projection, candidate geometry, commit/cancel, and single-undo semantics.
- Cover the native-routing contract and non-ASCII range boundary with regression tests that can run without automating a machine-wide IME.

**Non-Goals:**

- Replace GPUI or maintain a general-purpose fork beyond the pinned backport.
- Change Markdown parsing, source serialization, or Visual Edit projection semantics.
- Automate Microsoft Pinyin installation or machine-wide language settings in the test suite.

## Decisions

### Vendor the published GPUI crate and backport the upstream input commit

Markion will patch crates.io `gpui` to a repository-local copy of 0.2.2 and apply the GPUI portions of upstream commit `2ead8c42fb6792095d7cb02f7b89e467421dc8a0`. The vendored package stays outside `crates/*`, is excluded from workspace membership, retains its upstream Apache-2.0 license, and is selected through `[patch.crates-io]`.

This is preferred over pointing directly at the Zed monorepo because the git package resolves numerous workspace-internal dependencies and would expand the dependency migration beyond the IME fix. It is preferred over editing Cargo's registry cache because such a fix is not reproducible for users or CI.

The backport keeps `VK_PROCESSKEY` under IME ownership and moves translation/accelerator handling to the conventional Win32 message-loop boundary as upstream intended. Non-Windows constructor updates required by the upstream `KeyDownEvent` contract carry a neutral value and do not change their input behavior.

### Validate native ranges before reading or mutating canonical text

Markion will centralize validation of UTF-16 platform ranges against the current document text. A converted range is accepted only when it is ordered, within the current text length, and on UTF-8 character boundaries. Comparisons use checked access rather than indexing. If GPUI supplies a stale range, Markion falls back to the current valid selection (or declines a geometry lookup) instead of panicking.

This is preferred over clamping arbitrary byte offsets because clamping can silently redirect a composition into adjacent Markdown syntax. It also preserves the canonical-source and undo invariants: only a validated edit reaches `MarkdownDocument::replace_range`.

### Split automated coverage at the platform seam

Deterministic tests will cover the upstream Windows routing helpers where possible and Markion's invalid/stale range behavior through `EntityInputHandler`. Existing CJK, emoji, combining-character, candidate-bounds, and undo tests remain the cross-platform behavioral suite. A manual Microsoft Pinyin smoke check is retained as release verification because CI cannot reliably configure or drive a machine-wide IME.

### Merge composition styling before handing ranges to StyledText

Visual Edit will treat marked-text styling as an overlay rather than appending it as another independent highlight. All inline-style and marked-range endpoints are partitioned into UTF-8-valid intervals; the IME underline is merged into the base style on overlapping intervals; adjacent identical styles are coalesced. The result is sorted, non-overlapping, and preserves bold/link/source styling while the composition is active.

This fixes the producer rather than merely clamping DirectWrite's consumer. Regression tests exercise both a plain-text composition before later bold content (the reported order failure) and composition overlapping an existing inline style.

### Avoid the unstable MSVC deterministic-object path

Markion will patch the resolved cc-rs 1.2.65 package locally. For Microsoft `cl.exe`, the patch omits `-Brepro` and compiles one build script's object list sequentially. `clang-cl` retains `-Brepro`, other toolchains retain cc-rs parallelism, and Cargo can still run Rust crates and independent build scripts concurrently. This is narrower than setting global Cargo jobs to one and avoids changing application runtime behavior.

## Risks / Trade-offs

- [Vendoring GPUI increases repository size and maintenance responsibility] -> Keep the copy pinned to 0.2.2, document the single upstream backport, and remove the patch when a compatible crates.io release contains it.
- [The upstream commit changes keyboard routing beyond Chinese IME] -> Backport the complete GPUI commit rather than an invented partial variant, and run the workspace keyboard/input tests on Windows.
- [Defensive fallback can hide a platform sequencing defect] -> Test the stale-range branch explicitly and keep the native Microsoft Pinyin smoke check in acceptance verification.
- [Inline overlays may overlap existing bold/link/source styles] -> Partition at every endpoint and merge only explicitly set overlay fields, then assert the output is ordered, non-overlapping, and on UTF-8 boundaries.
- [Patching a shared build helper affects all native dependencies] -> Keep the patch version-pinned, retain upstream licensing, omit `-Brepro` only for Microsoft `cl.exe`, and preserve the flag for `clang-cl`; normal cc-rs behavior remains unchanged elsewhere.

## Migration Plan

1. Add the vendored GPUI 0.2.2 package and Cargo patch, then regenerate `Cargo.lock`.
2. Apply the upstream GPUI input commit and compile all supported targets through normal CI.
3. Add Markion range hardening, composition-overlay merging, and regression tests.
4. Patch cc-rs's MSVC object loop and Microsoft `cl.exe` flags, then verify a default parallel Cargo build from invalidated native outputs.
5. On Windows, run the original document/location with Microsoft Pinyin through preedit, commit, cancellation, backspace, and undo; confirm no new WER `0xc0000409` event.

Rollback removes the `[patch.crates-io]` entry and vendored package, restoring the current lockfile resolution. Markion's checked range handling is independently safe to retain.
