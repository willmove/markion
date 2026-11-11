# P3 可视化编辑实机验证

普通段落，包含 **粗体**、*斜体*、[链接](https://example.com)、中文、emoji 😀 和组合字符 é。

第二段用于多块选择、复制、移动、删除和一步撤销。

- first sibling
- parent subtree
  - 子项目一
  - 子项目二

  ```rust
  fn nested() {
      println!("子树中的代码");
  }
  ```

- last sibling

> 前面的引用段落。
>
> 将这个中间段落转换为普通正文，检查两侧引用容器。
>
> 后面的引用段落。

| 名称 | 类型 | 数值 |
| :--- | :---: | ---: |
| 第一格 | **粗体** | 10 |
| 第二格 | emoji 😀 | 20 |
| 最后一行 | Tab 加行 | 最后一格 |

```rust
fn fold_me() {
    let message = "折叠代码后，点击展开或搜索导航";
    println!("{message}");
}
```

![Markion](../../assets/markion.png "图片尺寸与对齐测试")

结尾段落，检查滚动、光标和撤销后内容没有丢失。
