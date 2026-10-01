## Context

The audit-fix branch changed image-cache keys from full data URIs to bounded sampled fingerprints. That removed multi-megabyte cloning and hashing from repaint paths, but equal-length data URIs that differ outside the three sampled regions deterministically share a key and therefore a raster. The branch also hardened hand-built YAML strings, yet single-element custom containers and multiline title overrides still produce invalid or lossy front matter. A helper-level trailing-subscript fix does not reach the default parser because `pulldown-cmark` can split the authored run into adjacent text events when strikethrough is enabled.

The solution crosses the root GPUI application and the GPUI-free `markdown` and `export` workspace members. It must preserve the document-version caching invariant, bounded repaint work, the image cache's claim/pending/complete lifecycle, CommonMark/GFM precedence, and the existing no-GPUI member-crate boundary.

Current and proposed image flow:

```text
canonical Markdown source
  -> per-version preview/visual derivation
       -> extract image URL
       -> compute complete data-URI SHA-256 identity once per distinct source
       -> store ImageSourceIdentity with the derived image model
  -> repaint/claim collection
       -> reuse ImageSourceIdentity (constant-size key; no payload scan/clone)
  -> pending decode
       -> retain one Arc<str> payload until completion
       -> ready/error cache entry keyed by the complete identity
```

## Goals / Non-Goals

**Goals:**

- Guarantee that distinct data-URI source bytes do not deliberately alias because of sampling.
- Keep key construction and claim reconciliation bounded on every repaint.
- Produce valid, semantically round-trippable YAML for the complete front-matter value space supported by `YamlFrontMatter`.
- Route PDF/DOCX pandoc title overrides through the same YAML implementation.
- Recognize a trailing `~subscript~` through the default parser while retaining strikethrough and escaped-delimiter behavior.
- Keep malformed or stale UTF-8 ranges contained without overstating the reachability of the original Callout hypothesis.

**Non-Goals:**

- Changing image-cache capacity, decode concurrency, raster sizing, or source-elision presentation.
- Adding a second Markdown parser or a general-purpose front-matter editor.
- Changing the meaning of GFM strikethrough or Markion's extended-inline delimiters.
- Addressing full-document Visual Edit fallback, editor shaping, or synchronous file saves.

## Decisions

### 1. Use a complete SHA-256 digest computed during per-version derivation

Introduce a small `ImageSourceIdentity` carried by derived block/inline image models. Local and remote sources retain their normalized locator identity. Data URIs carry their byte length and SHA-256 digest of the complete URI. A derivation-local interner avoids hashing the same distinct source more than once in one document version. HTML image extraction must produce the same descriptor and be cached with the derived HTML presentation rather than rebuilding the digest during paint.

The root crate will use the existing workspace `sha2` dependency directly. `PreviewImageKey` will accept the precomputed identity; repaint collection, claim reconciliation, entry lookup, and failure lookup will not read the full data URI. The existing `Arc<str>` payload side table remains responsible for the one payload copy needed by a newly reserved decode and removes it on completion/removal.

Alternatives considered:

- Hash every URI in `PreviewImageKey::from_url`: correct, but restores O(payload) work on repaint.
- Retain the full URI in every cache key: correct, but restores large key cloning and retention.
- Increase or randomize samples: still permits deterministic aliases outside sampled regions.
- Use only a 64-bit complete hash: much better than sampling but needlessly weaker when SHA-256 is already available in the workspace.

### 2. Serialize the complete front-matter object, never individual YAML fragments

`crates/markdown` will own one canonical helper that serializes `YamlFrontMatter` as a complete YAML mapping and wraps it in Markdown front-matter delimiters. `render_to_markdown` will call that helper rather than selecting scalar/container layout manually. This naturally handles single-element sequences and mappings, multiline/control characters, YAML-looking strings, Unicode separators, and nested values.

DOCX and PDF pandoc input construction will clone or build metadata, apply `title_override` to the typed `YamlFrontMatter`, and invoke the canonical Markdown renderer. The export crate will remove `escape_yaml_string` and line-oriented front-matter surgery. This favors semantic correctness over preserving the original YAML formatting in an intermediate export-only document; canonical source in the editor remains unchanged.

Alternatives considered:

- Expand the manual escape table: fragile because scalar escaping does not solve container indentation or YAML type resolution.
- Branch on `serde_yaml::Value` while keeping hand-built lines: possible, but duplicates the serializer's structural rules and leaves export overrides on a separate path.

### 3. Normalize only proven extended-inline candidates across adjacent text events

The Markdown parser will add a narrow adapter around contiguous text events. It may combine event fragments only when original source offsets prove an unescaped single-tilde opener and closer belonging to one text run. It must reject `~~` delimiters, escaped markers, and boundaries across code, links, emphasis, HTML, breaks, or other semantic events. The normal extended-inline parser remains the owner after a candidate run is reconstructed.

The implementation begins with event-shape characterization tests for default `pulldown-cmark` options. If offset provenance cannot prove the candidate without broad parser restructuring, the implementation must leave it literal rather than guess.

Alternatives considered:

- Concatenate all adjacent text events: simple, but can reinterpret escaped markers whose backslash has already been consumed.
- Disable GFM strikethrough: fixes the event shape by breaking required syntax.

### 4. Treat caret clamps as invariant containment

Keep boundary clamping and safe selection consumers. Rename or rewrite synthetic Callout tests so they establish the general contract—stale, out-of-range, or mid-codepoint offsets do not panic—without presenting a synthetic block as proof of an ordinary user path. No new Callout syntax is introduced.

## Risks / Trade-offs

- [SHA-256 derivation adds O(payload) work when a document version is derived] → Hash once per distinct source per version, reuse through all paint/cache paths, and gate repaint counters rather than elapsed time.
- [Adding identity fields touches several preview and visual model constructors] → Use one typed identity and exhaustive compiler-guided propagation; retain URL fields for decoding and display.
- [Canonical YAML serialization may reorder custom mapping keys or change formatting in export input] → Assert semantic round trips; export input is transient and the editor's canonical source is not rewritten.
- [Selective event reconstruction may miss unusual valid subscript forms] → Cover leading/middle/trailing, Unicode, escaped, and strikethrough fixtures; preserve literal text whenever provenance is ambiguous.
- [This change overlaps an active data-URI change] → Document that this change supersedes only task 5.1's implementation assumption while retaining its user-visible elision and bounded-frame requirements.

## Migration Plan

1. Land model and derivation support for complete image identities with compatibility constructors updated in one commit.
2. Switch all cache/claim/decode call sites to the precomputed identity and remove sampled hashing.
3. Replace YAML fragment writers and export override surgery with canonical typed serialization.
4. Complete the parser event-boundary fix and reframe the UTF-8 containment tests.
5. Run targeted collision/round-trip/parser tests, then the full repository quality gate.

Rollback is a normal branch revert before merge; there is no persisted cache format or user-data migration. Runtime image caches are rebuilt on launch.

## Open Questions

- Confirm during implementation whether preview HTML parts already have a per-version cache seam; if not, add the narrowest derived HTML-image descriptor cache needed to keep data-URI hashing out of paint.
- Confirm the exact `pulldown-cmark` offset-event sequence for escaped and trailing single-tilde candidates before choosing the smallest reconstruction helper.
