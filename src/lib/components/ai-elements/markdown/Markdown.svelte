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

  const supportedLangs: SupportedLanguage[] = ['bash', 'diff', 'javascript', 'json', 'svelte', 'typescript', 'python', 'tsx', 'jsx', 'css', 'rust', 'text'];
  
  function getLang(lang: string | undefined): SupportedLanguage {
    if (!lang) return 'text';
    const lower = lang.toLowerCase();
    if (supportedLangs.includes(lower as SupportedLanguage)) {
      return lower as SupportedLanguage;
    }
    // Common aliases
    if (lower === 'js') return 'javascript';
    if (lower === 'ts') return 'typescript';
    if (lower === 'sh' || lower === 'shell' || lower === 'zsh') return 'bash';
    if (lower === 'py') return 'python';
    if (lower === 'rs') return 'rust';
    return 'text';
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
