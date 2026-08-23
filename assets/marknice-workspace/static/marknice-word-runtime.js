/* Generated from MarkNice c009c1ec7e7c92f89afa5a32edcb126b5296bda7 by scripts/sync-marknice-workspace.ps1. */
// ===== Word export helpers =====
const WORD_EXPORT_MAX_IMAGE_WIDTH_PX = 560;

function wordPxToPt(px) {
  return ((px * 72) / 96).toFixed(2).replace(/\.?0+$/, '');
}

function wordUpsertStyle(style, prop, value) {
  const pattern = new RegExp(`(^|;)\\s*${prop}\\s*:[^;]*`, 'i');
  if (pattern.test(style)) return style.replace(pattern, `$1${prop}:${value}`);
  return `${style.replace(/;?\s*$/, '')};${prop}:${value}`;
}

function wordStyleValue(style, prop) {
  const match = String(style || '').match(new RegExp(`(?:^|;)\\s*${prop}\\s*:\\s*([^;]+)`, 'i'));
  return match ? match[1].trim() : '';
}

function wordHalfPxLength(value, fallback) {
  const match = String(value || '').trim().match(/^(-?\d+(?:\.\d+)?)px$/i);
  if (!match) return fallback;
  return `${Math.max(Math.min(Number(match[1]) / 2, 2), 1).toFixed(2).replace(/\.?0+$/, '')}px`;
}

function wordPxLengthToPt(value, fallback) {
  const match = String(value || '').trim().match(/^(-?\d+(?:\.\d+)?)px$/i);
  if (!match) return fallback;
  return `${wordPxToPt(Number(match[1]))}pt`;
}

function wordCompactTableBlockStyle(style) {
  let nextStyle = style || '';
  nextStyle = wordUpsertStyle(nextStyle, 'margin', '5pt 0cm');
  nextStyle = wordUpsertStyle(nextStyle, 'margin-top', '5pt');
  nextStyle = wordUpsertStyle(nextStyle, 'margin-bottom', '5pt');
  nextStyle = wordUpsertStyle(nextStyle, 'mso-para-margin', '5pt 0cm 5pt 0cm');
  nextStyle = wordUpsertStyle(nextStyle, 'mso-para-margin-top', '5pt');
  nextStyle = wordUpsertStyle(nextStyle, 'mso-para-margin-bottom', '5pt');
  nextStyle = wordUpsertStyle(nextStyle, 'mso-margin-top-alt', '5pt');
  nextStyle = wordUpsertStyle(nextStyle, 'mso-margin-bottom-alt', '5pt');
  nextStyle = wordUpsertStyle(nextStyle, 'line-height', '1.25');
  nextStyle = wordUpsertStyle(nextStyle, 'mso-line-height-rule', 'at-least');
  return nextStyle;
}

function wordAddClass(el, className) {
  const classes = (el.getAttribute('class') || '').split(/\s+/).filter(Boolean);
  if (!classes.includes(className)) classes.push(className);
  el.setAttribute('class', classes.join(' '));
}

function wordLeftAlignStyle(style) {
  let nextStyle = style || '';
  nextStyle = wordUpsertStyle(nextStyle, 'text-align', 'left');
  nextStyle = wordUpsertStyle(nextStyle, 'text-align-last', 'left');
  nextStyle = wordUpsertStyle(nextStyle, 'text-justify', 'none');
  return nextStyle;
}

function wordNodeHasVisibleContent(node) {
  if (node.nodeType === 3) return !!node.nodeValue.trim();
  if (node.nodeType !== 1) return false;
  if (/^(br)$/i.test(node.tagName)) return false;
  return !!(node.textContent || '').trim() || /^(img|svg|math)$/i.test(node.tagName);
}

function wordSegmentTextLength(nodes) {
  return nodes
    .map(node => node.textContent || '')
    .join('')
    .replace(/\s+/g, '')
    .length;
}

function wordCreateSoftBreakParagraph(doc, nodes, sourceStyle, compact) {
  if (!nodes.some(wordNodeHasVisibleContent)) return null;
  const p = doc.createElement('p');
  wordAddClass(p, 'MsoNormal');
  let style = wordLeftAlignStyle(sourceStyle || '');
  style = wordUpsertStyle(style, 'margin', compact ? '0 0 4pt 0' : '0 0 6pt 0');
  style = wordUpsertStyle(style, 'mso-para-margin', compact ? '0 0 4pt 0' : '0 0 6pt 0');
  style = wordUpsertStyle(style, 'line-height', '1.45');
  style = wordUpsertStyle(style, 'mso-line-height-rule', 'at-least');
  p.setAttribute('style', style);
  nodes.forEach(node => p.appendChild(node.cloneNode(true)));
  return p;
}

function wordSplitDirectSoftBreaks(doc, block) {
  const children = Array.from(block.childNodes);
  if (!children.some(node => node.nodeType === 1 && node.tagName === 'BR')) return;

  const segments = [];
  let current = [];
  children.forEach(node => {
    if (node.nodeType === 1 && node.tagName === 'BR') {
      segments.push(current);
      current = [];
    } else {
      current.push(node);
    }
  });
  segments.push(current);

  const firstLength = wordSegmentTextLength(segments[0] || []);
  const inList = block.tagName === 'LI' || !!block.closest('li');
  if (!inList && firstLength > 24) return;

  const sourceStyle = wordLeftAlignStyle(block.getAttribute('style') || '');
  const paragraphs = segments
    .map((segment, index) => wordCreateSoftBreakParagraph(doc, segment, sourceStyle, index === 0 && firstLength <= 24))
    .filter(Boolean);
  if (!paragraphs.length) return;

  if (block.tagName === 'LI') {
    block.innerHTML = '';
    block.setAttribute('style', sourceStyle);
    paragraphs.forEach(p => block.appendChild(p));
    return;
  }

  const parent = block.parentNode;
  paragraphs.forEach(p => parent.insertBefore(p, block));
  parent.removeChild(block);
}

function normalizeWordSoftBreaks(doc) {
  doc.body.querySelectorAll('li').forEach(li => {
    li.setAttribute('style', wordLeftAlignStyle(li.getAttribute('style') || ''));
  });
  Array.from(doc.body.querySelectorAll('li, p, div, section')).forEach(block => {
    if (block.querySelector('br')) block.setAttribute('style', wordLeftAlignStyle(block.getAttribute('style') || ''));
    wordSplitDirectSoftBreaks(doc, block);
  });
}

function wordWrapInlineCellContent(doc, cell) {
  const hasBlock = Array.from(cell.children).some(child => /^(p|div|section|ul|ol|table|blockquote|pre)$/i.test(child.tagName));
  if (hasBlock) return;
  const nodes = Array.from(cell.childNodes);
  const hasContent = nodes.some(node => node.nodeType === 1 || (node.nodeType === 3 && node.nodeValue.trim()));
  if (!hasContent) return;

  const p = doc.createElement('p');
  wordAddClass(p, 'MsoNormal');
  p.setAttribute('style', wordCompactTableBlockStyle(''));
  nodes.forEach(node => p.appendChild(node));
  cell.appendChild(p);
}

function wordCompactTableCellStyle(style) {
  const padding = wordStyleValue(style, 'padding');
  const parts = padding ? padding.split(/\s+/).filter(Boolean) : [];
  let top = '2px';
  let right = '10px';
  let bottom = '2px';
  let left = '10px';

  if (parts.length === 1) {
    top = bottom = wordHalfPxLength(parts[0], top);
    right = left = parts[0];
  } else if (parts.length === 2) {
    top = bottom = wordHalfPxLength(parts[0], top);
    right = left = parts[1];
  } else if (parts.length === 3) {
    top = wordHalfPxLength(parts[0], top);
    right = left = parts[1];
    bottom = wordHalfPxLength(parts[2], bottom);
  } else if (parts.length >= 4) {
    top = wordHalfPxLength(parts[0], top);
    right = parts[1];
    bottom = wordHalfPxLength(parts[2], bottom);
    left = parts[3];
  }

  let nextStyle = wordUpsertStyle(style, 'padding', `${top} ${right} ${bottom} ${left}`);
  nextStyle = wordUpsertStyle(nextStyle, 'mso-padding-alt', `${wordPxLengthToPt(top, '1.5pt')} ${wordPxLengthToPt(right, '7.5pt')} ${wordPxLengthToPt(bottom, '1.5pt')} ${wordPxLengthToPt(left, '7.5pt')}`);
  nextStyle = wordUpsertStyle(nextStyle, 'line-height', '1.25');
  nextStyle = wordUpsertStyle(nextStyle, 'mso-line-height-rule', 'at-least');
  return nextStyle;
}

function compactWordTableCells(doc) {
  doc.body.querySelectorAll('table').forEach(table => {
    let tableStyle = table.getAttribute('style') || '';
    tableStyle = wordUpsertStyle(tableStyle, 'border-collapse', 'collapse');
    tableStyle = wordUpsertStyle(tableStyle, 'border-spacing', '0');
    tableStyle = wordUpsertStyle(tableStyle, 'mso-table-lspace', '0pt');
    tableStyle = wordUpsertStyle(tableStyle, 'mso-table-rspace', '0pt');
    table.setAttribute('style', tableStyle);
    table.setAttribute('cellpadding', '0');
    table.setAttribute('cellspacing', '0');
  });
  doc.body.querySelectorAll('td, th').forEach(cell => {
    cell.setAttribute('style', wordCompactTableCellStyle(cell.getAttribute('style') || ''));
    wordWrapInlineCellContent(doc, cell);
    cell.querySelectorAll('p, div, section').forEach(block => {
      wordAddClass(block, 'MsoNormal');
      block.setAttribute('style', wordCompactTableBlockStyle(block.getAttribute('style') || ''));
    });
  });
}

function wordBytesFromDataUrl(src) {
  const match = String(src || '').match(/^data:([^;,]+)(;base64)?,([\s\S]*)$/i);
  if (!match) return null;
  try {
    if (match[2]) {
      const binary = atob(match[3]);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      return bytes;
    }
    const decoded = decodeURIComponent(match[3]);
    const bytes = new Uint8Array(decoded.length);
    for (let i = 0; i < decoded.length; i++) bytes[i] = decoded.charCodeAt(i);
    return bytes;
  } catch (err) {
    return null;
  }
}

function wordReadUint32BE(bytes, offset) {
  return ((bytes[offset] << 24) | (bytes[offset + 1] << 16) | (bytes[offset + 2] << 8) | bytes[offset + 3]) >>> 0;
}

function wordReadUint16BE(bytes, offset) {
  return (bytes[offset] << 8) | bytes[offset + 1];
}

function wordReadUint16LE(bytes, offset) {
  return bytes[offset] | (bytes[offset + 1] << 8);
}

function wordReadUint24LE(bytes, offset) {
  return bytes[offset] | (bytes[offset + 1] << 8) | (bytes[offset + 2] << 16);
}

function wordReadUint32LE(bytes, offset) {
  return (bytes[offset] | (bytes[offset + 1] << 8) | (bytes[offset + 2] << 16) | (bytes[offset + 3] << 24)) >>> 0;
}

function wordAscii(bytes, start, length) {
  let out = '';
  for (let i = start; i < start + length && i < bytes.length; i++) out += String.fromCharCode(bytes[i]);
  return out;
}

function wordTextFromDataUrl(src) {
  const bytes = wordBytesFromDataUrl(src);
  if (!bytes) return null;
  try {
    return new TextDecoder('utf-8').decode(bytes);
  } catch (err) {
    let out = '';
    for (const byte of bytes) out += String.fromCharCode(byte);
    return out;
  }
}

function wordCssLengthToPx(value) {
  const match = String(value || '').trim().match(/^([\d.]+)\s*(px|pt|in|cm|mm)?$/i);
  if (!match) return null;
  const n = Number.parseFloat(match[1]);
  if (!Number.isFinite(n) || n <= 0) return null;
  const unit = (match[2] || 'px').toLowerCase();
  if (unit === 'pt') return (n * 96) / 72;
  if (unit === 'in') return n * 96;
  if (unit === 'cm') return (n * 96) / 2.54;
  if (unit === 'mm') return (n * 96) / 25.4;
  return n;
}

function wordSvgDimensions(src) {
  if (!/^data:image\/svg\+xml/i.test(src || '')) return null;
  const text = wordTextFromDataUrl(src);
  if (!text) return null;

  const width = /<svg\b[^>]*\bwidth=["']?([^"'\s>]+)/i.exec(text)?.[1];
  const height = /<svg\b[^>]*\bheight=["']?([^"'\s>]+)/i.exec(text)?.[1];
  const widthPx = width ? wordCssLengthToPx(width) : null;
  const heightPx = height ? wordCssLengthToPx(height) : null;
  if (widthPx && heightPx) return { width: Math.round(widthPx), height: Math.round(heightPx) };

  const viewBox = /<svg\b[^>]*\bviewBox=["']\s*([\d.+-]+)\s+([\d.+-]+)\s+([\d.+-]+)\s+([\d.+-]+)\s*["']/i.exec(text);
  if (!viewBox) return null;
  const vbWidth = Number.parseFloat(viewBox[3]);
  const vbHeight = Number.parseFloat(viewBox[4]);
  if (!Number.isFinite(vbWidth) || !Number.isFinite(vbHeight) || vbWidth <= 0 || vbHeight <= 0) return null;
  return { width: Math.round(vbWidth), height: Math.round(vbHeight) };
}

function wordDataImageDimensions(src) {
  const svg = wordSvgDimensions(src);
  if (svg) return svg;

  const bytes = wordBytesFromDataUrl(src);
  if (!bytes || bytes.length < 10) return null;

  const png =
    bytes[0] === 0x89 &&
    bytes[1] === 0x50 &&
    bytes[2] === 0x4e &&
    bytes[3] === 0x47 &&
    bytes[12] === 0x49 &&
    bytes[13] === 0x48 &&
    bytes[14] === 0x44 &&
    bytes[15] === 0x52;
  if (png && bytes.length >= 24) return { width: wordReadUint32BE(bytes, 16), height: wordReadUint32BE(bytes, 20) };

  const gif = bytes[0] === 0x47 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes.length >= 10;
  if (gif) return { width: wordReadUint16LE(bytes, 6), height: wordReadUint16LE(bytes, 8) };

  if (bytes[0] === 0xff && bytes[1] === 0xd8) {
    let offset = 2;
    while (offset + 9 < bytes.length) {
      if (bytes[offset] !== 0xff) {
        offset++;
        continue;
      }
      const marker = bytes[offset + 1];
      const length = wordReadUint16BE(bytes, offset + 2);
      if (length < 2) return null;
      if (
        marker === 0xc0 ||
        marker === 0xc1 ||
        marker === 0xc2 ||
        marker === 0xc3 ||
        marker === 0xc5 ||
        marker === 0xc6 ||
        marker === 0xc7 ||
        marker === 0xc9 ||
        marker === 0xca ||
        marker === 0xcb ||
        marker === 0xcd ||
        marker === 0xce ||
        marker === 0xcf
      ) {
        return { width: wordReadUint16BE(bytes, offset + 7), height: wordReadUint16BE(bytes, offset + 5) };
      }
      offset += 2 + length;
    }
  }

  const webp = wordAscii(bytes, 0, 4) === 'RIFF' && wordAscii(bytes, 8, 4) === 'WEBP' && bytes.length >= 30;
  if (webp) {
    const chunk = wordAscii(bytes, 12, 4);
    if (chunk === 'VP8X') return { width: wordReadUint24LE(bytes, 24) + 1, height: wordReadUint24LE(bytes, 27) + 1 };
    if (chunk === 'VP8L' && bytes[20] === 0x2f) {
      const bits = wordReadUint32LE(bytes, 21);
      return { width: (bits & 0x3fff) + 1, height: ((bits >> 14) & 0x3fff) + 1 };
    }
    if (chunk === 'VP8 ' && bytes[23] === 0x9d && bytes[24] === 0x01 && bytes[25] === 0x2a) {
      return { width: wordReadUint16LE(bytes, 26) & 0x3fff, height: wordReadUint16LE(bytes, 28) & 0x3fff };
    }
  }

  return null;
}

function wordLoadImage(src) {
  return new Promise((resolve, reject) => {
    const image = new Image();
    const timeout = window.setTimeout(() => reject(new Error('Image load timeout')), 8000);
    image.onload = () => {
      window.clearTimeout(timeout);
      resolve(image);
    };
    image.onerror = () => {
      window.clearTimeout(timeout);
      reject(new Error('Image load failed'));
    };
    image.src = src;
  });
}

async function wordMeasureImage(src, fallbackDimensions) {
  if (fallbackDimensions?.width && fallbackDimensions.height) return fallbackDimensions;
  if (!src) return null;
  try {
    const image = await wordLoadImage(src);
    const width = image.naturalWidth || image.width || 0;
    const height = image.naturalHeight || image.height || 0;
    if (!width || !height) return null;
    return { width, height };
  } catch (err) {
    return fallbackDimensions;
  }
}

async function prepareHtmlForWordExport(html) {
  const doc = new DOMParser().parseFromString(`<body>${html}</body>`, 'text/html');
  normalizeWordSoftBreaks(doc);
  compactWordTableCells(doc);
  for (const img of Array.from(doc.body.querySelectorAll('img'))) {
    const src = img.getAttribute('src') || '';
    const dimensions = await wordMeasureImage(src, wordDataImageDimensions(src));
    const hasDimensions = !!dimensions && dimensions.width > 0 && dimensions.height > 0;
    const displayWidth = Math.max(1, Math.min(hasDimensions ? dimensions.width : WORD_EXPORT_MAX_IMAGE_WIDTH_PX, WORD_EXPORT_MAX_IMAGE_WIDTH_PX));
    const displayHeight = hasDimensions ? Math.max(1, Math.round((dimensions.height * displayWidth) / dimensions.width)) : null;
    const scale = hasDimensions ? Math.round((displayWidth / dimensions.width) * 100) : 100;
    const style = [
      `width:${wordPxToPt(displayWidth)}pt`,
      displayHeight ? `height:${wordPxToPt(displayHeight)}pt` : '',
      `max-width:${wordPxToPt(WORD_EXPORT_MAX_IMAGE_WIDTH_PX)}pt`,
      'display:block',
      'margin:10px auto',
      'border-radius:0',
      hasDimensions ? `mso-width-percent:${scale * 10}` : '',
      hasDimensions ? `mso-height-percent:${scale * 10}` : '',
    ].filter(Boolean).join(';') + ';';
    img.setAttribute('style', style);
    img.setAttribute('width', String(displayWidth));
    if (displayHeight) img.setAttribute('height', String(displayHeight));
    else img.removeAttribute('height');

    const parent = img.parentElement;
    if (parent && /^(figure|p|div|section)$/i.test(parent.tagName)) {
      let parentStyle = parent.getAttribute('style') || '';
      parentStyle = wordUpsertStyle(parentStyle, 'text-align', 'center');
      parentStyle = wordUpsertStyle(parentStyle, 'margin', '14px 0');
      parentStyle = wordUpsertStyle(parentStyle, 'page-break-inside', 'avoid');
      parent.setAttribute('style', parentStyle);
    }
  }
  return doc.body.innerHTML;
}
