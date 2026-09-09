use crate::html_to_markdown;

fn md(html: &str) -> String {
    html_to_markdown(html)
}

#[test]
fn headings_and_paragraphs() {
    assert_eq!(
        md("<h1>Title</h1><p>First</p><h2>Sub</h2><p>Second</p><h6>Deep</h6>"),
        "# Title\n\nFirst\n\n## Sub\n\nSecond\n\n###### Deep"
    );
}

#[test]
fn empty_blocks_produce_nothing() {
    assert_eq!(md("<p></p><p> </p><div><p>x</p></div>"), "x");
}

#[test]
fn inline_formatting_tags() {
    assert_eq!(
        md("<p><strong>a</strong> <em>b</em> <s>c</s> <u>d</u> <sub>e</sub> <sup>f</sup></p>"),
        "**a** *b* ~~c~~ <u>d</u> <sub>e</sub> <sup>f</sup>"
    );
    assert_eq!(
        md("<p><b>a</b> <i>b</i> <strike>c</strike> <del>d</del></p>"),
        "**a** *b* ~~c~~ ~~d~~"
    );
}

#[test]
fn empty_wrappers_are_skipped() {
    assert_eq!(md("<p>a<b> </b><i></i>c</p>"), "a c");
}

#[test]
fn adjacent_markers_merge() {
    assert_eq!(
        md("<p><b>a</b><b>b</b> <i>x</i><i>y</i> <s>m</s><s>n</s></p>"),
        "**ab** *xy* ~~mn~~"
    );
    assert_eq!(md("<p><b>a</b> <b>b</b></p>"), "**a b**");
}

#[test]
fn style_attribute_formatting() {
    assert_eq!(
        md(
            "<p><b style=\"font-weight:normal\"><span style=\"font-weight:700\">Bold</span></b> <span style=\"font-style:italic\">It</span> <span style=\"text-decoration:underline\">U</span> <span style=\"text-decoration:line-through\">S</span> <span style=\"font-weight:bold;font-style:italic\">BI</span></p>"
        ),
        "**Bold** *It* <u>U</u> ~~S~~ ***BI***"
    );
    assert_eq!(
        md("<p><span style=\"text-decoration: underline line-through\">x</span></p>"),
        "~~<u>x</u>~~"
    );
    assert_eq!(
        md(
            "<p><span style=\"font-weight:600\">w</span> <span style=\"font-weight:400\">n</span></p>"
        ),
        "**w** n"
    );
    assert_eq!(
        md("<p><u style=\"text-decoration:none\">plain</u></p>"),
        "plain"
    );
}

#[test]
fn word_fixture_cleans_up() {
    let html = "<!--[if gte mso 9]><xml><o:OfficeDocumentSettings><o:AllowPNG/></o:OfficeDocumentSettings></xml><![endif]-->\n\
        <p class=MsoNormal><b style=\"mso-bidi-font-weight:bold\">Hello</b> <i>World</i><o:p></o:p></p>\n\
        <p class=MsoNormal style=\"margin-left:36.0pt\">Second <span style=\"mso-bidi-font-weight:bold\">Mso</span><o:p>&nbsp;</o:p></p>";
    assert_eq!(md(html), "**Hello** *World*\n\nSecond **Mso**");
}

#[test]
fn links_keep_safe_targets() {
    assert_eq!(
        md("<p><a href=\"https://example.com\">ok</a></p>"),
        "[ok](https://example.com)"
    );
    assert_eq!(
        md("<p><a href=\"http://example.com/x y(1)\">t</a></p>"),
        "[t](http://example.com/x%20y%281%29)"
    );
    assert_eq!(
        md("<p><a href=\"mailto:a@b.c\">mail</a></p>"),
        "[mail](mailto:a@b.c)"
    );
    assert_eq!(
        md("<p><a href=\"ftp://files.example.com/x\">f</a></p>"),
        "[f](ftp://files.example.com/x)"
    );
    assert_eq!(
        md("<p><a href=\"//cdn.example.com/x\">sr</a></p>"),
        "[sr](//cdn.example.com/x)"
    );
    assert_eq!(
        md("<p><a href=\"page.html\">rel</a></p>"),
        "[rel](page.html)"
    );
    assert_eq!(md("<p><a href=\"#frag\">f</a></p>"), "[f](#frag)");
}

#[test]
fn links_drop_unsafe_targets_to_text() {
    assert_eq!(md("<p><a href=\"javascript:alert(1)\">bad</a></p>"), "bad");
    assert_eq!(md("<p><a href=\"data:text/html,<b>x</b>\">d</a></p>"), "d");
    assert_eq!(md("<p><a href=\"vbscript:evil\">v</a></p>"), "v");
}

#[test]
fn links_with_empty_href_render_text_only() {
    assert_eq!(
        md("<p><a href=\"\">empty</a> <a>plain</a></p>"),
        "empty plain"
    );
}

#[test]
fn link_text_falls_back_to_url() {
    assert_eq!(
        md("<p><a href=\"https://example.com\"></a></p>"),
        "[https://example.com](https://example.com)"
    );
}

#[test]
fn link_label_escapes_brackets_in_fallback() {
    assert_eq!(
        md("<p><a href=\"https://example.com/a[1]\"></a></p>"),
        "[https://example.com/a\\[1\\]](https://example.com/a[1])"
    );
}

#[test]
fn images() {
    assert_eq!(
        md(
            "<p><img src=\"https://x.com/a.png\" alt=\"pic\"> <img src=\"file:///tmp/a.png\" alt=\"f\"> <img src=\"rel/q.png\" alt=\"r\"></p>"
        ),
        "![pic](https://x.com/a.png) ![f](file:///tmp/a.png) ![r](rel/q.png)"
    );
}

#[test]
fn image_data_uri_reduces_to_alt_text() {
    assert_eq!(
        md("<p><img src=\"data:image/png;base64,iVBORw0KGgoAAAANSUhEUg==\" alt=\"dot\"></p>"),
        "dot"
    );
    assert_eq!(md("<p><img src=\"data:image/png;base64,AAAA\"></p>"), "");
}

#[test]
fn image_without_src_renders_alt_or_nothing() {
    assert_eq!(md("<p><img alt=\"nosrc\"> <img></p>"), "nosrc");
}

#[test]
fn unordered_and_nested_lists() {
    assert_eq!(
        md("<ul><li>a<ul><li>b</li><li>c</li></ul></li><li>d</li></ul>"),
        "- a\n  - b\n  - c\n- d"
    );
}

#[test]
fn ordered_list_honors_start_and_nests() {
    assert_eq!(
        md("<ol start=\"3\"><li>x</li><li>y</li></ol>"),
        "3. x\n4. y"
    );
    assert_eq!(
        md("<ol><li>a<ol start=\"2\"><li>b</li></ol></li></ol>"),
        "1. a\n   2. b"
    );
}

#[test]
fn task_list_items() {
    assert_eq!(
        md(
            "<ul><li><input type=\"checkbox\" checked> done</li><li><input type=\"checkbox\"> todo</li></ul>"
        ),
        "- [x] done\n- [ ] todo"
    );
}

#[test]
fn list_item_with_multiple_paragraphs() {
    assert_eq!(
        md("<ul><li><p>a</p><p>b</p></li><li>c</li></ul>"),
        "- a\n\n  b\n- c"
    );
}

#[test]
fn blockquote_nesting_composes() {
    assert_eq!(
        md("<blockquote><p>a</p><blockquote><p>b</p></blockquote></blockquote>"),
        "> a\n>\n> > b"
    );
    assert_eq!(
        md("<blockquote><ul><li>a</li><li>b</li></ul></blockquote>"),
        "> - a\n> - b"
    );
}

#[test]
fn pre_fenced_block_with_language() {
    assert_eq!(
        md("<pre><code class=\"language-rust\">fn main() {\n    println!(\"hi\");\n}</code></pre>"),
        "```rust\nfn main() {\n    println!(\"hi\");\n}\n```"
    );
}

#[test]
fn pre_containing_backticks_uses_tilde_fence() {
    assert_eq!(
        md("<pre>let x = 1;\n```\nmore</pre>"),
        "~~~\nlet x = 1;\n```\nmore\n~~~"
    );
}

#[test]
fn pre_preserves_blank_lines_and_strips_one_edge_newline() {
    assert_eq!(md("<pre>\na\n\n\nb\n</pre>"), "```\na\n\n\nb\n```");
    assert_eq!(md("<pre>**a****b**</pre>"), "```\n**a****b**\n```");
}

#[test]
fn inline_code_uses_longer_fence_and_padding() {
    assert_eq!(md("<p><code>x == 1</code></p>"), "`x == 1`");
    assert_eq!(md("<p><code>a`b</code></p>"), "``a`b``");
    assert_eq!(md("<p><code>`lit`</code></p>"), "`` `lit` ``");
}

#[test]
fn inline_code_content_is_not_merged() {
    assert_eq!(md("<p><code>*a* *b*</code></p>"), "`*a* *b*`");
}

#[test]
fn table_with_header_row() {
    assert_eq!(
        md("<table><tr><th>Name</th><th>Age</th></tr><tr><td>Ann</td><td>3</td></tr></table>"),
        "| Name | Age |\n| --- | --- |\n| Ann | 3 |"
    );
}

#[test]
fn table_with_sections_and_inline_formatting() {
    assert_eq!(
        md(
            "<table><thead><tr><th><b>H</b></th></tr></thead><tbody><tr><td><em>x</em></td></tr></tbody></table>"
        ),
        "| **H** |\n| --- |\n| *x* |"
    );
}

#[test]
fn headerless_table_gets_empty_header() {
    assert_eq!(
        md("<table><tr><td>a</td><td>b</td></tr><tr><td>c</td><td>d</td></tr></table>"),
        "|  |  |\n| --- | --- |\n| a | b |\n| c | d |"
    );
}

#[test]
fn table_cells_protect_pipes_and_newlines() {
    assert_eq!(
        md("<table><tr><th>a|b</th></tr><tr><td>c<br>d</td></tr></table>"),
        "| a\\|b |\n| --- |\n| c<br>d |"
    );
}

#[test]
fn merged_cells_fall_back_to_raw_html() {
    assert_eq!(
        md("<table><tr><td colspan=\"2\">a</td></tr><tr><td>b</td><td>c</td></tr></table>"),
        "<table><tr><td colspan=\"2\">a</td></tr><tr><td>b</td><td>c</td></tr></table>"
    );
    assert_eq!(
        md("<table><tr><td rowspan=\"2\">a</td><td>b</td></tr></table>"),
        "<table><tr><td rowspan=\"2\">a</td><td>b</td></tr></table>"
    );
}

#[test]
fn nested_table_falls_back_to_raw_html() {
    assert_eq!(
        md("<table><tr><td><table><tr><td>x</td></tr></table></td></tr></table>"),
        "<table><tr><td><table><tr><td>x</td></tr></table></td></tr></table>"
    );
}

#[test]
fn entities_decode_and_passthrough() {
    assert_eq!(
        md("<p>a &amp; b&nbsp;c &#233; &#x4E2D; &unknown; &</p>"),
        "a & b c é 中 &unknown; &"
    );
    assert_eq!(
        md("<p>&lt;tag&gt; &quot;q&quot; &#x1F600;</p>"),
        "\\<tag\\> \"q\" \u{1F600}"
    );
    assert_eq!(md("<p>&#xZZ; &#; &amp</p>"), "&\\#xZZ; &\\#; &amp");
}

#[test]
fn whitespace_collapses_around_inline_elements() {
    assert_eq!(md("<p>a   <b>  bold  </b>   c</p>"), "a **bold** c");
    assert_eq!(
        md("<p>line one\n   line two\t\tend</p>"),
        "line one line two end"
    );
}

#[test]
fn crlf_input_is_tolerated() {
    assert_eq!(md("<p>a</p>\r\n<p>b</p>"), "a\n\nb");
    assert_eq!(md("<p\r\nclass=MsoNormal>x</p>"), "x");
    assert_eq!(md("<p>a<br>b</p>"), "a\nb");
}

#[test]
fn markdown_specials_are_escaped() {
    assert_eq!(
        md("<p>* [ ] # < > _ { } \\ `</p>"),
        "\\* \\[ \\] \\# \\< \\> \\_ \\{ \\} \\\\ \\`"
    );
}

#[test]
fn line_start_block_markers_are_escaped() {
    assert_eq!(md("<p>- item</p>"), "\\- item");
    assert_eq!(md("<p>+ item</p>"), "\\+ item");
    assert_eq!(md("<p>&gt; quote</p>"), "\\> quote");
    assert_eq!(md("<p>1. not a list</p>"), "1\\. not a list");
    assert_eq!(md("<p>2) not a list</p>"), "2\\) not a list");
    assert_eq!(md("<p>1.5 stays</p>"), "1.5 stays");
    assert_eq!(md("<p>a<br>- b</p>"), "a\n\\- b");
}

#[test]
fn non_content_html_produces_empty_string() {
    assert_eq!(
        md("<!-- comment --><script>alert(1)</script><style>.x{color:red}</style>"),
        ""
    );
    assert_eq!(md("<?xml version=\"1.0\"?><!-- only -->"), "");
    assert_eq!(
        md("<head><title>t</title><meta charset=\"utf-8\"></head>"),
        ""
    );
    assert_eq!(md("<script>var unterminated = 1;"), "");
}

#[test]
fn empty_input_produces_empty_string() {
    assert_eq!(md(""), "");
    assert_eq!(md("   \n\t  "), "");
}

#[test]
fn mismatched_and_stray_tags_are_tolerated() {
    assert_eq!(md("<p>a</div></span>b"), "ab");
    assert_eq!(md("<ul><li>a<li>b</ul>"), "- a\n- b");
    assert_eq!(md("<p>one<div>two</div>"), "one\n\ntwo");
    assert_eq!(
        md("<table><tr><td>x<td>y</table>"),
        "|  |  |\n| --- | --- |\n| x | y |"
    );
}

#[test]
fn unknown_tags_are_transparent() {
    assert_eq!(md("<custom-element><b>x</b></custom-element>"), "**x**");
}

#[test]
fn malformed_html_never_panics() {
    let fragments = [
        "<",
        ">",
        "</",
        "/>",
        "<p",
        "<div ",
        "class=",
        "\"",
        "'",
        "<script>",
        "</script>",
        "&",
        "&amp",
        ";",
        "&#x",
        "<!--",
        "-->",
        "<![CDATA[",
        "]]>",
        "<table><tr><td",
        "\\",
        "`",
        "**",
        "~~",
        "\u{1F600}",
        "<pre>",
        "<a href='x",
        "<img src=x",
    ];
    for fragment in &fragments {
        let _ = md(fragment);
    }
    for a in &fragments {
        for b in &fragments {
            let _ = md(&format!("{a}{b}"));
        }
    }
    // Pathological nesting depth must not overflow the stack.
    let deep = format!("{}x{}", "<b>".repeat(5000), "</b>".repeat(5000));
    let _ = md(&deep);
    // Every truncation of a realistic document prefix.
    let doc = "<div><p>Hello <b>wor</p><table><tr><td>x</td></tr></table><script>alert(1)</script>";
    for end in 0..=doc.len() {
        let _ = md(&doc[..end]);
    }
}

#[test]
fn cdata_is_literal_text() {
    assert_eq!(md("<p><![CDATA[ raw <b> ]]></p>"), "raw \\<b\\>");
}

#[test]
fn full_document_shell() {
    assert_eq!(
        md("<!DOCTYPE html><html><head><title>T</title></head><body><p>Hello</p></body></html>"),
        "Hello"
    );
}

#[test]
fn horizontal_rule() {
    assert_eq!(md("<p>a</p><hr><p>b</p>"), "a\n\n---\n\nb");
}
