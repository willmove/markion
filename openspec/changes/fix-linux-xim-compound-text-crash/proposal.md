## Why

On Linux X11, entering Chinese or Korean text through an XIM server such as IBus can terminate Markion when the server sends a valid legacy COMPOUND_TEXT preedit. Markion 0.3.3 resolves the published `zed-xim 0.4.0-zed` with `xim-ctext 0.3.0`; that decoder rejects the GB2312 and KS C 5601 designators, and `zed-xim` converts the recoverable decode error into a main-thread panic.

## What Changes

- Use a reproducible repository-controlled XIM dependency override that includes GB2312 and KS C 5601 COMPOUND_TEXT decoding instead of the stale published decoder.
- Make XIM reset, preedit, and commit decoding return a recoverable client error rather than panicking on malformed or genuinely unsupported server data.
- Add deterministic regressions for Chinese and Korean COMPOUND_TEXT payloads at the XIM request boundary, plus Linux X11 acceptance coverage for composition continuity.
- Preserve Markion's canonical document mutation, undo grouping, and per-document-version derived-state/cache invariants; decoded text continues through the existing GPUI input callbacks.

Non-goals: replacing GPUI or XIM, redesigning Visual Edit composition, changing Wayland text input, changing Windows IME routing, or adding a custom input-method implementation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: require valid Chinese and Korean Linux X11 preedit/commit payloads to remain in the running editor process, and require undecodable XIM payloads to fail without terminating Markion or corrupting canonical source.

## Impact

- Root Cargo patching and lockfile resolution for the Linux-only `zed-xim` dependency.
- A repository-local, license-preserving XIM compatibility patch outside `crates/*`.
- Linux X11 XIM request decoding and GPUI error logging; no public Markion API changes.
- Focused dependency tests, root/workspace regression suites, Linux CI compilation, and manual IBus/Fcitx X11 verification.
