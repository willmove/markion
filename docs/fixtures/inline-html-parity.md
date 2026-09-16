# 行内 HTML 跨模式回归样例

**目  录**

[<u>第一章 项目概述</u>    4](#_toc232450996)

前 <span style="color:#c00"><b title="a > b">红色粗体</b> <font color="#08c">蓝色</font> 红色恢复</span> 普通。

前 <a href="https://example.com/?a=1&amp;b=2"><u>HTML 链接</u></a> 后。

前 <a href="https://example.com"><img src="missing-badge.png" alt="可点击图片占位"></a> 后。

前 <kbd>Ctrl</kbd> + <samp>输出</samp>，<span class="metadata" data-kind="example">普通容器</span>。

前 <span style="font-weight:bold;font-style:italic;text-decoration:underline;color:rgb(20,80,120)">组合样式</span> 后。

X<sup>上标</sup>，H<sub>下标</sub>，<mark>高亮</mark>，<ins>插入</ins>。

## 标题 <u>下划线</u> X<sup>2</sup>

- 列表 <font color="#c00">颜色</font> H<sub>2</sub>O

> 引用 <kbd>Ctrl</kbd> <a href="https://example.com">链接</a> X<sup>2</sup>

| GFM 单元格 | 相邻单元格 |
| --- | --- |
| <u>下划线</u> <mark>高亮</mark> <span style="color:#c00">红色</span> H<sub>2</sub>O X<sup>2</sup> | 普通 |
| <br>第一行<br><br>第三行<br> | 换行 |

<p>HTML 段落 <u>下划线</u> <span style="color:#c00">红色</span> <mark>高亮</mark> H<sub>2</sub>O X<sup>2</sup></p>

<table><tr><td><u>下划线</u> <span style="color:#c00">红色</span> <mark>高亮</mark> H<sub>2</sub>O X<sup>2</sup></td><td>普通</td></tr><tr><td><br>第一行<br><br>第三行<br></td><td><b><a href="https://example.com">未闭合样式</td><td>无继承污染</td></tr></table>

<br>第一行<br><br>第三行<br>

\*\*字面星号\*\*，\<u>字面标签\</u>，&lt;span&gt;实体标签&lt;/span&gt;，`<u>代码</u>`。

前 <em>错配</i> 后。前 <span style="position:fixed">未支持布局属性保留源码</span> 后。
