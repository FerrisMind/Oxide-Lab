<script lang="ts">
  /**
   * Chat Placeholder Component
   * 
   * Displays when no model is loaded.
   */
  import { ConversationEmptyState } from '$lib/components/ai-elements/conversation';
  import Sparkle from 'phosphor-svelte/lib/Sparkle';
  import { t } from '$lib/i18n';
</script>

<ConversationEmptyState
  title={$t('chat.placeholder.title')}
  description={$t('chat.placeholder.description')}
>
  {#snippet icon()}
    <Sparkle size={48} weight="duotone" class="text-muted-foreground" />
  {/snippet}
</ConversationEmptyState>
