## CodeMirror Integration Test

This file tests that CodeMirror is working correctly with the enhanced markdown rendering.

### Basic Code Block Test

```javascript
// This should render with CodeMirror
function testFunction() {
    console.log("CodeMirror is working!");
    return "Success";
}

const result = testFunction();
```

### Python Code Test

```python
# Python code should also work
def test_python():
    print("Python syntax highlighting works!")
    return True

if __name__ == "__main__":
    test_python()
```

### JSON Test

```json
{
  "test": "JSON syntax highlighting",
  "working": true,
  "features": [
    "syntax highlighting",
    "copy button",
    "language detection"
  ]
}
```

### CSS Test

```css
/* CSS code block */
.test-class {
    color: var(--primary-color);
    background: var(--bg-color);
    border-radius: 8px;
    padding: 1rem;
}

.test-class:hover {
    opacity: 0.8;
    transform: scale(1.02);
}
```

### SQL Test

```sql
-- SQL syntax highlighting
SELECT 
    id,
    name,
    email,
    created_at
FROM users 
WHERE active = true 
AND last_login > '2024-01-01'
ORDER BY created_at DESC
LIMIT 10;
```

### Plain Text Code Block

```
This is a plain text code block
It should still render with CodeMirror
But without syntax highlighting
Multiple lines work fine
```

### Inline Code Test

This paragraph contains `inline code` which should render normally, and longer code blocks should get the CodeMirror treatment.

---

**Expected Results:**
- All code blocks above should render with CodeMirror
- Each should have a toolbar with language label and copy button
- Syntax highlighting should work for recognized languages
- Copy button should use Phosphor icons (not emojis)
- Code blocks should be interactive and properly styled