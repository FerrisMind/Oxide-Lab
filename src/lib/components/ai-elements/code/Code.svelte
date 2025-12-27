<script lang="ts">
  import { cn } from '../../../utils';
  import { Button } from '$lib/components/ui/button';
  import { codeVariants } from '.';
  import type { CodeRootProps } from './types';
  import { useCode } from './code.svelte.js';
  import { box } from 'svelte-toolbelt';
  import Copy from 'phosphor-svelte/lib/Copy';
  import Check from 'phosphor-svelte/lib/Check';
  import CaretDown from 'phosphor-svelte/lib/CaretDown';
  import CaretRight from 'phosphor-svelte/lib/CaretRight';
  import {
    siJavascript,
    siTypescript,
    siPython,
    siRust,
    siGnubash,
    siJson,
    siCss,
    siSvelte,
    siReact,
  } from 'simple-icons';

  let {
    ref = $bindable(null),
    variant = 'default',
    lang = 'typescript',
    code,
    class: className,
    hideLines = false,
    highlight = [],
    children,
    ...rest
  }: CodeRootProps = $props();

  const codeState = useCode({
    code: box.with(() => code),
    hideLines: box.with(() => hideLines),
    highlight: box.with(() => highlight),
    lang: box.with(() => lang),
  });

  let collapsed = $state(false);
  let copied = $state(false);

  async function handleCopy() {
    await navigator.clipboard.writeText(code);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function toggleCollapse() {
    collapsed = !collapsed;
  }

  // Language display names and icons
  const langConfig: Record<string, { name: string; icon?: { svg: string; hex: string } }> = {
    javascript: { name: 'JavaScript', icon: siJavascript },
    typescript: { name: 'TypeScript', icon: siTypescript },
    python: { name: 'Python', icon: siPython },
    rust: { name: 'Rust', icon: siRust },
    bash: { name: 'Bash', icon: siGnubash },
    json: { name: 'JSON', icon: siJson },
    css: { name: 'CSS', icon: siCss },
    svelte: { name: 'Svelte', icon: siSvelte },
    jsx: { name: 'JSX', icon: siReact },
    tsx: { name: 'TSX', icon: siReact },
    diff: { name: 'Diff' },
    text: { name: 'Text' },
  };

  const config = $derived(langConfig[lang] || { name: lang });
  const displayLang = $derived(config.name);
  const iconSvg = $derived(config.icon?.svg);
  const iconColor = $derived(config.icon ? `#${config.icon.hex}` : undefined);
</script>

<div {...rest} bind:this={ref} class={cn(codeVariants({ variant }), className)}>
  <!-- Header -->
  <div class="flex items-center gap-2 px-2.5 sm:px-3 py-1.5 sm:py-2 bg-muted border-b border-border rounded-t-xl">
    <button 
      class="flex items-center justify-center p-1 bg-transparent border-none rounded text-muted-foreground cursor-pointer transition-colors hover:text-foreground hover:bg-accent" 
      onclick={toggleCollapse} 
      aria-label={collapsed ? 'Expand' : 'Collapse'}
    >
      {#if collapsed}
        <CaretRight size={14} weight="bold" />
      {:else}
        <CaretDown size={14} weight="bold" />
      {/if}
    </button>
    <div class="flex items-center gap-1.5">
      {#if iconSvg}
        <span class="lang-icon flex items-center justify-center w-4 h-4" style:color={iconColor}>
          {@html iconSvg}
        </span>
      {/if}
      <span class="text-xs sm:text-sm font-medium text-muted-foreground">{displayLang}</span>
    </div>
    <div class="ml-auto flex items-center gap-1">
      <Button
        variant="ghost"
        size="icon"
        class="h-6 w-6 sm:h-7 sm:w-7 text-muted-foreground hover:text-foreground"
        onclick={handleCopy}
        aria-label="Copy code"
      >
        {#if copied}
          <Check size={14} />
        {:else}
          <Copy size={14} />
        {/if}
      </Button>
    </div>
  </div>

  <!-- Code content -->
  <div class="code-content" class:collapsed>
    <div class="ai-code-wrapper">
      {@html codeState.highlighted}
    </div>
  </div>
</div>

<style>
  /* ===== Language Icon (CSS for SVG fill) ===== */
  .lang-icon :global(svg) {
    width: 100%;
    height: 100%;
    fill: currentColor;
  }

  /* ===== Collapse Animation (CSS for transitions) ===== */

  .code-content {
    overflow: hidden;
    transition: max-height 0.3s ease, opacity 0.2s ease;
    max-height: 800px;
    opacity: 1;
  }

  .code-content.collapsed {
    max-height: 0;
    opacity: 0;
  }

  /* Scoped global styles - only affect elements within .ai-code-wrapper */
  /* Dark mode: check dark class on parent, then scope to wrapper */
  :global(.dark) .ai-code-wrapper :global(.shiki),
  :global(.dark) .ai-code-wrapper :global(.shiki span) {
    color: var(--shiki-dark) !important;
    font-style: var(--shiki-dark-font-style) !important;
    font-weight: var(--shiki-dark-font-weight) !important;
    text-decoration: var(--shiki-dark-text-decoration) !important;
  }

  /* Shiki see: https://shiki.matsu.io/guide/dual-themes#class-based-dark-mode */
  :global(html.dark) .ai-code-wrapper :global(.shiki),
  :global(html.dark) .ai-code-wrapper :global(.shiki span) {
    color: var(--shiki-dark) !important;
    font-style: var(--shiki-dark-font-style) !important;
    font-weight: var(--shiki-dark-font-weight) !important;
    text-decoration: var(--shiki-dark-text-decoration) !important;
  }

  .ai-code-wrapper :global(pre.shiki) {
    overflow-x: auto;
    border-radius: 0 0 0.75rem 0.75rem;
    background: inherit;
    padding-top: 1rem;
    padding-bottom: 1rem;
    font-size: 0.875rem;
    margin: 0;
  }

  .ai-code-wrapper :global(pre.shiki:not([data-code-overflow] *):not([data-code-overflow])) {
    overflow-y: auto;
    max-height: min(100%, 650px);
  }

  .ai-code-wrapper :global(pre.shiki code) {
    display: grid;
    min-width: 100%;
    border-radius: 0;
    border: 0;
    background: transparent;
    padding: 0;
    word-break: break-word;
    counter-reset: line;
    box-decoration-break: clone;
  }

  .ai-code-wrapper :global(pre.line-numbers) {
    counter-reset: step;
    counter-increment: step 0;
  }

  .ai-code-wrapper :global(pre.line-numbers .line::before) {
    content: counter(step);
    counter-increment: step;
    display: inline-block;
    width: 1.8rem;
    margin-right: 1.4rem;
    text-align: right;
    color: var(--muted-foreground);
  }

  .ai-code-wrapper :global(pre .line.line--highlighted) {
    background: var(--secondary);
  }

  .ai-code-wrapper :global(pre .line.line--highlighted span) {
    position: relative;
  }

  .ai-code-wrapper :global(pre .line) {
    display: inline-block;
    min-height: 1rem;
    width: 100%;
    padding: 0.125rem 1rem;
  }

  .ai-code-wrapper :global(pre.line-numbers .line) {
    padding: 0 0.5rem;
  }
</style>
