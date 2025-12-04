<script lang="ts">
  import _CodeMirror from './CodeMirror.svelte';
  import { renderMarkdownToSafeHtml } from '$lib/chat/markdown';
  import { getCodeMirrorRenderer, cleanupRenderer } from '$lib/chat/codemirror-renderer';
  import { enableExternalLinks } from '$lib/chat/external-links';
  import { onMount, onDestroy } from 'svelte';

  let contentEl: HTMLDivElement | undefined = $state();
  let renderer: any;

  const sampleMarkdown = `# CodeMirror Integration Demo

This demonstrates how CodeMirror is integrated into the chat system.

## JavaScript Example

\`\`\`javascript
function fibonacci(n) {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}

console.log(fibonacci(10)); // 55
\`\`\`

## Python Example

\`\`\`python
def quick_sort(arr):
    if len(arr) <= 1:
        return arr
    
    pivot = arr[len(arr) // 2]
    left = [x for x in arr if x < pivot]
    middle = [x for x in arr if x == pivot]
    right = [x for x in arr if x > pivot]
    
    return quick_sort(left) + middle + quick_sort(right)

# Example usage
numbers = [3, 6, 8, 10, 1, 2, 1]
print(quick_sort(numbers))
\`\`\`

## CSS Example

\`\`\`css
.beautiful-button {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  padding: 12px 24px;
  border-radius: 12px;
  color: white;
  font-weight: 600;
   cursor: default;
  transition: transform 0.2s ease;
}

.beautiful-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 16px rgba(0,0,0,0.2);
}
\`\`\`

## JSON Example

\`\`\`json
{
  "name": "oxide-lab",
  "version": "0.1.0",
  "description": "A powerful desktop AI chat application",
  "dependencies": {
    "codemirror": "^6.0.0",
    "@codemirror/view": "^6.0.0",
    "@codemirror/state": "^6.0.0",
    "@codemirror/lang-javascript": "^6.0.0"
  }
}
\`\`\`

Inline code also works: \`const message = "Hello, CodeMirror!";\`

Regular text continues to render normally with **bold** and *italic* formatting.`;

  onMount(() => {
    if (contentEl) {
      contentEl.innerHTML = renderMarkdownToSafeHtml(sampleMarkdown);
      
      // Enable external links
      enableExternalLinks(contentEl);
      
      // Apply CodeMirror rendering
      renderer = getCodeMirrorRenderer(contentEl);
      renderer.startWatching(contentEl);
    }
  });

  onDestroy(() => {
    if (renderer && contentEl) {
      cleanupRenderer(contentEl);
    }
  });
</script>

<div class="demo-container">
  <h2>CodeMirror Integration Demo</h2>
  <p>This shows how code blocks are enhanced with CodeMirror in the chat interface:</p>
  
  <div class="content-wrapper">
    <div bind:this={contentEl} class="demo-content md-content"></div>
  </div>
</div>

<style>
  .demo-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }

  .demo-container h2 {
    color: var(--text);
    margin-bottom: 12px;
  }

  .demo-container p {
    color: var(--muted);
    margin-bottom: 24px;
  }

  .content-wrapper {
    background: var(--panel-bg);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    padding: 24px;
  }

  .demo-content {
    color: var(--text);
    line-height: 1.6;
  }

  :global(.demo-content h1, .demo-content h2, .demo-content h3) {
    color: var(--text);
    margin: 1.5em 0 0.5em 0;
  }

  :global(.demo-content h1:first-child) {
    margin-top: 0;
  }

  :global(.demo-content p) {
    margin: 1em 0;
  }

  :global(.demo-content code:not(.cm-content code)) {
    background: var(--code-bg);
    color: var(--code-fg);
    padding: 0.2em 0.4em;
    border-radius: 12px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    font-size: 0.9em;
  }

  :global(.demo-content strong) {
    font-weight: 600;
    color: var(--text);
  }

  :global(.demo-content em) {
    font-style: italic;
  }
</style>