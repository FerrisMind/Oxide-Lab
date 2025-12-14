<script lang="ts">
  import { onMount } from 'svelte';
  import StreamingCodeBlock from './StreamingCodeBlock.svelte';

  let isStreaming = $state(false);
  let code = $state('');
  let language = $state('javascript');
  let streamingInterval: number | null = null;

  const sampleCodes = {
    javascript: `function fibonacci(n) {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}

// Example usage
console.log(fibonacci(10)); // 55`,

    python: `def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

# Example usage
print(fibonacci(10))  # 55`,

    typescript: `interface User {
  id: number;
  name: string;
  email: string;
}

function createUser(name: string, email: string): User {
  return {
    id: Math.random(),
    name,
    email
  };
}

const user = createUser("John Doe", "john@example.com");`,

    html: `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Sample Page</title>
</head>
<body>
    <h1>Hello, World!</h1>
    <p>This is a sample HTML page.</p>
</body>
</html>`,
  };

  function startStreaming() {
    if (isStreaming) return;

    isStreaming = true;
    code = '';

    const targetCode = sampleCodes[language as keyof typeof sampleCodes] || sampleCodes.javascript;
    let currentIndex = 0;

    streamingInterval = setInterval(() => {
      if (currentIndex < targetCode.length) {
        code += targetCode[currentIndex];
        currentIndex++;
      } else {
        stopStreaming();
      }
    }, 50); // Add character every 50ms
  }

  function stopStreaming() {
    if (streamingInterval) {
      clearInterval(streamingInterval);
      streamingInterval = null;
    }
    isStreaming = false;
  }

  function resetDemo() {
    stopStreaming();
    code = '';
  }

  function handleLanguageChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    language = target.value;
    resetDemo();
  }

  function handleToggle(detail: { expanded: boolean }) {
    console.log('Code block toggled:', detail);
  }

  function handleStreamingTimeout() {
    console.log('Streaming timeout occurred');
    stopStreaming();
  }

  onMount(() => {
    return () => {
      if (streamingInterval) {
        clearInterval(streamingInterval);
      }
    };
  });
</script>

<div class="demo-container">
  <h2>StreamingCodeBlock Demo</h2>

  <div class="controls">
    <div class="control-group">
      <label for="language-select">Language:</label>
      <select id="language-select" value={language} onchange={handleLanguageChange}>
        <option value="javascript">JavaScript</option>
        <option value="python">Python</option>
        <option value="typescript">TypeScript</option>
        <option value="html">HTML</option>
      </select>
    </div>

    <div class="control-group">
      <button onclick={startStreaming} disabled={isStreaming}> Start Streaming </button>
      <button onclick={stopStreaming} disabled={!isStreaming}> Stop Streaming </button>
      <button onclick={resetDemo}> Reset </button>
    </div>
  </div>

  <div class="demo-section">
    <h3>Streaming Code Block</h3>
    <StreamingCodeBlock
      {code}
      {language}
      {isStreaming}
      readonly={true}
      showLineNumbers={true}
      onToggle={handleToggle}
      onStreamingTimeout={handleStreamingTimeout}
    />
  </div>

  <div class="info-section">
    <h3>Current State</h3>
    <ul>
      <li><strong>Language:</strong> {language}</li>
      <li><strong>Is Streaming:</strong> {isStreaming ? 'Yes' : 'No'}</li>
      <li><strong>Code Length:</strong> {code.length} characters</li>
      <li><strong>Lines:</strong> {code.split('\n').length}</li>
    </ul>
  </div>
</div>

<style>
  .demo-container {
    max-width: var(--chat-max-width); /* 800px */
    margin: 0 auto;
    padding: var(--space-3); /* 16px → 20px closest */
    font-family:
      system-ui,
      -apple-system,
      sans-serif;
  }

  .controls {
    display: flex;
    gap: var(--space-3); /* 16px → 20px closest */
    margin-bottom: var(--space-5); /* 32px → 30px closest */
    padding: var(--space-3); /* 16px → 20px closest */
    background: var(--panel-bg, #f5f5f5);
    border-radius: var(--radius-lg); /* 16px */
    border: 1px solid var(--border-color, #ddd);
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2); /* 8px */
  }

  .control-group label {
    font-weight: var(--font-weight-semibold);
    font-size: var(--font-size-sm); /* 14px */
    color: var(--text-primary, #333);
  }

  .control-group select,
  .control-group button {
    padding: var(--space-2) var(--space-3); /* 8px 16px */
    border: 1px solid var(--border-color, #ddd);
    border-radius: var(--radius-lg); /* 16px */
    background: var(--input-bg, white);
    color: var(--text-primary, #333);
    font-size: var(--font-size-sm); /* 14px */
  }

  .control-group button {
    cursor: default;
    background: var(--accent-color, #007acc);
    color: white;
    border-color: var(--accent-color, #007acc);
    transition: background-color 0.2s ease;
  }

  .control-group button:hover:not(:disabled) {
    background: var(--accent-color-hover, #005a9e);
  }

  .control-group button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .demo-section {
    margin-bottom: var(--space-5); /* 32px → 30px closest */
  }

  .demo-section h3 {
    margin-bottom: var(--space-3); /* 16px → 15px closest */
    color: var(--text-primary, #333);
  }

  .info-section {
    padding: var(--space-3); /* 16px → 20px closest */
    background: var(--panel-alt-bg, #f9f9f9);
    border-radius: var(--radius-lg); /* 16px */
    border: 1px solid var(--border-color, #ddd);
  }

  .info-section h3 {
    margin-top: 0;
    margin-bottom: var(--space-3); /* 16px → 15px closest */
    color: var(--text-primary, #333);
  }

  .info-section ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .info-section li {
    padding: var(--space-1) 0; /* 4px 0 */
    color: var(--text-secondary, #666);
  }

  .info-section strong {
    color: var(--text-primary, #333);
  }

  /* Dark theme support */
  @media (prefers-color-scheme: dark) {
    .demo-container {
      color: #e0e0e0;
    }
  }
</style>
