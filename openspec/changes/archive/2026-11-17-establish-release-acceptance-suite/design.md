## Context

See proposal.md for motivation. Existing Rust/GPUI gates and specialist evidence remain authoritative at their own layers. Several active changes alter the current UI without having synchronized stable specs; the suite must name that baseline and avoid obsolete P1 expectations. This design is needed for fixture safety, cross-platform isolation, and evidence attribution; application rendering and caches are untouched.

## Goals / Non-Goals

**Goals:** Give human and Codex operators the same observable contract, reproducible inputs, byte comparisons, explicit execution variants, and honest release decisions.

**Non-Goals:** Native UI automation implementation, real producer qualification, changing the updater trust model, or proving every case by writing this documentation.

## Decisions

1. Use a Chinese entry point and eight domain case files with retained D/E/R/W/X/G/A/P identifiers. Each case has explicit setup, steps, expectations and evidence; parameterized branches become separate execution rows. This stays readable compared with one enormous checklist or historical change folders.
2. Keep static synthetic Markdown/HTML/text and a standard-library Python fixture utility together. The utility creates a fresh run, independent baselines, LF/CRLF/EOF variants, local raster/data-URI images, synthetic DOCX, scratch Git remotes, AI notes, and large files. It refuses existing destinations; comparisons return failure on unequal bytes. Actual Word/WPS/LibreOffice corpus files must be supplied with provenance, not fabricated as compatibility proof.
3. Use a report template, per-variant CSV and a blank corpus manifest. Distinguish NOT_RUN, PASS, FAIL, BLOCKED, SKIP and N/A and native OS events versus callback replay. Record hashes and dirty-source diff identity; resets use a new run, not destructive cleanup of shared work.
4. Document Windows process-local APPDATA/LOCALAPPDATA and Unix XDG isolation, with an observable startup check. Use a separately copied candidate binary and preserve package resource layout. macOS uses a dedicated test account for actual installed/signing acceptance and its HOME-based log path. Isolation paths must be verified against current `src/paths.rs` behavior before tests continue.
5. Integrate candidate acceptance in release step 4 and final-tag checks in steps 6/8. Keep existing manual MarkNice gates and release-specific exceptions explicit. No new CI native-pass claim is introduced.

## Risks / Trade-offs

- UI wording and active deltas drift → name reviewed baseline, use menu actions rather than stale shortcuts, require suite updates with behavioral changes.
- Synthetic DOCX looks like producer qualification → label it synthetic and keep real-producer rows NOT_RUN until supplied.
- Isolation differs by OS → verify emitted logs/config location before mutation; require dedicated test account when redirection is insufficient.
- Docs can be internally valid while native tests are unavailable → completion means the suite is delivered, never that release acceptance passed.

## Migration Plan

Add the suite and references without deleting historical reports. Verify links, all case IDs and record rows, fixture determinism and comparison failures, then validate/archive this change. Revert only these documentation/tooling additions if needed; no user data or application migration is involved.
