## 1. Diagnostic baseline

- [x] 1.1 Add the diagnostic VisualBlock profile and selectable benchmark sizes/stroke counts without changing the ordinary cache contract
- [x] 1.2 Run and record a short baseline with stage timings and identity-candidate checks

## 2. Indexed identity reconciliation

- [x] 2.1 Replace repeated whole-list lookups with occurrence-safe source-range indexes and matched-old tracking
- [x] 2.2 Avoid zero-delta clones when existing metadata already proves direct comparison is valid

## 3. Regression evidence

- [x] 3.1 Add deterministic candidate-work and duplicate/empty-range tests
- [x] 3.2 Run focused source-mapped, Visual Edit identity, and full-derivation equivalence tests

## 4. Comparative diagnostics and report

- [x] 4.1 Run the same benchmark shapes after optimization and document stage timings, candidate counts, and residual costs
- [x] 4.2 Write `docs/visual-p0-performance.md` with scope, baseline, implementation, verification, and follow-up boundaries

## 5. Quality gate

- [x] 5.1 Run formatting, the complete workspace test suite, and `openspec validate profile-visual-edit-hotspots`
- [x] 5.2 Review generated files and repository status; record P0 verification and unresolved manual GUI measurements
