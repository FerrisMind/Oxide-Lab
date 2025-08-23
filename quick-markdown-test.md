// Quick Markdown Rendering Test Script
// You can paste this in the chat to test the enhanced markdown features

/**
 * Test all basic markdown syntax features
 */

## Quick Markdown Test

### Text Formatting
**Bold text** and *italic text* and ***bold italic***

~~Strikethrough~~ and `inline code` and <u>underlined</u>

### Lists and Tasks
1. Numbered item
2. Another numbered item
   - Nested bullet
   - Another nested

- [x] Completed task
- [ ] Incomplete task

### Code Blocks
```javascript
// JavaScript with syntax highlighting
function hello(name) {
    return `Hello, ${name}!`;
}
```

```python
# Python code
def greet(name):
    print(f"Hello, {name}!")
```

### Tables
| Feature | Status | Notes |
|---------|--------|-------|
| Headers | ✅ | Working |
| Alignment | ✅ | Left/Center/Right |
| Responsive | ✅ | Mobile friendly |

### Links and Images
[Link to example](https://example.com "Link title")

![Sample Image](https://via.placeholder.com/300x200 "Sample image")

### Blockquotes
> This is a blockquote
> 
> > With nested content
> 
> And `code` inside

### Advanced Features

#### Mathematical Expressions
Inline math: $E = mc^2$

Block math:
$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$

#### Technical Terms (Auto-Enhanced)
The following should be automatically wrapped in code tags:
pytorch tensorflow rust javascript python docker kubernetes

#### Collapsible Sections
<details>
<summary>📁 Click to expand details</summary>

This content is hidden by default and can be toggled.

- Item 1
- Item 2
- Item 3

</details>

#### Keyboard Keys
Press <kbd>Ctrl</kbd>+<kbd>C</kbd> to copy

#### Subscript and Superscript
Water formula: H~2~O

Power notation: x^2^ + y^2^ = z^2^

#### Definition Lists
HTML
: HyperText Markup Language

CSS
: Cascading Style Sheets

#### Special Characters and Escaping
\*Not italic\* and \#Not a header

---

### Horizontal Rule Above

This test covers most of the enhanced markdown features. All syntax should render properly according to the mdsynt.md specification.