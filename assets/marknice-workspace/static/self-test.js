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
document.addEventListener('DOMContentLoaded', async () => {
  const [corpus, golden] = await Promise.all([
    fetch('compatibility-corpus.json').then(response => response.json()),
    fetch('compatibility-golden.json').then(response => response.json())
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
  const passed = results.every(result => result.pass);
  statusEl.textContent = passed ? `PASS (${results.length} themes)` : 'FAIL';
  statusEl.dataset.result = passed ? 'pass' : 'fail';
  document.getElementById('results').textContent = JSON.stringify(results, null, 2);
});
