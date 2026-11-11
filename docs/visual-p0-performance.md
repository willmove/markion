# Visual Edit P0：诊断与身份协调优化报告

## 范围

P0 对应 OpenSpec change `profile-visual-edit-hotspots`，目标是找出大文档 Visual Edit 局部输入后的主要成本，并在不改变文档模型、VisualBlock 内容或身份语义的前提下优化明确热点。

本阶段不做 rope 迁移、VisualBlock 区域模型重写、异步派生或任何 P1 交互变更。完整 VisualBlock 区域局部化及输入后副作用调度仍属后续工作。

## 诊断方法

在既有 `MarkdownDocument::visual_blocks_shared` 接缝内增加只读诊断 profile，区分：

- semantic update：语义区域更新、组装及必要的全量 fallback；
- VisualBlock build：由 preview blocks 构造全部 VisualBlock；
- identity reconcile：把上一版本的稳定身份映射到新块；
- identity candidates：经过 source-range 查找后实际检查的候选数。

普通调用仍返回原来的 `Arc<Vec<VisualBlock>>`。诊断调用仅用于基准和测试；缓存命中会返回同一 `Arc` 并标记 `cache_hit`。

基准命令支持以下环境变量，默认值保持原有 Visual 基准行为：

```bash
BENCH_SIZES_KB=100,300,600 BENCH_STROKES=20 BENCH_VISUAL_ONLY=1 \
  cargo run --release --example bench_large_doc
```

所有前后对比均使用相同混合 Markdown fixture、文档大小、插入位置和 20 次输入。数值是本次 Windows 机器上的模型层诊断，不是键盘到屏幕延迟，也不作为 CI 固定耗时门槛。

## 基线结果

优化前，身份协调会在每个旧块上反复扫描新块列表；候选检查随块数呈平方级增长，并且是该 fixture 中最大的单项成本。

| 文档 | 总耗时 P50 | semantic P50 | Visual build P50 | identity P50 | 候选检查 |
|---|---:|---:|---:|---:|---:|
| 100 KB | 10.23 ms | 1.02 ms | 3.77 ms | 4.46 ms | 4,519,864 |
| 300 KB | 81.81 ms | 2.96 ms | 14.44 ms | 60.84 ms | 40,005,596 |
| 600 KB | 686.69 ms | 12.29 ms | 86.65 ms | 561.96 ms | 158,709,580 |

600 KB 基线中，身份协调约占总耗时 82%。这证明 P0 应优先修复身份查找，而不是先更换文本容器或重写全部派生逻辑。

## 实现

### Source-range 索引

`src/source_mapped.rs` 现在一次建立 `Range<usize> -> Vec<usize>` 新块索引。向量保留重复/空范围的文档顺序，不以单值索引覆盖它们。两个身份协调阶段都只检查具有目标 source range 的候选；第二阶段继续按 kind 和 claimed 状态做唯一性判断。

旧身份是否已复用改用 matched-old 布尔向量记录，不再为每个旧块重新扫描新块寻找该身份。

### 零位移直接比较

当块没有字节位移且 quote-group 元数据一致时，直接比较现有块，避免仅为身份比较而克隆并平移整个 VisualBlock。非零位移或 quote-group 变化仍走原来的 clone-and-shift 证明。

### 语义保持不变

- 第一阶段仍只使用目标 range 的第一个候选，保持既有重复范围处理规则。
- 一对多或不明确的第二阶段匹配继续被拒绝，不把身份捐给无关块。
- 重复空范围新增回归用例验证：首个合法匹配保留身份，其余候选保持 fresh id，不发生错配。
- 现有随机增量/全量等价、UTF-8、容器、拆分/合并、kind change、直接编辑器和缓存命中测试继续作为正确性 oracle。

## 优化后结果

| 文档 | 总耗时 P50 | semantic P50 | Visual build P50 | identity P50 | 候选检查 |
|---|---:|---:|---:|---:|---:|
| 100 KB | 7.47 ms | 1.08 ms | 3.96 ms | 1.27 ms | 2,125 |
| 300 KB | 23.61 ms | 2.63 ms | 14.00 ms | 3.89 ms | 6,324 |
| 600 KB | 52.44 ms | 5.30 ms | 33.30 ms | 8.09 ms | 12,597 |

相对收益：

| 文档 | 总耗时下降 | identity 耗时下降 | 候选检查下降 |
|---|---:|---:|---:|
| 100 KB | 27.0%（1.37×） | 71.5%（3.51×） | 约 2,127× |
| 300 KB | 71.1%（3.47×） | 93.6%（15.63×） | 约 6,325× |
| 600 KB | 92.4%（13.10×） | 98.6%（69.43×） | 约 12,597× |

普通唯一范围文档现在每轮候选数等于块数，满足“每个旧块至多两次候选检查”的确定性上限。耗时仍受机器负载影响，因此语义与工作量断言比耗时倍数更重要。

## 验证

已完成的针对性验证：

- `cargo test --lib source_mapped`：25 个测试中，24 个首轮通过；新增的重复范围测试因初版测试预期错误而修正后单独复测通过，其余测试未出现实现回归。
- 大文档确定性测试：复用至少 499 个区域、至多重新解析 1 个区域、候选检查不超过旧块数的两倍、缓存命中复用同一 `Arc`。
- 重复空范围测试：验证文档顺序、claimed/ambiguity 保护和 fresh id 保留。
- `cargo check --example bench_large_doc` 与 release 基准均成功构建并运行。

完整 workspace 测试、格式化和 OpenSpec 验证结果见下文质量门禁记录。

## 剩余问题

P0 已消除当前最大热点，但没有把整个 Visual Edit 输入路径变成恒定或局部工作量：

1. 语义区域仍会进行全文 split、组装、克隆和偏移平移。
2. 每个文本版本仍会重建全部 VisualBlock，并扫描全文引用定义、逐块生成 inline runs。
3. 输入后的导航快照、搜索、预览及其他订阅者仍有独立失效成本。
4. 600 KB 的总耗时 P50 仍高于 60 fps 的 16.7 ms 帧预算；本阶段不能据此宣称大文档实时输入已达标。

后续应在独立变更中让区域缓存直接持有可复用的 VisualBlock 切片，并以 ChangeSet 按变更范围调度搜索、预览和几何状态。只有 profile 证明 `String.replace_range` 成为主要瓶颈后，才值得评估 rope 迁移。

## 质量门禁

- `cargo test --workspace`：全部通过。根库 723 passed / 1 ignored，应用 768 passed / 2 ignored；其余 workspace crate、集成测试和 doctest 也全部通过。
- `cargo check --release --lib`：通过。
- `rustfmt --check`（仅本阶段 Rust 文件）：通过。`cargo fmt --all` 会顺带改写与本阶段无关的既有格式漂移，已撤回这些无关改动，没有把它们混入 P0。
- `git diff --check`：通过。
- `openspec validate profile-visual-edit-hotspots`：通过。
- 构建日志存在既有依赖 `proc-macro-error2` 的 future-incompat 提示，以及与本阶段无关的既有测试 import 警告；未发现 P0 新增警告。

本阶段没有人工 GPUI 键盘到屏幕测量；该验证需按 P1 完成后提供的人工实机步骤另行执行。
