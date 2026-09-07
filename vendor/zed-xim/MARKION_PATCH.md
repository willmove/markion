# Markion compatibility patch

This directory starts from the published crates.io package `zed-xim
0.4.0-zed` (MIT), checksum
`0c0b46ed118eba34d9ba53d94ddc0b665e0e06a2cf874cfa2dd5dec278148642`.
The unmodified upstream license is retained in `LICENSE`.

Markion carries three narrow changes until a compatible GPUI/XIM release
contains them:

- pin `xim-ctext` to `0.4.1`, which decodes the GB2312 and KS C 5601
  COMPOUND_TEXT designators used by Chinese and Korean XIM servers;
- return a typed `ClientError` for inbound compound-text decoding failures
  instead of panicking in reset, commit, or preedit handling, and log inbound
  request names without payload bytes or user text;
- cover those paths with deterministic request-boundary tests;
- make four elided `AttributeBuilder` output lifetimes explicit so the vendored
  package remains warning-free under Markion's current stable Rust toolchain.

The root `Cargo.toml` selects this package through `[patch.crates-io]`. Remove
that override and this directory, then regenerate `Cargo.lock`, once the GPUI
version used by Markion resolves an upstream package with equivalent behavior.
