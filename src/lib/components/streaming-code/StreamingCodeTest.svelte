<script lang="ts">
  import { onMount } from 'svelte';
  import { StreamingCodeBlock } from '$lib/components/streaming-code';

  let code = $state('');
  let isStreaming = $state(false);
  let isComplete = $state(false);
  let language = $state('javascript');

  // Simulate streaming code
  onMount(() => {
    // Start streaming
    isStreaming = true;
    
    // Simulate receiving code chunks
    setTimeout(() => {
      code = 'console';
    }, 500);
    
    setTimeout(() => {
      code = 'console.log';
    }, 1000);
    
    setTimeout(() => {
      code = 'console.log("Hello';
    }, 1500);
    
    setTimeout(() => {
      code = 'console.log("Hello, World!';
    }, 2000);
    
    setTimeout(() => {
      code = 'console.log("Hello, World!");';
      isStreaming = false;
      isComplete = true;
    }, 2500);
  });
</script>

<div style="padding: var(--space-4);">
  <h2>Streaming Code Block Test</h2>
  <StreamingCodeBlock 
    {code} 
    {language} 
    {isStreaming} 
    {isComplete}
    readonly={true}
    showLineNumbers={false}
  />
</div>