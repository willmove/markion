# Markion patch

This directory vendors `cc` 1.2.65 from crates.io.

On Windows with MSVC 14.44, compiling multiple native objects concurrently
intermittently fails with C1056 ("cannot update the time date stamp field")
even though every object has a distinct output path. Markion's dependency graph
enables cc-rs's `parallel` feature through `libgit2-sys`, which makes unrelated
native dependencies such as `onig_sys`, `libz-sys`, and `ring` hit the same
compiler failure.

The local patch keeps cc-rs parallel compilation on other toolchains and makes
only MSVC object compilation sequential inside each build script. It also omits
cc-rs's undocumented `/Brepro` flag for Microsoft's `cl.exe` 19.44, because the
same C1056 still occurs sequentially while cl.exe rewrites the deterministic
timestamp field. `clang-cl` retains `/Brepro`. Cargo remains free to compile
Rust crates and independent build scripts in parallel.
