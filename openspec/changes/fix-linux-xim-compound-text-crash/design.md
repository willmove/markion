## Context

See `proposal.md` for motivation and `specs/markdown-editing/spec.md` for the observable contract.

Markion already selects a repository-local GPUI 0.2.2 through `[patch.crates-io]` for the Windows IME backport. That GPUI manifest resolves its Linux-only XIM dependency to the published `zed-xim 0.4.0-zed`, whose normalized Cargo dependency selects `xim-ctext 0.3.0`. The published decoder returns `UnsupportedEncoding` for the valid 94N Chinese (`ESC $ ( A`, GB2312) and Korean (`ESC $ ( C`, KS C 5601) designators. `zed-xim` calls that decoder with `expect` in reset, commit, and preedit handling, so the error unwinds on GPUI's main X11 event thread.

A deterministic decoder harness reproduced the reported `Encoding Error: UnsupportedEncoding` failure with 0.3.0 on every run and passed unchanged with `xim-ctext 0.4.1`. Current Zed source pins an XIM revision containing equivalent Chinese and Korean decoding, but the published `zed-xim` package consumed by GPUI 0.2.2 does not contain it.

The editing data flow remains:

```text
X11 ClientMessage bytes
  -> zed-xim protocol request
  -> COMPOUND_TEXT decoder
  -> GPUI XIM preedit/commit callback carrying UTF-8
  -> EntityInputHandler checked range conversion
  -> MarkdownDocument canonical mutation
  -> normal document-version cache invalidation

decode error
  -> recoverable XIM client error
  -> clear the active mark at the last accepted UTF-8 state
  -> drop the failed XIM connection and retain direct keyboard input
```

No decoder state enters `MarkdownDocument`, and no preview, outline, statistics, syntax-highlight, or text-handle cache is added or recomputed outside the existing versioned mutation path.

The completed but unarchived `fix-windows-microsoft-pinyin-crash` change also modifies the `Visual Edit IME composition fidelity` requirement. Archive-order reconciliation is required so neither platform scenario is lost when delta specs are synced.

## Goals / Non-Goals

**Goals:**

- Decode valid GB2312 and KS C 5601 XIM preedit, reset, and commit text into UTF-8.
- Convert every COMPOUND_TEXT decode failure at the XIM request boundary into a normal `ClientError` rather than a panic.
- End a failed active composition at its last successfully decoded text, clear its marked state, and keep canonical source, selection, and undo history valid.
- Keep the dependency patch reproducible, reviewable, license-preserving, and removable when a compatible GPUI/XIM release contains both fixes.

**Non-Goals:**

- Guarantee support for every historical COMPOUND_TEXT character set.
- Retry or reconnect an XIM server after a decode failure in the same session.
- Change Wayland text-input-v3, Windows IME routing, or Markion's composition projection model.
- Introduce decoding or derived-state work on the render path.

## Decisions

### Vendor a narrow `zed-xim` compatibility patch

Add a repository-local copy of the published `zed-xim 0.4.0-zed` package under `vendor/zed-xim`, retain its upstream license and metadata, and select it from the root `[patch.crates-io]`. Record the source version and Markion-only edits in `MARKION_PATCH.md`. The local manifest will resolve `xim-ctext 0.4.1`, whose released decoder covers GB2312, KS C 5601, mixed escape segments, and the other compound-text forms already supported upstream.

This is preferred over editing Cargo's registry cache, which is not reproducible; overriding only `xim-ctext`, which cannot cross `zed-xim`'s incompatible 0.3 version requirement and would leave fatal `expect` calls; or upgrading all of GPUI, which would pull unrelated framework changes into a crash fix. Pinning Zed's corrected git revision was also considered, but a read-only git dependency cannot carry the additional panic-to-error hardening.

### Represent compound-text failures in `ClientError`

Extend the local XIM client's error model with a compound-text decode variant and convert decoder results with `?` in all three inbound text paths: reset reply, character commit, and preedit draw. The error display identifies the operation and decoder failure without logging payload bytes or user text.

This keeps the failure inside the existing `filter_event -> Result` contract. A lossy decoder fallback was rejected because replacing unknown bytes could silently commit the wrong source text. Catching panics in GPUI was rejected because it is broader than the known failure and can hide unrelated invariant violations after partial mutation.

### Degrade the failed XIM session while preserving the last accepted text

GPUI already logs an `XIMClientError`, drops the XIM client, and routes subsequent keys through its direct input path when `filter_event` returns an error. Extend that branch so, when a preedit is active, it first clears the composing flag and invokes the existing unmark callback for the owning window. This finalizes Markion's current IME undo capture and retains only the last successfully decoded UTF-8 preedit; the undecodable packet is never forwarded.

Retaining the last accepted preedit is preferred over deleting it because a failure may occur on the final commit packet after the user has already seen the complete candidate. Reconnection is deferred: continuing without XIM is the current GPUI failure policy and provides a deterministic safe fallback.

### Test at the decoder, request, and editor boundaries

The vendored XIM package will contain byte fixtures for GB2312, KS C 5601, mixed CJK/ASCII segments, and an invalid designator. Request-boundary tests will exercise reset, preedit, and commit dispatch with a minimal client/handler and assert decoded text or a returned `ClientError`, so the regression covers the former `expect` sites without requiring a desktop input method.

A focused GPUI/Markion regression will model an active preedit followed by XIM failure cleanup and assert that the last accepted text remains, the marked range and undo capture are finalized, canonical UTF-8 boundaries remain valid, and later direct input succeeds. Manual IBus and Fcitx verification on Linux X11 remains necessary because CI cannot reliably install and drive a desktop-wide input method.

## Risks / Trade-offs

- [Vendoring another upstream crate adds maintenance and repository size] -> Keep the patch limited to one small package, retain licensing and a patch manifest, and remove it once GPUI publishes an equivalent dependency.
- [A decoder upgrade changes handling for compound-text encodings beyond Chinese and Korean] -> Pin 0.4.1 exactly and retain byte-fixture coverage for UTF-8 and Japanese alongside the new cases.
- [Dropping XIM after one malformed packet disables composition for the rest of the process] -> Preserve GPUI's existing safe-degradation policy, keep direct typing active, and log one actionable error without user text.
- [Unmarking the last accepted preedit may preserve text the input method intended to replace] -> Prefer visible, already accepted UTF-8 over destructive rollback; assert source and selection integrity and keep the result undoable as one composition action.
- [Two active changes modify the same IME requirement] -> Keep both deltas textually aligned with the Windows and Linux scenarios before either change is synced or archived, and validate both changes after reconciliation.

## Migration Plan

1. Add the licensed `vendor/zed-xim` snapshot, patch record, root Cargo override, and regenerated lockfile.
2. Upgrade its compound-text decoder dependency and replace fatal inbound decode calls with typed errors.
3. Add XIM error cleanup at the vendored GPUI boundary and the automated regressions described above.
4. Run the focused vendor tests, root tests, `cargo test --workspace`, formatting, OpenSpec validation, and Linux-target CI compilation.
5. On Linux X11, verify Chinese and Korean composition where available with IBus/Fcitx, including preedit updates, commit, cancellation, undo/redo, and continued process liveness.

Rollback removes the `zed-xim` Cargo patch and vendored package and regenerates `Cargo.lock`; no document or preference migration is required. The regression tests should remain as evidence if the override is later replaced by an upstream release.
