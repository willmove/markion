const markdownEl = document.getElementById('markdown');
const previewEl = document.getElementById('preview');
const statusEl = document.getElementById('status');
const themeSelect = document.getElementById('themeSelect');
const fontSizeLabel = document.getElementById('fontSizeLabel');
const fontSizeDown = document.getElementById('fontSizeDown');
const fontSizeUp = document.getElementById('fontSizeUp');
let fontSizeOffset = 0;
let paraSpacingOffset = 0;
const __mnHooks = { beforeRender: [], afterRender: [], ready: [] };
const locale = { opened: 'rendered' };
const selfTestFormatMessages = {
  applied: 'formatted', headingPlaceholder: 'Heading', listPlaceholder: 'List item',
  codePlaceholder: 'code', codeBlockPlaceholder: 'code content', linkPlaceholder: 'link text',
  imagePlaceholder: 'image description', imageUrlPlaceholder: 'image URL', boldPlaceholder: 'bold text',
  italicPlaceholder: 'italic text', underlinePlaceholder: 'underlined text', quotePlaceholder: 'quote',
  tableTemplate: '| Header 1 | Header 2 | Header 3 |\n| --- | --- | --- |\n| Content | Content | Content |'
};
function markdownFormatText(key) { return selfTestFormatMessages[key] || ''; }
const protectedImages = new Map();
const fixtureGif = Uint8Array.from(atob('R0lGODlhAQABAAAAACw='), value => value.charCodeAt(0));
protectedImages.set('fixture', { id: 'fixture', blob: new Blob([fixtureGif], { type: 'image/gif' }) });
function applyLocalImages(root) {
  root.querySelectorAll('img[src="note.assets/local.png"]').forEach(image => {
    image.setAttribute('src', 'data:image/gif;base64,R0lGODlhAQABAAAAACw=');
    image.setAttribute('data-mn-local-image-id', 'fixture');
  });
}
function localImageStatusSuffix() { return ''; }
marked.setOptions({ breaks: true, gfm: true });
function escapeMathHtml(value) {
  return String(value || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}
marked.use({ extensions: [
  { name: 'mathBlock', level: 'block', start: src => (src.match(/\$\$/) || { index: -1 }).index, tokenizer(src) { const m = src.match(/^\$\$([\s\S]+?)\$\$/); if (m) return { type: 'mathBlock', raw: m[0], tex: m[1].trim() }; }, renderer: token => { const tex = escapeMathHtml(token.tex); return `<div class="math-block" data-tex="${tex}">\\[${tex}\\]</div>`; } },
  { name: 'mathInline', level: 'inline', start: src => (src.match(/\$/) || { index: -1 }).index, tokenizer(src) { const m = src.match(/^\$([^$\n]+?)\$/); if (m) return { type: 'mathInline', raw: m[0], tex: m[1].trim() }; }, renderer: token => { const tex = escapeMathHtml(token.tex); return `<span class="math-inline" data-tex="${tex}">\\(${tex}\\)</span>`; } }
] });
function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function setMarkdownSelection(value, start = 0, end = start) {
  markdownEl.value = value;
  markdownEl.focus();
  markdownEl.setSelectionRange(start, end);
  MarkionMarkdownFormat.rememberMarkdownSelection();
}

function runFormattingChecks() {
  const stateBefore = {
    markdown: markdownEl.value,
    selectionStart: markdownEl.selectionStart,
    selectionEnd: markdownEl.selectionEnd,
    theme: themeSelect.value,
    fontSizeOffset,
    paraSpacingOffset,
  };
  const format = MarkionMarkdownFormat.runMarkdownAction;

  setMarkdownSelection('hello', 0, 5);
  format('bold');
  assert(markdownEl.value === '**hello**' && markdownEl.selectionStart === 2 && markdownEl.selectionEnd === 7, 'bold did not wrap and preserve the selection');
  format('bold');
  assert(markdownEl.value === 'hello' && markdownEl.selectionStart === 0 && markdownEl.selectionEnd === 5, 'bold did not toggle off');

  setMarkdownSelection('one\ntwo', 0, 7);
  format('heading2');
  assert(markdownEl.value === '## one\n## two', 'multiline heading formatting failed');
  format('heading2');
  assert(markdownEl.value === 'one\ntwo', 'multiline heading toggle failed');

  setMarkdownSelection('one\ntwo', 0, 7);
  format('orderedList');
  assert(markdownEl.value === '1. one\n2. two', 'ordered-list formatting failed');
  format('orderedList');
  assert(markdownEl.value === 'one\ntwo', 'ordered-list toggle failed');

  const emptyCases = [
    ['heading1', '# Heading'], ['italic', '*italic text*'], ['underline', '<u>underlined text</u>'],
    ['unorderedList', '- List item'], ['code', '`code`'], ['link', '[link text](https://example.com)'],
    ['quote', '> quote'], ['codeBlock', '```js\ncode content\n```'],
    ['image', '![image description](image URL)'], ['table', '| Header 1 | Header 2 | Header 3 |']
  ];
  emptyCases.forEach(([action, expected]) => {
    setMarkdownSelection('');
    format(action);
    assert(markdownEl.value.includes(expected), `${action} did not insert its localized empty-selection template`);
  });

  const shortcuts = [
    ['b', '**shortcut**'], ['i', '*shortcut*'], ['u', '<u>shortcut</u>'], ['k', '[shortcut](https://example.com)']
  ];
  shortcuts.forEach(([key, expected]) => {
    setMarkdownSelection('shortcut', 0, 8);
    const event = new KeyboardEvent('keydown', { key, ctrlKey: true, bubbles: true, cancelable: true });
    markdownEl.dispatchEvent(event);
    assert(event.defaultPrevented && markdownEl.value === expected, `Ctrl/Cmd+${key.toUpperCase()} was not handled`);
  });
  setMarkdownSelection('unchanged', 0, 9);
  const altEvent = new KeyboardEvent('keydown', { key: 'b', ctrlKey: true, altKey: true, bubbles: true, cancelable: true });
  markdownEl.dispatchEvent(altEvent);
  assert(!altEvent.defaultPrevented && markdownEl.value === 'unchanged', 'Alt-modified shortcut was consumed');
  const unsupported = new KeyboardEvent('keydown', { key: 'x', ctrlKey: true, bubbles: true, cancelable: true });
  markdownEl.dispatchEvent(unsupported);
  assert(!unsupported.defaultPrevented && markdownEl.value === 'unchanged', 'unsupported shortcut was consumed');

  setMarkdownSelection('preview', 0, 7);
  format('bold');
  assert(previewEl.querySelector('strong')?.textContent === 'preview', 'formatting did not refresh the preview immediately');
  setMarkdownSelection('toolbar', 0, 7);
  document.querySelector('[data-md-action="bold"]').click();
  assert(markdownEl.value === '**toolbar**' && document.activeElement === markdownEl, 'toolbar action lost the remembered selection or textarea focus');
  setMarkdownSelection('safe');
  format('imageUpload');
  assert(markdownEl.value === 'safe', 'excluded local-image upload action changed the editor');
  assert(themeSelect.value === stateBefore.theme && fontSizeOffset === stateBefore.fontSizeOffset && paraSpacingOffset === stateBefore.paraSpacingOffset, 'formatting changed presentation state');

  markdownEl.value = stateBefore.markdown;
  themeSelect.value = stateBefore.theme;
  fontSizeOffset = stateBefore.fontSizeOffset;
  paraSpacingOffset = stateBefore.paraSpacingOffset;
  render();
  markdownEl.setSelectionRange(stateBefore.selectionStart, stateBefore.selectionEnd);
}

function readU16(bytes, offset) { return bytes[offset] | (bytes[offset + 1] << 8); }
function readU32(bytes, offset) { return (bytes[offset] | (bytes[offset + 1] << 8) | (bytes[offset + 2] << 16) | (bytes[offset + 3] << 24)) >>> 0; }

async function unzipEntry(bytes, name) {
  let eocd = -1;
  for (let offset = bytes.length - 22; offset >= Math.max(0, bytes.length - 65_557); offset -= 1) {
    if (readU32(bytes, offset) === 0x06054b50) { eocd = offset; break; }
  }
  assert(eocd >= 0, 'DOCX has no ZIP directory');
  let offset = readU32(bytes, eocd + 16);
  const decoder = new TextDecoder();
  while (offset + 46 <= bytes.length && readU32(bytes, offset) === 0x02014b50) {
    const compression = readU16(bytes, offset + 10);
    const compressedSize = readU32(bytes, offset + 20);
    const filenameLength = readU16(bytes, offset + 28);
    const extraLength = readU16(bytes, offset + 30);
    const commentLength = readU16(bytes, offset + 32);
    const filename = decoder.decode(bytes.slice(offset + 46, offset + 46 + filenameLength));
    const localOffset = readU32(bytes, offset + 42);
    if (filename === name) {
      assert(readU32(bytes, localOffset) === 0x04034b50, `DOCX entry ${name} has no local header`);
      const localNameLength = readU16(bytes, localOffset + 26);
      const localExtraLength = readU16(bytes, localOffset + 28);
      const data = bytes.slice(localOffset + 30 + localNameLength + localExtraLength, localOffset + 30 + localNameLength + localExtraLength + compressedSize);
      if (compression === 0) return data;
      assert(compression === 8 && typeof DecompressionStream !== 'undefined', `DOCX entry ${name} cannot be deflated`);
      return new Uint8Array(await new Response(new Blob([data]).stream().pipeThrough(new DecompressionStream('deflate-raw'))).arrayBuffer());
    }
    offset += 46 + filenameLength + extraLength + commentLength;
  }
  throw new Error(`DOCX entry missing: ${name}`);
}

async function runExportChecks(docxFixture) {
  const stateBefore = {
    markdown: markdownEl.value,
    selectionStart: markdownEl.selectionStart,
    selectionEnd: markdownEl.selectionEnd,
    theme: themeSelect.value,
    fontSizeOffset,
    paraSpacingOffset,
  };
  themeSelect.value = 'night';
  fontSizeOffset = 2;
  paraSpacingOffset = 4;
  markdownEl.value = docxFixture.markdown;
  window.__markionRenderTimer = window.setTimeout(() => { markdownEl.value = 'stale debounce value'; }, 0);
  const snapshot = MarkionWorkspaceExports.buildExportSnapshot();
  await new Promise(resolve => window.setTimeout(resolve, 10));
  assert(snapshot.markdown === markdownEl.value, 'export snapshot did not use current textarea content');
  assert(snapshot.theme === 'night' && snapshot.fontSizeOffset === 2 && snapshot.paraSpacingOffset === 4, 'export snapshot did not retain presentation settings');
  assert(MarkionWorkspaceExports.filenameFor('A/title.html', 'html') === 'A title.html', 'filename normalization failed');
  assert(MarkionWorkspaceExports.filenameFor('文档.docx', 'docx') === '文档.docx', 'duplicate extension was retained');
  assert(MarkionWorkspaceExports.filenameFor('', 'html') === 'MarkNice export.html', 'filename fallback failed');
  const clipboardDescriptor = Object.getOwnPropertyDescriptor(navigator, 'clipboard');
  const execCommandDescriptor = Object.getOwnPropertyDescriptor(document, 'execCommand');
  const copiedSources = [];
  const previewBeforeCopy = previewEl.innerHTML;
  try {
    markdownEl.value = 'session edit\n\n  preserved whitespace';
    markdownEl.setSelectionRange(4, 11);
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: async value => copiedSources.push(value) } });
    assert(await MarkionWorkspaceExports.copyMarkdown(), 'preferred Markdown clipboard path failed');
    assert(copiedSources[0] === markdownEl.value, 'preferred Markdown clipboard path changed source text');
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: async () => { throw new Error('denied'); } } });
    Object.defineProperty(document, 'execCommand', { configurable: true, value: command => command === 'copy' });
    assert(await MarkionWorkspaceExports.copyMarkdown(), 'fallback Markdown clipboard path failed');
    assert(markdownEl.selectionStart === 4 && markdownEl.selectionEnd === 11 && previewEl.innerHTML === previewBeforeCopy, 'Markdown copy mutated selection or preview');
    Object.defineProperty(document, 'execCommand', { configurable: true, value: () => false });
    assert(!(await MarkionWorkspaceExports.copyMarkdown()), 'clipboard denial was reported as success');
    markdownEl.value = '';
    assert(!(await MarkionWorkspaceExports.copyMarkdown()), 'empty Markdown was reported as copied');
  } finally {
    if (clipboardDescriptor) Object.defineProperty(navigator, 'clipboard', clipboardDescriptor);
    else delete navigator.clipboard;
    if (execCommandDescriptor) Object.defineProperty(document, 'execCommand', execCommandDescriptor);
    else delete document.execCommand;
  }
  markdownEl.value = snapshot.markdown;
  markdownEl.setSelectionRange(0, 0);
  const unsafeSnapshot = {
    ...snapshot,
    articleHtml: '<section data-mn-local-image-id="leak"><script>throw new Error(1)</script><style>body{display:none}</style><form action="https://invalid.example"><input></form><a href="javascript:alert(1)" onclick="alert(1)">unsafe</a><iframe src="https://invalid.example"></iframe><img src="blob:secret" alt="blob image"><img src="file:///private.png" alt="file image"><img src="http://127.0.0.1:1234/secret" alt="loopback image"><img data-mn-local-image-id="fixture" src="blob:managed" alt="managed"><img src="https://example.com/image.png" alt="remote"></section>',
  };
  const prepared = await MarkionWorkspaceExports.prepareExportArticle(unsafeSnapshot);
  assert(!/(?:script|iframe|form|javascript:|blob:|file:|127\.0\.0\.1|data-mn-|onclick)/i.test(prepared.articleHtml), 'unsafe durable-artifact content remained');
  assert(prepared.articleHtml.includes('data:image/gif;base64,'), 'managed image was not embedded');
  assert(prepared.remoteImageCount === 1, 'remote image classification failed');
  assert(prepared.fallbackCount === 3, 'unsafe image fallback count failed');
  const documentPrepared = await MarkionWorkspaceExports.prepareWordArticle(snapshot);
  assert(documentPrepared.articleHtml.includes('background:#0f1220') && documentPrepared.articleHtml.includes('font-size:17px'), 'HTML export did not preserve the selected theme or typography offsets');
  assert(documentPrepared.articleHtml.includes('E=mc²') && documentPrepared.articleHtml.includes('x²+y²=z²'), 'Word export did not degrade formulas to linear text');
  const oversizedResource = {
    id: 'oversized',
    blob: new Blob([new Uint8Array(MarkionWorkspaceExports.MAX_EMBEDDED_IMAGE_BYTES + 1)], { type: 'image/png' }),
  };
  const limited = await MarkionWorkspaceExports.prepareExportArticle({
    ...snapshot,
    protectedResources: Object.freeze([oversizedResource]),
    articleHtml: '<section><img data-mn-local-image-id="oversized" src="blob:oversized" alt="oversized image"></section>',
  });
  assert(limited.fallbackCount === 1 && limited.articleHtml.includes('oversized image'), 'managed-image byte limit did not use a safe fallback');
  const standalone = MarkionWorkspaceExports.standaloneHtml(snapshot, prepared);
  assert(standalone.includes("default-src 'none'") && !standalone.includes('static/vendor') && !standalone.includes('script src=') && !/url\((?:['"])?fonts\//i.test(standalone), 'standalone HTML is not inert and self-contained');
  const offlineHtml = MarkionWorkspaceExports.standaloneHtml(snapshot, {
    articleHtml: '<section style="color:#123"><h1 style="font-size:29px">Offline export</h1></section>', fallbackCount: 0, remoteImageCount: 0
  });
  const offlineFrame = document.createElement('iframe');
  offlineFrame.sandbox = 'allow-same-origin';
  offlineFrame.srcdoc = offlineHtml;
  document.body.appendChild(offlineFrame);
  await new Promise((resolve, reject) => {
    offlineFrame.addEventListener('load', resolve, { once: true });
    window.setTimeout(() => reject(new Error('offline HTML did not reopen')), 2_000);
  });
  assert(offlineFrame.contentDocument?.querySelector('h1')?.textContent === 'Offline export', 'standalone HTML did not reopen offline');
  offlineFrame.remove();
  assert(typeof htmlDocx?.asBlob === 'function', 'pinned DOCX converter did not load');
  const expectedWordHtml = MarkionWorkspaceExports.wordHtml(snapshot, documentPrepared.articleHtml);
  docxFixture.expected_word_fragments.forEach(fragment => assert(expectedWordHtml.includes(fragment), `DOCX fixture lost ${fragment}`));
  const docx = await MarkionWorkspaceExports.createDocxBlob(snapshot, documentPrepared);
  assert(docx instanceof Blob && docx.size >= 64, 'DOCX converter produced an invalid blob');
  const archive = new Uint8Array(await docx.arrayBuffer());
  const contentTypes = new TextDecoder().decode(await unzipEntry(archive, '[Content_Types].xml'));
  const documentXml = new TextDecoder().decode(await unzipEntry(archive, 'word/document.xml'));
  const rootRelationships = new TextDecoder().decode(await unzipEntry(archive, '_rels/.rels'));
  const documentRelationships = new TextDecoder().decode(await unzipEntry(archive, 'word/_rels/document.xml.rels'));
  const mht = new TextDecoder().decode(await unzipEntry(archive, 'word/afchunk.mht'));
  assert(contentTypes.includes('wordprocessingml.document') && documentXml.includes('altChunk'), 'DOCX package structure is incomplete');
  assert(!/(?:blob:|file:|127\.0\.0\.1|javascript:|markion\.wechat\.session)/i.test(rootRelationships + documentRelationships), 'DOCX relationship target leaked a local/session reference');
  const leakedDocxReference = mht.match(/(?:blob:|file:|127\.0\.0\.1|javascript:|markion\.wechat\.session)/i);
  assert(!leakedDocxReference, 'DOCX package leaked a local/session reference');
  assert(mht.includes('Content-Location: urn:markion:image0.gif') && mht.includes('R0lGODlhAQABAAAAACw='), 'DOCX package did not retain embedded managed image data');
  docxFixture.expected_package_text.forEach(text => assert(mht.includes(text), `DOCX package fixture lost ${text}`));
  const converterDescriptor = Object.getOwnPropertyDescriptor(window, 'htmlDocx');
  try {
    Object.defineProperty(window, 'htmlDocx', { configurable: true, value: undefined });
    await MarkionWorkspaceExports.createDocxBlob(snapshot, prepared).then(() => { throw new Error('missing DOCX converter was accepted'); }, () => {});
    Object.defineProperty(window, 'htmlDocx', { configurable: true, value: { asBlob: () => new Blob(['bad']) } });
    await MarkionWorkspaceExports.createDocxBlob(snapshot, prepared).then(() => { throw new Error('malformed DOCX converter output was accepted'); }, () => {});
  } finally {
    if (converterDescriptor) Object.defineProperty(window, 'htmlDocx', converterDescriptor);
    else delete window.htmlDocx;
  }
  markdownEl.value = stateBefore.markdown;
  themeSelect.value = stateBefore.theme;
  fontSizeOffset = stateBefore.fontSizeOffset;
  paraSpacingOffset = stateBefore.paraSpacingOffset;
  render();
  markdownEl.setSelectionRange(stateBefore.selectionStart, stateBefore.selectionEnd);
  assert(markdownEl.value === stateBefore.markdown && themeSelect.value === stateBefore.theme && fontSizeOffset === stateBefore.fontSizeOffset && paraSpacingOffset === stateBefore.paraSpacingOffset, 'export checks mutated live workspace state');
}

function setPreviewMode(phone) {
  document.getElementById('previewContainer')?.classList.toggle('phone-mode', phone);
  document.querySelectorAll('.mode-btn').forEach(button => button.classList.remove('active'));
  document.getElementById(phone ? 'phoneModeBtn' : 'desktopModeBtn')?.classList.add('active');
}

async function runLocaleChecks() {
  const bridge = await fetch('bridge.js').then(response => response.text());
  ['importWord', 'importWordSuccess', 'savePdf', 'pdfPrintOpened', 'copyMarkdown', 'downloadHtml', 'downloadDocx', 'editorTitle', 'toggleMode']
    .forEach(key => {
      const count = bridge.split(`${key}:`).length - 1;
      assert(count >= 7, `locale key ${key} is missing from some languages (${count})`);
    });
}

async function loadWorkspaceShell() {
  const candidates = ['/', '../index.html'];
  for (const url of candidates) {
    try {
      const response = await fetch(url, { cache: 'no-store' });
      if (!response.ok) continue;
      const text = await response.text();
      if (text.includes('editor-panel')) return text;
    } catch (_) { /* try the next shell location */ }
  }
  throw new Error('workspace shell is missing editor-panel');
}

async function runSkinChecks() {
  const [shell, css] = await Promise.all([
    loadWorkspaceShell(),
    fetch('workspace.css').then(response => response.text()),
  ]);
  ['editor-panel', 'panel-header', 'panel-dot', 'mode-btn', 'preview-container', 'format-btn', 'font-size-ctrl', 'wordFileInput', 'savePdfBtn', 'importWordLabel']
    .forEach(token => assert(shell.includes(token), `workspace shell is missing ${token}`));
  assert(css.includes('--accent: #6366f1'), 'editor skin is missing the indigo accent');
  assert(css.includes('Inter') && css.includes('PingFang SC') && css.includes('Microsoft YaHei'), 'chrome font stack is incomplete');
  assert(css.includes('SF Mono') && css.includes('Fira Code'), 'editor font stack is incomplete');
  assert(/width:\s*375px/.test(css), 'phone frame is not 375px wide');
  assert(css.includes('.preview-container.phone-mode .preview-area::before'), 'phone frame is missing the notch');
  assert(!/(?:id=["'](?:fileInput|imageFileInput|importPdf|sampleBtn)["']|class=["'](?:navbar|hero|features|footer)["'])/.test(shell), 'workspace restored marketing or excluded import chrome');
  assert(!shell.includes('Import MD') && !shell.includes('导入 Markdown') && !shell.includes('Import PDF') && !shell.includes('导入 PDF'), 'workspace includes excluded import actions');
  assert(shell.includes('<svg') && shell.includes('data-md-action="italic"'), 'formatting toolbar is missing SVG icons');
  const italicButton = document.querySelector('#skinFixture .format-btn[data-md-action="italic"]');
  assert(italicButton?.querySelector('svg'), 'self-test skin fixture is missing an SVG format icon');
  setPreviewMode(true);
  assert(document.getElementById('previewContainer').classList.contains('phone-mode'), 'phone mode did not apply the device frame class');
  assert(document.querySelectorAll('.mode-btn.active').length === 1 && document.getElementById('phoneModeBtn').classList.contains('active'), 'phone mode left a stale desktop active state');
  setPreviewMode(false);
  assert(!document.getElementById('previewContainer').classList.contains('phone-mode'), 'desktop mode did not leave the preview card undressed');
  assert(document.querySelectorAll('.mode-btn.active').length === 1 && document.getElementById('desktopModeBtn').classList.contains('active'), 'desktop mode is not exclusive');
}

async function buildWordImportFixture() {
  assert(typeof JSZip === 'function', 'pinned JSZip did not load');
  assert(typeof MarkionWordImportRuntime?.parseDocx === 'function', 'Word import runtime did not load');
  const zip = new JSZip();
  zip.file('[Content_Types].xml', `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Default Extension="gif" ContentType="image/gif"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>
</Types>`);
  zip.file('_rels/.rels', `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>`);
  zip.file('word/_rels/document.xml.rels', `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdNum" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>
  <Relationship Id="rIdImg" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image1.gif"/>
</Relationships>`);
  zip.file('word/numbering.xml', `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:abstractNum w:abstractNumId="0">
    <w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="•"/></w:lvl>
  </w:abstractNum>
  <w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>
</w:numbering>`);
  zip.file('word/media/image1.gif', fixtureGif);
  zip.file('word/document.xml', `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <w:body>
    <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Imported Title</w:t></w:r></w:p>
    <w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>Bullet item</w:t></w:r></w:p>
    <w:tbl>
      <w:tr>
        <w:tc><w:p><w:r><w:t>Alpha</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:r><w:t>Beta</w:t></w:r></w:p></w:tc>
      </w:tr>
      <w:tr>
        <w:tc><w:p><w:r><w:t>One</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:r><w:t>Two</w:t></w:r></w:p></w:tc>
      </w:tr>
    </w:tbl>
    <w:p>
      <m:oMath>
        <m:sSup>
          <m:e><m:r><m:t>x</m:t></m:r></m:e>
          <m:sup><m:r><m:t>2</m:t></m:r></m:sup>
        </m:sSup>
      </m:oMath>
    </w:p>
    <w:p>
      <w:r>
        <w:drawing>
          <wp:inline>
            <wp:docPr descr="fixture-gif"/>
            <a:graphic>
              <a:graphicData>
                <pic:pic>
                  <pic:blipFill>
                    <a:blip r:embed="rIdImg"/>
                  </pic:blipFill>
                </pic:pic>
              </a:graphicData>
            </a:graphic>
          </wp:inline>
        </w:drawing>
      </w:r>
    </w:p>
  </w:body>
</w:document>`);
  return zip.generateAsync({ type: 'blob' });
}

async function runWordImportChecks() {
  const stateBefore = {
    markdown: markdownEl.value,
    selectionStart: markdownEl.selectionStart,
    selectionEnd: markdownEl.selectionEnd,
  };
  markdownEl.value = 'keep on failure';
  render();
  const preserved = markdownEl.value;
  assert(!(await MarkionWorkspaceExports.importWordFile(new File(['plain'], 'notes.md', { type: 'text/markdown' }))), 'non-docx import was accepted');
  assert(markdownEl.value === preserved, 'invalid import replaced session Markdown');
  assert(/docx/i.test(statusEl.textContent), 'invalid import did not explain the .docx requirement');
  const oversized = {
    name: 'huge.docx',
    size: MarkionWorkspaceExports.WORD_IMPORT_MAX_BYTES + 1,
    type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    arrayBuffer() { throw new Error('oversized Word file was read'); },
  };
  assert(!(await MarkionWorkspaceExports.importWordFile(oversized)), 'oversized import was accepted');
  assert(markdownEl.value === preserved, 'oversized import replaced session Markdown');
  assert(/20/.test(statusEl.textContent), 'oversized import did not mention the size bound');
  assert(!(await MarkionWorkspaceExports.importWordFile(new File(['not-a-zip'], 'broken.docx'))), 'unreadable docx was accepted');
  assert(markdownEl.value === preserved, 'failed parse replaced session Markdown');
  const blob = await buildWordImportFixture();
  const file = new File([blob], 'fixture.docx', { type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document' });
  assert(await MarkionWorkspaceExports.importWordFile(file), 'representative Word import failed');
  const imported = markdownEl.value;
  assert(imported.includes('# Imported Title'), 'imported Markdown lost the heading');
  assert(imported.includes('- Bullet item') || imported.includes('• Bullet item'), 'imported Markdown lost the list');
  assert(imported.includes('| Alpha | Beta |') && imported.includes('| One | Two |'), 'imported Markdown lost the table');
  assert(/\$\$?x\^\{2\}/.test(imported) || imported.includes('$x^{2}$') || imported.includes('$$x^{2}$$'), 'imported Markdown lost math');
  assert(imported.includes('data:image/gif;base64,'), 'imported Markdown lost the embedded image data URI');
  assert(!/(?:file:|[A-Za-z]:\\|note\.assets\/|127\.0\.0\.1|localhost)/i.test(imported), 'imported Markdown leaked a filesystem or loopback path');
  assert(/session|tab|Copy Markdown|Markdown/i.test(statusEl.textContent), 'successful import did not disclose the session-local recovery path');
  markdownEl.value = stateBefore.markdown;
  render();
  markdownEl.setSelectionRange(stateBefore.selectionStart, stateBefore.selectionEnd);
}

async function runPdfChecks() {
  const stateBefore = {
    markdown: markdownEl.value,
    theme: themeSelect.value,
    fontSizeOffset,
    paraSpacingOffset,
  };
  const fetches = [];
  const originalFetch = window.fetch;
  const originalPrint = window.print;
  let printInvoked = false;
  window.print = () => { printInvoked = true; };
  window.fetch = function patchedFetch(resource, init) {
    fetches.push(String(resource));
    return originalFetch.call(this, resource, init);
  };
  try {
    markdownEl.value = '';
    render();
    assert(!(await MarkionWorkspaceExports.savePdf({ capturePrintHtml() { throw new Error('empty print captured'); } })), 'empty content opened print');
    assert(!printInvoked, 'empty content invoked the print dialog');
    themeSelect.value = 'night';
    fontSizeOffset = 2;
    paraSpacingOffset = 4;
    markdownEl.value = '# Print title\n\nParagraph for themed PDF.';
    document.getElementById('previewContainer')?.classList.add('phone-mode');
    let captured = '';
    assert(await MarkionWorkspaceExports.savePdf({ capturePrintHtml: html => { captured = html; } }), 'themed print capture failed');
    assert(!printInvoked, 'capture path still invoked window.print');
    assert(captured.includes('background:#0f1220') && captured.includes('font-size:17px'), 'print clone lost the selected theme or typography offsets');
    assert(!/h1\s*\{[^}]*font-size:\s*24px/.test(captured) && !/body\s*\{[^}]*color:\s*#333/.test(captured), 'print clone used a generic unthemed article stylesheet');
    assert(!captured.includes('phone-mode') && !captured.includes('preview-container') && !captured.includes('sessionDisclosure') && !captured.includes('markdown-format-toolbar'), 'print clone included workspace chrome or the phone bezel');
    assert(!/(?:blob:|file:|127\.0\.0\.1|localhost|markion\.wechat\.session)/i.test(captured), 'print clone leaked a session or local URL');
    assert(!/(?:pandoc|\/api\/pdf|native pdf)/i.test(captured), 'print clone referenced a native PDF path');
    assert(/Save as PDF|print dialog|打印|列印/i.test(statusEl.textContent), 'status claimed a PDF file was written');
    assert(!/saved as pdf|pdf download started|已保存为 PDF/i.test(statusEl.textContent), 'status claimed a PDF was saved');
    assert(!fetches.some(url => /pdf|pandoc/i.test(url)), 'Save as PDF contacted a native PDF endpoint');
  } finally {
    window.fetch = originalFetch;
    window.print = originalPrint;
    document.getElementById('previewContainer')?.classList.remove('phone-mode');
    markdownEl.value = stateBefore.markdown;
    themeSelect.value = stateBefore.theme;
    fontSizeOffset = stateBefore.fontSizeOffset;
    paraSpacingOffset = stateBefore.paraSpacingOffset;
    render();
  }
}

document.addEventListener('DOMContentLoaded', async () => {
  const [corpus, golden, docxFixture] = await Promise.all([
    fetch('compatibility-corpus.json').then(response => response.json()),
    fetch('compatibility-golden.json').then(response => response.json()),
    fetch('docx-compatibility-fixture.json').then(response => response.json())
  ]);
  const results = [];
  for (const option of themeSelect.options) {
    themeSelect.value = option.value || option.textContent;
    markdownEl.value = corpus.markdown;
    render();
    const html = previewEl.dataset.html || '';
    const missing = corpus.required_fragments.filter(fragment => !html.includes(fragment));
    const normalized = html.replace(/\s+/g, ' ').trim();
    const bytes = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(normalized));
    const digest = Array.from(new Uint8Array(bytes)).map(value => value.toString(16).padStart(2, '0')).join('');
    results.push({
      theme: themeSelect.value,
      pass: missing.length === 0 && digest === golden[themeSelect.value],
      missing,
      digest,
      expected: golden[themeSelect.value]
    });
  }
  const themeCount = results.length;
  try {
    runFormattingChecks();
  } catch (error) {
    results.push({ formatting: 'Markdown toolbar and shortcuts', pass: false, error: String(error?.message || error) });
  }
  try {
    await runExportChecks(docxFixture);
  } catch (error) {
    results.push({ export: 'browser artifacts', pass: false, error: String(error?.message || error) });
  }
  try {
    await runLocaleChecks();
  } catch (error) {
    results.push({ locales: 'workspace locales', pass: false, error: String(error?.message || error) });
  }
  try {
    await runSkinChecks();
  } catch (error) {
    results.push({ skin: 'editor chrome', pass: false, error: String(error?.message || error) });
  }
  try {
    await runWordImportChecks();
  } catch (error) {
    results.push({ import: 'Word import', pass: false, error: String(error?.message || error) });
  }
  try {
    await runPdfChecks();
  } catch (error) {
    results.push({ pdf: 'themed print-to-PDF', pass: false, error: String(error?.message || error) });
  }
  const passed = results.every(result => result.pass);
  statusEl.textContent = passed ? `PASS (${themeCount} themes + formatting + exports + skin + word + pdf)` : 'FAIL';
  statusEl.dataset.result = passed ? 'pass' : 'fail';
  document.getElementById('results').textContent = JSON.stringify(results, null, 2);
});
