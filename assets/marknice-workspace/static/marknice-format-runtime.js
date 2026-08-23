/* Generated from MarkNice c009c1ec7e7c92f89afa5a32edcb126b5296bda7 by scripts/sync-marknice-workspace.ps1. */
const markdownFormatToolbar = document.querySelector('.markdown-format-toolbar');
let lastMarkdownSelection = { start: 0, end: 0 };

function rememberMarkdownSelection() {
  lastMarkdownSelection = {
    start: markdownEl.selectionStart || 0,
    end: markdownEl.selectionEnd || 0
  };
}

function getMarkdownSelection() {
  if (document.activeElement === markdownEl) rememberMarkdownSelection();
  const length = markdownEl.value.length;
  const start = Math.min(lastMarkdownSelection.start, length);
  const end = Math.min(lastMarkdownSelection.end, length);
  return {
    start: Math.min(start, end),
    end: Math.max(start, end)
  };
}

function updateMarkdownValue(value, selectionStart, selectionEnd, message) {
  markdownEl.value = value;
  markdownEl.focus();
  markdownEl.setSelectionRange(selectionStart, selectionEnd);
  rememberMarkdownSelection();
  render();
  if (message) statusEl.textContent = markdownFormatText('applied');
}

function wrapMarkdownSelection(before, after, placeholder, message) {
  const value = markdownEl.value;
  const selection = getMarkdownSelection();
  const start = selection.start;
  const end = selection.end;
  const selected = value.slice(start, end);
  const hasWrappedSelection = selected &&
    selected.startsWith(before) &&
    selected.endsWith(after) &&
    selected.length >= before.length + after.length;
  const hasOuterMarkers = start >= before.length &&
    value.slice(start - before.length, start) === before &&
    value.slice(end, end + after.length) === after;

  if (hasWrappedSelection) {
    const inner = selected.slice(before.length, selected.length - after.length);
    const next = value.slice(0, start) + inner + value.slice(end);
    updateMarkdownValue(next, start, start + inner.length, message);
    return;
  }

  if (hasOuterMarkers) {
    const next = value.slice(0, start - before.length) + selected + value.slice(end + after.length);
    const nextStart = start - before.length;
    updateMarkdownValue(next, nextStart, nextStart + selected.length, message);
    return;
  }

  const text = selected || placeholder;
  const next = value.slice(0, start) + before + text + after + value.slice(end);
  const innerStart = start + before.length;
  updateMarkdownValue(next, innerStart, innerStart + text.length, message);
}

function selectedLineRange() {
  const value = markdownEl.value;
  const selection = getMarkdownSelection();
  const start = selection.start;
  const end = selection.end;
  const lineStart = value.lastIndexOf('\n', Math.max(0, start - 1)) + 1;
  const endAnchor = end > start && value.charAt(end - 1) === '\n' ? end - 1 : end;
  let lineEnd = value.indexOf('\n', endAnchor);
  if (lineEnd === -1) lineEnd = value.length;
  return { value, start, end, lineStart, lineEnd };
}

function toggleSimpleLinePrefix(prefix, placeholder, message) {
  const range = selectedLineRange();
  const segment = range.value.slice(range.lineStart, range.lineEnd);

  if (!segment.trim()) {
    const inserted = prefix + placeholder;
    const next = range.value.slice(0, range.lineStart) + inserted + range.value.slice(range.lineEnd);
    updateMarkdownValue(next, range.lineStart + prefix.length, range.lineStart + inserted.length, message);
    return;
  }

  const lines = segment.split('\n');
  const nonEmptyLines = lines.filter(line => line.trim());
  const shouldRemove = nonEmptyLines.length > 0 && nonEmptyLines.every(line => line.startsWith(prefix));
  const nextSegment = lines.map(line => {
    if (!line.trim()) return line;
    return shouldRemove ? line.slice(prefix.length) : prefix + line;
  }).join('\n');
  const next = range.value.slice(0, range.lineStart) + nextSegment + range.value.slice(range.lineEnd);
  updateMarkdownValue(next, range.lineStart, range.lineStart + nextSegment.length, message);
}

function toggleHeading(level) {
  const range = selectedLineRange();
  const segment = range.value.slice(range.lineStart, range.lineEnd);
  const safeLevel = Math.min(Math.max(Number(level) || 1, 1), 6);
  const prefix = '#'.repeat(safeLevel) + ' ';

  if (!segment.trim()) {
    const inserted = prefix + markdownFormatText('headingPlaceholder');
    const next = range.value.slice(0, range.lineStart) + inserted + range.value.slice(range.lineEnd);
    updateMarkdownValue(next, range.lineStart + prefix.length, range.lineStart + inserted.length, `已插入 H${safeLevel} 标题`);
    return;
  }

  const lines = segment.split('\n');
  const nonEmptyLines = lines.filter(line => line.trim());
  const headingPattern = new RegExp('^#{' + safeLevel + '}\\s+');
  const shouldRemove = nonEmptyLines.length > 0 && nonEmptyLines.every(line => headingPattern.test(line));
  const nextSegment = lines.map(line => {
    if (!line.trim()) return line;
    return shouldRemove ? line.replace(/^#{1,6}\s+/, '') : line.replace(/^(#{1,6}\s+)?/, prefix);
  }).join('\n');
  const next = range.value.slice(0, range.lineStart) + nextSegment + range.value.slice(range.lineEnd);
  updateMarkdownValue(next, range.lineStart, range.lineStart + nextSegment.length, `已切换 H${safeLevel} 标题`);
}

function toggleOrderedList() {
  const range = selectedLineRange();
  const segment = range.value.slice(range.lineStart, range.lineEnd);

  if (!segment.trim()) {
    const inserted = '1. ' + markdownFormatText('listPlaceholder');
    const next = range.value.slice(0, range.lineStart) + inserted + range.value.slice(range.lineEnd);
    updateMarkdownValue(next, range.lineStart + 3, range.lineStart + inserted.length, '已插入有序列表');
    return;
  }

  const lines = segment.split('\n');
  const nonEmptyLines = lines.filter(line => line.trim());
  const shouldRemove = nonEmptyLines.length > 0 && nonEmptyLines.every(line => /^\d+\.\s+/.test(line));
  let index = 1;
  const nextSegment = lines.map(line => {
    if (!line.trim()) return line;
    if (shouldRemove) return line.replace(/^\d+\.\s+/, '');
    return (index++) + '. ' + line.replace(/^\d+\.\s+/, '');
  }).join('\n');
  const next = range.value.slice(0, range.lineStart) + nextSegment + range.value.slice(range.lineEnd);
  updateMarkdownValue(next, range.lineStart, range.lineStart + nextSegment.length, '已切换有序列表');
}

function insertCodeMarkdown() {
  const value = markdownEl.value;
  const selection = getMarkdownSelection();
  const start = selection.start;
  const end = selection.end;
  const selected = value.slice(start, end);

  wrapMarkdownSelection('`', '`', markdownFormatText('codePlaceholder'), '已插入代码标记');
}

function insertCodeBlock() {
  const value = markdownEl.value;
  const selection = getMarkdownSelection();
  const start = selection.start;
  const end = selection.end;
  const selected = value.slice(start, end) || markdownFormatText('codeBlockPlaceholder');
  const before = start === 0 ? '' : (value.charAt(start - 1) === '\n' ? '\n' : '\n\n');
  const after = value.charAt(end) === '\n' ? '\n' : '\n\n';
  const inserted = before + '```js\n' + selected + '\n```' + after;
  const codeStart = start + before.length + 6;
  const next = value.slice(0, start) + inserted + value.slice(end);
  updateMarkdownValue(next, codeStart, codeStart + selected.length, '已插入代码块');
}

function insertMarkdownLink() {
  const value = markdownEl.value;
  const selection = getMarkdownSelection();
  const start = selection.start;
  const end = selection.end;
  const selected = value.slice(start, end) || markdownFormatText('linkPlaceholder');
  const url = 'https://example.com';
  const inserted = '[' + selected + '](' + url + ')';
  const next = value.slice(0, start) + inserted + value.slice(end);
  const selectionStart = start + 1;
  updateMarkdownValue(next, selectionStart, selectionStart + selected.length, '已插入链接');
}

function insertMarkdownImage() {
  const value = markdownEl.value;
  const selection = getMarkdownSelection();
  const start = selection.start;
  const end = selection.end;
  const selected = value.slice(start, end) || markdownFormatText('imagePlaceholder');
  const url = markdownFormatText('imageUrlPlaceholder');
  const inserted = '![' + selected + '](' + url + ')';
  const next = value.slice(0, start) + inserted + value.slice(end);
  const selectionStart = start + 2;
  updateMarkdownValue(next, selectionStart, selectionStart + selected.length, '已插入图片语法');
}

function insertMarkdownTable() {
  const value = markdownEl.value;
  const selection = getMarkdownSelection();
  const start = selection.start;
  const end = selection.end;
  const before = start === 0 ? '' : (value.charAt(start - 1) === '\n' ? '\n' : '\n\n');
  const after = value.charAt(end) === '\n' ? '\n' : '\n\n';
  const table = markdownFormatText('tableTemplate');
  const inserted = before + table + after;
  const next = value.slice(0, start) + inserted + value.slice(end);
  const cellStart = start + before.length + 2;
  updateMarkdownValue(next, cellStart, cellStart + 3, '已插入表格');
}

function insertHorizontalRule() {
  const value = markdownEl.value;
  const selection = getMarkdownSelection();
  const start = selection.start;
  const end = selection.end;
  const before = start === 0 ? '' : (value.charAt(start - 1) === '\n' ? '\n' : '\n\n');
  const after = value.charAt(end) === '\n' ? '\n' : '\n\n';
  const inserted = before + '---' + after;
  const next = value.slice(0, start) + inserted + value.slice(end);
  const nextCursor = start + inserted.length;
  updateMarkdownValue(next, nextCursor, nextCursor, '已插入分割线');
}

function runMarkdownAction(action) {
  if (action === 'bold') wrapMarkdownSelection('**', '**', markdownFormatText('boldPlaceholder'), '已插入粗体标记');
  else if (action === 'italic') wrapMarkdownSelection('*', '*', markdownFormatText('italicPlaceholder'), '已插入斜体标记');
  else if (action === 'underline') wrapMarkdownSelection('<u>', '</u>', markdownFormatText('underlinePlaceholder'), '已插入下划线标记');
  else if (action === 'heading1') toggleHeading(1);
  else if (action === 'heading2') toggleHeading(2);
  else if (action === 'heading3') toggleHeading(3);
  else if (action === 'quote') toggleSimpleLinePrefix('> ', markdownFormatText('quotePlaceholder'), '已切换引用');
  else if (action === 'unorderedList') toggleSimpleLinePrefix('- ', markdownFormatText('listPlaceholder'), '已切换无序列表');
  else if (action === 'orderedList') toggleOrderedList();
  else if (action === 'code') insertCodeMarkdown();
  else if (action === 'codeBlock') insertCodeBlock();
  else if (action === 'link') insertMarkdownLink();
  else if (action === 'image') insertMarkdownImage();
  else if (action === 'table') insertMarkdownTable();
  else if (action === 'hr') insertHorizontalRule();
}

markdownFormatToolbar?.addEventListener('click', (e) => {
  const button = e.target.closest('[data-md-action]');
  if (!button) return;
  e.preventDefault();
  runMarkdownAction(button.dataset.mdAction);
});

markdownFormatToolbar?.addEventListener('mousedown', (e) => {
  if (e.target.closest('[data-md-action]')) e.preventDefault();
});

['focus', 'input', 'keyup', 'mouseup', 'select'].forEach(eventName => {
  markdownEl.addEventListener(eventName, rememberMarkdownSelection);
});

markdownEl.addEventListener('keydown', (e) => {
  if (!(e.ctrlKey || e.metaKey) || e.altKey) return;
  const key = e.key.toLowerCase();
  if (key === 'b') {
    e.preventDefault();
    runMarkdownAction('bold');
  } else if (key === 'i') {
    e.preventDefault();
    runMarkdownAction('italic');
  } else if (key === 'u') {
    e.preventDefault();
    runMarkdownAction('underline');
  } else if (key === 'k') {
    e.preventDefault();
    runMarkdownAction('link');
  }
});
window.MarkionMarkdownFormat = Object.freeze({ runMarkdownAction, rememberMarkdownSelection });
