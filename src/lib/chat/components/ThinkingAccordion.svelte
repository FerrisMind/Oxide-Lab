<script lang="ts">
  import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '$lib/components/ui/accordion';
  import { Badge } from '$lib/components/ui/badge';
  import { cn } from '$lib/utils.js';

  let {
    value = 'thinking',
    labelLoading = 'Thinking...',
    labelReady = 'Thoughts',
    streaming = false,
    autoCollapse = true,
    onToggle = undefined,
  }: {
    value?: string;
    labelLoading?: string;
    labelReady?: string;
    streaming?: boolean;
    autoCollapse?: boolean;
    onToggle?: ((open: boolean) => void) | undefined;
  } = $props();

  let accordionValue = $state<string | undefined>(streaming ? value : undefined);
  let bodyHtml = $state('');

  $effect(() => {
    onToggle?.(accordionValue === value);
  });

  export function setOpen(next: boolean) {
    accordionValue = next ? value : undefined;
  }

  export function setStreaming(next: boolean) {
    streaming = next;
    if (streaming) {
      accordionValue = value;
    } else if (autoCollapse) {
      accordionValue = undefined;
    }
  }

  export function appendHtml(html: string) {
    bodyHtml += html;
  }

  export function resetContent() {
    bodyHtml = '';
  }
</script>

<Accordion type="single" bind:value={accordionValue} class="w-full">
  <AccordionItem value={value}>
    <AccordionTrigger class="items-center gap-2">
      <Badge variant={streaming ? 'secondary' : 'outline'} class="flex items-center gap-2">
        <span
          class={cn(
            'inline-flex h-2 w-2 rounded-full',
            streaming ? 'bg-primary animate-pulse' : 'bg-muted-foreground/60',
          )}
        ></span>
        <span>{streaming ? labelLoading : labelReady}</span>
      </Badge>
    </AccordionTrigger>
    <AccordionContent>
      <div class="space-y-2 text-sm leading-relaxed text-muted-foreground">
        {@html bodyHtml}
      </div>
    </AccordionContent>
  </AccordionItem>
</Accordion>
