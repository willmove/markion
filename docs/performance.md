# Measured performance · 实测性能

[English overview](../README.md#performance) · [中文概览](../README.zh-CN.md#performance)

These are **recorded November 11, 2026 results**, retained to make Markion's performance claims inspectable. They predate v0.6.0; they are not a fresh benchmark of that release. Lower times are better. KiB/MiB use 1,024-based units, and fixture sizes are approximate because complete Markdown sections are repeated.

以下是 **2026-11-11 的历史实测**，早于 v0.6.0，不能视作该发布版的重新测试。耗时越低越好；KiB/MiB 按 1,024 进制，混合文档由完整 Markdown 片段重复生成，因此大小为近似值。

## Environment and workload

| Item | Recorded environment |
| --- | --- |
| OS | Windows 11 Pro, build 10.0.26200 |
| CPU / RAM | Intel Core Ultra 5 225U, 12 cores / 14 threads, approximately 31 GiB physical memory |
| Graphics | Intel Graphics; VRAM was not measured separately |
| Build | Optimized Rust release build; the final OS-input record identifies its binary by SHA-256 |
| Document | Repeated headings, formatted prose, links, lists, quotes, fenced Rust code and GFM tables |
| Model sampling | 100 localized middle-of-document insertions per size, after cache warm-up; no concurrent compilation in the final run |
| OS-input sampling | 1,080 independently injected Windows key events in a mixed 1 MiB document; full saved-text equality checked at 180, 360, 720 and 1,080 inputs |

The [implementation report](fluent-editing-implementation.md#验证口径) records hardware and model method. The [machine-readable model and native-replay record](verification/fluent-editing-windows-2026-11-11.json) retains the per-size results. The [Windows system-input report](verification/windows-native-input-2026-11-11.md) records the later frame-scheduling correction and its complete history.

## Model work across document sizes

This measures **a source mutation plus the source-mapped Visual Edit model read**. It excludes GPUI element layout, paint, OS input delivery and GPU presentation; it does not measure frame time or full application memory.

| Mixed Markdown | Initial model build | Local edit P50 | Local edit P95 | Rebuilt / reused blocks, median |
| --- | ---: | ---: | ---: | ---: |
| 100 KiB | 10.77 ms | 0.64 ms | 1.21 ms | 4 / 2,121 |
| 300 KiB | 28.67 ms | 2.55 ms | 3.69 ms | 4 / 6,320 |
| 600 KiB | 55.03 ms | 5.81 ms | 6.86 ms | 11 / 12,586 |
| 1 MiB | 96.84 ms | 10.33 ms | 11.24 ms | 10 / 21,409 |

Markion shares derived Markdown through `Arc`, memoizes highlighting and reuses independently parseable unchanged regions. Uncertain or global semantics still trigger correctness-preserving full fallbacks. The text buffer remains a `String`; these figures do not imply constant-time edits for arbitrary documents.

本表只测量源码修改及可视化模型派生，不含 GPUI 排版绘制、系统输入传输或 GPU 呈现。复用不变区域可减少工作量；全局语义或无法确定的边界仍会回退完整派生，文本缓冲区仍为 `String`。

## System key events to caret paint

The later final run measures **the application input callback to the caret's paint checkpoint**, including model and UI work. It excludes hardware/OS delivery before the callback and GPU/display presentation after paint. The 1,080 events were sent by the official Computer Use tool, not a physical keyboard latency rig.

| 1 MiB mixed document | Result |
| --- | ---: |
| Already-running window: read file → first body paint | 167.07 ms |
| Input P50 / P95 / maximum | 26.63 / 33.14 / 47.95 ms |
| Resident / private memory after 1,080 inputs | 303.53 / 261.02 MiB |
| Resident / private memory after returning to a small document | 199.29 / 156.88 MiB |

[Summary and build identity](verification/fluent-editing-input-frame-final-2026-11-11.json) · [all final input samples](verification/fluent-editing-input-frame-after-2026-11-11.json) · [earlier results, including long-tail peaks](verification/windows-native-input-2026-11-11.md#连续输入发现的尾延迟).

该组是应用收到输入回调到光标 paint 的计时，包含模型和界面工作，但不等于硬件按键到显示器呈现的完整延迟。原始记录保留了修复前的峰值，没有用较快样本替换旧结果。

## Reproduce

From the repository root, in PowerShell 7 with Rust stable and the app's native build prerequisites installed:

```powershell
$env:BENCH_SIZES_KB = '100,300,600,1024'
$env:BENCH_STROKES = '100'
$env:BENCH_VISUAL_ONLY = '1'
cargo run --release --example bench_large_doc
```

For a native window replay with isolated app data and generated fixtures:

```powershell
pwsh ./scripts/Invoke-FluentEditingSmoke.ps1
```

The native script replays application callbacks and reports paint checkpoints; it does **not** reproduce the independent OS-key injection run. To repeat that experiment, use real system key events against a dedicated copy of the mixed 1 MiB fixture, set `MARKION_PROFILE_VISUAL_INPUT=1` before launch, keep concurrent compilation off, and compare the full saved text to the expected input sequence. See the [recorded procedure](verification/windows-native-input-2026-11-11.md).

Record your commit, binary hash, OS, CPU, RAM, build profile, fixture, workload and sample count alongside results. Do not mix model timings, native callback replay, and OS-input samples into one ranking. Image-heavy documents, very large single blocks, global references, other machines and other platforms require their own measurements.

## Comparisons

The README compares **Markion at different document sizes**. It does not rank Markion against quiet-md, Typora, Obsidian or another editor: their published results use different machines, document structures and timing boundaries. A fair competitor table requires the same machine, workload, build class and measurement stages, with raw samples for every application.

README 的性能对比是同一组 Markion 实测在不同文档大小下的对照。quiet-md 等产品的公开数据使用不同设备、文档和指标，因此不据此推导产品间的速度排名。
