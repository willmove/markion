## Context

See proposal.md — Why. Today `build_publishing_snapshot` (src/publishing.rs) resolves only images inside `document_asset_dir()` (`<stem>.assets/`), enforced lexically by `validate_authored_reference` (rejects `../`, absolute paths, URLs) and physically by canonical containment inside `PublishingResource::from_path` (crates/wechat-workspace/src/resource.rs). The browser side (`bridge.js`) already matches arbitrary authored URLs against `resources[].authored_url` after normalization, so the frontend needs no change. The app has a GPUI `window.prompt` confirmation pattern (used by delete/reset flows) and a single-undo whole-text splice (`MarkdownDocument::apply_transformed_text`) suitable for a one-shot link rewrite. Conventions: spec prose in English, user-facing strings via `src/i18n.rs` (7 languages: en, zh-hans, zh-hant, ja, fr, de, es).

## Goals / Non-Goals

**Goals:**

- Local images referenced by a saved document preview in the MarkNice workspace when they live in the document's directory tree or up to one directory level above it — with zero user action.
- A confirmed, undoable "organize local images" escape hatch for references outside that scope (deeper parent escapes, absolute paths).
- Preserve every existing loopback protection: loopback-only bind, claim/bearer session auth, `no-store`, opaque resource IDs, extension whitelist, canonical containment with read-time symlink re-check, no filesystem-path disclosure.

**Non-Goals (design-level):**

- No WeChat media upload / draft API (separate `add-wechat-draft-publishing` change).
- No change to rich-copy stripping of local images or to any bundled MarkNice file (`bridge.js`, `index.html`, CSP, manifest stay byte-identical).
- No `file://` scheme resolution, no new image formats, no directory browsing or bulk import UI.

## Decisions

### D1: The publishing image scope is the canonical parent of the document's directory

For a saved document at `<dir>/<stem>.md`, the containment root becomes `canonicalize(parent(<dir>))`. A referenced image resolves when its canonical path starts with that root.

One rule covers the requested behavior with no per-case logic:

```
doc: C:\notes\sub\my-note.md          scope root: C:\notes
  ![](pic.png)              ✓ same level            (under C:\notes\sub)
  ![](img/deep/nested.png)  ✓ any depth below       (under C:\notes\sub)
  ![](../banner.png)        ✓ exactly one level up  (under C:\notes)
  ![](../assets/logo.png)   ✓ parent tree           (under C:\notes)
  ![](../../x.png)          ✗ escapes above the parent level → organize/unresolved
  ![](C:\pictures\x.png)    ✗ absolute → organize/unresolved
```

Alternative considered — enumerating zones as "document tree ∪ files directly in the parent directory only". Rejected: two asymmetric rules (allowing `../logo.png` but denying `../assets/logo.png` is inexplicable to users), and it adds no real safety because exposure is bounded by reference enumeration (D4), not by tree shape. Depth *below* an allowed anchor is irrelevant; the only boundary that matters is never escaping *above* the parent level.

### D2: `PublishingResource::from_path` takes the scope root as an explicit parameter

The second argument (currently the implicit asset directory) becomes the caller-supplied scope root; `src/publishing.rs` computes it as `document_dir.parent()` (falling back consistently when the document sits at a filesystem root — such a document has no parent level, so the scope degenerates to its own directory tree).

`validate_authored_reference` relaxes to allow `ParentDir` components but still rejects absolute paths, drive prefixes, `://` URLs, and NUL. The canonical containment check in `from_path` (and the re-check in `read()`) remains the authoritative boundary; the lexical check stays as a cheap prefilter. Untitled documents keep the existing behavior: no path, no resources, all local references unresolved.

### D3: Server routes, IDs, and the MarkNice bundle are untouched

Resources keep flowing through `/api/resource/{id}` with SHA256(authored URL + canonical path) IDs. `bridge.js` matching already normalizes both sides identically (`decodeURI`, strip `?#`, backslash→slash, strip leading `./`, lowercase), so `../img.png` and `IMG/sub.PNG` variants match without bundle changes. Consequence: no manifest regeneration is needed for this change — an important scope saver, since bundle bytes are digest-pinned.

### D4: The organize command plans first, mutates once

New `OrganizeLocalImages` action in the Export menu beside `PublishWechat`. Flow:

```
scan publishing_image_references()
   ├─ in-scope (D1)            → skip
   ├─ out-of-scope, resolvable → organizable candidates
   │    (../ any depth, absolute local paths; ext whitelist + regular file + readable)
   └─ missing/unsupported/file: → skipped, counted for the status note

window.prompt (Info): "N local image(s) will be copied into <stem>.assets/ and links rewritten"
   ├─ Cancel → nothing happens (no file, no version bump)
   └─ OK    → import_image_file() per candidate (content-hash naming, byte-identical reuse)
              → compute rewritten destinations from parser-derived image ranges ONLY
              → apply_transformed_text(): ONE splice, ONE version bump, ONE undo step
              → status: organized count (+ failures if any)
```

Rewrites use the same parser scan that feeds `publishing_image_references()` (pulldown-cmark `Tag::Image` destinations plus `html_preview_parts` inline `<img src>`) to get exact destination byte ranges — never raw string replace, so a plain link `[x](url)` sharing the URL is not rewritten. Candidates whose copy fails are left untouched; already-copied files are harmless (content-addressed). Copying happens before the text splice so a failed copy cannot strand a rewritten link. The document becomes dirty; no silent save.

The planner (candidate resolution + copy plan) lives in `src/storage/resources.rs` next to the existing import helpers; the action/prompt/status wiring lives in `src/app/`. The `wechat-workspace` crate stays GPUI-free and gains no new API beyond the parameter rename in D2.

Data flow against the caching invariant:

```
publish (read-only):  references ─▶ classify ─▶ resources / unresolved   (no cache touch, no version bump)
organize (mutating):  plan ─▶ confirm ─▶ copy files ─▶ one text splice   (single invalidation of derived caches)
```

## Risks / Trade-offs

- [Wider containment root lets a claimed session read referenced images outside the asset dir] → Exposure is exactly the images the document references at launch (enumerated into opaque IDs), still bearer-auth-gated, extension-whitelisted, canonicalized with symlink re-check at read time, and path-disclosure-free. A document saved directly under a wide directory (home, `Documents`) exposes no more than its referenced images.
- [Absolute-path organize copies large or odd files into the asset dir] → Same guards as the existing import path: extension whitelist, regular-file check, content-hash reuse. No size cap, matching current paste/drop import behavior.
- [Rewrite could corrupt unrelated text] → Parser-derived destination ranges only; single undo step restores the whole transformation; app tests cover MD syntax, inline HTML `src`, and a shared-URL non-image link that must not change.
- [Normalization mismatch between Rust classification and bridge.js matching (case, backslashes, percent-encoding)] → Both sides normalize identically today; tests pin `..\`, `./`, `%2e%2e`, uppercase-extension variants end to end through the snapshot payload.
- [Users may expect the preview to also fix WeChat pasting] → Rich copy still strips local images (loopback blobs are unpublishable); the spec and status wording keep calling that out rather than implying publication support.

## Migration Plan

Pure code change with no persisted-format impact; rollout and rollback are a normal release/revert. Spec deltas sync into `openspec/specs/` at archive time.

## Open Questions

- Should publishing that detects unresolved-but-organizable images surface a hint toward the new command (status-bar note only, no modal)? Deferred; does not affect the APIs or tasks above.
- Should the organize confirmation list each file individually versus a count? Deferred to implementation polish; the prompt API supports a detail string either way.
