/* Generated from MarkNice c009c1ec7e7c92f89afa5a32edcb126b5296bda7 by scripts/sync-marknice-workspace.ps1. */
function tokenHeadingOptions(tagName) {
  return tagName === 'H1'
    ? {
      margin: '28px 0 18px',
      fontSize: 24,
      lineHeight: 1.4,
      fontWeight: 800,
      paddingLeft: 12,
      bgPadding: '8px 14px 7px',
      bgRadius: 3,
      tailHeight: 48,
      tailWidth: 26,
    }
    : {
      margin: '24px 0 14px',
      fontSize: 21,
      lineHeight: 1.45,
      fontWeight: 800,
      paddingLeft: 10,
      bgPadding: '7px 13px 6px',
      bgRadius: 3,
      tailHeight: 42,
      tailWidth: 23,
    };
}

function tokenHeadingStyle(theme, opts) {
  if (theme.headingVariant === 'ribbon') {
    return `margin:${opts.margin};border-bottom:2px solid ${theme.headingLine || theme.accent};line-height:0;`;
  }
  if (theme.headingBg) {
    return `margin:${opts.margin};padding:${opts.bgPadding};background:${theme.headingBg};border-radius:${opts.bgRadius}px;color:${theme.headingText || theme.heading};font-size:${opts.fontSize}px;line-height:${opts.lineHeight};font-weight:${opts.fontWeight};box-shadow:0 10px 24px rgba(79,70,229,0.16);`;
  }
  return `margin:${opts.margin};padding-left:${opts.paddingLeft}px;border-left:4px solid ${theme.accent};font-size:${opts.fontSize}px;line-height:${opts.lineHeight};color:${theme.heading};font-weight:${opts.fontWeight};`;
}

function makeTokenTheme(theme) {
  const strongColor = theme.strong || theme.heading;
  const codeText = theme.codeText || theme.text;
  const pageBgStyle = theme.pageBg
    ? `background:${theme.pageBg};${theme.pageBgSize ? `background-size:${theme.pageBgSize};` : ''}padding:24px 20px;border-radius:8px;`
    : '';
  return {
    section: `font-family:${theme.bodyFont};font-size:16px;color:${theme.text};line-height:1.9;letter-spacing:0.5px;word-break:break-word;${pageBgStyle}`,
    headingVariant: theme.headingVariant,
    headingBg: theme.headingBg,
    headingText: theme.headingText,
    headingTailBg: theme.headingTailBg,
    headingLine: theme.headingLine,
    accent: theme.accent,
    styles: {
      p: `margin:16px 0;line-height:1.9;color:${theme.text};font-size:16px;word-break:break-word;text-align:justify;`,
      h1: tokenHeadingStyle(theme, tokenHeadingOptions('H1')),
      h2: tokenHeadingStyle(theme, tokenHeadingOptions('H2')),
      h3: `margin:20px 0 12px;font-size:18px;line-height:1.5;color:${theme.heading};font-weight:700;`,
      'h4,h5,h6': `margin:18px 0 10px;font-size:17px;line-height:1.6;color:${theme.heading};font-weight:600;`,
      blockquote: `margin:18px 0;padding:12px 16px;background:${theme.quoteBg};border-left:4px solid ${theme.quoteBorder};color:${theme.text};border-radius:6px;`,
      ul: `margin:14px 0 14px 1.2em;padding:0;color:${theme.text};line-height:1.9;`,
      ol: `margin:14px 0 14px 1.2em;padding:0;color:${theme.text};line-height:1.9;`,
      li: 'margin:6px 0;font-size:16px;',
      a: `color:${theme.accent};text-decoration:none;border-bottom:1px solid ${theme.accent};`,
      img: 'max-width:100%;height:auto;border-radius:8px;display:block;margin:20px auto;',
      pre: `margin:18px 0;padding:14px 16px;overflow:auto;background:${theme.codeBg};border-radius:8px;color:${codeText};font-family:Menlo,Consolas,monospace;font-size:14px;line-height:1.7;white-space:pre-wrap;word-break:break-word;overflow-wrap:anywhere;`,
      code: `font-family:Menlo,Consolas,monospace;background:${theme.codeBg};color:${theme.accent};display:inline;white-space:normal;padding:2px 6px;border-radius:4px;font-size:0.92em;`,
      'strong,b': `color:${strongColor};font-weight:700;`,
      mark: `background:${theme.markBg || theme.quoteBg};color:${theme.heading};padding:1px 4px;border-radius:3px;`,
      table: 'width:100%;border-collapse:collapse;margin:18px 0;font-size:14px;',
      th: `border:1px solid ${theme.hr};padding:8px 10px;background:${theme.quoteBg};font-weight:700;color:${theme.heading};text-align:left;`,
      td: `border:1px solid ${theme.hr};padding:8px 10px;color:${theme.text};`,
      hr: `border:none;border-top:1px solid ${theme.hr};margin:28px 0;`,
    }
  };
}

const themes = {
  claude: makeTokenTheme({
    bodyFont: '-apple-system,BlinkMacSystemFont,"Helvetica Neue","PingFang SC","Hiragino Sans GB","Microsoft YaHei",sans-serif',
    text: '#3d3929',
    heading: '#181815',
    accent: '#d97757',
    strong: '#c2613f',
    quoteBg: '#faf9f5',
    quoteBorder: '#d97757',
    codeBg: '#f5f4ef',
    hr: '#e8e6dc',
    markBg: '#f9e8e0',
  }),
  simple: { section: 'font-family:-apple-system,BlinkMacSystemFont,"PingFang SC","Microsoft YaHei",sans-serif;word-break:break-word;color:#222;', styles: {
    h1:'font-size:24px;line-height:1.6;font-weight:700;margin:24px 0 16px;color:#111;',h2:'font-size:20px;line-height:1.6;font-weight:700;margin:22px 0 14px;color:#111;',h3:'font-size:17px;line-height:1.6;font-weight:700;margin:20px 0 12px;color:#222;',h4:'font-size:15px;line-height:1.6;font-weight:700;margin:16px 0 10px;color:#222;',h5:'font-size:14px;line-height:1.6;font-weight:700;margin:14px 0 8px;color:#333;',h6:'font-size:13px;line-height:1.6;font-weight:700;margin:12px 0 8px;color:#333;',p:'font-size:14px;line-height:1.85;margin:10px 0;color:#222;text-align:justify;',blockquote:'margin:14px 0;padding:10px 14px;border-left:4px solid #3e7bfa;background:#f4f7ff;color:#3a4a77;font-size:13px;line-height:1.8;',ul:'margin:10px 0;padding-left:24px;line-height:1.85;color:#222;font-size:14px;',ol:'margin:10px 0;padding-left:24px;line-height:1.85;color:#222;font-size:14px;',li:'margin:6px 0;',a:'color:#276ef1;text-decoration:none;border-bottom:1px solid #8eb2ff;',img:'max-width:100%;display:block;margin:16px auto;border-radius:6px;',pre:'background:#f6f8fa;border:1px solid #e2e8f0;border-radius:8px;padding:12px;overflow:auto;line-height:1.6;font-size:12px;',code:'background:#f1f4f8;padding:2px 5px;border-radius:4px;font-size:90%;font-family:Menlo,Consolas,monospace;',table:'border-collapse:collapse;width:100%;margin:12px 0;font-size:12px;',th:'border:1px solid #d9e2f0;padding:8px;background:#f7faff;text-align:left;',td:'border:1px solid #d9e2f0;padding:8px;',hr:'border:none;border-top:1px solid #dbe3ef;margin:24px 0;'
  }},
  tech: { section: 'font-family:-apple-system,BlinkMacSystemFont,"PingFang SC","Microsoft YaHei",sans-serif;word-break:break-word;color:#1a2433;', styles: {
    h1:'font-size:25px;line-height:1.55;font-weight:800;margin:36px 0 22px;color:#0b3b8b;border-bottom:2px solid #dce9ff;padding-bottom:8px;',h2:'font-size:21px;line-height:1.6;font-weight:700;margin:32px 0 20px;color:#1456c5;',h3:'font-size:17px;line-height:1.6;font-weight:700;margin:28px 0 10px;color:#1d3f6f;',h4:'font-size:15px;line-height:1.6;font-weight:700;margin:16px 0 10px;color:#1d3f6f;',p:'font-size:14px;line-height:1.9;margin:10px 0;color:#1f2d3d;text-align:justify;',blockquote:'margin:14px 0;padding:12px 14px;border-left:4px solid #5b8dff;background:#f5f8ff;color:#2a4f8a;font-size:13px;line-height:1.85;',ul:'margin:10px 0;padding-left:24px;line-height:1.9;color:#1f2d3d;font-size:14px;',ol:'margin:10px 0;padding-left:24px;line-height:1.9;color:#1f2d3d;font-size:14px;',li:'margin:6px 0;',a:'color:#1456c5;text-decoration:none;border-bottom:1px dashed #7aa2ff;',img:'max-width:100%;display:block;margin:18px auto;border-radius:8px;box-shadow:0 2px 10px rgba(0,0,0,.08);',pre:'background:#eef6ff;color:#1b3f75;border:1px solid #cfe2ff;border-radius:8px;padding:12px;overflow:auto;line-height:1.65;font-size:12px;',code:'background:#eef4ff;padding:2px 6px;border-radius:4px;font-size:90%;font-family:Menlo,Consolas,monospace;color:#174ea6;',table:'border-collapse:collapse;width:100%;margin:12px 0;font-size:12px;',th:'border:1px solid #d3e3ff;padding:8px;background:#edf3ff;text-align:left;color:#1d3f6f;',td:'border:1px solid #d3e3ff;padding:8px;',hr:'border:none;border-top:1px solid #dce6ff;margin:24px 0;'
  }},
  green: { section: 'font-family:-apple-system,BlinkMacSystemFont,"PingFang SC","Microsoft YaHei",sans-serif;word-break:break-word;color:#2b2b2b;', styles: {
    h1:'font-size:25px;line-height:1.6;font-weight:800;margin:36px 0 22px;color:#2e5b1f;',h2:'font-size:21px;line-height:1.6;font-weight:700;margin:32px 0 20px;color:#3f7f2f;',h3:'font-size:17px;line-height:1.6;font-weight:700;margin:28px 0 10px;color:#4a6b2f;',h4:'font-size:15px;line-height:1.6;font-weight:700;margin:16px 0 10px;color:#4a6b2f;',p:'font-size:14px;line-height:1.95;margin:10px 0;color:#2f2f2f;text-align:justify;',blockquote:'margin:14px 0;padding:12px 14px;border-left:4px solid #7abf45;background:#f6fbef;color:#486a30;font-size:13px;line-height:1.85;',ul:'margin:10px 0;padding-left:24px;line-height:1.9;color:#2f2f2f;font-size:14px;',ol:'margin:10px 0;padding-left:24px;line-height:1.9;color:#2f2f2f;font-size:14px;',li:'margin:6px 0;',a:'color:#3f7f2f;text-decoration:none;border-bottom:1px solid #9fd57a;',img:'max-width:100%;display:block;margin:16px auto;border-radius:6px;',pre:'background:#f6f8f0;border:1px solid #d8e6c6;border-radius:8px;padding:12px;overflow:auto;line-height:1.6;font-size:12px;',code:'background:#eef5e7;padding:2px 6px;border-radius:4px;font-size:90%;font-family:Menlo,Consolas,monospace;color:#496e2d;',table:'border-collapse:collapse;width:100%;margin:12px 0;font-size:12px;',th:'border:1px solid #d5e4c5;padding:8px;background:#f2f8ea;text-align:left;color:#496e2d;',td:'border:1px solid #d5e4c5;padding:8px;',hr:'border:none;border-top:1px solid #dbe8cf;margin:24px 0;'
  }},
  health: makeTokenTheme({
    bodyFont: '-apple-system,BlinkMacSystemFont,"Segoe UI","PingFang SC","Microsoft YaHei",sans-serif',
    text: '#183d34',
    heading: '#123f34',
    accent: '#0f8f67',
    strong: '#087857',
    quoteBg: '#f7fcf9',
    quoteBorder: '#5a8f75',
    codeBg: '#edf7f2',
    codeText: '#0f3129',
    hr: '#dcebe3',
    markBg: '#e5f5ee',
    pageBg: 'linear-gradient(rgba(18,63,52,0.045) 1px,transparent 1px),linear-gradient(90deg,rgba(18,63,52,0.045) 1px,transparent 1px),#f4faf7',
    pageBgSize: '48px 48px,48px 48px,auto',
  }),
  newspaper: { section: 'font-family:Georgia,"PingFang SC","Microsoft YaHei",serif;word-break:break-word;color:#1f1f1f;', styles: {
    h1:'font-size:28px;line-height:1.45;font-weight:700;margin:36px 0 22px;color:#111;',h2:'font-size:22px;line-height:1.5;font-weight:700;margin:32px 0 20px;color:#1b1b1b;',h3:'font-size:18px;line-height:1.55;font-weight:700;margin:28px 0 10px;color:#2b2b2b;',h4:'font-size:16px;line-height:1.55;font-weight:700;margin:16px 0 10px;color:#2b2b2b;',p:'font-size:15px;line-height:2;margin:10px 0;color:#1f1f1f;text-align:justify;',blockquote:'margin:14px 0;padding:10px 14px;border-left:4px solid #999;background:#f8f8f8;color:#444;font-size:13px;line-height:1.85;',ul:'margin:10px 0;padding-left:24px;line-height:1.9;color:#1f1f1f;font-size:15px;',ol:'margin:10px 0;padding-left:24px;line-height:1.9;color:#1f1f1f;font-size:15px;',li:'margin:6px 0;',a:'color:#1155cc;text-decoration:underline;',img:'max-width:100%;display:block;margin:18px auto;',pre:'background:#f5f5f5;border:1px solid #e3e3e3;border-radius:4px;padding:12px;overflow:auto;line-height:1.6;font-size:12px;',code:'background:#f0f0f0;padding:2px 5px;border-radius:3px;font-size:90%;font-family:Menlo,Consolas,monospace;',table:'border-collapse:collapse;width:100%;margin:12px 0;font-size:12px;',th:'border:1px solid #ddd;padding:8px;background:#f7f7f7;text-align:left;',td:'border:1px solid #ddd;padding:8px;',hr:'border:none;border-top:1px solid #ddd;margin:24px 0;'
  }},
  magazine: makeTokenTheme({
    bodyFont: '"PingFang SC","Hiragino Sans GB","Microsoft YaHei",sans-serif',
    text: '#2f2f33',
    heading: '#7a1f1f',
    accent: '#c23a3a',
    strong: '#992d2d',
    quoteBg: '#fff3f2',
    quoteBorder: '#c23a3a',
    codeBg: '#ffeef0',
    codeText: '#9c2f3a',
    hr: '#e8c2c2',
    markBg: '#ffe0de',
  }),
  ocean: makeTokenTheme({
    bodyFont: '-apple-system,BlinkMacSystemFont,"PingFang SC","Microsoft YaHei",sans-serif',
    text: '#1f3a3d',
    heading: '#0b4f55',
    accent: '#0e9594',
    strong: '#0a7e7d',
    quoteBg: '#effbfa',
    quoteBorder: '#0e9594',
    codeBg: '#e8f6f5',
    codeText: '#0b6b6a',
    hr: '#cdeae8',
    markBg: '#d6f3f1',
  }),
  minimal: { section: 'font-family:Inter,"PingFang SC","Microsoft YaHei",sans-serif;word-break:break-word;color:#111827;', styles: {
    h1:'font-size:26px;line-height:1.45;font-weight:700;margin:30px 0 18px;color:#111827;',h2:'font-size:21px;line-height:1.5;font-weight:700;margin:24px 0 14px;color:#1f2937;',h3:'font-size:17px;line-height:1.6;font-weight:600;margin:20px 0 10px;color:#374151;',h4:'font-size:15px;line-height:1.6;font-weight:600;margin:16px 0 8px;color:#374151;',p:'font-size:15px;line-height:2.05;margin:14px 0;color:#111827;',blockquote:'margin:16px 0;padding:0 0 0 14px;border-left:3px solid #111827;background:transparent;color:#374151;font-size:14px;line-height:1.95;',ul:'margin:12px 0;padding-left:22px;line-height:2;color:#111827;font-size:15px;',ol:'margin:12px 0;padding-left:22px;line-height:2;color:#111827;font-size:15px;',li:'margin:6px 0;',a:'color:#111827;text-decoration:underline;',img:'max-width:100%;display:block;margin:22px auto;border-radius:2px;',pre:'background:#fafafa;border:1px solid #ececec;border-radius:6px;padding:12px;overflow:auto;line-height:1.65;font-size:12px;',code:'background:#f3f4f6;padding:2px 5px;border-radius:4px;font-size:90%;font-family:Menlo,Consolas,monospace;',table:'border-collapse:collapse;width:100%;margin:12px 0;font-size:12px;',th:'border-bottom:1px solid #d1d5db;padding:8px 6px;text-align:left;',td:'border-bottom:1px solid #e5e7eb;padding:8px 6px;',hr:'border:none;border-top:1px solid #e5e7eb;margin:28px 0;'
  }},
  retro: { section: 'font-family:Georgia,"Times New Roman","PingFang SC",serif;word-break:break-word;color:#2d2418;', styles: {
    h1:'font-size:28px;line-height:1.5;font-weight:700;margin:24px 0 14px;color:#4a3215;',h2:'font-size:22px;line-height:1.55;font-weight:700;margin:20px 0 12px;color:#5f3f17;',h3:'font-size:18px;line-height:1.6;font-weight:700;margin:16px 0 10px;color:#704a1a;',h4:'font-size:16px;line-height:1.6;font-weight:700;margin:14px 0 8px;color:#704a1a;',p:'font-size:15px;line-height:2;margin:12px 0;color:#2f261b;text-align:justify;',blockquote:'margin:16px 0;padding:12px 14px;border-left:4px solid #8b6a35;background:#f8f2e8;color:#5a4423;font-size:14px;line-height:1.9;',ul:'margin:10px 0;padding-left:24px;line-height:1.95;color:#2f261b;font-size:15px;',ol:'margin:10px 0;padding-left:24px;line-height:1.95;color:#2f261b;font-size:15px;',li:'margin:6px 0;',a:'color:#7a4e14;text-decoration:none;border-bottom:1px solid #c8a870;',img:'max-width:100%;display:block;margin:20px auto;border:6px solid #f0e3cc;border-radius:2px;',pre:'background:#f8f2e8;border:1px solid #e4d1ad;border-radius:6px;padding:12px;overflow:auto;line-height:1.65;font-size:12px;',code:'background:#f2e7d4;padding:2px 5px;border-radius:3px;font-size:90%;font-family:Menlo,Consolas,monospace;color:#704a1a;',table:'border-collapse:collapse;width:100%;margin:12px 0;font-size:12px;',th:'border:1px solid #d7c19a;padding:8px;background:#f7ebd6;text-align:left;',td:'border:1px solid #d7c19a;padding:8px;',hr:'border:none;border-top:1px solid #d7c19a;margin:24px 0;'
  }},
  night: { section: 'font-family:-apple-system,BlinkMacSystemFont,"PingFang SC","Microsoft YaHei",sans-serif;word-break:break-word;color:#cbd5ff;background:#0f1220;padding:14px;border-radius:12px;', styles: {
    h1:'font-size:27px;line-height:1.5;font-weight:800;margin:24px 0 14px;color:#9ec5ff;',h2:'font-size:22px;line-height:1.55;font-weight:700;margin:20px 0 12px;color:#84f0ff;',h3:'font-size:18px;line-height:1.6;font-weight:700;margin:16px 0 10px;color:#b9a3ff;',h4:'font-size:16px;line-height:1.6;font-weight:700;margin:14px 0 8px;color:#b9a3ff;',p:'font-size:15px;line-height:1.95;margin:12px 0;color:#d5dbff;text-align:justify;',blockquote:'margin:16px 0;padding:12px 14px;border-left:4px solid #7c8cff;background:#171b2f;color:#aebcff;font-size:14px;line-height:1.85;',ul:'margin:10px 0;padding-left:24px;line-height:1.9;color:#d5dbff;font-size:15px;',ol:'margin:10px 0;padding-left:24px;line-height:1.9;color:#d5dbff;font-size:15px;',li:'margin:6px 0;',a:'color:#7de3ff;text-decoration:none;border-bottom:1px dashed #7de3ff;',img:'max-width:100%;display:block;margin:20px auto;border-radius:10px;box-shadow:0 0 0 1px #2c3255,0 8px 24px rgba(0,0,0,.35);',pre:'background:#171b2f;border:1px solid #2f3763;border-radius:8px;padding:12px;overflow:auto;line-height:1.65;font-size:12px;color:#d6e0ff;',code:'background:#23294a;padding:2px 6px;border-radius:4px;font-size:90%;font-family:Menlo,Consolas,monospace;color:#9ec5ff;',table:'border-collapse:collapse;width:100%;margin:12px 0;font-size:12px;',th:'border:1px solid #2f3763;padding:8px;background:#1f2440;text-align:left;color:#9ec5ff;',td:'border:1px solid #2f3763;padding:8px;color:#d5dbff;',hr:'border:none;border-top:1px solid #2f3763;margin:24px 0;'
  }},
  elegant: makeTokenTheme({
    bodyFont: '"PingFang SC","Hiragino Sans GB","Microsoft YaHei",sans-serif',
    text: '#4a3a30',
    heading: '#5a3826',
    accent: '#b08968',
    strong: '#8a5a36',
    quoteBg: '#fffaf2',
    quoteBorder: '#d8b891',
    codeBg: '#f7efe4',
    codeText: '#6b4a35',
    hr: '#eadcc9',
    markBg: '#f6e6cf',
    pageBg: '#fff8ee',
  }),
  vivid: makeTokenTheme({
    bodyFont: '"PingFang SC","Microsoft YaHei",sans-serif',
    text: '#233044',
    heading: '#d94841',
    accent: '#ff6b6b',
    headingBg: '#f1665d',
    headingText: '#ffffff',
    headingVariant: 'ribbon',
    headingTailBg: '#e5e7eb',
    headingLine: '#ff6b6b',
    strong: '#e05252',
    quoteBg: '#fff4f4',
    quoteBorder: '#ff6b6b',
    codeBg: '#fff8f1',
    hr: '#ffd6d6',
    markBg: '#ffe3e3',
  }),
  warmred: { section: 'font-family:-apple-system,BlinkMacSystemFont,"PingFang SC","Microsoft YaHei",sans-serif;word-break:break-word;color:#3b2e2a;background:#fffcfa;padding:16px;', styles: {
    h1:'font-size:24px;line-height:1.6;font-weight:800;margin:36px 0 18px;color:#c0392b;text-align:center;',h2:'font-size:20px;line-height:1.6;font-weight:700;margin:32px 0 16px;color:#c0392b;text-align:center;border-bottom:2px solid #c0392b;padding-bottom:8px;display:inline-block;',h3:'font-size:17px;line-height:1.6;font-weight:700;margin:28px 0 12px;color:#c0392b;',h4:'font-size:15px;line-height:1.6;font-weight:700;margin:22px 0 10px;color:#a93226;',h5:'font-size:14px;line-height:1.6;font-weight:700;margin:18px 0 8px;color:#a93226;',h6:'font-size:13px;line-height:1.6;font-weight:700;margin:14px 0 8px;color:#a93226;',p:'font-size:15px;line-height:2.0;margin:14px 0;color:#3b2e2a;text-align:justify;letter-spacing:.3px;',strong:'color:#c0392b;font-weight:700;',blockquote:'margin:18px 0;padding:14px 18px;border-left:4px solid #c0392b;background:#fef5f0;color:#7a2e1f;font-size:14px;line-height:1.9;border-radius:0 8px 8px 0;',ul:'margin:14px 0;padding-left:22px;line-height:2.0;color:#3b2e2a;font-size:15px;',ol:'margin:14px 0;padding-left:22px;line-height:2.0;color:#3b2e2a;font-size:15px;',li:'margin:8px 0;',a:'color:#c0392b;text-decoration:none;border-bottom:1px solid #e8a89a;',img:'max-width:100%;display:block;margin:20px auto;border-radius:8px;',pre:'background:#f7ede0;border:1px solid #ecdcc8;border-radius:10px;padding:14px;overflow:auto;line-height:1.7;font-size:12px;',code:'background:#f7ede0;padding:2px 6px;border-radius:4px;font-size:90%;font-family:Menlo,Consolas,monospace;color:#a93226;',table:'border-collapse:collapse;width:100%;margin:14px 0;font-size:13px;',th:'border:1px solid #ecdcc8;padding:10px;background:#f7ede0;text-align:left;color:#7a2e1f;font-weight:700;',td:'border:1px solid #ecdcc8;padding:10px;',hr:'border:none;border-top:2px solid #c0392b;margin:28px 0;width:40%;margin-left:auto;margin-right:auto;'
  }},
  amber: { section: 'font-family:-apple-system,BlinkMacSystemFont,"PingFang SC","Microsoft YaHei",sans-serif;word-break:break-word;color:#2c2c2c;', styles: {
    h1:'font-size:20px;line-height:1.6;font-weight:800;margin:40px 0 28px;color:#fff;text-align:center;background:#c8722a;border-radius:8px;padding:10px 28px;display:inline-block;width:auto;',h2:'font-size:18px;line-height:1.6;font-weight:800;margin:36px 0 24px;color:#fff;text-align:center;background:#c8722a;border-radius:8px;padding:8px 24px;display:inline-block;width:auto;',h3:'font-size:16px;line-height:1.6;font-weight:700;margin:30px 0 18px;color:#c8722a;font-weight:700;',h4:'font-size:15px;line-height:1.6;font-weight:700;margin:24px 0 14px;color:#c8722a;',h5:'font-size:14px;line-height:1.6;font-weight:700;margin:20px 0 12px;color:#c8722a;',h6:'font-size:13px;line-height:1.6;font-weight:700;margin:16px 0 10px;color:#c8722a;',p:'font-size:15px;line-height:2.0;margin:18px 0;color:#2c2c2c;text-align:justify;letter-spacing:.2px;',strong:'color:#c8722a;font-weight:700;',blockquote:'margin:20px 0;padding:14px 18px;border-left:4px solid #c8722a;background:#fdf5ec;color:#7a4010;font-size:14px;line-height:1.95;border-radius:0 8px 8px 0;',ul:'margin:16px 0;padding-left:22px;line-height:2.0;color:#2c2c2c;font-size:15px;',ol:'margin:16px 0;padding-left:8px;line-height:2.0;color:#2c2c2c;font-size:15px;list-style:none;',li:'margin:10px 0;',a:'color:#c8722a;text-decoration:none;border-bottom:1px solid #e8b07a;',img:'max-width:100%;display:block;margin:24px auto;border-radius:8px;',pre:'background:#fdf5ec;border:1px solid #f0d5b0;border-radius:8px;padding:14px;overflow:auto;line-height:1.65;font-size:12px;',code:'background:#faebd7;padding:2px 6px;border-radius:4px;font-size:90%;font-family:Menlo,Consolas,monospace;color:#a05a20;',table:'border-collapse:collapse;width:100%;margin:16px 0;font-size:13px;',th:'border:1px solid #f0d5b0;padding:10px;background:#fdf0e0;text-align:left;color:#7a4010;',td:'border:1px solid #f0d5b0;padding:10px;',hr:'border:none;border-top:1px solid #f0d5b0;margin:28px 0;'
  }}
};

function scaledStyle(styleText, fontOffset = 0, spacingOffset = 0) {
  let s = styleText;
  if (fontOffset) s = s.replace(/font-size:(\d+)px/g, (_, n) => `font-size:${Math.max(Number(n) + fontOffset, 9)}px`);
  if (spacingOffset) {
    // match margin with 3 values: top h bot (e.g. margin:36px 0 22px)
    s = s.replace(/margin:([-\d]+)px ([-\d]+(?:px)?) ([-\d]+)px/g, (_, top, mid, bot) =>
      `margin:${Math.max(Number(top) + spacingOffset, 0)}px ${mid} ${Math.max(Number(bot) + spacingOffset, 0)}px`
    );
    // match margin with 4 values: top h bot h (e.g. margin:36px 0 22px 0)
    s = s.replace(/margin:([-\d]+)px ([-\d]+(?:px)?) ([-\d]+)px ([-\d]+(?:px)?)/g, (_, top, h1, bot, h2) =>
      `margin:${Math.max(Number(top) + spacingOffset, 0)}px ${h1} ${Math.max(Number(bot) + spacingOffset, 0)}px ${h2}`
    );
  }
  return s;
}

function applyInlineStyles(container, styleMap, offset = 0) {
  Object.entries(styleMap).forEach(([tag, style]) => {
    const styleWithScale = scaledStyle(style, offset, paraSpacingOffset);
    container.querySelectorAll(tag).forEach(el => {
      const prev = el.getAttribute('style') || '';
      el.setAttribute('style', prev ? `${prev};${styleWithScale}` : styleWithScale);
    });
  });
}

function stripMarkdownInline(text) {
  return String(text || '')
    .replace(/!\[([^\]]*)\]\((?:[^()\\]|\\.)*\)/g, '$1')
    .replace(/\[([^\]]+)\]\((?:[^()\\]|\\.)*\)/g, '$1')
    .replace(/[*_`~]/g, '')
    .replace(/<[^>]+>/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

function extractDocumentTitle(md) {
  const lines = String(md || '').split(/\r?\n/);
  let firstTextLine = '';
  for (const rawLine of lines) {
    const line = rawLine.trim();
    if (!line) continue;
    if (!firstTextLine) firstTextLine = line;
    const headingMatch = line.match(/^#{1,6}\s+(.+?)\s*#*$/);
    if (headingMatch) {
      const title = stripMarkdownInline(headingMatch[1]);
      if (title) return title;
    }
  }
  if (!firstTextLine) return '';
  return stripMarkdownInline(firstTextLine.replace(/^[>+-]\s+/, '').replace(/^\d+[.)\u3001]\s+/, ''));
}

function sanitizeFileNamePart(text, fallback) {
  const safe = String(text || '')
    .replace(/[\\/:*?"<>|]/g, ' ')
    .replace(/[\u0000-\u001f\u007f]/g, ' ')
    .replace(/\s+/g, ' ')
    .replace(/[. ]+$/g, '')
    .trim();
  return (safe || fallback || 'MarkNice-导出文档').slice(0, 80);
}

function getExportTitle() {
  return extractDocumentTitle(markdownEl.value) || 'MarkNice 导出文档';
}

function getExportBaseName() {
  return sanitizeFileNamePart(getExportTitle(), 'MarkNice-导出文档');
}

function detectHeadingNumberPrefix(text) {
  const patterns = [
    /^\s*[（(]\s*([0-9]{1,2}|[一二三四五六七八九十百]+)\s*[)）][、.．:：]?\s*/,
    /^\s*([0-9]{1,2}|[一二三四五六七八九十百]+)\s*[、.．:：]\s*/,
    /^\s*([0-9]{1,2})\s+/
  ];
  for (const pattern of patterns) {
    const match = text.match(pattern);
    if (match) return { value: match[1], length: match[0].length };
  }
  return null;
}

function stripTextPrefix(node, count) {
  if (!count) return 0;
  Array.from(node.childNodes).forEach(child => {
    if (!count) return;
    if (child.nodeType === 3) {
      const text = child.nodeValue || '';
      if (text.length <= count) {
        count -= text.length;
        child.nodeValue = '';
      } else {
        child.nodeValue = text.slice(count);
        count = 0;
      }
    } else if (child.nodeType === 1) {
      count = stripTextPrefix(child, count);
    }
  });
  return count;
}

function createWarmredCircle(label, block) {
  const outerStyle = block
    ? 'display:block;text-align:center;margin-bottom:8px;'
    : 'display:inline-block;vertical-align:middle;margin-right:10px;';
  const circleStyle = block
    ? "display:inline-block;box-sizing:border-box;width:52px;height:52px;line-height:48px;border:2px solid #c0392b;border-radius:50%;background:transparent;color:#c0392b;font-size:24px;font-weight:900;text-align:center;font-family:'DIN Alternate','Impact','Arial Black',sans-serif;letter-spacing:1px;"
    : "display:inline-block;box-sizing:border-box;min-width:40px;height:40px;line-height:36px;padding:0 8px;border:2px solid #c0392b;border-radius:999px;background:transparent;color:#c0392b;font-size:16px;font-weight:900;text-align:center;font-family:'DIN Alternate','Impact','Arial Black',sans-serif;letter-spacing:0;";
  return `<span style="${outerStyle}"><span style="${circleStyle}">${label}</span></span>`;
}

function sanitizeForWechat(html) {
  const theme = themes[themeSelect.value] || themes.simple;
  const offset = fontSizeOffset;
  const doc = new DOMParser().parseFromString(`<section>${html}</section>`, 'text/html');
  const root = doc.body.firstElementChild;
  root.setAttribute('style', theme.section);
  root.querySelectorAll('script,style,iframe,object,embed').forEach(n => n.remove());
  root.querySelectorAll('*').forEach(el => {
    [...el.attributes].forEach(attr => {
      const n = attr.name.toLowerCase();
      if (n.startsWith('on') || n === 'id') el.removeAttribute(attr.name);
      if (n === 'class') {
        const cls = el.getAttribute('class') || '';
        if (cls !== 'math-inline' && cls !== 'math-block') el.removeAttribute(attr.name);
      }
    });
  });
  applyLocalImages(root);
  applyInlineStyles(root, theme.styles, offset);
  // Reset code styles inside pre blocks to avoid extra indentation
  root.querySelectorAll('pre code').forEach(el => {
    el.setAttribute('style', 'background:none;padding:0;border-radius:0;font-size:inherit;font-family:Menlo,Consolas,monospace;white-space:inherit;word-break:inherit;overflow-wrap:inherit;');
  });
  // Insert LRM (U+200E) before inline <code> and <strong> inside list items and table cells
  // to prevent WeChat editor from breaking them into separate lines
  root.querySelectorAll('li code, td code, th code').forEach(el => {
    if (el.closest('pre')) return; // skip code blocks
    el.parentNode.insertBefore(doc.createTextNode('\u200E'), el);
  });
  root.querySelectorAll('li strong, td strong, th strong, li b, td b, th b').forEach(el => {
    if (el.previousSibling) return; // only first child (line-start bold)
    el.parentNode.insertBefore(doc.createTextNode('\u200E'), el);
  });
  // For amber theme: center h1/h2 wrapper + colored ol numbers
  if (themeSelect.value === 'amber') {
    // Wrap h1/h2 in a centered div so inline-block centering works in WeChat
    root.querySelectorAll('h1, h2').forEach(el => {
      const wrapper = doc.createElement('div');
      wrapper.setAttribute('style', 'text-align:center;margin:0;padding:0;');
      el.parentNode.insertBefore(wrapper, el);
      wrapper.appendChild(el);
    });
    // Style ol items with amber numbered prefix
    root.querySelectorAll('ol').forEach(ol => {
      ol.setAttribute('style', (ol.getAttribute('style') || '') + ';list-style:none;padding-left:0;');
      ol.querySelectorAll(':scope > li').forEach((li, i) => {
        const marker = `<span style="color:#c8722a;font-weight:700;">${i + 1}、</span>`;
        li.innerHTML = marker + li.innerHTML;
      });
    });
  }
  // For warmred theme: add or reuse chapter numbers with a hollow circle.
  // Use h1 for sections, except when the only h1 is the article title at the start.
  if (themeSelect.value === 'warmred') {
    let headingCounter = 0;
    const h1List = Array.from(root.querySelectorAll('h1'));
    const hasOnlyOpeningTitle = h1List.length === 1 && h1List[0] === root.firstElementChild;
    const headingTag = h1List.length && !hasOnlyOpeningTitle ? 'h1' : 'h2';
    const baseTopMargin = headingTag === 'h1' ? 36 : 32;
    const baseBottomMargin = headingTag === 'h1' ? 18 : 16;
    const headingTopMargin = Math.max((baseTopMargin + paraSpacingOffset) * 2, 0);
    const headingBottomMargin = Math.max((baseBottomMargin + paraSpacingOffset) / 2, 0);
    root.querySelectorAll(headingTag).forEach(el => {
      const numbered = detectHeadingNumberPrefix(el.textContent || '');
      const wrapper = doc.createElement('div');
      wrapper.setAttribute('style', `text-align:center;margin:${headingTopMargin}px 0 ${headingBottomMargin}px;padding:0;`);
      el.parentNode.insertBefore(wrapper, el);
      wrapper.appendChild(el);
      const headingStyle = (el.getAttribute('style') || '').replace(/margin:[^;]+;?/g, '');
      el.setAttribute('style', `${headingStyle};margin:0;`);
      if (numbered) {
        const numericValue = /^[0-9]+$/.test(numbered.value) ? Number(numbered.value) : null;
        headingCounter = numericValue || headingCounter + 1;
        stripTextPrefix(el, numbered.length);
        el.insertAdjacentHTML('afterbegin', createWarmredCircle(numbered.value, false));
      } else {
        headingCounter++;
        wrapper.insertAdjacentHTML('afterbegin', createWarmredCircle(String(headingCounter).padStart(2, '0'), true));
      }
    });
  }
  if (theme.headingVariant === 'ribbon') {
    root.querySelectorAll('h1,h2').forEach(h => {
      const opts = tokenHeadingOptions(h.tagName);
      const wrapper = doc.createElement('section');
      wrapper.setAttribute('style', scaledStyle(tokenHeadingStyle(theme, opts), offset, paraSpacingOffset));
      const label = doc.createElement('section');
      label.setAttribute('style', scaledStyle(
        `display:inline-block;box-sizing:border-box;max-width:88%;padding:${opts.bgPadding};background:${theme.headingBg || theme.accent};border-radius:${opts.bgRadius}px ${opts.bgRadius}px 0 0;color:${theme.headingText || '#ffffff'};font-size:${opts.fontSize}px;line-height:${opts.lineHeight};font-weight:${opts.fontWeight};letter-spacing:0;vertical-align:bottom;`,
        offset,
        paraSpacingOffset
      ));
      label.textContent = h.textContent || '';
      const tail = doc.createElement('span');
      tail.setAttribute('style', `display:inline-block;width:0;height:0;border-left:${opts.tailWidth}px solid ${theme.headingTailBg || '#e5e7eb'};border-top:${opts.tailHeight}px solid transparent;vertical-align:bottom;`);
      wrapper.appendChild(label);
      wrapper.appendChild(tail);
      h.replaceWith(wrapper);
    });
  }
  return root.outerHTML;
}

function renderMath(container) {
  if (typeof katex === 'undefined') return;
  container.querySelectorAll('.math-block').forEach(function(el) {
    var tex = el.textContent.replace(/^\\\[/, '').replace(/\\\]$/, '').trim();
    try { el.innerHTML = katex.renderToString(tex, { displayMode: true, throwOnError: false }); } catch(e) {}
  });
  container.querySelectorAll('.math-inline').forEach(function(el) {
    var tex = el.textContent.replace(/^\\\(/, '').replace(/\\\)$/, '').trim();
    try { el.innerHTML = katex.renderToString(tex, { displayMode: false, throwOnError: false }); } catch(e) {}
  });
}

function render() {
  let md = markdownEl.value || '';
  for (const fn of __mnHooks.beforeRender) {
    try { const r = fn(md); if (typeof r === 'string') md = r; }
    catch (e) { console.error('[MarkNice] beforeRender hook error:', e); }
  }
  const html = sanitizeForWechat(marked.parse(md));
  previewEl.innerHTML = html;
  renderMath(previewEl);
  for (const fn of __mnHooks.afterRender) {
    try { fn(previewEl); }
    catch (e) { console.error('[MarkNice] afterRender hook error:', e); }
  }
  previewEl.dataset.html = previewEl.innerHTML;
  statusEl.textContent = md.trim() ? locale.opened + localImageStatusSuffix() : '';
}

async function copyRichHtml(html) {
  const plainText = (() => {
    try {
      const doc = new DOMParser().parseFromString(html, 'text/html');
      return doc.body.innerText || doc.body.textContent || previewEl.innerText;
    } catch (_) {
      return previewEl.innerText;
    }
  })();
  if (navigator.clipboard && window.ClipboardItem) {
    const blobHtml = new Blob([html], { type: 'text/html' });
    const blobText = new Blob([plainText], { type: 'text/plain' });
    await navigator.clipboard.write([new ClipboardItem({ 'text/html': blobHtml, 'text/plain': blobText })]);
    return;
  }
  const holder = document.createElement('div');
  holder.style.cssText = 'position:fixed;left:-99999px;top:0;width:1px;height:1px;overflow:hidden;';
  holder.innerHTML = html;
  document.body.appendChild(holder);
  try {
    const range = document.createRange();
    range.selectNodeContents(holder);
    const sel = window.getSelection();
    sel.removeAllRanges();
    sel.addRange(range);
    if (!document.execCommand('copy')) throw new Error('clipboard denied');
    sel.removeAllRanges();
  } finally {
    holder.remove();
  }
}
