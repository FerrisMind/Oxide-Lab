# StreamingCodeBlock

Компонент для отображения кода с поддержкой стриминга в реальном времени. Подсветка выполняется через Highlight.js; редактирования нет, только чтение/копирование.

## Особенности

- ✅ Инкрементальное обновление содержимого во время стриминга
- ✅ Плавные анимированные переходы между состояниями
- ✅ Поддержка множества языков программирования через Highlight.js
- ✅ Интерактивная полоса прогресса с возможностью разворачивания
- ✅ Поддержка темной и светлой темы
- ✅ Accessibility (ARIA, клавиатурная навигация, screen readers)
- ✅ Поддержка `prefers-reduced-motion`
- ✅ Автоматический timeout для зависшего стриминга
- ✅ Graceful degradation при ошибках подсветки

## Использование

### Базовое использование

```svelte
<script>
  import { StreamingCodeBlock } from '$lib/components/streaming-code';

  let code = '';
  let isStreaming = true;
</script>

<StreamingCodeBlock
  {code}
  language="javascript"
  {isStreaming}
  readonly={true}
  showLineNumbers={true}
  on:toggle={(e) => console.log('Toggled:', e.detail.expanded)}
  on:streamingTimeout={() => console.log('Streaming timeout')}
/>
```

### Интеграция с markdown рендерингом

```typescript
import { appendMarkdownText, finalizeMarkdownStreaming } from '$lib/chat/stream/markdown_block';

// Во время стриминга
ctx = appendMarkdownText(ctx, newText, true);

// По завершении стриминга
ctx = finalizeMarkdownStreaming(ctx);
```

## API

### StreamingCodeBlock Props

| Prop              | Type                          | Default  | Description                        |
| ----------------- | ----------------------------- | -------- | ---------------------------------- |
| `code`            | `string`                      | `''`     | Содержимое кода                    |
| `language`        | `string`                      | `''`     | Язык программирования              |
| `isStreaming`     | `boolean`                     | `false`  | Флаг активного стриминга           |
| `theme`           | `'light' \| 'dark' \| 'auto'` | `'auto'` | Тема оформления                    |
| `showLineNumbers` | `boolean`                     | `true`   | (зарезервировано, подсветка HLJS)  |
| `readonly`        | `boolean`                     | `true`   | Режим только для чтения            |

### События

| Event              | Detail                  | Description                          |
| ------------------ | ----------------------- | ------------------------------------ |
| `toggle`           | `{ expanded: boolean }` | Переключение состояния разворачивания|
| `streamingTimeout` | `void`                  | Timeout стриминга (30 секунд)        |

### ProgressBar Props

| Prop          | Type      | Default | Description              |
| ------------- | --------- | ------- | ------------------------ |
| `language`    | `string`  | `''`    | Язык программирования    |
| `isStreaming` | `boolean` | `false` | Флаг активного стриминга |
| `isExpanded`  | `boolean` | `false` | Состояние разворачивания |

## Поддерживаемые языки

- JavaScript (`js`, `javascript`)
- TypeScript (`ts`, `typescript`)
- Python (`py`, `python`)
- HTML (`html`)
- CSS (`css`)
- JSON (`json`)
- XML (`xml`)
- SQL (`sql`)
- React JSX (`jsx`)
- React TSX (`tsx`)

## Архитектура

### Состояния компонента

1. **Idle** - Начальное состояние, показывается только прогресс-бар
2. **Streaming** - Активный стриминг, инкрементальные обновления подсветки
3. **Completed** - Стриминг завершен, полный контент подсвечен

### Подсветка

- Highlight.js рендерит код на лету через `highlight()`/`highlightAuto()`
- Во время стриминга обновления коалесцируются через `requestAnimationFrame`
- Автоопределение языка при отсутствии явного `language`

### Обработка ошибок

- Fallback на простое отображение текста при ошибках подсветки
- Timeout механизм для автоматического завершения зависшего стриминга
- Graceful degradation при ошибках загрузки компонентов

## Accessibility

- ARIA метки для screen readers
- Поддержка навигации с клавиатуры
- Объявления изменений состояния
- Поддержка `prefers-reduced-motion`
- Высококонтрастный режим

## Производительность

- Коалесcирование частых обновлений в rAF
- Оптимизированный re-rendering
- Правильная очистка ресурсов

## Стилизация

Компонент использует CSS переменные для кастомизации:

```css
:root {
  --accent-color: #007acc;
  --accent-color-alpha: rgba(0, 122, 204, 0.1);
  --border-color: #e1e5e9;
  --panel-bg: #ffffff;
  --text-primary: #24292f;
  --code-bg: #f6f8fa;
}
```

## Демо

Запустите демонстрацию для тестирования всех функций:

```svelte
<script>
  import { StreamingCodeDemo } from '$lib/components/streaming-code';
</script>

<StreamingCodeDemo />
```
