import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js/lib/common';
import { markedHighlight } from 'marked-highlight';

// Глобальные настройки markdown-рендера:
// - gfm: поддержка таблиц/списков/код-блоков/ссылок/задач;
// - breaks: false — одиночные \n не превращаем в <br>, чтобы избежать «лестницы» из строк
// - pedantic: false — более либеральный парсинг для лучшей совместимости
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

// Настройки marked.js согласно спецификации
marked.setOptions({ 
  gfm: true,     // GitHub Flavored Markdown
  breaks: false, // Не превращаем одиночные переносы в <br>
  pedantic: false // Более либеральный парсинг
});

// Простая обёртка для безопасного преобразования Markdown → HTML
export function renderMarkdownToSafeHtml(markdownText: string): string {
  try {
    let input = markdownText ?? '';
    // Нормализуем переводы строк
    input = input.replace(/\r\n?/g, '\n');

    // Быстрый путь: разворачиваем полноценные блоки ```md|markdown|gfm ...```
    let enhanced = input.replace(/```(?:markdown|md|gfm)\s*\n([\s\S]*?)```/gi, (_m, inner) => inner);
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
    const dirty = (typeof (marked as any).parse === 'function'
      ? (marked as any).parse(enhanced)
      : (marked as any)(enhanced)) as string;
    // Разрешаем типичные теги Markdown, остальное вычищаем
    const sanitized = typeof window !== 'undefined'
      ? DOMPurify.sanitize(dirty, {
          ALLOWED_TAGS: [
            // Заголовки и структура
            'h1','h2','h3','h4','h5','h6','p','br','hr','div','section','article','aside',
            'nav','header','footer','main','address',
            // Форматирование текста
            'strong','em','b','i','u','s','mark','small','del','ins','sub','sup',
            // Списки и задачи
            'ul','ol','li','dl','dt','dd',
            // Цитаты и код
            'blockquote','code','pre','kbd','samp','var',
            // Ссылки и медиа
            'a','img','figure','figcaption','picture','source',
            // Таблицы
            'table','thead','tbody','tfoot','tr','th','td','caption','colgroup','col',
            // Интерактивные элементы
            'details','summary',
            // Семантические элементы
            'span','abbr','dfn','q','cite','time','data','output',
            // Математика и формулы (для MathJax/KaTeX)
            'math','mi','mo','mn','ms','mtext','mrow','msup','msub','mfrac','msqrt','mroot',
            // Ruby аннотации (для восточных языков)
            'ruby','rt','rp',
            // Дополнительные элементы
            'wbr','bdi','bdo'
          ],
          ALLOWED_ATTR: [
            // Ссылки и навигация
            'href','title','target','rel','download','hreflang',
            // Медиа атрибуты
            'src','alt','width','height','sizes','srcset','loading','decoding',
            // Идентификаторы и метаданные
            'id','name','class','lang','dir','translate',
            // Стили и данные
            'style','data-*','content',
            // Доступность
            'aria-*','role','tabindex','accesskey',
            // Таблицы
            'colspan','rowspan','scope','headers','summary',
            // Формы и интерактивность
            'type','value','placeholder','readonly','disabled','checked',
            'min','max','step','pattern','required','autocomplete',
            // Дата и время
            'datetime','cite','open','reversed','start',
            // Медиа контент
            'controls','autoplay','muted','loop','preload','poster',
            // Математика
            'mathvariant','mathsize','mathcolor','mathbackground',
            // Дополнительные
            'hidden','contenteditable','spellcheck','draggable'
          ]
        })
      : dirty;
    return sanitized;
  } catch {
    return DOMPurify.sanitize(markdownText ?? '');
  }
}


