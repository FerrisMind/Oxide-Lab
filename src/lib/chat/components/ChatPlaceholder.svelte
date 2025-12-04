<script lang="ts">
  import Sparkle from "phosphor-svelte/lib/Sparkle";
  import { t } from '$lib/i18n';

  export let variant: 'full' | 'inline' = 'full';
</script>

<section class="chat-placeholder" class:compact={variant === 'inline'}>
  <div class="placeholder-content">
    <Sparkle size={48} weight="duotone" style="color: var(--muted);" />
    <div class="placeholder-text">
      <h3>{$t('chat.placeholder.title')}</h3>
      <p>{$t('chat.placeholder.description')}</p>
    </div>
  </div>
</section>


