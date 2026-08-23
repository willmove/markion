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
marked.use({ extensions: [
  { name: 'mathBlock', level: 'block', start: src => (src.match(/\$\$/) || { index: -1 }).index, tokenizer(src) { const m = src.match(/^\$\$([\s\S]+?)\$\$/); if (m) return { type: 'mathBlock', raw: m[0], tex: m[1].trim() }; }, renderer: token => `<div class="math-block">\\[${token.tex}\\]</div>` },
  { name: 'mathInline', level: 'inline', start: src => (src.match(/\$/) || { index: -1 }).index, tokenizer(src) { const m = src.match(/^\$([^$\n]+?)\$/); if (m) return { type: 'mathInline', raw: m[0], tex: m[1].trim() }; }, renderer: token => `<span class="math-inline">\\(${token.tex}\\)</span>` }
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
  const documentPrepared = await MarkionWorkspaceExports.prepareExportArticle(snapshot);
  assert(documentPrepared.articleHtml.includes('background:#0f1220') && documentPrepared.articleHtml.includes('font-size:17px'), 'HTML export did not preserve the selected theme or typography offsets');
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
  const katexCss = await MarkionWorkspaceExports.katexCssWithEmbeddedFonts();
  const standalone = MarkionWorkspaceExports.standaloneHtml(snapshot, prepared, katexCss);
  assert(standalone.includes("default-src 'none'") && !standalone.includes('static/vendor') && !standalone.includes('script src=') && !/url\((?:['"])?fonts\//i.test(standalone), 'standalone HTML is not inert and self-contained');
  const offlineHtml = MarkionWorkspaceExports.standaloneHtml(snapshot, {
    articleHtml: '<section style="color:#123"><h1 style="font-size:29px">Offline export</h1></section>', fallbackCount: 0, remoteImageCount: 0
  }, katexCss);
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
  const passed = results.every(result => result.pass);
  statusEl.textContent = passed ? `PASS (${themeCount} themes + formatting + exports)` : 'FAIL';
  statusEl.dataset.result = passed ? 'pass' : 'fail';
  document.getElementById('results').textContent = JSON.stringify(results, null, 2);
});
