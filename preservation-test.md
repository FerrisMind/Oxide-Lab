## Preservation Mechanism Test

This tests that the preservation mechanism works correctly and doesn't leave placeholders.

### Code Blocks Should Work

```javascript
// This code block should render with CodeMirror
function testPreservation() {
    console.log("Testing preservation mechanism");
    return "success";
}
```

```python
# Python code block
def test_python():
    print("Python code should work")
    return True
```

### Inline Code Should Work

This paragraph has `inline code` that should be preserved correctly.

### Technical Terms Should Be Auto-Enhanced

The following technical terms should be wrapped in code tags:
- pytorch should become `pytorch`
- tensorflow should become `tensorflow`
- javascript should become `javascript`
- python should become `python`

### Mixed Content

Here's a paragraph with `inline code` and technical terms like pytorch and tensorflow mixed together.

```json
{
  "test": "json code block",
  "pytorch": "should not be enhanced inside code",
  "tensorflow": "should also not be enhanced"
}
```

But outside the code block, pytorch and tensorflow should be enhanced.

### Expected Results

1. No "preserve" or "TECHTERM_PRESERVE" text should appear
2. Code blocks should render with CodeMirror
3. Technical terms outside code blocks should be wrapped in backticks
4. Technical terms inside code blocks should remain unchanged
5. Inline code should work normally