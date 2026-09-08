## 1. Reproduction

- [x] 1.1 Add and run a deterministic GPUI reproduction of short-document typewriter jitter in a small window.
- [x] 1.2 Minimize the failing sequence and record tested hypotheses and the root cause.

## 2. Fix

- [x] 2.1 Correct the responsible layout/scroll interaction while preserving per-version caches and normal scroll behavior.
- [x] 2.2 Cover newline, composition, wrapped input, and unchanged-frame stability with regression tests.

## 3. Verification

- [x] 3.1 Run focused scrolling tests and the root package tests; record any environment limitations.
- [x] 3.2 Validate the OpenSpec change and document evidence and remaining manual verification.

## 4. Follow-up: Repeated Enter

- [x] 4.1 Reproduce consecutive Enter presses without intervening text and capture the alternating viewport state.
- [x] 4.2 Correct the repeated-empty-line scroll behavior and add focused regression coverage.
- [x] 4.3 Re-run typewriter and root-package tests and update verification evidence.
