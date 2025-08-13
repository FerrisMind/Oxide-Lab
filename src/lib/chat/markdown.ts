import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js/lib/common';
import { markedHighlight } from 'marked-highlight';

// Глобальные настройки markdown-рендера:
// - gfm: поддержка таблиц/списков/код-блоков;
// - breaks: false — одиночные \n не превращаем в <br>, чтобы избежать «лестницы» из строк
marked.use(markedHighlight({
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
  }
}));

marked.setOptions({ gfm: true, breaks: true });

// Простая обёртка для безопасного преобразования Markdown → HTML
export function renderMarkdownToSafeHtml(markdownText: string): string {
  try {
    let input = markdownText ?? '';
    // Нормализуем переводы строк
    input = input.replace(/\r\n?/g, '\n');
    // Быстрый путь: разворачиваем полноценные блоки ```md|markdown|gfm ...```
    input = input.replace(/```(?:markdown|md|gfm)\s*\n([\s\S]*?)```/gi, (_m, inner) => inner);
    input = input.replace(/~~~(?:markdown|md|gfm)\s*\n([\s\S]*?)~~~/gi, (_m, inner) => inner);
    // Робастный путь: если встречаются незакрытые/странно размеченные блоки с тегом markdown —
    // снимем ограждения построчно, чтобы контент парсился как обычный Markdown
    if (/^```(?:markdown|md|gfm)\s*$/im.test(input)) {
      const lines = input.split(/\n/);
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
      input = out.join('\n');
    }
    const dirty = marked.parse(input) as string;
    // Разрешаем типичные теги Markdown, остальное вычищаем
    const clean = DOMPurify.sanitize(dirty, {
      ALLOWED_TAGS: [
        'h1','h2','h3','h4','h5','h6','p','br','hr','strong','em','b','i','u','s',
        'ul','ol','li','blockquote','code','pre','a','table','thead','tbody','tr','th','td','span'
      ],
      ALLOWED_ATTR: ['href','title','target','rel','class']
    });
    return clean;
  } catch {
    return DOMPurify.sanitize(markdownText ?? '');
  }
}


