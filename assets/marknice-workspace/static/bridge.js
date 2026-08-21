/* Markion bridge: immutable handoff, protected images, and tab-local edits. */
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

const messages = {
  en: {
    session: 'Edits in this browser tab are session-local and are not saved to Markion.',
    privacy: 'App assets stay local. Remote images in the document may contact their authored hosts without a referrer.',
    opened: 'Publishing workspace opened', expired: 'This publishing session expired. Return to Markion and launch it again.',
    setup: 'The publishing workspace could not load. Return to Markion and launch it again.',
    unresolved: n => `${n} local image(s) could not be resolved.`, resolved: n => `${n} protected local preview image(s)`,
    omitPrompt: n => `${n} local image(s) cannot be pasted directly into WeChat. Choose OK to copy without them, or Cancel to keep editing.`,
    copied: 'Copied rich HTML and plain text.', partial: n => `Copied without ${n} local image(s).`,
    denied: 'Clipboard access was denied. Allow clipboard permission and try again.', empty: 'Enter content before copying.',
    desktop: 'Desktop', phone: 'Phone', copy: 'Copy for WeChat', theme: 'Theme', font: 'Font size', spacing: 'Spacing',
    fontDown: 'Decrease font size', fontUp: 'Increase font size', spacingDown: 'Decrease paragraph spacing', spacingUp: 'Increase paragraph spacing'
  },
  'zh-hans': {
    session: '浏览器中的修改仅保留在当前标签页，不会保存回 Markion。',
    privacy: '应用资源完全本地；文档中的远程图片可能在无来源信息的情况下连接原始站点。',
    opened: '本地发布工作区已打开', expired: '发布会话已过期，请返回 Markion 重新打开。',
    setup: '发布工作区加载失败，请返回 Markion 重新打开。',
    unresolved: n => `${n} 张本地图片无法解析。`, resolved: n => `已保护加载 ${n} 张本地预览图片`,
    omitPrompt: n => `${n} 张本地图片无法直接粘贴到公众号。确定“复制但不含本地图片”，取消则继续编辑。`,
    copied: '已复制富文本和纯文本。', partial: n => `复制成功，已省略 ${n} 张本地图片。`,
    denied: '剪贴板权限被拒绝，请允许权限后重试。', empty: '请先输入内容再复制。',
    desktop: '桌面', phone: '手机', copy: '复制到公众号', theme: '主题', font: '字号', spacing: '段距',
    fontDown: '减小字号', fontUp: '增大字号', spacingDown: '减小段距', spacingUp: '增大段距'
  },
  'zh-hant': {
    session: '瀏覽器中的修改只保留在目前分頁，不會儲存回 Markion。', privacy: '應用資源完全在本機；文件中的遠端圖片可能連線至原始站點。',
    opened: '本機發佈工作區已開啟', expired: '發佈工作階段已過期，請回到 Markion 重新開啟。', setup: '發佈工作區載入失敗，請回到 Markion 重新開啟。',
    unresolved: n => `${n} 張本機圖片無法解析。`, resolved: n => `已保護載入 ${n} 張本機預覽圖片`, omitPrompt: n => `${n} 張本機圖片無法直接貼到公眾號。確定會略過圖片複製，取消則繼續編輯。`,
    copied: '已複製富文字和純文字。', partial: n => `複製成功，已略過 ${n} 張本機圖片。`, denied: '剪貼簿權限遭拒，請允許後重試。', empty: '請先輸入內容。',
    desktop: '桌面', phone: '手機', copy: '複製到公眾號', theme: '主題', font: '字號', spacing: '段距',
    fontDown: '縮小字號', fontUp: '放大字號', spacingDown: '縮小段距', spacingUp: '放大段距'
  },
  ja: { session: 'ブラウザーの編集はこのタブだけに保持され、Markion には保存されません。', privacy: 'アプリ資産はローカルです。文書のリモート画像は配信元へ接続する場合があります。', opened: '公開ワークスペースを開きました', expired: 'セッションの期限が切れました。Markion から開き直してください。', setup: 'ワークスペースを読み込めません。Markion から開き直してください。', unresolved: n => `${n} 件のローカル画像を解決できません。`, resolved: n => `${n} 件の保護された画像`, omitPrompt: n => `${n} 件のローカル画像は貼り付けできません。画像なしでコピーするには OK、編集を続けるにはキャンセルを選択してください。`, copied: 'リッチ HTML とテキストをコピーしました。', partial: n => `${n} 件の画像を除いてコピーしました。`, denied: 'クリップボード権限が拒否されました。', empty: '先に内容を入力してください。', desktop: 'デスクトップ', phone: 'スマートフォン', copy: 'WeChat 用にコピー', theme: 'テーマ', font: '文字サイズ', spacing: '間隔', fontDown: '文字を小さく', fontUp: '文字を大きく', spacingDown: '段落間隔を縮小', spacingUp: '段落間隔を拡大' },
  fr: { session: 'Les modifications restent dans cet onglet et ne sont pas enregistrées dans Markion.', privacy: 'Les ressources de l’application restent locales. Les images distantes peuvent contacter leur hôte.', opened: 'Espace de publication ouvert', expired: 'La session a expiré. Relancez-la depuis Markion.', setup: 'Impossible de charger l’espace. Relancez-le depuis Markion.', unresolved: n => `${n} image(s) locale(s) non résolue(s).`, resolved: n => `${n} image(s) locale(s) protégée(s)`, omitPrompt: n => `${n} image(s) locale(s) ne peut pas être collée. OK pour copier sans elles, Annuler pour continuer.`, copied: 'HTML enrichi et texte copiés.', partial: n => `Copié sans ${n} image(s) locale(s).`, denied: 'Accès au presse-papiers refusé.', empty: 'Saisissez du contenu avant de copier.', desktop: 'Bureau', phone: 'Téléphone', copy: 'Copier pour WeChat', theme: 'Thème', font: 'Taille', spacing: 'Espacement', fontDown: 'Réduire la taille', fontUp: 'Augmenter la taille', spacingDown: 'Réduire l’espacement', spacingUp: 'Augmenter l’espacement' },
  de: { session: 'Browser-Änderungen bleiben in diesem Tab und werden nicht in Markion gespeichert.', privacy: 'App-Ressourcen bleiben lokal. Entfernte Dokumentbilder können ihren Host kontaktieren.', opened: 'Veröffentlichungsbereich geöffnet', expired: 'Die Sitzung ist abgelaufen. Bitte aus Markion neu öffnen.', setup: 'Der Bereich konnte nicht geladen werden. Bitte aus Markion neu öffnen.', unresolved: n => `${n} lokale Bilder nicht aufgelöst.`, resolved: n => `${n} geschützte lokale Bilder`, omitPrompt: n => `${n} lokale Bilder können nicht eingefügt werden. OK kopiert ohne Bilder, Abbrechen setzt die Bearbeitung fort.`, copied: 'Rich-HTML und Text kopiert.', partial: n => `Ohne ${n} lokale Bilder kopiert.`, denied: 'Zwischenablagezugriff verweigert.', empty: 'Bitte zuerst Inhalt eingeben.', desktop: 'Desktop', phone: 'Telefon', copy: 'Für WeChat kopieren', theme: 'Design', font: 'Schrift', spacing: 'Abstand', fontDown: 'Schrift verkleinern', fontUp: 'Schrift vergrößern', spacingDown: 'Absatzabstand verkleinern', spacingUp: 'Absatzabstand vergrößern' },
  es: { session: 'Los cambios del navegador permanecen en esta pestaña y no se guardan en Markion.', privacy: 'Los recursos de la aplicación son locales. Las imágenes remotas pueden contactar su servidor.', opened: 'Espacio de publicación abierto', expired: 'La sesión caducó. Vuelve a abrirla desde Markion.', setup: 'No se pudo cargar el espacio. Vuelve a abrirlo desde Markion.', unresolved: n => `${n} imagen(es) local(es) sin resolver.`, resolved: n => `${n} imagen(es) local(es) protegida(s)`, omitPrompt: n => `${n} imagen(es) local(es) no se pueden pegar. Acepta para copiar sin ellas o cancela para seguir editando.`, copied: 'HTML enriquecido y texto copiados.', partial: n => `Copiado sin ${n} imagen(es) local(es).`, denied: 'Se denegó el acceso al portapapeles.', empty: 'Escribe contenido antes de copiar.', desktop: 'Escritorio', phone: 'Teléfono', copy: 'Copiar para WeChat', theme: 'Tema', font: 'Tamaño', spacing: 'Espacio', fontDown: 'Reducir tamaño', fontUp: 'Aumentar tamaño', spacingDown: 'Reducir espacio', spacingUp: 'Aumentar espacio' }
};

let locale = messages.en;
let sessionToken = '';
const protectedImages = new Map();
const objectUrls = new Set();
let unresolvedCount = 0;
let resolvedCount = 0;

marked.setOptions({ breaks: true, gfm: true });
marked.use({ extensions: [
  { name: 'mathBlock', level: 'block', start: src => (src.match(/\$\$/) || { index: -1 }).index, tokenizer(src) { const m = src.match(/^\$\$([\s\S]+?)\$\$/); if (m) return { type: 'mathBlock', raw: m[0], tex: m[1].trim() }; }, renderer: token => `<div class="math-block">\\[${token.tex}\\]</div>` },
  { name: 'mathInline', level: 'inline', start: src => (src.match(/\$/) || { index: -1 }).index, tokenizer(src) { const m = src.match(/^\$([^$\n]+?)\$/); if (m) return { type: 'mathInline', raw: m[0], tex: m[1].trim() }; }, renderer: token => `<span class="math-inline">\\(${token.tex}\\)</span>` }
] });

function normalizeImageReference(value) {
  try { value = decodeURI(String(value || '')); } catch (_) { value = String(value || ''); }
  return value.replace(/[?#].*$/, '').replace(/\\/g, '/').replace(/^\.\//, '').toLowerCase();
}
function imageSrcIsLocalCandidate(value) {
  return !!value && !/^(https?:|data:|blob:|about:|javascript:|\/\/)/i.test(value);
}
function applyLocalImages(root) {
  resolvedCount = 0;
  root.querySelectorAll('img').forEach(img => {
    const authored = img.getAttribute('src') || '';
    const resource = protectedImages.get(normalizeImageReference(authored));
    if (!resource) return;
    img.setAttribute('src', resource.objectUrl);
    img.setAttribute('data-mn-local-image-id', resource.id);
    resolvedCount++;
  });
}
function localImageStatusSuffix() {
  if (unresolvedCount) return ` · ${locale.unresolved(unresolvedCount)}`;
  if (resolvedCount) return ` · ${locale.resolved(resolvedCount)}`;
  return '';
}

async function api(path, options = {}) {
  const headers = new Headers(options.headers || {});
  if (sessionToken) headers.set('Authorization', `Bearer ${sessionToken}`);
  const response = await fetch(path, { ...options, headers, cache: 'no-store', referrerPolicy: 'no-referrer' });
  if (response.status === 401) expire();
  if (!response.ok) throw new Error(`workspace request failed (${response.status})`);
  return response;
}
function expire() {
  sessionToken = '';
  sessionStorage.removeItem('markion.wechat.session');
  document.querySelector('.workspace').innerHTML = `<div class="expired"><p>${locale.expired}</p></div>`;
  statusEl.textContent = locale.expired;
}
async function exchangeClaim(claim) {
  const response = await fetch('/api/claim', { method: 'POST', headers: { Authorization: `Bearer ${claim}` }, cache: 'no-store', referrerPolicy: 'no-referrer' });
  if (!response.ok) throw new Error('claim denied');
  return (await response.json()).session_token;
}
async function loadProtectedImages(resources) {
  await Promise.all(resources.map(async resource => {
    const response = await api(`/api/resource/${encodeURIComponent(resource.id)}`);
    const objectUrl = URL.createObjectURL(await response.blob());
    objectUrls.add(objectUrl);
    protectedImages.set(normalizeImageReference(resource.authored_url), { ...resource, objectUrl });
  }));
}

function localImagesRemovedFromCopy(html) {
  const doc = new DOMParser().parseFromString(html, 'text/html');
  let omitted = 0;
  doc.querySelectorAll('img').forEach(img => {
    const src = img.getAttribute('src') || '';
    if (img.hasAttribute('data-mn-local-image-id') || imageSrcIsLocalCandidate(src) || /^(blob:|file:)/i.test(src)) {
      img.remove(); omitted++;
    }
  });
  const output = doc.body.innerHTML;
  if (/(?:blob:|file:|https?:\/\/(?:127\.0\.0\.1|\[::1\]|localhost)(?::\d+)?)/i.test(output)) {
    throw new Error('unsafe local URL remained in copy output');
  }
  return { html: output, omitted };
}

async function copyCurrentPreview() {
  const html = previewEl.dataset.html || '';
  if (!html.trim()) { statusEl.textContent = locale.empty; return; }
  const localCount = previewEl.querySelectorAll('img[data-mn-local-image-id]').length + unresolvedCount;
  try {
    let payload = { html, omitted: 0 };
    if (localCount) {
      if (!window.confirm(locale.omitPrompt(localCount))) return;
      payload = localImagesRemovedFromCopy(html);
    }
    await copyRichHtml(payload.html);
    statusEl.textContent = payload.omitted ? locale.partial(payload.omitted) : locale.copied;
  } catch (error) {
    console.error(error);
    statusEl.textContent = locale.denied;
  }
}

function localize(language) {
  locale = messages[language] || messages.en;
  document.documentElement.lang = language;
  document.getElementById('sessionDisclosure').textContent = locale.session;
  document.getElementById('privacyDisclosure').textContent = locale.privacy;
  document.getElementById('themeLabel').textContent = locale.theme;
  document.getElementById('fontSizeText').textContent = locale.font;
  document.getElementById('spacingText').textContent = locale.spacing;
  document.getElementById('desktopModeBtn').textContent = locale.desktop;
  document.getElementById('phoneModeBtn').textContent = locale.phone;
  document.getElementById('copyBtn').textContent = locale.copy;
  fontSizeDown.setAttribute('aria-label', locale.fontDown);
  fontSizeUp.setAttribute('aria-label', locale.fontUp);
  document.getElementById('paraSpacingDown').setAttribute('aria-label', locale.spacingDown);
  document.getElementById('paraSpacingUp').setAttribute('aria-label', locale.spacingUp);
}

async function bootstrapWorkspace() {
  const claimParams = new URLSearchParams(location.hash.slice(1));
  const claim = claimParams.get('claim');
  history.replaceState(null, '', `${location.pathname}${location.search}`);
  try {
    sessionToken = claim ? await exchangeClaim(claim) : (sessionStorage.getItem('markion.wechat.session') || '');
    if (!sessionToken) throw new Error('missing session');
    sessionStorage.setItem('markion.wechat.session', sessionToken);
    const payload = await (await api('/api/document')).json();
    localize(payload.language);
    unresolvedCount = payload.unresolved_local_images.length;
    await loadProtectedImages(payload.resources);
    document.getElementById('documentName').textContent = payload.display_name;
    markdownEl.value = payload.markdown;
    render();
    statusEl.textContent = `${locale.opened}${localImageStatusSuffix()}`;
    if (unresolvedCount) {
      const warning = document.getElementById('warning');
      warning.hidden = false;
      warning.textContent = locale.unresolved(unresolvedCount);
    }
    setInterval(() => api('/api/heartbeat', { method: 'POST' }).catch(() => {}), 60_000);
  } catch (error) {
    console.error(error);
    statusEl.textContent = locale.setup;
    document.querySelector('.workspace').innerHTML = `<div class="expired"><p>${locale.setup}</p></div>`;
  }
}

document.addEventListener('DOMContentLoaded', () => {
  themeSelect.addEventListener('change', render);
  fontSizeDown.addEventListener('click', () => { fontSizeOffset = Math.max(-6, fontSizeOffset - 1); fontSizeLabel.textContent = fontSizeOffset; render(); });
  fontSizeUp.addEventListener('click', () => { fontSizeOffset = Math.min(6, fontSizeOffset + 1); fontSizeLabel.textContent = fontSizeOffset; render(); });
  const spacingLabel = document.getElementById('paraSpacingLabel');
  document.getElementById('paraSpacingDown').addEventListener('click', () => { paraSpacingOffset = Math.max(-16, paraSpacingOffset - 2); spacingLabel.textContent = paraSpacingOffset; render(); });
  document.getElementById('paraSpacingUp').addEventListener('click', () => { paraSpacingOffset = Math.min(24, paraSpacingOffset + 2); spacingLabel.textContent = paraSpacingOffset; render(); });
  markdownEl.addEventListener('input', () => { clearTimeout(window.__markionRenderTimer); window.__markionRenderTimer = setTimeout(render, 180); });
  document.getElementById('copyBtn').addEventListener('click', copyCurrentPreview);
  document.getElementById('desktopModeBtn').addEventListener('click', event => { document.getElementById('previewContainer').classList.remove('phone-mode'); document.querySelectorAll('.mode-btn').forEach(button => button.classList.remove('active')); event.currentTarget.classList.add('active'); });
  document.getElementById('phoneModeBtn').addEventListener('click', event => { document.getElementById('previewContainer').classList.add('phone-mode'); document.getElementById('desktopModeBtn').classList.remove('active'); event.currentTarget.classList.add('active'); });
  bootstrapWorkspace();
});
window.addEventListener('pagehide', () => objectUrls.forEach(url => URL.revokeObjectURL(url)));
