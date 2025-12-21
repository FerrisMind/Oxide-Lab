import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js/lib/common';
import { markedHighlight } from 'marked-highlight';
import { THINK_CLOSE_TOKEN } from './parser/constants.js';

// Флаг: в последнем вызове renderMarkdownToSafeHtml был вставлен авто-закрывающий </think>
export let lastAutoClosedThink = false;

// Глобальные настройки markdown-рендера:
// - gfm: поддержка таблиц/списков/код-блоков/ссылок/задач;
// - breaks: false — одиночные \n не превращаем в <br>, чтобы избежать «лестницы» из строк
// - pedantic: false — более либеральный парсинг для лучшей совместимости
marked.use(
  markedHighlight({
    langPrefix: 'hljs language-',
    highlight(code, lang) {
      try {
        if (lang && hljs.getLanguage(lang)) {
          return hljs.highlight(code, { language: lang }).value;
        }
        return hljs.highlightAuto(code).value;
      } catch {
        return code;
      }
    },
  }),
);

// Настройки marked.js согласно спецификации
marked.setOptions({
  gfm: true, // GitHub Flavored Markdown
  breaks: false, // Не превращаем одиночные переносы в <br>
  pedantic: false, // Более либеральный парсинг
});

// Простая обёртка для безопасного преобразования Markdown → HTML
export function renderMarkdownToSafeHtml(markdownText: string): string {
  try {
    let input = markdownText ?? '';
    // Нормализуем переводы строк
    input = input.replace(/\r\n?/g, '\n');

    // Обрабатываем GitHub-style callouts перед основным парсингом
    input = processCallouts(input);

    // Быстрый путь: разворачиваем полноценные блоки ```md|markdown|gfm ...```
    let enhanced = input.replace(
      /```(?:markdown|md|gfm)\s*\n([\s\S]*?)```/gi,
      (_m, inner) => inner,
    );
    enhanced = enhanced.replace(/~~~(?:markdown|md|gfm)\s*\n([\s\S]*?)~~~/gi, (_m, inner) => inner);
    // Робастный путь: если встречаются незакрытые/странно размеченные блоки с тегом markdown —
    // снимем ограждения построчно, чтобы контент парсился как обычный Markdown
    if (/^```(?:markdown|md|gfm)\s*$/im.test(enhanced)) {
      const lines = enhanced.split(/\n/);
      let inMdFence = false;
      const out: string[] = [];
      for (const line of lines) {
        if (/^```(?:markdown|md|gfm)\s*$/i.test(line)) {
          inMdFence = true; // пропускаем строку-ограждение
          continue;
        }
        if (inMdFence && /^```+\s*$/.test(line)) {
          inMdFence = false; // закрытие — тоже пропускаем
          continue;
        }
        out.push(line);
      }
      enhanced = out.join('\n');
    }

    const openCountRaw = countThink(enhanced, '<think>');
    const closeCountRaw = countThink(enhanced, '</think>');
    const forceStreaming = openCountRaw > closeCountRaw;
    if (forceStreaming && openCountRaw > closeCountRaw) {
      // Автозакрытие незавершённого <think>, чтобы остальной текст не попадал внутрь
      enhanced += THINK_CLOSE_TOKEN;
      lastAutoClosedThink = true;
    } else {
      lastAutoClosedThink = false;
    }

    const dirty = (
      typeof (marked as any).parse === 'function'
        ? (marked as any).parse(enhanced)
        : (marked as any)(enhanced)
    ) as string;

    // Разрешаем типичные теги Markdown, остальное вычищаем
    const sanitizedLocal =
      typeof window !== 'undefined'
        ? DOMPurify.sanitize(dirty, {
            ALLOWED_TAGS: [
              // Заголовки и структура
              'h1',
              'h2',
              'h3',
              'h4',
              'h5',
              'h6',
              'p',
              'br',
              'hr',
              'div',
              'section',
              'article',
              'aside',
              'nav',
              'header',
              'footer',
              'main',
              'address',
              // Форматирование текста
              'strong',
              'em',
              'b',
              'i',
              'u',
              's',
              'mark',
              'small',
              'del',
              'ins',
              'sub',
              'sup',
              // Списки и задачи
              'ul',
              'ol',
              'li',
              'dl',
              'dt',
              'dd',
              // Цитаты и код
              'blockquote',
              'code',
              'pre',
              'kbd',
              'samp',
              'var',
              // Ссылки и медиа
              'a',
              'img',
              'figure',
              'figcaption',
              'picture',
              'source',
              // Таблицы
              'table',
              'thead',
              'tbody',
              'tfoot',
              'tr',
              'th',
              'td',
              'caption',
              'colgroup',
              'col',
              // Интерактивные элементы
              'details',
              'summary',
              'think',
              // Семантические элементы
              'span',
              'abbr',
              'dfn',
              'q',
              'cite',
              'time',
              'data',
              'output',
              // Математика и формулы (для MathJax/KaTeX)
              'math',
              'mi',
              'mo',
              'mn',
              'ms',
              'mtext',
              'mrow',
              'msup',
              'msub',
              'mfrac',
              'msqrt',
              'mroot',
              // Ruby аннотации (для восточных языков)
              'ruby',
              'rt',
              'rp',
              // SVG для иконок callout
              'svg',
              'path',
              // Дополнительные элементы
              'wbr',
              'bdi',
              'bdo',
            ],
            ALLOWED_ATTR: [
              // Ссылки и навигация
              'href',
              'title',
              'target',
              'rel',
              'download',
              'hreflang',
              // Медиа атрибуты
              'src',
              'alt',
              'width',
              'height',
              'sizes',
              'srcset',
              'loading',
              'decoding',
              // Идентификаторы и метаданные
              'id',
              'name',
              'class',
              'lang',
              'dir',
              'translate',
              // Стили и данные
              'style',
              'data-*',
              'content',
              // Доступность
              'aria-*',
              'role',
              'tabindex',
              'accesskey',
              // Таблицы
              'colspan',
              'rowspan',
              'scope',
              'headers',
              'summary',
              // Формы и интерактивность
              'type',
              'value',
              'placeholder',
              'readonly',
              'disabled',
              'checked',
              'min',
              'max',
              'step',
              'pattern',
              'required',
              'autocomplete',
              // Дата и время
              'datetime',
              'cite',
              'open',
              'reversed',
              'start',
              // Медиа контент
              'controls',
              'autoplay',
              'muted',
              'loop',
              'preload',
              'poster',
              // Математика
              'mathvariant',
              'mathsize',
              'mathcolor',
              'mathbackground',
              // SVG атрибуты для иконок
              'viewBox',
              'fill',
              'd',
              'width',
              'height',
              // Дополнительные
              'hidden',
              'contenteditable',
              'spellcheck',
              'draggable',
            ],
          })
        : dirty;

    const wrappedLocal = wrapThinkBlocks(sanitizedLocal, forceStreaming);
    return wrappedLocal;
  } catch {
    const fallback = DOMPurify.sanitize(markdownText ?? '');
    return fallback;
  }
}

// Оборачиваем завершённые блоки <think>...</think> в плейсхолдеры под аккордеон
function wrapThinkBlocks(html: string, forceStreaming: boolean = false): string {
  const OPEN = '<think>';
  const CLOSE = '</think>';
  if (!html.includes(OPEN)) {
    return html;
  }

  let out = '';
  let cursor = 0;
  let idx = 0;

  while (true) {
    const start = html.indexOf(OPEN, cursor);
    if (start === -1) {
      out += html.slice(cursor);
      break;
    }
    const end = html.indexOf(CLOSE, start + OPEN.length);
    const hasClose = end !== -1;
    const streaming = !hasClose; // управление break/обрезкой
    const placeholderStreaming = streaming || forceStreaming; // UI держим открытым, если закрытие авто
    const endPos = hasClose ? end : html.length;

    const before = html.slice(cursor, start);
    const inner = html.slice(start + OPEN.length, endPos);
    const key = `think-${idx++}`;

    out += before;
    out += `<div class="thinking-placeholder" data-think-id="${key}" data-streaming="${placeholderStreaming}">`;
    out += `<div class="thinking-content" data-think-slot>${inner}</div>`;
    out += `<div class="thinking-mount"></div>`;
    out += `</div>`;

    if (streaming) {
      // всё оставшееся сейчас в размышлении — ждём закрывающий тег в следующих чанках
      cursor = html.length;
      break;
    }

    cursor = endPos + CLOSE.length;
  }

  return out;
}

function countThink(text: string, token: string): number {
  let count = 0;
  let idx = 0;
  while (true) {
    const pos = text.indexOf(token, idx);
    if (pos === -1) break;
    count += 1;
    idx = pos + token.length;
  }
  return count;
}

// Функция для обработки GitHub-style callouts
function processCallouts(text: string): string {
  const lines = text.split('\n');
  const result: string[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];
    const calloutMatch = line.match(/^>\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\](.*)$/);

    if (calloutMatch) {
      const [, type, title] = calloutMatch;
      const calloutType = type.toLowerCase();
      const customTitle = title.trim();

      // Начинаем обработку callout
      const calloutLines = [`<div class="callout callout-${calloutType}">`];
      calloutLines.push(`<div class="callout-body">`);
      calloutLines.push(`<div class="callout-icon">${getCalloutIcon(type)}</div>`);
      calloutLines.push(`<div class="callout-content">`);

      // Если есть кастомный заголовок, добавляем его как первый элемент контента
      if (customTitle) {
        calloutLines.push(`<div class="callout-custom-title">${customTitle}</div>`);
      }

      i++; // переходим к следующей строке

      // Собираем содержимое callout
      const contentLines: string[] = [];
      while (i < lines.length) {
        const contentLine = lines[i];

        // Если строка не начинается с '>', то callout закончился
        if (!contentLine.startsWith('>')) {
          break;
        }

        // Удаляем '> ' или '>' в начале строки
        const cleanLine = contentLine.replace(/^>\s?/, '');
        contentLines.push(cleanLine);
        i++;
      }

      // Обрабатываем содержимое как markdown
      if (contentLines.length > 0) {
        const contentMarkdown = contentLines.join('\n');
        // Рекурсивно обрабатываем содержимое (без callouts, чтобы избежать бесконечной рекурсии)
        const contentHtml = renderMarkdownContent(contentMarkdown);
        calloutLines.push(contentHtml);
      }

      calloutLines.push(`</div>`);
      calloutLines.push(`</div>`);
      calloutLines.push(`</div>`);

      result.push(...calloutLines);

      // i уже указывает на следующую строку после callout
      continue;
    }

    result.push(line);
    i++;
  }

  return result.join('\n');
}

// Вспомогательная функция для рендеринга markdown без callout обработки
function renderMarkdownContent(markdownText: string): string {
  try {
    const dirty = (
      typeof (marked as any).parse === 'function'
        ? (marked as any).parse(markdownText)
        : (marked as any)(markdownText)
    ) as string;
    return dirty;
  } catch {
    return markdownText;
  }
}

// Получение заголовка по умолчанию для типа callout
function _getDefaultCalloutTitle(type: string): string {
  const titles: Record<string, string> = {
    NOTE: 'Note',
    TIP: 'Tip',
    IMPORTANT: 'Important',
    WARNING: 'Warning',
    CAUTION: 'Caution',
  };
  return titles[type] || type;
}

// Получение SVG иконки для типа callout (Phosphor Icons)
function getCalloutIcon(type: string): string {
  const icons: Record<string, string> = {
    // Phosphor Info icon for NOTE
    NOTE: '<svg width="16" height="16" viewBox="0 0 256 256"><path fill="currentColor" d="M128 24a104 104 0 1 0 104 104A104.11 104.11 0 0 0 128 24m0 192a88 88 0 1 1 88-88a88.1 88.1 0 0 1-88 88m16-40a8 8 0 0 1-8 8h-24a8 8 0 0 1 0-16h8v-40h-8a8 8 0 0 1 0-16h16a8 8 0 0 1 8 8v48h8a8 8 0 0 1 8 8M124 96a12 12 0 1 1 12-12a12 12 0 0 1-12 12"/></svg>',
    // Phosphor Lightbulb icon for TIP
    TIP: '<svg width="16" height="16" viewBox="0 0 256 256"><path fill="currentColor" d="M176 232a8 8 0 0 1-8 8H88a8 8 0 0 1 0-16h80a8 8 0 0 1 8 8m40-128a87.55 87.55 0 0 1-33.64 69.21A16.24 16.24 0 0 0 176 186v6a16 16 0 0 1-16 16H96a16 16 0 0 1-16-16v-6a16 16 0 0 0-6.23-12.66A87.59 87.59 0 0 1 40 104.49C39.74 56.83 78.26 17.14 125.88 16A88 88 0 0 1 216 104m-16 0a72 72 0 0 0-73.74-72c-36 .92-66.13 30.77-67.26 66.75A71.64 71.64 0 0 0 83.18 151a32.17 32.17 0 0 1 12.82 25.82V192h64v-15.18A32.17 32.17 0 0 1 172.82 151A71.65 71.65 0 0 0 200 104m-72 56.61V136a8 8 0 0 0-16 0v24.61a16 16 0 1 0 16 0"/></svg>',
    // Phosphor SealWarning icon for IMPORTANT
    IMPORTANT:
      '<svg width="16" height="16" viewBox="0 0 256 256"><path fill="currentColor" d="M225.86 102.82c-3.77-3.94-7.67-8-9.14-11.57c-1.36-3.27-1.44-8.69-1.52-13.94c-.15-9.76-.31-20.82-8-28.51s-18.75-7.85-28.51-8c-5.25-.08-10.67-.16-13.94-1.52c-3.56-1.47-7.63-5.37-11.57-9.14C146.28 23.51 138.44 16 128 16s-18.27 7.51-25.18 14.14c-3.94 3.77-8 7.67-11.57 9.14C88 40.64 82.58 40.72 77.33 40.8c-9.76.15-20.82.31-28.51 8s-7.85 18.75-8 28.51c-.08 5.25-.16 10.67-1.52 13.94c-1.47 3.56-5.37 7.63-9.14 11.57C23.51 109.72 16 117.56 16 128s7.51 18.27 14.14 25.18c3.77 3.94 7.67 8 9.14 11.57c1.36 3.27 1.44 8.69 1.52 13.94c.15 9.76.31 20.82 8 28.51s18.75 7.85 28.51 8c5.25.08 10.67.16 13.94 1.52c3.56 1.47 7.63 5.37 11.57 9.14c6.9 6.63 14.74 14.14 25.18 14.14s18.27-7.51 25.18-14.14c3.94-3.77 8-7.67 11.57-9.14c3.27-1.36 8.69-1.44 13.94-1.52c9.76-.15 20.82-.31 28.51-8s7.85-18.75 8-28.51c.08-5.25.16-10.67 1.52-13.94c1.47-3.56 5.37-7.63 9.14-11.57c6.63-6.9 14.14-14.74 14.14-25.18s-7.51-18.27-14.14-25.18M120 80a8 8 0 0 1 16 0v56a8 8 0 0 1-16 0Zm8 104a12 12 0 1 1 12-12a12 12 0 0 1-12 12"/></svg>',
    // Phosphor Warning icon for WARNING
    WARNING:
      '<svg width="16" height="16" viewBox="0 0 256 256"><path fill="currentColor" d="m236.8 188.09l-99.21-169.86a16 16 0 0 0-27.18 0L11.2 188.09a16 16 0 0 0 13.59 24.17h198.42a16 16 0 0 0 13.59-24.17M120 104a8 8 0 0 1 16 0v40a8 8 0 0 1-16 0Zm8 88a12 12 0 1 1 12-12a12 12 0 0 1-12 12"/></svg>',
    // Phosphor ShieldWarning icon for CAUTION
    CAUTION:
      '<svg width="16" height="16" viewBox="0 0 256 256"><path fill="currentColor" d="M208 40H48a16 16 0 0 0-16 16v58.78c0 89.61 75.82 119.34 91 124.39a15.53 15.53 0 0 0 10 0c15.2-5.05 91-34.78 91-124.39V56a16 16 0 0 0-16-16M120 80a8 8 0 0 1 16 0v56a8 8 0 0 1-16 0Zm8 104a12 12 0 1 1 12-12a12 12 0 0 1-12 12"/></svg>',
  };
  return icons[type] || icons['NOTE'];
}
