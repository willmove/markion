/* Generated from MarkNice c009c1ec7e7c92f89afa5a32edcb126b5296bda7 by scripts/sync-marknice-workspace.ps1. */
/**
 * Custom DOCX parser - handles numbering, merged cells, OMML formulas, images.
 * Uses manual XML traversal to avoid querySelector namespace issues.
 */

// ===== XML helper: find child element by localName =====
function _qn(parent, name) {
  // Find first descendant element matching localName (handles w:xxx, m:xxx etc.)
  if (!parent) return null;
  var parts = name.split('>').map(function(s) { return s.trim(); });
  var cur = parent;
  for (var i = 0; i < parts.length; i++) {
    var found = null;
    for (var c = cur.firstChild; c; c = c.nextSibling) {
      if (c.nodeType === 1 && c.localName === parts[i]) { found = c; break; }
    }
    if (!found) return null;
    cur = found;
  }
  return cur;
}
// Find ALL direct children matching localName
function _qnAll(parent, name) {
  var result = [];
  if (!parent) return result;
  for (var c = parent.firstChild; c; c = c.nextSibling) {
    if (c.nodeType === 1 && c.localName === name) result.push(c);
  }
  return result;
}
// Find ALL descendants matching localName
function _qnDeep(parent, name) {
  if (!parent) return [];
  var els = parent.getElementsByTagName('*');
  var result = [];
  for (var i = 0; i < els.length; i++) {
    if (els[i].localName === name) result.push(els[i]);
  }
  return result;
}
// Get attribute value (tries w:xxx then plain xxx)
function _attr(el, name) {
  if (!el) return '';
  return el.getAttribute('w:' + name) || el.getAttribute('m:' + name) || el.getAttribute('r:' + name) || el.getAttribute(name) || '';
}

// ===== Number formatting =====
function _fmtNum(val, fmt) {
  switch (fmt) {
    case 'upperRoman': return _toRoman(val);
    case 'lowerRoman': return _toRoman(val).toLowerCase();
    case 'upperLetter': return String.fromCharCode(64 + ((val - 1) % 26) + 1);
    case 'lowerLetter': return String.fromCharCode(96 + ((val - 1) % 26) + 1);
    case 'chineseCounting': case 'ideographTraditional': return _toCN(val);
    default: return String(val);
  }
}
function _toRoman(n) {
  var v=[1000,900,500,400,100,90,50,40,10,9,5,4,1], s=['M','CM','D','CD','C','XC','L','XL','X','IX','V','IV','I'], r='';
  for (var i=0;i<v.length;i++) while(n>=v[i]){r+=s[i];n-=v[i];} return r;
}
function _toCN(n) {
  var c=['零','一','二','三','四','五','六','七','八','九','十'];
  if(n<=10)return c[n]; if(n<20)return '十'+(n%10?c[n%10]:''); return String(n);
}

// ===== OMML to LaTeX =====
function ommlToLatex(node) {
  var parts = [];
  for (var ch = node.firstChild; ch; ch = ch.nextSibling) {
    if (ch.nodeType !== 1) continue;
    var tag = ch.localName;
    switch (tag) {
      case 'r': { var t = _qn(ch,'t'); if(t) parts.push(t.textContent); break; }
      case 'f': {
        var num=_qn(ch,'num'), den=_qn(ch,'den');
        parts.push('\\frac{'+(num?ommlToLatex(num):'')+'}{' +(den?ommlToLatex(den):'')+'}'); break;
      }
      case 'rad': {
        var deg=_qn(ch,'deg'), e=_qn(ch,'e'), base=e?ommlToLatex(e):'', dg=deg?ommlToLatex(deg).trim():'';
        parts.push(dg?'\\sqrt['+dg+']{'+base+'}':'\\sqrt{'+base+'}'); break;
      }
      case 'sSup': { var e=_qn(ch,'e'),sup=_qn(ch,'sup'); parts.push(ommlToLatex(e)+'^{'+ommlToLatex(sup)+'}'); break; }
      case 'sSub': { var e=_qn(ch,'e'),sub=_qn(ch,'sub'); parts.push(ommlToLatex(e)+'_{'+ommlToLatex(sub)+'}'); break; }
      case 'sSubSup': { var e=_qn(ch,'e'),sub=_qn(ch,'sub'),sup=_qn(ch,'sup'); parts.push(ommlToLatex(e)+'_{'+ommlToLatex(sub)+'}^{'+ommlToLatex(sup)+'}'); break; }
      case 'nary': {
        var chr=_qn(_qn(ch,'naryPr'),'chr'), sym=_attr(chr,'val')||'∫';
        var sub=_qn(ch,'sub'),sup=_qn(ch,'sup'),e=_qn(ch,'e');
        var map={'∑':'\\sum','∫':'\\int','∏':'\\prod','∮':'\\oint'}, s=map[sym]||sym;
        if(sub)s+='_{'+ommlToLatex(sub)+'}'; if(sup)s+='^{'+ommlToLatex(sup)+'}'; if(e)s+=' '+ommlToLatex(e);
        parts.push(s); break;
      }
      case 'd': {
        var dPr=_qn(ch,'dPr'), beg=_attr(_qn(dPr,'begChr'),'val')||'(', end=_attr(_qn(dPr,'endChr'),'val')||')';
        var eNodes=_qnAll(ch,'e'), inner=eNodes.map(function(e){return ommlToLatex(e);}).join(', ');
        var lm={'(':'(','[':'[','{':'\\{','|':'|'}, rm={')':')',']':']','}':'\\}','|':'|'};
        parts.push('\\left'+(lm[beg]||beg)+inner+'\\right'+(rm[end]||end)); break;
      }
      case 'func': { var fn=_qn(ch,'fName'),e=_qn(ch,'e'); parts.push('\\'+(fn?ommlToLatex(fn).trim():'')+'{'+ommlToLatex(e)+'}'); break; }
      case 'acc': {
        var ac=_attr(_qn(_qn(ch,'accPr'),'chr'),'val')||'\u0302', e=_qn(ch,'e');
        var am={'\u0302':'\\hat','\u0304':'\\bar','\u0303':'\\tilde','\u0307':'\\dot','\u0308':'\\ddot','\u20D7':'\\vec','\u0305':'\\overline'};
        parts.push((am[ac]||'\\hat')+'{'+ommlToLatex(e)+'}'); break;
      }
      case 'bar': { var pos=_attr(_qn(_qn(ch,'barPr'),'pos'),'val')||'top',e=_qn(ch,'e'); parts.push(pos==='bot'?'\\underline{'+ommlToLatex(e)+'}':'\\overline{'+ommlToLatex(e)+'}'); break; }
      case 'm': {
        var mrs=_qnAll(ch,'mr'), rows=mrs.map(function(mr){return _qnAll(mr,'e').map(function(e){return ommlToLatex(e);}).join(' & ');});
        parts.push('\\begin{matrix}'+rows.join(' \\\\ ')+'\\end{matrix}'); break;
      }
      case 'eqArr': { var es=_qnAll(ch,'e'); parts.push('\\begin{aligned}'+es.map(function(e){return ommlToLatex(e);}).join(' \\\\ ')+'\\end{aligned}'); break; }
      case 'limLow': { var e=_qn(ch,'e'),lim=_qn(ch,'lim'); parts.push(ommlToLatex(e)+'_{'+ommlToLatex(lim)+'}'); break; }
      case 'limUpp': { var e=_qn(ch,'e'),lim=_qn(ch,'lim'); parts.push(ommlToLatex(e)+'^{'+ommlToLatex(lim)+'}'); break; }
      case 'box': case 'borderBox': case 'groupChr': { var e=_qn(ch,'e'); if(e)parts.push(ommlToLatex(e)); break; }
      default: { var inner=ommlToLatex(ch); if(inner)parts.push(inner); }
    }
  }
  return parts.join('');
}

// ===== Main parser =====
async function parseDocx(arrayBuffer) {
  var zip = await JSZip.loadAsync(arrayBuffer);

  // --- numbering.xml ---
  var abstractNums = {}, numToAbstract = {};
  var nf = zip.file('word/numbering.xml');
  if (nf) {
    var nd = new DOMParser().parseFromString(await nf.async('string'), 'application/xml');
    _qnDeep(nd,'abstractNum').forEach(function(an) {
      var id = _attr(an,'abstractNumId'), levels = {};
      _qnDeep(an,'lvl').forEach(function(lvl) {
        var il = _attr(lvl,'ilvl');
        levels[il] = {
          numFmt: _attr(_qn(lvl,'numFmt'),'val') || 'decimal',
          lvlText: _attr(_qn(lvl,'lvlText'),'val') || '',
          start: parseInt(_attr(_qn(lvl,'start'),'val') || '1', 10)
        };
      });
      abstractNums[id] = levels;
    });
    _qnDeep(nd,'num').forEach(function(n) {
      var nid = _attr(n,'numId'), ar = _qn(n,'abstractNumId');
      if (!ar) return;
      var aid = _attr(ar,'val');
      numToAbstract[nid] = aid;
      _qnDeep(n,'lvlOverride').forEach(function(ov) {
        var il = _attr(ov,'ilvl'), so = _qn(ov,'startOverride');
        if (so) {
          if (!numToAbstract['_ov_'+nid]) numToAbstract['_ov_'+nid] = {};
          numToAbstract['_ov_'+nid][il] = { start: parseInt(_attr(so,'val'),10) };
        }
      });
    });
  }

  // --- styles.xml ---
  var styleNumIds = {}, styleOutlineLvl = {};
  var sf = zip.file('word/styles.xml');
  if (sf) {
    var sd = new DOMParser().parseFromString(await sf.async('string'), 'application/xml');
    _qnDeep(sd,'style').forEach(function(st) {
      var sid = _attr(st,'styleId');
      var pPr = _qn(st,'pPr'); if (!pPr) return;
      var numPr = _qn(pPr,'numPr');
      if (numPr) {
        var ni = _attr(_qn(numPr,'numId'),'val'), il = _attr(_qn(numPr,'ilvl'),'val') || '0';
        if (ni && ni !== '0') styleNumIds[sid] = { numId: ni, ilvl: il };
      }
      var ol = _qn(pPr,'outlineLvl');
      if (ol) { var lv = parseInt(_attr(ol,'val')||'-1',10); if(lv>=0&&lv<=8) styleOutlineLvl[sid]=lv+1; }
    });
  }

  // --- relationships ---
  var rels = {};
  var rf = zip.file('word/_rels/document.xml.rels');
  if (rf) {
    var rd = new DOMParser().parseFromString(await rf.async('string'), 'application/xml');
    _qnDeep(rd,'Relationship').forEach(function(r) {
      rels[r.getAttribute('Id')] = r.getAttribute('Target');
    });
  }

  async function getImageDataUri(rId) {
    var target = rels[rId]; if (!target) return '';
    var p = target.startsWith('/') ? target.substring(1) : 'word/' + target;
    var imgFile = zip.file(p); if (!imgFile) return '';
    var data = await imgFile.async('base64');
    var ext = target.split('.').pop().toLowerCase();
    var mm = {png:'image/png',jpg:'image/jpeg',jpeg:'image/jpeg',gif:'image/gif',bmp:'image/bmp',svg:'image/svg+xml'};
    return 'data:'+(mm[ext]||'image/png')+';base64,'+data;
  }

  // --- document.xml ---
  var df = zip.file('word/document.xml');
  if (!df) return '<p>无法读取文档内容</p>';
  var dd = new DOMParser().parseFromString(await df.async('string'), 'application/xml');

  var numCounters = {}, seqCounters = {}, currentChapter = 0;

  function getHeadingLevel(pStyle, pPr) {
    if (pStyle) {
      var m = pStyle.match(/^[Hh][Ee][Aa][Dd][Ii][Nn][Gg]\s*(\d)$/);
      if (m) return parseInt(m[1],10);
      m = pStyle.match(/^标题\s*(\d)$/);
      if (m) return parseInt(m[1],10);
      m = pStyle.match(/^(\d)$/);
      if (m) return parseInt(m[1],10);
      if (styleOutlineLvl[pStyle]) return styleOutlineLvl[pStyle];
    }
    if (pPr) {
      var ol = _qn(pPr,'outlineLvl');
      if (ol) { var lv=parseInt(_attr(ol,'val')||'-1',10); if(lv>=0&&lv<=8) return lv+1; }
    }
    return 0;
  }

  function isTocHeading(text) { return /^(目\s*录|table\s+of\s+contents|contents|toc)$/i.test(text.trim()); }

  function resolveNumbering(numId, ilvl) {
    var is = String(ilvl), aid = numToAbstract[numId];
    if (!aid) return '';
    var levels = abstractNums[aid]; if (!levels) return '';
    var ld = levels[is]; if (!ld) return '';

    // Bullet lists: don't increment counters, return special marker
    if (ld.numFmt === 'bullet') return '\x00BULLET';

    if (!numCounters[numId]) numCounters[numId] = {};
    var ov = numToAbstract['_ov_'+numId];
    if (numCounters[numId][is] === undefined) {
      numCounters[numId][is] = (ov&&ov[is]) ? ov[is].start : (ld.start||1);
    } else { numCounters[numId][is]++; }
    for (var i=ilvl+1;i<=8;i++) delete numCounters[numId][String(i)];
    var pfx = ld.lvlText || '';
    if (pfx) {
      for (var j=0;j<=ilvl;j++) {
        var v=numCounters[numId][String(j)]||1, f=(levels[String(j)]&&levels[String(j)].numFmt)||'decimal';
        pfx = pfx.replace('%'+(j+1), _fmtNum(v,f));
      }
    } else {
      var pp=[]; for(var k=0;k<=ilvl;k++) pp.push(numCounters[numId][String(k)]||1); pfx=pp.join('.');
    }
    return pfx;
  }

  function getRunText(r) {
    var t='';
    for(var c=r.firstChild;c;c=c.nextSibling){
      if(c.nodeType!==1)continue; var ln=c.localName;
      if(ln==='t')t+=c.textContent; else if(ln==='tab')t+='\t'; else if(ln==='br')t+='\n';
      else if(ln==='sym'){var code=_attr(c,'char'); if(code)t+=String.fromCodePoint(parseInt(code,16));}
    }
    return t;
  }
  function escapeHtml(s){return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');}

  function processField(instr, result) {
    var tr=instr.trim(), sm=tr.match(/SEQ\s+(\S+)/i);
    if(sm){var sn=sm[1]; if(!seqCounters[sn])seqCounters[sn]=0; seqCounters[sn]++; return String(seqCounters[sn]);}
    if(/STYLEREF/i.test(tr)&&result.trim()) return escapeHtml(result.trim());
    if(result.trim()) return escapeHtml(result.trim());
    return '';
  }

  async function processDrawing(drawing) {
    var blip = _qnDeep(drawing,'blip')[0]; if(!blip) return '';
    var rId = blip.getAttribute('r:embed')||blip.getAttribute('r:link')||''; if(!rId) return '';
    var uri = await getImageDataUri(rId); if(!uri) return '';
    var dp = _qnDeep(drawing,'docPr')[0];
    return '<img src="'+uri+'" alt="'+escapeHtml(dp?dp.getAttribute('descr')||'':'')+'" />';
  }

  // ===== Process paragraph content into HTML =====
  // Helper: extract formatting flags from rPr
  function _getRunFmt(rPr) {
    if (!rPr) return { b: false, i: false, u: false, s: false, va: '' };
    var bNode = _qn(rPr, 'b');
    var bVal = bNode ? _attr(bNode, 'val') : null;
    var bold = bNode ? (bVal !== 'false' && bVal !== '0') : false;
    var iNode = _qn(rPr, 'i');
    var iVal = iNode ? _attr(iNode, 'val') : null;
    var italic = iNode ? (iVal !== 'false' && iVal !== '0') : false;
    var uline = !!_qn(rPr, 'u');
    var strike = !!_qn(rPr, 'strike');
    var vAlign = _qn(rPr, 'vertAlign');
    var va = vAlign ? (_attr(vAlign, 'val') || '') : '';
    return { b: bold, i: italic, u: uline, s: strike, va: va };
  }
  function _fmtKey(f) { return (f.b?'B':'')+(f.i?'I':'')+(f.u?'U':'')+(f.s?'S':'')+f.va; }
  function _wrapFmt(text, fmt) {
    var s = text;
    if (fmt.b) s = '<strong>' + s + '</strong>';
    if (fmt.i) s = '<em>' + s + '</em>';
    if (fmt.u) s = '<u>' + s + '</u>';
    if (fmt.s) s = '<s>' + s + '</s>';
    if (fmt.va === 'superscript') s = '<sup>' + s + '</sup>';
    else if (fmt.va === 'subscript') s = '<sub>' + s + '</sub>';
    return s;
  }

  async function processParagraphContent(p) {
    // Collect segments: each is { text, fmt, raw } where raw is pre-formatted HTML (non-run content)
    var segments = [];
    var fieldInstr = '', fieldResult = '', inField = false, fieldDepth = 0;

    for (var ch = p.firstChild; ch; ch = ch.nextSibling) {
      if (ch.nodeType !== 1) continue;
      var ln = ch.localName;

      // --- Field handling ---
      if (ln === 'fldChar') {
        var ft = _attr(ch, 'fldCharType');
        if (ft === 'begin') { inField = true; fieldDepth++; fieldInstr = ''; fieldResult = ''; }
        else if (ft === 'separate') { /* switch to result collection */ }
        else if (ft === 'end') {
          fieldDepth--;
          if (fieldDepth <= 0) {
            inField = false;
            var fr = processField(fieldInstr, fieldResult);
            if (fr) segments.push({ raw: fr });
          }
        }
        continue;
      }
      if (ln === 'instrText') { fieldInstr += ch.textContent; continue; }
      if (inField && ln === 'r') { fieldResult += getRunText(ch); continue; }

      // --- Hyperlinks ---
      if (ln === 'hyperlink') {
        var rId = ch.getAttribute('r:id') || _attr(ch, 'id') || '';
        var href = rels[rId] || '';
        var linkText = '';
        for (var lc = ch.firstChild; lc; lc = lc.nextSibling) {
          if (lc.nodeType === 1 && lc.localName === 'r') linkText += getRunText(lc);
        }
        if (href) segments.push({ raw: '<a href="' + escapeHtml(href) + '">' + escapeHtml(linkText) + '</a>' });
        else segments.push({ raw: escapeHtml(linkText) });
        continue;
      }

      // --- OMML math (inline) ---
      if (ln === 'oMath') {
        var latex = ommlToLatex(ch);
        if (latex) segments.push({ raw: '<span class="math-inline">\\(' + latex + '\\)</span>' });
        continue;
      }

      // --- Runs ---
      if (ln === 'r') {
        var rPr = _qn(ch, 'rPr');
        var text = getRunText(ch);
        if (!text) {
          var drawing = _qn(ch, 'drawing');
          if (drawing) { var imgHtml = await processDrawing(drawing); if (imgHtml) segments.push({ raw: imgHtml }); continue; }
          var pict = _qn(ch, 'pict');
          if (pict) continue;
          continue;
        }
        var fmt = _getRunFmt(rPr);
        segments.push({ text: escapeHtml(text), fmt: fmt });
        continue;
      }

      if (ln === 'bookmarkStart' || ln === 'bookmarkEnd') continue;

      if (ln === 'sdt') {
        var sdtContent = _qn(ch, 'sdtContent');
        if (sdtContent) {
          var innerPs = _qnAll(sdtContent, 'p');
          for (var ip = 0; ip < innerPs.length; ip++) {
            var innerHtml = await processParagraphContent(innerPs[ip]);
            if (innerHtml) segments.push({ raw: innerHtml });
          }
        }
        continue;
      }
    }

    // Merge adjacent segments with same formatting, then wrap
    var html = '';
    var i = 0;
    while (i < segments.length) {
      var seg = segments[i];
      if (seg.raw !== undefined) {
        html += seg.raw;
        i++;
        continue;
      }
      // Text segment — merge with following same-format text segments
      var merged = seg.text;
      var key = _fmtKey(seg.fmt);
      var j = i + 1;
      while (j < segments.length && segments[j].text !== undefined && segments[j].raw === undefined && _fmtKey(segments[j].fmt) === key) {
        merged += segments[j].text;
        j++;
      }
      html += _wrapFmt(merged, seg.fmt);
      i = j;
    }
    return html;
  }

  // ===== Process table =====
  async function processTable(tblNode) {
    var html = '<table>';
    var grid = _qn(tblNode, 'tblGrid');
    var gridCols = grid ? _qnAll(grid, 'gridCol') : [];
    var colCount = gridCols.length;

    // Track vMerge state: which columns are continuing a vertical merge
    var vMergeMap = {}; // col index -> { remaining rows }

    var rows = _qnAll(tblNode, 'tr');
    for (var ri = 0; ri < rows.length; ri++) {
      html += '<tr>';
      var cells = _qnAll(rows[ri], 'tc');
      var colIdx = 0;

      for (var ci = 0; ci < cells.length; ci++) {
        var tc = cells[ci];
        var tcPr = _qn(tc, 'tcPr');

        // gridSpan (horizontal merge)
        var gs = tcPr ? _qn(tcPr, 'gridSpan') : null;
        var colspan = gs ? parseInt(_attr(gs, 'val') || '1', 10) : 1;

        // vMerge (vertical merge)
        var vm = tcPr ? _qn(tcPr, 'vMerge') : null;
        var vmVal = vm ? (_attr(vm, 'val') || 'continue') : null;

        // Skip cells that are continuation of vertical merge
        if (vmVal === 'continue') {
          colIdx += colspan;
          continue;
        }

        // Count how many rows this cell spans (look ahead for vMerge continue)
        var rowspan = 1;
        if (vmVal === 'restart' || (vm && vmVal === '')) {
          for (var rr = ri + 1; rr < rows.length; rr++) {
            var futureCells = _qnAll(rows[rr], 'tc');
            var futureColIdx = 0;
            var found = false;
            for (var fc = 0; fc < futureCells.length; fc++) {
              if (futureColIdx === colIdx) {
                var ftcPr = _qn(futureCells[fc], 'tcPr');
                var fvm = ftcPr ? _qn(ftcPr, 'vMerge') : null;
                if (fvm && (_attr(fvm, 'val') || 'continue') === 'continue') {
                  rowspan++;
                  found = true;
                }
                break;
              }
              var fgs = _qn(_qn(futureCells[fc], 'tcPr'), 'gridSpan');
              futureColIdx += fgs ? parseInt(_attr(fgs, 'val') || '1', 10) : 1;
            }
            if (!found) break;
          }
        }

        // Build cell content
        var cellHtml = '';
        var cellPs = _qnAll(tc, 'p');
        for (var cpi = 0; cpi < cellPs.length; cpi++) {
          var cp = cellPs[cpi];
          var cpPr = _qn(cp, 'pPr');
          var cpNumPr = cpPr ? _qn(cpPr, 'numPr') : null;
          var cellText = await processParagraphContent(cp);

          // Check for numbering in cell paragraphs
          if (cpNumPr) {
            var cnid = _attr(_qn(cpNumPr, 'numId'), 'val');
            var cilvl = parseInt(_attr(_qn(cpNumPr, 'ilvl'), 'val') || '0', 10);
            if (cnid && cnid !== '0') {
              var cpfx = resolveNumbering(cnid, cilvl);
              if (cpfx === '\x00BULLET') cellText = '• ' + cellText;
              else if (cpfx) cellText = cpfx + ' ' + cellText;
            }
          }

          if (cpi > 0) cellHtml += '<br/>';
          cellHtml += cellText;
        }

        // Determine tag
        var cellTag = (ri === 0) ? 'th' : 'td';
        var attrs = '';
        if (colspan > 1) attrs += ' colspan="' + colspan + '"';
        if (rowspan > 1) attrs += ' rowspan="' + rowspan + '"';
        html += '<' + cellTag + attrs + '>' + cellHtml + '</' + cellTag + '>';

        colIdx += colspan;
      }
      html += '</tr>';
    }
    html += '</table>';
    return html;
  }

  // ===== Process document body =====
  var body = _qn(dd.documentElement, 'body');
  if (!body) return '<p>无法读取文档内容</p>';

  var output = '';
  var children = [];
  for (var bc = body.firstChild; bc; bc = bc.nextSibling) {
    if (bc.nodeType === 1) children.push(bc);
  }

  for (var bi = 0; bi < children.length; bi++) {
    var node = children[bi];
    var nodeName = node.localName;

    // --- Table ---
    if (nodeName === 'tbl') {
      output += await processTable(node);
      continue;
    }

    // --- Structured document tag (e.g. TOC wrapper) ---
    if (nodeName === 'sdt') {
      var sdtBody = _qn(node, 'sdtContent');
      if (sdtBody) {
        var sdtChildren = [];
        for (var sc = sdtBody.firstChild; sc; sc = sc.nextSibling) {
          if (sc.nodeType === 1) sdtChildren.push(sc);
        }
        for (var si = 0; si < sdtChildren.length; si++) {
          if (sdtChildren[si].localName === 'p') {
            // Process as paragraph (below logic duplicated inline)
            var sp = sdtChildren[si];
            var spPr = _qn(sp, 'pPr');
            var spStyle = spPr ? _attr(_qn(spPr, 'pStyle'), 'val') : '';
            var shl = getHeadingLevel(spStyle, spPr);
            var sContent = await processParagraphContent(sp);
            if (!sContent.trim()) continue;
            if (shl > 0) {
              output += '<h' + shl + '>' + sContent + '</h' + shl + '>';
            } else {
              output += '<p>' + sContent + '</p>';
            }
          } else if (sdtChildren[si].localName === 'tbl') {
            output += await processTable(sdtChildren[si]);
          }
        }
      }
      continue;
    }

    // --- Paragraph ---
    if (nodeName === 'p') {
      var pPr = _qn(node, 'pPr');
      var pStyle = pPr ? _attr(_qn(pPr, 'pStyle'), 'val') : '';
      var headingLevel = getHeadingLevel(pStyle, pPr);

      // Check for oMathPara (block math paragraph)
      var oMathParas = _qnDeep(node, 'oMathPara');
      if (oMathParas.length > 0) {
        for (var mi = 0; mi < oMathParas.length; mi++) {
          var oMaths = _qnAll(oMathParas[mi], 'oMath');
          for (var mj = 0; mj < oMaths.length; mj++) {
            var latex = ommlToLatex(oMaths[mj]);
            if (latex) output += '<div class="math-block">\\[' + latex + '\\]</div>';
          }
        }
        continue;
      }

      // Also check for standalone oMath directly in paragraph
      var directMath = _qnAll(node, 'oMath');
      var hasOnlyMath = directMath.length > 0;
      if (hasOnlyMath) {
        // Check if paragraph has any non-math, non-empty content
        var hasText = false;
        for (var tc = node.firstChild; tc; tc = tc.nextSibling) {
          if (tc.nodeType === 1 && tc.localName === 'r') {
            var rt = getRunText(tc);
            if (rt.trim()) { hasText = true; break; }
          }
        }
        if (!hasText && directMath.length === 1) {
          // Single math expression as block
          var blatex = ommlToLatex(directMath[0]);
          if (blatex) { output += '<div class="math-block">\\[' + blatex + '\\]</div>'; continue; }
        }
      }

      var content = await processParagraphContent(node);
      if (!content.trim()) continue;

      // Numbering
      var numPr = pPr ? _qn(pPr, 'numPr') : null;
      var numId = null, ilvl = 0;
      if (numPr) {
        numId = _attr(_qn(numPr, 'numId'), 'val');
        ilvl = parseInt(_attr(_qn(numPr, 'ilvl'), 'val') || '0', 10);
      }
      // Check style-based numbering
      if (!numId || numId === '0') {
        if (pStyle && styleNumIds[pStyle]) {
          numId = styleNumIds[pStyle].numId;
          ilvl = parseInt(styleNumIds[pStyle].ilvl || '0', 10);
        }
      }

      if (headingLevel > 0) {
        // Heading
        var prefix = '';
        if (numId && numId !== '0') {
          // Check if this is a TOC heading - skip numbering
          if (!isTocHeading(content.replace(/<[^>]*>/g, ''))) {
            prefix = resolveNumbering(numId, ilvl);
            if (prefix) prefix = prefix + ' ';
          }
        }
        if (headingLevel === 1) currentChapter++;
        output += '<h' + headingLevel + '>' + prefix + content + '</h' + headingLevel + '>';
      } else if (numId && numId !== '0') {
        // List item with numbering
        var listPrefix = resolveNumbering(numId, ilvl);
        if (listPrefix === '\x00BULLET') {
          output += '<ul><li>' + content + '</li></ul>';
        } else if (listPrefix) {
          output += '<p>' + listPrefix + ' ' + content + '</p>';
        } else {
          output += '<p>' + content + '</p>';
        }
      } else {
        output += '<p>' + content + '</p>';
      }
      continue;
    }
  }

  return output;
}

window.parseDocx = parseDocx;

function mathNodeTex(node, displayMode) {
  const fromAttribute = node.getAttribute && node.getAttribute('data-tex');
  if (fromAttribute) return fromAttribute;
  const text = node.textContent || '';
  return (displayMode
    ? text.replace(/^\\\[/, '').replace(/\\\]$/, '')
    : text.replace(/^\\\(/, '').replace(/\\\)$/, '')
  ).trim();
}

// ===== HTML to Markdown converter =====
function htmlToMarkdown(html) {
  const doc = new DOMParser().parseFromString(html, 'text/html');

  function processNode(node) {
    if (node.nodeType === Node.TEXT_NODE) return node.textContent;
    if (node.nodeType !== Node.ELEMENT_NODE) return '';
    const tag = node.tagName.toLowerCase();
    const children = Array.from(node.childNodes).map(processNode).join('');
    switch (tag) {
      case 'h1': return '# ' + children.trim() + '\n\n';
      case 'h2': return '## ' + children.trim() + '\n\n';
      case 'h3': return '### ' + children.trim() + '\n\n';
      case 'h4': return '#### ' + children.trim() + '\n\n';
      case 'h5': return '##### ' + children.trim() + '\n\n';
      case 'h6': return '###### ' + children.trim() + '\n\n';
      case 'p': return children.trim() + '\n\n';
      case 'br': return '\n';
      case 'strong': case 'b': return '**' + children.trim() + '**';
      case 'em': case 'i': return '*' + children.trim() + '*';
      case 'u': return '<u>' + children.trim() + '</u>';
      case 's': case 'strike': case 'del': return '~~' + children.trim() + '~~';
      case 'sub': return '<sub>' + children.trim() + '</sub>';
      case 'sup': return '<sup>' + children.trim() + '</sup>';
      case 'a': {
        const href = node.getAttribute('href') || '';
        const text = children.trim();
        if (!text && !href) return '';
        if (!href) return text;
        return '[' + text + '](' + href + ')';
      }
      case 'img': {
        const src = node.getAttribute('src') || '';
        const alt = node.getAttribute('alt') || '';
        return '![' + alt + '](' + src + ')\n\n';
      }
      case 'ul': return children + '\n';
      case 'ol': return children + '\n';
      case 'li': {
        const parent = node.parentElement;
        const isOl = parent && parent.tagName.toLowerCase() === 'ol';
        const idx = isOl ? Array.from(parent.children).indexOf(node) + 1 : 0;
        const prefix = isOl ? idx + '. ' : '- ';
        return prefix + children.trim() + '\n';
      }
      case 'blockquote': {
        const lines = children.trim().split('\n');
        return lines.map(function(l) { return '> ' + l; }).join('\n') + '\n\n';
      }
      case 'pre': return '```\n' + node.textContent.trim() + '\n```\n\n';
      case 'code': {
        if (node.parentElement && node.parentElement.tagName.toLowerCase() === 'pre') return children;
        return '`' + children.trim() + '`';
      }
      case 'hr': return '---\n\n';
      case 'table': return processTable(node);
      case 'span': {
        if (node.classList && node.classList.contains('math-inline')) {
          var tex = mathNodeTex(node, false);
          return tex ? ('$' + tex + '$') : children;
        }
        return children;
      }
      case 'div': {
        if (node.classList && node.classList.contains('math-block')) {
          var tex = mathNodeTex(node, true);
          return tex ? ('$$' + tex + '$$\n\n') : children;
        }
        return children;
      }
      default: return children;
    }
  }

  function processTable(table) {
    const rows = Array.from(table.querySelectorAll(':scope > tr, :scope > thead > tr, :scope > tbody > tr'));
    if (!rows.length) return '';
    let hasMerge = false;
    rows.forEach(function(row) {
      row.querySelectorAll('th, td').forEach(function(cell) {
        if (parseInt(cell.getAttribute('colspan') || '1', 10) > 1 ||
            parseInt(cell.getAttribute('rowspan') || '1', 10) > 1) hasMerge = true;
      });
    });
    if (hasMerge) {
      let html = '<table>\n';
      rows.forEach(function(row) {
        html += '<tr>';
        row.querySelectorAll('th, td').forEach(function(cell) {
          const tag = cell.tagName.toLowerCase();
          const cs = cell.getAttribute('colspan');
          const rs = cell.getAttribute('rowspan');
          let attrs = '';
          if (cs && cs !== '1') attrs += ' colspan="' + cs + '"';
          if (rs && rs !== '1') attrs += ' rowspan="' + rs + '"';
          html += '<' + tag + attrs + '>' + cell.textContent.trim() + '</' + tag + '>';
        });
        html += '</tr>\n';
      });
      html += '</table>\n\n';
      return html;
    }
    let md = '';
    rows.forEach(function(row, ri) {
      const cells = Array.from(row.querySelectorAll('th, td'));
      md += '| ' + cells.map(function(c) { return c.textContent.trim(); }).join(' | ') + ' |\n';
      if (ri === 0) md += '| ' + cells.map(function() { return '---'; }).join(' | ') + ' |\n';
    });
    return md + '\n';
  }

  const result = processNode(doc.body);
  var merged = result.replace(/\n{3,}/g, '\n\n');
  // Merge adjacent bold markers: **A** **B** or **A****B** → **A B** or **AB**
  for (var _m = 0; _m < 5; _m++) {
    merged = merged.replace(/\*\*([^*]+?)\*\*( ?)\*\*([^*]+?)\*\*/g, '**$1$2$3**');
  }
  // Merge adjacent italic markers: *A* *B* or *A**B* → *A B* or *AB*
  for (var _n = 0; _n < 5; _n++) {
    merged = merged.replace(/(?<!\*)\*([^*]+?)\*( ?)\*([^*]+?)\*(?!\*)/g, '*$1$2$3*');
  }
  return merged.trim() + '\n';
}

window.MarkionWordImportRuntime = Object.freeze({ parseDocx: window.parseDocx, htmlToMarkdown });
