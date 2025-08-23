# Pure Markdown Specification Test

This test verifies that markdown rendering follows the pure specification from mdsynt.txt.

## Inline Code Test

According to the specification: `код` — выделяет inline-код моноширинным шрифтом

### Correctly Marked Inline Code (Should be highlighted)

- `pytorch` - explicitly marked with backticks
- `transformers` - explicitly marked with backticks  
- `javascript` - explicitly marked with backticks
- `model` - explicitly marked with backticks

### Normal Text (Should NOT be highlighted)

- pytorch - without backticks, should remain normal text
- transformers - without backticks, should remain normal text
- javascript - without backticks, should remain normal text
- model - without backticks, should remain normal text
- api - common word, should remain normal text
- data - common word, should remain normal text
- git - common word, should remain normal text

## Mixed Content Test

In this sentence, only `pytorch` should be highlighted because it has backticks, while words like model, api, data, and git should remain as normal text.

## Code Blocks Test

```python
# This entire block should be syntax highlighted
pytorch = "framework"
model = "some model"
api = "some api"
data = "some data"
```

The words pytorch, model, api, and data inside the code block should be syntax highlighted as part of the Python code, not individually enhanced.

## Expected Results

✅ Only text explicitly wrapped in backticks should be rendered as inline code
✅ Normal text should remain as normal text, regardless of technical terms
✅ Code blocks should work with normal syntax highlighting
✅ No random words should be automatically highlighted