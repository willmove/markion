// Run without GPUI dependencies to isolate the counter algorithm and benchmark.
// rustc --edition 2024 --test tests/stats_standalone.rs -o stats-tests
// rustc --edition 2024 -O tests/stats_standalone.rs -o stats-bench
// Run the resulting binaries (use .exe output names on Windows).
mod model {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DocumentBasicStats {
        pub bytes: usize,
        pub chars: usize,
        pub words: usize,
        pub lines: usize,
    }
}
#[path = "../src/incremental_stats.rs"]
mod incremental_stats;

fn full(text: &str) -> model::DocumentBasicStats {
    model::DocumentBasicStats {
        bytes: text.len(),
        chars: text.chars().count(),
        words: text.split_whitespace().count(),
        lines: text.lines().count().max(1),
    }
}

fn main() {
    use std::{hint::black_box, time::Instant};
    // Same source and logical edit; measures statistics only, not GUI latency.
    for mib in [1, 10] {
        let source = "日本語 **本文** word\r\n".repeat(mib * 1024 * 1024 / 28);
        let old = full(&source);
        let at = source.char_indices().nth(20).unwrap().0;
        let mut edited = source.clone();
        edited.insert_str(at, "入力 ");
        let start = Instant::now();
        for _ in 0..100 {
            black_box(full(black_box(&edited)));
        }
        let baseline = start.elapsed().as_secs_f64() * 1e6 / 100.0;
        let start = Instant::now();
        for _ in 0..100_000 {
            black_box(incremental_stats::after_edit(
                black_box(&source),
                at..at,
                black_box("入力 "),
                old,
            ));
        }
        let changed = start.elapsed().as_secs_f64() * 1e6 / 100_000.0;
        assert_eq!(
            incremental_stats::after_edit(&source, at..at, "入力 ", old),
            full(&edited)
        );
        println!(
            "bytes={} full_us={baseline:.3} incremental_us={changed:.3}",
            source.len()
        );
    }
}
