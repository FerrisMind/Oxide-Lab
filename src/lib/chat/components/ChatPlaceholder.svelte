<script lang="ts">
  import Sparkle from "phosphor-svelte/lib/Sparkle";

  export let variant: 'full' | 'inline' = 'full';
</script>

<section class="chat-placeholder" class:compact={variant === 'inline'}>
  <div class="placeholder-content">
    <Sparkle size={48} weight="duotone" style="color: var(--muted); margin-bottom: 16px;" />
    <h3>Модель не загружена</h3>
    <p>Загрузите GGUF модель, чтобы начать общение</p>
  </div>
</section>


