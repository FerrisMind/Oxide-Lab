<script lang="ts">
  import * as Popover from "$lib/components/ui/popover";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import Check from "phosphor-svelte/lib/Check";
  import Translate from "phosphor-svelte/lib/Translate";
  import { cn } from "$lib/utils";

  interface Props {
    selectedLanguage?: string | null;
    onSelect: (lang: string) => void;
  }

  let { selectedLanguage = "auto", onSelect }: Props = $props();

  const LANGUAGES = [
    { code: "auto", label: "Auto Detect" },
    { code: "en", label: "English" },
    { code: "ru", label: "Russian" },
    { code: "zh", label: "Chinese" },
    { code: "de", label: "German" },
    { code: "es", label: "Spanish" },
    { code: "fr", label: "French" },
    { code: "ja", label: "Japanese" },
    { code: "ko", label: "Korean" },
    { code: "pt", label: "Portuguese" },
    { code: "it", label: "Italian" },
    { code: "tr", label: "Turkish" },
    { code: "pl", label: "Polish" },
    { code: "uk", label: "Ukrainian" },
    { code: "nl", label: "Dutch" },
    { code: "ar", label: "Arabic" },
    { code: "hi", label: "Hindi" },
  ];
</script>

<Popover.Root>
  <Popover.Trigger>
    {#snippet child({ props })}
        <Button 
            variant="ghost" 
            size="sm" 
            {...props} 
            class="h-8 w-auto px-2 text-xs gap-1.5 text-muted-foreground hover:text-foreground"
        >
            <Translate size={14} />
            <span class="capitalize">{LANGUAGES.find(l => l.code === (selectedLanguage || "auto"))?.label || "Auto"}</span>
        </Button>
    {/snippet}
  </Popover.Trigger>
  <Popover.Content class="w-[180px] p-0" side="top" align="start">
    <ScrollArea class="h-[300px]">
        <div class="p-1 flex flex-col gap-0.5">
            {#each LANGUAGES as lang}
                <Button
                    variant="ghost"
                    size="sm"
                    class={cn(
                        "justify-start h-8 px-2 text-xs font-normal",
                        selectedLanguage === lang.code && "bg-accent text-accent-foreground"
                    )}
                    onclick={() => onSelect(lang.code)}
                >
                    <span class="flex-1 text-left">{lang.label}</span>
                    {#if selectedLanguage === lang.code}
                        <Check size={14} />
                    {/if}
                </Button>
            {/each}
        </div>
    </ScrollArea>
  </Popover.Content>
</Popover.Root>
