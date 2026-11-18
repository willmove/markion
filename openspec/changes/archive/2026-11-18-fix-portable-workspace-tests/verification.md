# Verification

2026-11-18, macOS Apple Silicon.

- `cargo test --workspace --no-fail-fast --quiet`: 2,253 passed, 0 failed, 5 already ignored, and 0 filtered, using the host's unchanged temporary-directory configuration.
- Both PDF integration tests passed with installed system fonts. Text extraction from the sample PDF confirmed the original mixed CJK/Latin heading, Chinese body text, and Chinese footnote.
- The synthetic PDF regression exercised rejection of an invalid font face and checked that the exported PDF embeds the compatible bundled replacement.
- `cargo fmt --all -- --check`, `git diff --check`, and strict validation of this change passed.
