//! Ad-hoc benchmark: per-keystroke cost of Markion's markdown derive pipeline on
//! large documents.
//!
//! This measures the CPU work that a single keystroke forces the render path to
//! redo: the three memoized derived values (`preview_blocks_shared`, `outline`,
//! `stats`) all get invalidated by any text mutation and recomputed the next
//! time they are read. That recompute is a *full-document* pulldown-cmark parse
//! today, and it is exactly the cost an incremental parser could reduce.
//!
//! What this does NOT measure: GPUI's re-render of the preview element tree
//! (element construction + layout + paint). That happens on the UI thread and
//! cannot be exercised headlessly here. For large documents the render cost is
//! typically the larger term — see the notes printed at the end.
//!
//! Run with: `cargo run --release --example bench_large_doc`

use std::time::Instant;

use markion::MarkdownDocument;

/// Build a representative Markdown document of roughly `target_bytes` bytes by
/// repeating a mixed section (headings, paragraphs with inline styles, lists,
/// a code block, and a table) — the kind of content the parser actually walks.
fn make_doc(target_bytes: usize) -> String {
    let section = "\
## Section heading number {n}

This is a paragraph with **bold**, *italic*, `inline code`, and a [link](https://example.com/{n}). \
Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore \
et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation.

- First bullet with some text {n}
- Second bullet with `code` {n}
- Third bullet with a [link](https://example.com/list/{n})

1. Ordered item one {n}
2. Ordered item two {n}

> A block quote paragraph number {n} with **emphasis** inside it to exercise inline parsing.

```rust
fn compute_{n}(x: i64) -> i64 {
    let mut acc = 0;
    for i in 0..x { acc += i * {n}; }
    acc
}
```

| Name {n} | Kind | Score |
|----------|------|-------|
| alpha    | a    | 1     |
| beta     | b    | 22    |
| gamma    | c    | 333   |

";
    let mut out = String::with_capacity(target_bytes + section.len());
    let mut n = 0usize;
    while out.len() < target_bytes {
        out.push_str(&section.replace("{n}", &n.to_string()));
        n += 1;
    }
    out
}

/// Time N simulated keystrokes: each inserts one char near the middle of the
/// document, then reads the three derived values (as a render would), forcing a
/// full recompute. Returns per-keystroke durations in microseconds.
fn bench_keystrokes(base: &str, strokes: usize) -> Vec<f64> {
    let mut doc = MarkdownDocument::from_text(base);
    // Warm the caches once so the first measured stroke is not paying for the
    // very first parse of an otherwise-cold document.
    let _ = doc.preview_blocks_shared();
    let _ = doc.outline();
    let _ = doc.stats();

    let mut samples = Vec::with_capacity(strokes);
    for i in 0..strokes {
        // Insert at a char boundary near the middle of the current text.
        let mut mid = doc.text().len() / 2;
        while !doc.text().is_char_boundary(mid) {
            mid += 1;
        }
        let ch = if i % 10 == 0 { "x" } else { "a" };

        let start = Instant::now();
        doc.insert(mid, ch); // bumps text_version, invalidates all derived caches
        let _ = doc.preview_blocks_shared(); // full parse #1 (the heavy one)
        let _ = doc.outline(); // full parse #2
        let _ = doc.stats(); // full parse #3
        let elapsed = start.elapsed();

        samples.push(elapsed.as_secs_f64() * 1_000_000.0);
    }
    samples
}

fn report(label: &str, bytes: usize, samples: &mut [f64]) {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = samples.len();
    let sum: f64 = samples.iter().sum();
    let mean = sum / n as f64;
    let median = samples[n / 2];
    let p95 = samples[(n as f64 * 0.95) as usize];
    let min = samples[0];
    let max = samples[n - 1];
    println!(
        "{label:<26} size={:>6} KB  strokes={n:>4}  \
         median={median:>8.1}us  mean={mean:>8.1}us  p95={p95:>8.1}us  min={min:>7.1}us  max={max:>8.1}us",
        bytes / 1024
    );
}

fn main() {
    println!("Markion large-document keystroke benchmark (release build recommended)\n");
    println!("Measures per-keystroke recompute of preview_blocks + outline + stats");
    println!("(the full-document parse an incremental parser could shrink).\n");

    for target in [100 * 1024, 300 * 1024, 600 * 1024, 1024 * 1024] {
        let doc = make_doc(target);
        let bytes = doc.len();
        let mut samples = bench_keystrokes(&doc, 300);
        report("mixed markdown", bytes, &mut samples);
    }

    // Phase 0 result: in Edit mode the app no longer reads preview_blocks, and a
    // collapsed sidebar no longer reads the outline. Model that mode by editing
    // WITHOUT reading any derived value — this is the per-keystroke markdown-
    // pipeline cost after Phase 0 for an Edit-mode / collapsed-sidebar user.
    println!("\nPhase 0 (Edit mode / sidebar collapsed — no derived reads):");
    for target in [300 * 1024, 600 * 1024, 1024 * 1024] {
        let doc = make_doc(target);
        let bytes = doc.len();
        let mut d = MarkdownDocument::from_text(&doc);
        let mut samples = Vec::with_capacity(300);
        for i in 0..300 {
            let mut mid = d.text().len() / 2;
            while !d.text().is_char_boundary(mid) {
                mid += 1;
            }
            let start = Instant::now();
            d.insert(mid, if i % 10 == 0 { "x" } else { "a" });
            samples.push(start.elapsed().as_secs_f64() * 1_000_000.0);
        }
        report("edit-mode (phase 0)", bytes, &mut samples);
    }

    // Isolate the single heaviest derived value (preview blocks) so we can see
    // how much of the per-keystroke cost is the preview parse alone.
    println!("\nPreview-blocks-only (no outline/stats):");
    for target in [300 * 1024, 600 * 1024] {
        let doc = make_doc(target);
        let bytes = doc.len();
        let mut d = MarkdownDocument::from_text(&doc);
        let _ = d.preview_blocks_shared();
        let mut samples = Vec::with_capacity(300);
        for i in 0..300 {
            let mut mid = d.text().len() / 2;
            while !d.text().is_char_boundary(mid) {
                mid += 1;
            }
            let start = Instant::now();
            d.insert(mid, if i % 10 == 0 { "x" } else { "a" });
            let _ = d.preview_blocks_shared();
            samples.push(start.elapsed().as_secs_f64() * 1_000_000.0);
        }
        report("preview only", bytes, &mut samples);
    }

    println!(
        "\nNote: a 60 fps frame budget is ~16600us. Values well under that mean the\n\
         parse is not the bottleneck for that document size; the GPUI preview\n\
         re-render (not measured here) dominates instead."
    );
}
