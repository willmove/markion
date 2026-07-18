## 1. Current Contract and Documentation

- [x] 1.1 Add the Visual Edit support/fallback and parser-ownership matrix with canonical ranges and verification evidence
- [x] 1.2 Update the English and Simplified Chinese READMEs for direct code, math, image, and table editing plus current incremental behavior
- [x] 1.3 Link both READMEs to the detailed matrix and document the one-command contributor quality gate
- [x] 1.4 Correct stale OpenSpec project context and `markdown-editing` purpose metadata through this tracked change

## 2. Executable Quality Gates

- [x] 2.1 Add `scripts/check-quality.ps1` with fail-fast format, workspace-test, and strict OpenSpec gates
- [x] 2.2 Add a dedicated pull-request/main quality workflow with pinned Rust, Node, and OpenSpec setup
- [x] 2.3 Keep packaging isolated in the release workflow and document intentionally ignored external tests
- [x] 2.4 Add a documentation contract test that covers every direct editor and conservative fallback family

## 3. Deterministic Visual Edit Discipline

- [x] 3.1 Inventory semantic parser, exact boundary helper, table mutation, member parser, and exporter ownership
- [x] 3.2 Ensure localized large-document tests bound parsed regions, prove reuse/stable identity, and compare against fresh full derivation
- [x] 3.3 Ensure interaction-only direct-widget tests preserve document version and shared derived-cache identity
- [x] 3.4 Update the large-document benchmark description and cases for the incremental source-mapped Visual Edit model
- [x] 3.5 Add proposal/review guidance requiring matrix, fallback, UTF-8 range, IME/history, and cross-crate evidence for future visual strategies

## 4. Verification and Archive

- [x] 4.1 Run focused documentation, source-mapped, direct-widget, and quality-script tests
- [x] 4.2 Run `cargo fmt --all -- --check` and `cargo test --workspace`
- [x] 4.3 Run `openspec validate --all --strict --no-interactive` and resolve every error
- [x] 4.4 Archive the completed change and verify its delta specs and metadata are current
