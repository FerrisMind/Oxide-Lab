<script lang="ts">
  import { cn } from '../../../utils';
  import { Streamdown } from 'svelte-streamdown';
  import { Code } from '$lib/components/ai-elements/code';
  import type { SupportedLanguage } from '$lib/components/ai-elements/code/shiki';
  import type { HTMLAttributes } from 'svelte/elements';

  type Props = {
    content: string;
    id?: string;
    class?: string;
  } & Omit<HTMLAttributes<HTMLDivElement>, 'content'>;

  let { content, id, class: className, ...restProps }: Props = $props();

  // All supported languages for Oxide Lab
  const supportedLangs: SupportedLanguage[] = [
    // Fallback & CLI
    'text', 'ansi',
    // Documentation
    'markdown', 'mdx',
    // Rust ecosystem
    'rust', 'toml',
    // Configs
    'yaml', 'json', 'ini',
    // Shell & DevOps
    'bash', 'powershell', 'dockerfile', 'nginx',
    // Web frontend
    'javascript', 'typescript', 'tsx', 'jsx', 'html', 'css', 'scss', 'svelte',
    // Data & Query
    'sql', 'graphql', 'regex',
    // Misc
    'diff', 'python',
  ];
  
  function getLang(lang: string | undefined): SupportedLanguage {
    if (!lang) return 'text';
    const lower = lang.toLowerCase();
    
    // Direct match
    if (supportedLangs.includes(lower as SupportedLanguage)) {
      return lower as SupportedLanguage;
    }
    
    // Aliases
    const aliases: Record<string, SupportedLanguage> = {
      // Text fallbacks
      'txt': 'text',
      'plain': 'text',
      'plaintext': 'text',
      // JavaScript
      'js': 'javascript',
      'mjs': 'javascript',
      'cjs': 'javascript',
      // TypeScript
      'ts': 'typescript',
      'mts': 'typescript',
      'cts': 'typescript',
      // Shell
      'sh': 'bash',
      'shell': 'bash',
      'zsh': 'bash',
      'ps1': 'powershell',
      'pwsh': 'powershell',
      // Rust
      'rs': 'rust',
      // Python
      'py': 'python',
      // HTML/CSS
      'htm': 'html',
      'sass': 'scss',
      // Config
      'yml': 'yaml',
      'jsonc': 'json',
      'json5': 'json',
      // Documentation
      'md': 'markdown',
      // Data
      'gql': 'graphql',
      're': 'regex',
      'regexp': 'regex',
      // Docker
      'docker': 'dockerfile',
    };
    
    return aliases[lower] || 'text';
  }
</script>

<div {id} class={cn("markdown-content", className)} {...restProps}>
  <Streamdown
    {content}
    class="[&>*:first-child]:mt-0 [&>*:last-child]:mb-0"
    shikiTheme="github-dark-default"
    baseTheme="shadcn"
  >
    {#snippet code({ token })}
      <Code 
        code={token.text} 
        lang={getLang(token.lang)}
        variant="secondary"
        hideLines={true}
      />
    {/snippet}
  </Streamdown>
</div>

<style>
  /* Inline code styling */
  .markdown-content :global(code:not(pre code)) {
    background: var(--muted);
    color: var(--foreground);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    font-size: 0.875em;
  }
</style>
