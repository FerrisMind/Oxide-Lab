# Comprehensive Markdown Syntax Test

This file tests all markdown syntax features mentioned in the mdsynt.md specification.

## Basic Syntax (Основной синтаксис)

### Headings (Заголовки)

# Heading 1 (Заголовок 1)
## Heading 2 (Заголовок 2) 
### Heading 3 (Заголовок 3)
#### Heading 4 (Заголовок 4)
##### Heading 5 (Заголовок 5)
###### Heading 6 (Заголовок 6)

Alternative heading syntax:

Alternative Heading 1
===

Alternative Heading 2
---

### Text Formatting (Форматирование текста)

**Bold text (жирный текст)** — makes text bold

*Italic text (курсивный текст)* — makes text italic

***Bold italic (жирный курсив)*** — makes text bold and italic

~~Strikethrough (зачеркнутый текст)~~ — strikes through text

`inline code (код)` — highlights inline code

<u>Underlined text (подчеркнутый)</u> — underlines text (HTML)

### Paragraphs and Line Breaks

This is a paragraph.

This is another paragraph separated by an empty line.

Line with two spaces at the end  
Forces a line break

<br>HTML tag for line break

### Lists (Списки)

#### Numbered Lists (Нумерованные списки):

1. Item 1 (Пункт 1)
2. Item 2 (Пункт 2)
   1. Sub-item (Подпункт)
   2. Another sub-item

#### Bulleted Lists (Маркированные списки):

- Item with dash (Пункт с дефисом)
* Item with asterisk (Пункт со звездочкой)
+ Item with plus (Пункт с плюсом)
  - Sub-item (Подпункт)
  - Another sub-item

#### Task Lists (Списки задач):

- [x] Completed task (Выполненная задача)
- [ ] Incomplete task (Невыполненная задача)
- [x] Another completed task

### Links (Ссылки)

#### Inline Links:

[Link text](https://example.com) — creates a link

[Link with title](https://example.com "This is a title") — link with tooltip

#### Reference Links:

[Reference link][id] — link with identifier

[id]: https://example.com "Reference title"

#### Auto Links:

<https://example.com> — auto-converts URL to link

<email@example.com> — auto-converts email to link

### Images (Изображения)

#### Inline Images:

![Alt text](https://via.placeholder.com/150x100 "Image title") — inserts image

#### Reference Images:

![Alt text][img-id] — image with identifier

[img-id]: https://via.placeholder.com/200x150 "Reference image"

### Blockquotes (Блоки цитат)

> This is a quote (Цитата)

>> This is a nested quote (Вложенная цитата)

> * List in quote (Список в цитате)
> * Another item

### Code (Код)

#### Inline Code:

This is `inline code` in a sentence.

#### Code Blocks:

```
Code block without syntax highlighting
Multiple lines
Of code
```

```python
# Python code with syntax highlighting
def hello():
    print("Hello, World!")
```

```javascript
// JavaScript code
function greet(name) {
    console.log(`Hello, ${name}!`);
}
```

### Horizontal Rules (Горизонтальные линии)

---

***

___

### Escaping Characters (Экранирование символов)

\* Escaped asterisk
\# Escaped hash
\[ Escaped bracket
\\ Escaped backslash

## Extended Syntax (Расширенный синтаксис)

### Tables (Таблицы)

| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |

#### Table with Alignment:

| Left | Center | Right |
|:-----|:------:|------:|
| Left | Center | Right |
| Text | Text   | Text  |

### Definition Lists (Списки определений)

Term 1
: Definition of term 1

Term 2
: Definition of term 2
: Another definition of term 2

### Highlighting (Выделение)

==Highlighted text== — highlights text with marker

### Subscript and Superscript (Подстрочный и надстрочный индексы)

H~2~O — subscript (подстрочный индекс)

X^2^ — superscript (надстрочный индекс)

### Mathematical Expressions (Математические выражения)

$E = mc^2$ — inline mathematical expression

$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$

### Emoji (Эмодзи)

:smile: — emoji by code (depends on platform support)

😀 — direct Unicode emoji

### Buttons (Кнопки)

[Button](https://example.com){.button} — creates styled button

### Abbreviations (Аббревиатуры)

*[HTML]: HyperText Markup Language

We use HTML for markup.

### Keyboard Keys (Клавиши клавиатуры)

Press <kbd>Ctrl</kbd>+<kbd>C</kbd> to copy

++ctrl+c++ — alternative syntax for key combinations

### Collapsible Sections (Свертываемые секции)

<details>
<summary>Click to expand (Кликните, чтобы развернуть)</summary>

Hidden content here.

This content is initially hidden and can be expanded by clicking the summary.

</details>

### Inline HTML (Включения HTML)

<div style="color: red;">HTML block</div>

<span style="color: blue;">colored text</span> — inline HTML

### User Mentions (Упоминания пользователей)

@username — user mention (platform dependent)

### Issue References (Ссылки на задачи)

#123 — reference to issue number 123

### Color Models (Цветовые модели)

`#ff0000` — HEX color code

`rgb(255, 0, 0)` — RGB color code

### Comments (Комментарии)

<!-- This is a comment that won't be displayed -->

### Anchor Links (Якорные ссылки)

[Link to section](#basic-syntax) — link to anchor in same document

[Back to top](#comprehensive-markdown-syntax-test) — link to top of page

### Additional Elements

#### Time and Date

<time datetime="2024-08-23">August 23, 2024</time>

#### Data

<data value="123">One hundred twenty-three</data>

#### Variables and Code Samples

The <var>variable</var> contains the value.

Output: <samp>Hello, World!</samp>

#### Definitions and Citations

<dfn>Markdown</dfn> is a lightweight markup language.

According to <cite>the specification</cite>, this should work.

#### Quotes

<q>This is a quoted text</q> with automatic quotation marks.

#### Ruby Annotations (for East Asian languages)

<ruby>
  漢 <rp>(</rp><rt>kan</rt><rp>)</rp>
  字 <rp>(</rp><rt>ji</rt><rp>)</rp>
</ruby>

### Complex Nested Structures

> #### Quote with heading
> 
> 1. Numbered list in quote
> 2. With `inline code`
> 
> ```python
> # Code block in quote
> print("Complex nesting")
> ```
> 
> | Column 1 | Column 2 |
> |----------|----------|
> | Table    | In quote |

### Bidirectional Text Support

<div dir="rtl">هذا نص باللغة العربية</div>

<div dir="ltr">This is left-to-right text</div>

### Testing Technical Terms Auto-Enhancement

The following technical terms should be automatically wrapped in code tags:

transformers pytorch tensorflow onnx safetensors gguf ggml llama mistral qwen bert gpt tokenizer embedding attention inference quantization fine-tuning lora model dataset python javascript typescript rust cuda json yaml docker kubernetes git github api rest graphql

---

This completes the comprehensive test of all markdown syntax features mentioned in the mdsynt.md specification.