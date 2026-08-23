/* Markion math runtime: renders TeX placeholders as self-contained MathJax SVG,
   aligned with the MarkNice Obsidian plugin (mathjax-full tex-svg, fontCache 'none').
   The emitted SVG is pure <path>/<rect> vector output with inline sizing, so it
   survives the WeChat editor's paste-time CSS and tag filtering, and contains no
   hidden MathML duplicate. Must execute before static/vendor/tex-svg-full.js so the
   component picks up this configuration. */
window.MathJax = Object.assign({}, window.MathJax, {
  svg: { fontCache: 'none' },
  options: { enableMenu: false }
});
(() => {
  const MAX_RENDER_TEX_LENGTH = 12000;

  function mathTex(el, displayMode) {
    const fromAttribute = el.getAttribute('data-tex');
    if (fromAttribute) return fromAttribute;
    return (el.textContent || '')
      .replace(displayMode ? /^\\\[/ : /^\\\(/, '')
      .replace(displayMode ? /\\\]$/ : /\\\)$/, '')
      .trim();
  }

  /* WeChat-friendly SVG polish (same as the Obsidian plugin): width/height move
     from attributes into inline style (ex units, scaling with body font size),
     MathJax's inline vertical-align keeps inline formulas on the text baseline,
     and oversized display formulas scroll sideways instead of being squashed. */
  function polishMathSvg(el, displayMode) {
    const svg = el.querySelector('svg');
    if (!svg) return;
    const width = svg.getAttribute('width');
    const height = svg.getAttribute('height');
    if (width) svg.style.width = width;
    if (height) svg.style.height = height;
    svg.removeAttribute('width');
    svg.removeAttribute('height');
    if (displayMode) svg.style.setProperty('max-width', '300%', 'important');
  }

  function renderElement(el, displayMode) {
    const tex = mathTex(el, displayMode);
    if (!tex || tex.length > MAX_RENDER_TEX_LENGTH) return;
    const open = displayMode ? '\\[' : '\\(';
    const close = displayMode ? '\\]' : '\\)';
    try {
      const node = MathJax.tex2svg(tex, { display: displayMode });
      const svg = node.querySelector('svg');
      if (!svg) throw new Error('MathJax produced no SVG');
      el.replaceChildren(svg);
      polishMathSvg(el, displayMode);
    } catch (_) {
      el.textContent = `${open}${tex}${close}`;
    }
  }

  function renderInto(container) {
    if (!window.MathJax || typeof MathJax.tex2svg !== 'function') return;
    container.querySelectorAll('.math-block').forEach(el => renderElement(el, true));
    container.querySelectorAll('.math-inline').forEach(el => renderElement(el, false));
  }

  /* ------------------------------------------------------------------ */
  /* Word export fallback: Word renders the DOCX via altChunk(MHT),       */
  /* which supports neither MathML nor SVG math layout, so formulas are   */
  /* degraded to linear readable text (ported from the Obsidian plugin).  */
  /* Unicode symbols keep the text readable; unrecognized constructs fall */
  /* back to the original TeX so no information is lost.                  */
  /* ------------------------------------------------------------------ */

  const MATH_SYMBOLS = {
    alpha: 'α', beta: 'β', gamma: 'γ', delta: 'δ', epsilon: 'ε', varepsilon: 'ε',
    zeta: 'ζ', eta: 'η', theta: 'θ', vartheta: 'θ', iota: 'ι', kappa: 'κ',
    lambda: 'λ', mu: 'μ', nu: 'ν', xi: 'ξ', pi: 'π', varpi: 'π', rho: 'ρ',
    varrho: 'ρ', sigma: 'σ', varsigma: 'ς', tau: 'τ', upsilon: 'υ', phi: 'φ',
    varphi: 'φ', chi: 'χ', psi: 'ψ', omega: 'ω',
    Gamma: 'Γ', Delta: 'Δ', Theta: 'Θ', Lambda: 'Λ', Xi: 'Ξ', Pi: 'Π',
    Sigma: 'Σ', Upsilon: 'Υ', Phi: 'Φ', Psi: 'Ψ', Omega: 'Ω',
    leq: '≤', le: '≤', geq: '≥', ge: '≥', neq: '≠', ne: '≠', approx: '≈',
    equiv: '≡', sim: '∼', simeq: '≃', cong: '≅', propto: '∝',
    pm: '±', mp: '∓', times: '×', div: '÷', cdot: '·', ast: '∗',
    cap: '∩', cup: '∪', setminus: '∖', subset: '⊂', subseteq: '⊆',
    supset: '⊃', supseteq: '⊇', in: '∈', notin: '∉', emptyset: '∅', varnothing: '∅',
    forall: '∀', exists: '∃', neg: '¬', lnot: '¬',
    rightarrow: '→', to: '→', leftarrow: '←', gets: '←',
    Rightarrow: '⇒', Leftarrow: '⇐', leftrightarrow: '↔', Leftrightarrow: '⇔',
    mapsto: '↦', uparrow: '↑', downarrow: '↓',
    infty: '∞', partial: '∂', nabla: '∇', angle: '∠', perp: '⊥', parallel: '∥',
    sum: '∑', prod: '∏', coprod: '∐', int: '∫', oint: '∮', iint: '∬', iiint: '∭',
    bigcup: '⋃', bigcap: '⋂', bigvee: '⋁', bigwedge: '⋀',
    langle: '⟨', rangle: '⟩', lceil: '⌈', rceil: '⌉', lfloor: '⌊', rfloor: '⌋',
    cdots: '⋯', ldots: '…', vdots: '⋮', ddots: '⋱',
    bullet: '•', prime: '′', dagger: '†', ddagger: '‡',
    triangle: '△', square: '□',
    mathbb: '', mathcal: '', mathrm: '', mathbf: '', mathit: '', operatorname: '', text: '', textrm: '', textit: '', textbf: '',
  };

  const SUPERSCRIPT = {
    '0': '⁰', '1': '¹', '2': '²', '3': '³', '4': '⁴', '5': '⁵', '6': '⁶', '7': '⁷', '8': '⁸', '9': '⁹',
    '+': '⁺', '-': '⁻', '=': '⁼', '(': '⁽', ')': '⁾', 'n': 'ⁿ', 'i': 'ⁱ',
  };
  const SUBSCRIPT = {
    '0': '₀', '1': '₁', '2': '₂', '3': '₃', '4': '₄', '5': '₅', '6': '₆', '7': '₇', '8': '₈', '9': '₉',
    '+': '₊', '-': '₋', '=': '₌', '(': '₍', ')': '₎',
    'a': 'ₐ', 'e': 'ₑ', 'o': 'ₒ', 'x': 'ₓ', 'h': 'ₕ', 'k': 'ₖ', 'l': 'ₗ', 'm': 'ₘ', 'n': 'ₙ', 'p': 'ₚ', 's': 'ₛ', 't': 'ₜ',
  };

  function tokenizeTex(tex) {
    const tokens = [];
    for (let i = 0; i < tex.length;) {
      const ch = tex[i];
      if (ch === '\\') {
        tokens.push('\\');
        let j = i + 1;
        while (j < tex.length && /[a-zA-Z]/.test(tex[j])) {
          tokens.push(tex[j]);
          j++;
        }
        if (j === i + 1 && tex[j] && !/[a-zA-Z]/.test(tex[j])) {
          tokens.push(tex[j]);
          j++;
        }
        i = j;
      } else {
        tokens.push(ch);
        i++;
      }
    }
    return tokens;
  }

  function texGroupToLinear(tokens, i, lenRef) {
    let depth = 0;
    const out = [];
    let j = i;
    for (; j < tokens.length; j++) {
      if (tokens[j] === '{') {
        depth++;
        if (depth >= 2) out.push(tokens[j]);
      } else if (tokens[j] === '}') {
        depth--;
        if (depth === 0) break;
        out.push(tokens[j]);
      } else if (depth >= 1) {
        out.push(tokens[j]);
      }
    }
    lenRef.n = j - i + 1;
    return texToLinearInner(out);
  }

  function readArg(tokens, i, lenRef) {
    let j = i;
    while (j < tokens.length && tokens[j] === ' ') j++;
    if (tokens[j] === '{') return texGroupToLinear(tokens, j, lenRef);
    if (tokens[j] === '\\') {
      let k = j + 1;
      while (k < tokens.length && /[a-zA-Z]/.test(tokens[k])) k++;
      const cmd = tokens.slice(j + 1, k).join('');
      lenRef.n = (k - j) + (cmd.length > 0 ? 0 : 1);
      return cmd in MATH_SYMBOLS ? MATH_SYMBOLS[cmd] : `\\${cmd}`;
    }
    lenRef.n = j - i + 1;
    return tokens[j] ?? '';
  }

  function shiftMap(s, map) {
    let out = '';
    for (const ch of s) out += map[ch] ?? ch;
    return out;
  }

  function texToLinearInner(tokens) {
    let out = '';
    let i = 0;
    while (i < tokens.length) {
      const t = tokens[i];

      if (t === '\\') {
        let k = i + 1;
        while (k < tokens.length && /[a-zA-Z]/.test(tokens[k])) k++;
        const cmd = tokens.slice(i + 1, k).join('');
        const cmdLen = (k - (i + 1)) + 1;

        if (cmd === '') {
          out += ' ';
          i += 2;
          continue;
        }
        if (cmd === 'frac' || cmd === 'dfrac' || cmd === 'tfrac' || cmd === 'cfrac') {
          const r1 = { n: 0 };
          const a = readArg(tokens, k, r1);
          const r2 = { n: 0 };
          const b = readArg(tokens, k + r1.n, r2);
          const wrap = s => (/[+\-*/= ]/.test(s) && s.length > 1 ? `(${s})` : s);
          out += `${wrap(a)}/${wrap(b)}`;
          i = k + r1.n + r2.n;
          continue;
        }
        if (cmd === 'sqrt') {
          let k2 = k;
          while (k2 < tokens.length && tokens[k2] === ' ') k2++;
          let rootStr = '';
          if (tokens[k2] === '[') {
            const end = tokens.indexOf(']', k2);
            if (end !== -1) {
              rootStr = texToLinearInner(tokens.slice(k2 + 1, end));
              k2 = end + 1;
            }
          }
          const r = { n: 0 };
          const body = readArg(tokens, k2, r);
          out += rootStr ? `√[${rootStr}]{${body}}` : `√${body.length > 1 ? `(${body})` : body}`;
          i = k2 + r.n;
          continue;
        }
        if (cmd in MATH_SYMBOLS) {
          out += MATH_SYMBOLS[cmd];
          i += cmdLen;
          continue;
        }
        if (cmd === 'left' || cmd === 'right' || cmd === 'middle' || cmd === 'big' || cmd === 'Big' || cmd === 'Bigg' || cmd === 'bigg' || cmd === 'displaystyle' || cmd === 'textstyle' || cmd === 'limits' || cmd === 'nolimits' || cmd === 'scriptstyle' || cmd === 'noalign' || cmd === 'operatorname') {
          i += cmdLen;
          continue;
        }
        if (cmd === 'begin' || cmd === 'end') {
          let k2 = k;
          while (k2 < tokens.length && tokens[k2] === ' ') k2++;
          if (tokens[k2] === '{') {
            const r = { n: 0 };
            texGroupToLinear(tokens, k2, r);
            k2 += r.n;
          }
          i = k2;
          continue;
        }
        out += `\\${cmd}`;
        i += cmdLen;
        continue;
      }

      if (t === '^' || t === '_') {
        const map = t === '^' ? SUPERSCRIPT : SUBSCRIPT;
        const fallback = t === '^' ? '^' : '_';
        let k = i + 1;
        while (k < tokens.length && tokens[k] === ' ') k++;
        let arg;
        const r = { n: 0 };
        if (tokens[k] === '{') {
          arg = texGroupToLinear(tokens, k, r);
        } else if (tokens[k] === '\\') {
          arg = readArg(tokens, k, r);
        } else {
          arg = tokens[k] ?? '';
          r.n = 1;
        }
        const converted = shiftMap(arg, map);
        const allConvertible = [...arg].every(ch => ch in map);
        out += allConvertible ? converted : `${fallback}{${arg}}`;
        i = k + r.n;
        continue;
      }

      if (t === ' ' || t === '\t' || t === '\n') {
        i++;
        continue;
      }
      out += t;
      i++;
    }
    return out;
  }

  function texToLinearText(tex) {
    const trimmed = String(tex || '').trim();
    if (!trimmed) return '';
    try {
      const linear = texToLinearInner(tokenizeTex(trimmed)).replace(/\s{2,}/g, ' ').trim();
      return linear || trimmed;
    } catch (_) {
      return trimmed;
    }
  }

  /* Replaces every rendered formula in an exported article's HTML with linear
     readable text, restoring the TeX source from the placeholder's data-tex. */
  function rewriteForWordExport(html) {
    const doc = new DOMParser().parseFromString(`<body>${html}</body>`, 'text/html');
    const root = doc.body.firstElementChild;
    if (!root) return html;
    const rewrite = el => {
      const tex = el.getAttribute('data-tex');
      if (tex) el.textContent = texToLinearText(tex);
    };
    root.querySelectorAll('.math-block').forEach(rewrite);
    root.querySelectorAll('.math-inline').forEach(rewrite);
    return root.outerHTML;
  }

  window.MarkionMath = Object.freeze({ renderInto, texToLinearText, rewriteForWordExport });
})();
