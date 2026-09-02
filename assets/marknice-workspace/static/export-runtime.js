/* Markion export bridge: immutable browser-session artifacts only. */
(() => {
  const MAX_EMBEDDED_IMAGE_BYTES = 8 * 1024 * 1024;
  const MAX_EMBEDDED_IMAGE_TOTAL_BYTES = 24 * 1024 * 1024;
  const MAX_MANAGED_EXPORT_RESOURCES = 64;
  const WORD_IMPORT_MAX_BYTES = 20 * 1024 * 1024;
  const WORD_DOCX_TYPE = 'application/vnd.openxmlformats-officedocument.wordprocessingml.document';
  const PRINT_PAGE_CSS = '@page{margin:15mm 10mm}html,body{margin:0}body{padding:0 20px}img{max-width:100%;height:auto}table,figure,pre,blockquote{break-inside:avoid}';
  const SAFE_DATA_IMAGE = /^data:image\/(?:png|jpeg|gif|webp);base64,[a-z0-9+/=\s]+$/i;
  const BLOCKED_EXPORT_ELEMENTS = 'script,style,link,base,iframe,frame,frameset,object,embed,applet,form,input,button,select,textarea,meta,template';

  function exportedMessage(name, fallback) {
    return typeof locale?.[name] === 'function' || typeof locale?.[name] === 'string'
      ? locale[name]
      : fallback;
  }

  function updateStatus(message) {
    if (statusEl) statusEl.textContent = message;
  }

  function setBusy(button, busy) {
    if (!button) return;
    if ('disabled' in button) button.disabled = busy;
    button.setAttribute('aria-busy', String(busy));
  }

  function appendExportResourceStatus(result) {
    const details = [];
    if (result.fallbackCount) details.push(exportedMessage('exportFallback', n => `${n} images were replaced with text.`)(result.fallbackCount));
    if (result.remoteImageCount) details.push(exportedMessage('exportRemote', n => `${n} remote images remain linked.`)(result.remoteImageCount));
    return details.length ? ` ${details.join(' ')}` : '';
  }

  function escapeHtml(value) {
    const node = document.createElement('span');
    node.textContent = String(value || '');
    return node.innerHTML;
  }

  function launchName() {
    return (document.getElementById('documentName')?.textContent || '').trim();
  }

  function exportTitle(markdown) {
    return extractDocumentTitle(markdown) || launchName() || 'MarkNice export';
  }

  function filenameFor(title, extension) {
    const suffix = String(extension || '').replace(/^\.+/, '').toLowerCase() || 'txt';
    const suffixPattern = new RegExp(`(?:\\.${suffix.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})+$`, 'i');
    let base = String(title || '')
      .replace(/[\\/:*?"<>|]/g, ' ')
      .replace(/[\u0000-\u001f\u007f]/g, ' ')
      .replace(/\s+/g, ' ')
      .trim()
      .replace(suffixPattern, '')
      .replace(/[. ]+$/g, '')
      .trim();
    if (!base) base = 'MarkNice export';
    return `${base.slice(0, 80).replace(/[. ]+$/g, '') || 'MarkNice export'}.${suffix}`;
  }

  function buildExportSnapshot() {
    window.clearTimeout(window.__markionRenderTimer);
    render();
    const markdown = markdownEl.value;
    const parser = new DOMParser();
    const doc = parser.parseFromString(`<body>${previewEl.dataset.html || ''}</body>`, 'text/html');
    const article = doc.body.firstElementChild || doc.createElement('section');
    return Object.freeze({
      markdown,
      articleHtml: article.outerHTML,
      title: exportTitle(markdown),
      theme: themeSelect.value,
      fontSizeOffset,
      paraSpacingOffset,
      language: document.documentElement.lang || 'en',
      protectedResources: Object.freeze([...protectedImages.values()]
        .slice(0, MAX_MANAGED_EXPORT_RESOURCES)
        .map(resource => Object.freeze({ id: resource.id, blob: resource.blob || null }))),
    });
  }

  function isLoopbackHost(hostname) {
    const host = String(hostname || '').replace(/^\[|\]$/g, '').toLowerCase();
    return host === 'localhost' || host.endsWith('.localhost') || host === '0.0.0.0' || host === '::1' || /^127\./.test(host);
  }

  function safeHttpUrl(value) {
    try {
      const url = new URL(String(value || ''));
      return (url.protocol === 'https:' || url.protocol === 'http:') && !isLoopbackHost(url.hostname) ? url.href : '';
    } catch (_) {
      return '';
    }
  }

  function resourceForId(snapshot, id) {
    return snapshot.protectedResources.find(resource => resource.id === id) || null;
  }

  function dataUrlForBlob(blob) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onerror = () => reject(reader.error || new Error('could not read managed image'));
      reader.onload = () => resolve(String(reader.result || ''));
      reader.readAsDataURL(blob);
    });
  }

  function replaceWithImageFallback(documentNode, image, index) {
    const fallback = documentNode.createElement('span');
    fallback.setAttribute('style', 'display:block;border:1px dashed #999;padding:8px;color:#666;font-size:12px;');
    const label = (image.getAttribute('alt') || '').replace(/[\u0000-\u001f\u007f]/g, ' ').trim();
    fallback.textContent = label || `Image ${index + 1} unavailable`;
    image.replaceWith(fallback);
  }

  async function prepareExportArticle(snapshot) {
    const doc = new DOMParser().parseFromString(`<body>${snapshot.articleHtml}</body>`, 'text/html');
    const root = doc.body.firstElementChild || doc.createElement('section');
    root.querySelectorAll(BLOCKED_EXPORT_ELEMENTS).forEach(node => node.remove());
    root.querySelectorAll('svg').forEach(node => {
      if (!node.closest('.math-block, .math-inline')) node.remove();
    });
    let fallbackCount = 0;
    let remoteImageCount = 0;
    let embeddedBytes = 0;
    const images = [...root.querySelectorAll('img')];
    for (let index = 0; index < images.length; index += 1) {
      const image = images[index];
      const managed = image.getAttribute('data-mn-local-image-id');
      const original = image.getAttribute('src') || '';
      if (managed) {
        const resource = resourceForId(snapshot, managed);
        if (!resource?.blob || resource.blob.size > MAX_EMBEDDED_IMAGE_BYTES || embeddedBytes + resource.blob.size > MAX_EMBEDDED_IMAGE_TOTAL_BYTES || !/^image\//i.test(resource.blob.type || '')) {
          replaceWithImageFallback(doc, image, index);
          fallbackCount += 1;
          continue;
        }
        try {
          const dataUrl = await dataUrlForBlob(resource.blob);
          if (!SAFE_DATA_IMAGE.test(dataUrl)) throw new Error('unsupported managed image type');
          image.setAttribute('src', dataUrl);
          embeddedBytes += resource.blob.size;
        } catch (_) {
          replaceWithImageFallback(doc, image, index);
          fallbackCount += 1;
        }
        continue;
      }
      if (SAFE_DATA_IMAGE.test(original)) continue;
      const remote = safeHttpUrl(original);
      if (remote) {
        image.setAttribute('src', remote);
        remoteImageCount += 1;
        continue;
      }
      replaceWithImageFallback(doc, image, index);
      fallbackCount += 1;
    }
    [root, ...root.querySelectorAll('*')].forEach(node => {
      [...node.attributes].forEach(attribute => {
        const name = attribute.name.toLowerCase();
        if (name.startsWith('on') || name.startsWith('data-') || name === 'id' || name === 'name' || name === 'target' || name === 'formaction') {
          node.removeAttribute(attribute.name);
        } else if (name === 'class') {
          node.removeAttribute(attribute.name);
        } else if (name === 'style' && /(?:url\s*\(|expression\s*\(|behavior\s*:|-moz-binding)/i.test(attribute.value)) {
          node.removeAttribute(attribute.name);
        } else if (name === 'href') {
          const href = safeHttpUrl(attribute.value);
          if (href) node.setAttribute('href', href);
          else node.removeAttribute(attribute.name);
        } else if (name === 'src' && node.tagName !== 'IMG') {
          node.removeAttribute(attribute.name);
        } else if (name === 'srcset') {
          node.removeAttribute(attribute.name);
        }
      });
    });
    const localReference = [...root.querySelectorAll('[src],[href]')].some(node =>
      /(?:blob:|file:|https?:\/\/(?:127\.0\.0\.1|\[::1\]|localhost)(?::\d+)?)/i.test(node.getAttribute('src') || node.getAttribute('href') || '')
    );
    if (localReference || root.querySelector('[data-mn-local-image-id], [data-mn-resource], [data-mn-session]')) {
      throw new Error('unsafe local reference remained in export');
    }
    const html = root.outerHTML;
    return Object.freeze({ articleHtml: html, fallbackCount, remoteImageCount });
  }

  /* Word renders the DOCX body via altChunk(MHT), which supports neither MathML
     nor SVG math layout, so formulas are degraded to linear readable text from
     their preserved TeX source before the durable-artifact safety pass. */
  async function prepareWordArticle(snapshot) {
    const rewritten = typeof window.MarkionMath?.rewriteForWordExport === 'function'
      ? { ...snapshot, articleHtml: window.MarkionMath.rewriteForWordExport(snapshot.articleHtml) }
      : snapshot;
    return prepareExportArticle(rewritten);
  }

  function standaloneHtml(snapshot, prepared) {
    const title = escapeHtml(snapshot.title);
    return `<!doctype html>
<html lang="${escapeHtml(snapshot.language)}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; img-src data: http: https:; base-uri 'none'; form-action 'none'; frame-src 'none'; object-src 'none'">
<title>${title}</title>
</head>
<body>
${prepared.articleHtml}
</body>
</html>`;
  }

  function wordHtml(snapshot, bodyHtml) {
    return `<!doctype html>
<html xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:w="urn:schemas-microsoft-com:office:word" xmlns="http://www.w3.org/TR/REC-html40">
<head><meta charset="utf-8"><title>${escapeHtml(snapshot.title)}</title>
<style>
body{font-family:'PingFang SC','Microsoft YaHei',SimHei,sans-serif;font-size:14px;line-height:1.8;color:#333}h1{font-size:24px;margin:20px 0 10px;font-weight:bold}h2{font-size:20px;margin:18px 0 8px;font-weight:bold}h3{font-size:17px;margin:14px 0 6px;font-weight:bold}h4{font-size:15px;margin:12px 0 6px;font-weight:bold}h5{font-size:14px;margin:10px 0 6px;font-weight:bold}h6{font-size:13px;margin:10px 0 6px;font-weight:bold}p{margin:8px 0}ul,ol{margin:10px 0;padding-left:24px}li{margin:6px 0;text-align:left;text-align-last:left;text-justify:none}blockquote{margin:10px 0;padding:8px 16px;border-left:4px solid #ddd;background:#f8f8f8;color:#666}table{border-collapse:collapse;border-spacing:0;width:100%;margin:10px 0;mso-table-lspace:0pt;mso-table-rspace:0pt}td,th{border:1px solid #ccc;padding:2px 10px;mso-padding-alt:1.5pt 7.5pt 1.5pt 7.5pt;line-height:1.25;mso-line-height-rule:at-least}th{background:#f5f5f5;font-weight:bold}pre{background:#f6f6f6;padding:12px;white-space:pre-wrap}code{font-family:Consolas,Monaco,'Courier New',monospace;font-size:13px}img{max-width:560px;display:block;margin:10px auto}figure{text-align:center;margin:14px 0;page-break-inside:avoid}a{color:#0066cc;text-decoration:underline}
</style></head><body>${bodyHtml}</body></html>`;
  }

  async function createDocxBlob(snapshot, prepared) {
    if (typeof window.htmlDocx?.asBlob !== 'function' || typeof window.prepareHtmlForWordExport !== 'function') {
      throw new Error('DOCX converter unavailable');
    }
    const wordBody = await window.prepareHtmlForWordExport(prepared.articleHtml);
    const converted = window.htmlDocx.asBlob(wordHtml(snapshot, wordBody));
    if (!(converted instanceof Blob) || converted.size < 64) throw new Error('invalid DOCX converter output');
    return new Blob([converted], { type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document' });
  }

  function startDownload(blob, filename) {
    if (!(blob instanceof Blob) || blob.size === 0) throw new Error('empty export blob');
    const objectUrl = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = objectUrl;
    anchor.download = filename;
    anchor.style.display = 'none';
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(objectUrl), 1_000);
  }

  function hasRenderableContent(snapshot) {
    return !!snapshot.markdown.trim();
  }

  function legacyCopyMarkdown(source) {
    const previousFocus = document.activeElement;
    const selectionStart = markdownEl.selectionStart;
    const selectionEnd = markdownEl.selectionEnd;
    const helper = document.createElement('textarea');
    helper.value = source;
    helper.setAttribute('readonly', '');
    helper.style.cssText = 'position:fixed;left:-99999px;top:0;width:1px;height:1px;opacity:0;';
    document.body.appendChild(helper);
    try {
      helper.select();
      if (!document.execCommand('copy')) throw new Error('clipboard denied');
    } finally {
      helper.remove();
      if (previousFocus?.focus) previousFocus.focus();
      markdownEl.setSelectionRange(selectionStart, selectionEnd);
    }
  }

  async function copyMarkdown() {
    const source = markdownEl.value;
    if (!source.length) {
      updateStatus(exportedMessage('empty', 'Enter content before copying.'));
      return false;
    }
    try {
      try {
        if (!navigator.clipboard?.writeText) throw new Error('modern clipboard unavailable');
        await navigator.clipboard.writeText(source);
      } catch (_) {
        legacyCopyMarkdown(source);
      }
      updateStatus(exportedMessage('markdownCopied', 'Copied Markdown source.'));
      return true;
    } catch (error) {
      console.error(error);
      updateStatus(exportedMessage('markdownDenied', 'Markdown could not be copied.'));
      return false;
    }
  }

  async function downloadHtml() {
    const button = document.getElementById('downloadHtmlBtn');
    if (button?.disabled) return false;
    setBusy(button, true);
    try {
      const snapshot = buildExportSnapshot();
      if (!hasRenderableContent(snapshot)) {
        updateStatus(exportedMessage('empty', 'Enter content before exporting.'));
        return false;
      }
      updateStatus(exportedMessage('htmlPreparing', 'Preparing themed HTML…'));
      const prepared = await prepareExportArticle(snapshot);
      startDownload(new Blob([standaloneHtml(snapshot, prepared)], { type: 'text/html;charset=utf-8' }), filenameFor(snapshot.title, 'html'));
      updateStatus(`${exportedMessage('htmlDownloaded', 'Themed HTML download started.')}${appendExportResourceStatus(prepared)}`);
      return true;
    } catch (error) {
      console.error(error);
      updateStatus(exportedMessage('exportFailed', 'Export could not be completed.'));
      return false;
    } finally {
      setBusy(button, false);
    }
  }

  async function downloadDocx() {
    const button = document.getElementById('downloadDocxBtn');
    if (button?.disabled) return false;
    setBusy(button, true);
    try {
      const snapshot = buildExportSnapshot();
      if (!hasRenderableContent(snapshot)) {
        updateStatus(exportedMessage('empty', 'Enter content before exporting.'));
        return false;
      }
      updateStatus(exportedMessage('docxPreparing', 'Preparing browser-generated DOCX…'));
      const prepared = await prepareWordArticle(snapshot);
      startDownload(await createDocxBlob(snapshot, prepared), filenameFor(snapshot.title, 'docx'));
      updateStatus(`${exportedMessage('docxDownloaded', 'Browser-generated DOCX download started.')}${appendExportResourceStatus(prepared)} ${exportedMessage('docxNote', '')}`.trim());
      return true;
    } catch (error) {
      console.error(error);
      updateStatus(exportedMessage('exportFailed', 'Export could not be completed.'));
      return false;
    } finally {
      setBusy(button, false);
    }
  }

  function isSupportedWordFile(file) {
    if (!file) return false;
    const name = String(file.name || '').toLowerCase();
    return name.endsWith('.docx') || String(file.type || '').toLowerCase() === WORD_DOCX_TYPE;
  }

  async function importWordFile(file) {
    const previous = markdownEl.value;
    const label = document.getElementById('importWordLabel');
    if (label?.getAttribute('aria-busy') === 'true') return false;
    setBusy(label, true);
    try {
      if (!isSupportedWordFile(file)) {
        updateStatus(exportedMessage('importWordInvalid', 'Import failed. Choose a .docx file.'));
        return false;
      }
      if ((file.size || 0) > WORD_IMPORT_MAX_BYTES) {
        updateStatus(exportedMessage('importWordTooLarge', 'The Word file is too large for this workspace (20 MB limit).'));
        return false;
      }
      const runtime = window.MarkionWordImportRuntime;
      if (typeof JSZip === 'undefined' || typeof runtime?.parseDocx !== 'function' || typeof runtime?.htmlToMarkdown !== 'function') {
        throw new Error('Word import runtime unavailable');
      }
      updateStatus(exportedMessage('importWordPreparing', 'Importing Word document…'));
      const html = await runtime.parseDocx(await file.arrayBuffer());
      const markdown = runtime.htmlToMarkdown(html);
      if (!String(markdown || '').trim()) throw new Error('empty conversion');
      markdownEl.value = markdown;
      render();
      updateStatus(exportedMessage(
        'importWordSuccess',
        'Imported into this browser tab only. Copy Markdown or save the session Markdown to keep it in Markion.'
      ));
      return true;
    } catch (error) {
      console.error(error);
      markdownEl.value = previous;
      render();
      updateStatus(exportedMessage('importWordInvalid', 'Import failed. Choose a .docx file.'));
      return false;
    } finally {
      setBusy(label, false);
      const input = document.getElementById('wordFileInput');
      if (input) input.value = '';
    }
  }

  function buildPrintHtml(snapshot, prepared) {
    return `<!doctype html>
<html lang="${escapeHtml(snapshot.language)}">
<head>
<meta charset="utf-8">
<title>${escapeHtml(snapshot.title)}</title>
<style>${PRINT_PAGE_CSS}</style>
</head>
<body>
${prepared.articleHtml}
</body>
</html>`;
  }

  function openHiddenPrintDialog(html, title) {
    const iframe = document.createElement('iframe');
    iframe.setAttribute('title', title);
    iframe.setAttribute('aria-hidden', 'true');
    iframe.style.cssText = 'position:fixed;left:-9999px;top:0;width:800px;height:600px;border:none;';
    document.body.appendChild(iframe);
    const doc = iframe.contentDocument || iframe.contentWindow.document;
    doc.open();
    doc.write(html);
    doc.close();
    doc.title = title;
    const win = iframe.contentWindow;
    const cleanup = () => { if (iframe.parentNode) iframe.remove(); };
    if (win) win.onafterprint = cleanup;
    window.setTimeout(() => {
      win?.print();
      window.setTimeout(cleanup, 60_000);
    }, 300);
  }

  async function savePdf(options = {}) {
    const button = document.getElementById('savePdfBtn');
    if (button?.disabled) return false;
    setBusy(button, true);
    try {
      const snapshot = buildExportSnapshot();
      if (!hasRenderableContent(snapshot)) {
        updateStatus(exportedMessage('empty', 'Enter content before exporting.'));
        return false;
      }
      updateStatus(exportedMessage('pdfPreparing', 'Preparing themed preview for print…'));
      const prepared = await prepareExportArticle(snapshot);
      const html = buildPrintHtml(snapshot, prepared);
      if (typeof options.capturePrintHtml === 'function') {
        options.capturePrintHtml(html);
      } else {
        openHiddenPrintDialog(html, snapshot.title);
      }
      updateStatus(`${exportedMessage('pdfPrintOpened', 'Choose “Save as PDF” in the print dialog. This prints the MarkNice preview, not Markion’s native PDF export.')}${appendExportResourceStatus(prepared)}`);
      return true;
    } catch (error) {
      console.error(error);
      updateStatus(exportedMessage('exportFailed', 'Export could not be completed.'));
      return false;
    } finally {
      setBusy(button, false);
    }
  }

  window.MarkionWorkspaceExports = Object.freeze({
    buildExportSnapshot,
    filenameFor,
    prepareExportArticle,
    prepareWordArticle,
    standaloneHtml,
    wordHtml,
    createDocxBlob,
    copyMarkdown,
    downloadHtml,
    downloadDocx,
    importWordFile,
    buildPrintHtml,
    savePdf,
    MAX_EMBEDDED_IMAGE_BYTES,
    MAX_EMBEDDED_IMAGE_TOTAL_BYTES,
    MAX_MANAGED_EXPORT_RESOURCES,
    WORD_IMPORT_MAX_BYTES,
  });

  document.addEventListener('DOMContentLoaded', () => {
    document.getElementById('copyMarkdownBtn')?.addEventListener('click', copyMarkdown);
    document.getElementById('downloadHtmlBtn')?.addEventListener('click', downloadHtml);
    document.getElementById('downloadDocxBtn')?.addEventListener('click', downloadDocx);
    document.getElementById('savePdfBtn')?.addEventListener('click', () => savePdf());
    document.getElementById('wordFileInput')?.addEventListener('change', event => {
      const file = event.target.files && event.target.files[0];
      if (file) importWordFile(file);
    });
  });
})();
