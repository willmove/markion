## Context

The prior brand appears in Rust package metadata, application and bundle identifiers, file-system paths, generated resources, asset names, documentation, and OpenSpec records. A case-sensitive rename must update every product consumer together so that the built application and its installers present a single Markion identity.

## Goals / Non-Goals

**Goals:**

- Replace every prior product identifier with its matching Markion casing.
- Rename icon assets and repair all source, build, packaging, and documentation references.
- Confirm no prior case-insensitive product reference remains and preserve build/test behavior.

**Non-Goals:**

- Redesigning the logo artwork, changing application behavior, or preserving backwards compatibility with prior bundle/configuration paths.
- Preserving legacy names in historical records.

## Decisions

- Perform a case-preserving textual rename across tracked project files, then inspect every remaining case-insensitive legacy match. This provides broad coverage while preventing unintended casing changes.
- Rename the canonical asset files rather than keeping legacy-named aliases. The specifications require a single canonical source, and aliases would perpetuate the former product name.
- Update active OpenSpec specs through delta specifications, then archive the completed change so current specifications are synchronized.

## Risks / Trade-offs

- [Generated or binary files may retain stale names or content] → Regenerate icon outputs from the renamed source where supported and inspect filenames plus references.
- [An external consumer may rely on a legacy bundle/configuration path] → Treat the identifier change as breaking and record it in the proposal; no compatibility aliases are introduced.
- [Broad replacement can affect unrelated prose] → Review all remaining matches and run Cargo/OpenSpec validation after changes.

## Migration Plan

1. Rename code, metadata, assets, documentation, and active specifications.
2. Rebuild and run the test suite to verify the Markion package and resource paths.
3. Deliver the new application and installers as Markion; previous local application data is not migrated by this change.

## Open Questions

None.
