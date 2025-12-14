import type { ParseResult, StreamSegment } from './types.js';
import { escapeAttr, escapeHtml } from './escape.js';
import { KNOWN_TAG_PREFIXES, THINK_CLOSE_TOKEN, THINK_OPEN_TOKEN } from './constants.js';

export function createStreamParser() {
  // Состояния парсера для потокового разбора спец-тегов
  let inCode = false;
  let codeLang: string | null = null;
  let _codeBuf = '';
  let inToolCall = false;
  let _toolCallBuf = '';
  let inToolResponse = false;
  let _toolResponseBuf = '';
  let inMedia: null | 'image' | 'audio' | 'video' = null;
  let mediaBuf = '';

  function parse(streamBuf: string): ParseResult {
    let buf = streamBuf;
    // Защита от разрыва UTF-16 суррогатной пары на границе чанка:
    // если последний код-юнит — верхний суррогат (D800–DBFF), не выводим его сейчас,
    // оставляем в remainder, чтобы склеить с нижним суррогатом из следующего чанка.
    let trailingHighSurrogate = '';
    if (buf.length > 0) {
      const lastCode = buf.charCodeAt(buf.length - 1);
      if (lastCode >= 0xd800 && lastCode <= 0xdbff) {
        trailingHighSurrogate = buf.slice(-1);
        buf = buf.slice(0, -1);
      }
    }
    const segments: StreamSegment[] = [];

    const appendPlain = (s: string) => {
      if (s.length === 0) return;
      segments.push({ kind: 'text', data: s });
    };

    let i = 0;
    while (i < buf.length) {
      // Внутри кода (инкрементальный вывод)
      if (inCode) {
        const closeCandidates: Array<{ token: string; index: number }> = [];
        if (codeLang) {
          const langClose = `<|/${codeLang}|>`;
          closeCandidates.push({ token: langClose, index: buf.indexOf(langClose, i) });
        }
        const genericClose = '<|endcode|>';
        closeCandidates.push({ token: genericClose, index: buf.indexOf(genericClose, i) });

        let matchedClose: { token: string; index: number } | null = null;
        for (const candidate of closeCandidates) {
          if (candidate.index === -1) continue;
          if (!matchedClose || candidate.index < matchedClose.index) {
            matchedClose = candidate;
          }
        }

        if (!matchedClose) {
          const chunk = buf.slice(i);
          _codeBuf += chunk;
          i = buf.length;
          break;
        }

        const chunk = buf.slice(i, matchedClose.index);
        _codeBuf += chunk;
        const cls = codeLang ? ` class=\\"language-${escapeAttr(codeLang)}\\"` : '';
        const html = `<pre class=\"code\"><code${cls}>${escapeHtml(_codeBuf)}</code></pre>`;
        segments.push({ kind: 'html', data: html });
        _codeBuf = '';
        inCode = false;
        codeLang = null;
        i = matchedClose.index + matchedClose.token.length;
        continue;
      }

      // Внутри tool_call (инкрементальный вывод)
      if (inToolCall) {
        const endIdx = buf.indexOf('</tool_call>', i);
        if (endIdx === -1) {
          const chunk = buf.slice(i);
          _toolCallBuf += chunk;
          segments.push({ kind: 'html', data: escapeHtml(chunk) });
          i = buf.length;
          break;
        }
        const chunk = buf.slice(i, endIdx);
        _toolCallBuf += chunk;
        segments.push({ kind: 'html', data: escapeHtml(chunk) + `</pre></div>` });
        _toolCallBuf = '';
        inToolCall = false;
        i = endIdx + '</tool_call>'.length;
        continue;
      }

      // Внутри tool_response (инкрементальный вывод)
      if (inToolResponse) {
        const endIdx = buf.indexOf('</tool_response>', i);
        if (endIdx === -1) {
          const chunk = buf.slice(i);
          _toolResponseBuf += chunk;
          segments.push({ kind: 'html', data: escapeHtml(chunk) });
          i = buf.length;
          break;
        }
        const chunk = buf.slice(i, endIdx);
        _toolResponseBuf += chunk;
        segments.push({ kind: 'html', data: escapeHtml(chunk) + `</pre></div>` });
        _toolResponseBuf = '';
        inToolResponse = false;
        i = endIdx + '</tool_response>'.length;
        continue;
      }

      // Внутри мультимедиа (инкрементальный вывод)
      if (inMedia) {
        const close = `<|/${inMedia}|>`;
        const endIdx = buf.indexOf(close, i);
        if (endIdx === -1) {
          const chunk = buf.slice(i);
          mediaBuf += chunk;
          i = buf.length;
          break;
        }
        mediaBuf += buf.slice(i, endIdx);
        const src = mediaBuf.trim();
        if (inMedia === 'image') {
          segments.push({
            kind: 'html',
            data: `<img class=\"media-img\" src=\"${escapeAttr(src)}\" alt=\"image\" />`,
          });
          segments.push({ kind: 'text', data: `\n[image: ${src}]\n` });
        } else if (inMedia === 'audio') {
          segments.push({
            kind: 'html',
            data: `<audio class=\"media-audio\" controls src=\"${escapeAttr(src)}\"></audio>`,
          });
          segments.push({ kind: 'text', data: `\n[audio: ${src}]\n` });
        } else if (inMedia === 'video') {
          segments.push({
            kind: 'html',
            data: `<video class=\"media-video\" controls src=\"${escapeAttr(src)}\"></video>`,
          });
          segments.push({ kind: 'text', data: `\n[video: ${src}]\n` });
        }
        mediaBuf = '';
        inMedia = null;
        i = endIdx + close.length;
        continue;
      }

      // Обычный текст и поиск тегов
      const lt = buf.indexOf('<', i);
      if (lt === -1) {
        appendPlain(buf.slice(i));
        i = buf.length;
        break;
      }
      appendPlain(buf.slice(i, lt));
      i = lt;
      const rest = buf.slice(i);

      // ChatML / роли
      // Gemma-style turn markers
      if (rest.startsWith('<start_of_turn>')) {
        i += '<start_of_turn>'.length;
        continue;
      }
      if (rest.startsWith('<end_of_turn>')) {
        i += '<end_of_turn>'.length;
        continue;
      }

      // ChatML / роли
      if (rest.startsWith('<|im_start|>')) {
        i += '<|im_start|>'.length;
        const nl = buf.indexOf('\n', i);
        if (nl === -1) {
          i = lt;
          break;
        }
        // роль между <|im_start|> и переводом строки
        // Для стрима: просто поглощаем заголовок роли, ничего не выводим
        i = nl + 1;
        continue;
      }
      if (rest.startsWith('<|im_end|>')) {
        i += '<|im_end|>'.length;
        continue;
      }
      if (rest.startsWith('<|user|>')) {
        i += '<|user|>'.length;
        continue;
      }
      if (rest.startsWith('<|assistant|>')) {
        i += '<|assistant|>'.length;
        continue;
      }
      if (rest.startsWith('<|system|>')) {
        i += '<|system|>'.length;
        continue;
      }

      // Llama3 header
      if (rest.startsWith('<|start_header_id|>')) {
        const end = buf.indexOf('<|end_header_id|>', i);
        if (end === -1) {
          i = lt;
          break;
        } else {
          i = end + '<|end_header_id|>'.length;
          continue;
        }
      }
      // EOS/EOT/EOM
      if (rest.startsWith('<|eot_id|>')) {
        i += '<|eot_id|>'.length;
        continue;
      }
      if (rest.startsWith('<|eom_id|>')) {
        i += '<|eom_id|>'.length;
        continue;
      }
      if (rest.startsWith('<|endoftext|>')) {
        i += '<|endoftext|>'.length;
        continue;
      }
      if (rest.startsWith('<|end_of_text|>')) {
        i += '<|end_of_text|>'.length;
        continue;
      }
      if (rest.startsWith('<|begin_of_sentence|>')) {
        i += '<|begin_of_sentence|>'.length;
        continue;
      }
      if (rest.startsWith('<|end_of_sentence|>')) {
        i += '<|end_of_sentence|>'.length;
        continue;
      }

      // Tool calling
      if (rest.startsWith('<tool_call>')) {
        inToolCall = true;
        _toolCallBuf = '';
        i += '<tool_call>'.length;
        segments.push({
          kind: 'html',
          data: `<div class=\"tool-card\"><div class=\"tool-title\">Вызов инструмента</div><pre class=\"tool-pre\">`,
        });
        continue;
      }
      if (rest.startsWith('<tool_response>')) {
        inToolResponse = true;
        _toolResponseBuf = '';
        i += '<tool_response>'.length;
        segments.push({
          kind: 'html',
          data: `<div class=\"tool-response\"><div class=\"tool-title\">Ответ инструмента</div><pre class=\"tool-pre\">`,
        });
        continue;
      }

      // Кодовые блоки
      if (rest.startsWith('<|code|>')) {
        inCode = true;
        codeLang = null;
        _codeBuf = '';
        i += '<|code|>'.length;
        continue;
      }
      if (rest.startsWith('<|python|>')) {
        inCode = true;
        codeLang = 'python';
        _codeBuf = '';
        i += '<|python|>'.length;
        continue;
      }
      if (rest.startsWith('<|endcode|>')) {
        i += '<|endcode|>'.length;
        continue;
      }
      if (rest.startsWith('<|/python|>')) {
        i += '<|/python|>'.length;
        continue;
      }

      // Мультимедиа
      if (rest.startsWith('<|image|>')) {
        inMedia = 'image';
        mediaBuf = '';
        i += '<|image|>'.length;
        continue;
      }
      if (rest.startsWith('<|audio|>')) {
        inMedia = 'audio';
        mediaBuf = '';
        i += '<|audio|>'.length;
        continue;
      }
      if (rest.startsWith('<|video|>')) {
        inMedia = 'video';
        mediaBuf = '';
        i += '<|video|>'.length;
        continue;
      }

      // BOS/EOS
      if (rest.startsWith('<s>')) {
        i += 3;
        continue;
      }
      if (rest.startsWith('</s>')) {
        i += 4;
        continue;
      }

      // Потоковые размышления: передаем теги как есть, без спец-парсинга
      if (rest.startsWith(THINK_OPEN_TOKEN)) {
        // #region agent log
        fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            sessionId: 'debug-session',
            runId: 'run-pre-fix',
            hypothesisId: 'H12',
            location: 'parser.impl.ts:parse',
            message: 'THINK_OPEN_TOKEN encountered',
            data: { index: i, bufSnippet: rest.slice(0, 80) },
            timestamp: Date.now(),
          }),
        }).catch(() => {});
        // #endregion
        appendPlain(THINK_OPEN_TOKEN);
        i += THINK_OPEN_TOKEN.length;
        continue;
      }
      if (rest.startsWith(THINK_CLOSE_TOKEN)) {
        // #region agent log
        fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            sessionId: 'debug-session',
            runId: 'run-pre-fix',
            hypothesisId: 'H12',
            location: 'parser.impl.ts:parse',
            message: 'THINK_CLOSE_TOKEN encountered',
            data: { index: i, bufSnippet: rest.slice(0, 80) },
            timestamp: Date.now(),
          }),
        }).catch(() => {});
        // #endregion
        appendPlain(THINK_CLOSE_TOKEN);
        i += THINK_CLOSE_TOKEN.length;
        continue;
      }

      // Если теги размышлений пришли частично — ждём догрузку
      if (
        THINK_OPEN_TOKEN.startsWith(rest) ||
        THINK_CLOSE_TOKEN.startsWith(rest)
      ) {
        break;
      }

      // reserved tokens
      const m = rest.match(/^<\|reserved_[^|]*\|>/);
      if (m) {
        i += m[0].length;
        continue;
      }

      // неизвестный/возможно неполный тег
      // Если это начало любого из известных тегов, но он ещё не доприбыл — ждём следующую порцию
      const maybeKnown = KNOWN_TAG_PREFIXES.some((p) => p.startsWith(rest));
      if (maybeKnown || rest.startsWith('<|reserved_')) {
        // ждём догрузку оставшейся части тега
        break;
      }
      // Иначе считаем символ '<' обычным текстом и продолжаем разбор,
      // чтобы не «замораживать» вывод на незнакомых HTML-тегах
      appendPlain('<');
      i += 1;
      continue;
    }
    const remainder = (i < buf.length ? buf.slice(i) : '') + trailingHighSurrogate;
    return { segments, remainder };
  }

  function reset() {
    inCode = false;
    codeLang = null;
    _codeBuf = '';
    inToolCall = false;
    _toolCallBuf = '';
    inToolResponse = false;
    _toolResponseBuf = '';
    inMedia = null;
    mediaBuf = '';
  }

  return { parse, reset };
}
