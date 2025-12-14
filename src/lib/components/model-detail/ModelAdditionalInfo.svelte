<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';

  interface Props {
    detailedModel: HFModel | null;
  }

  let { detailedModel }: Props = $props();
</script>

{#if detailedModel}
  <div class="additional-info">
    <h3>Дополнительная информация</h3>
    <div class="info-grid">
      {#if detailedModel.pipeline_tag}
        <div class="info-item">
          <span class="info-label">Pipeline Tag:</span>
          <span class="info-value">{detailedModel.pipeline_tag}</span>
        </div>
      {/if}
      {#if detailedModel.library_name}
        <div class="info-item">
          <span class="info-label">Библиотека:</span>
          <span class="info-value">{detailedModel.library_name}</span>
        </div>
      {/if}
      {#if detailedModel.license}
        <div class="info-item">
          <span class="info-label">Лицензия:</span>
          <span class="info-value">{detailedModel.license}</span>
        </div>
      {/if}
      {#if detailedModel.language}
        <div class="info-item">
          <span class="info-label">Язык:</span>
          <span class="info-value">
            {Array.isArray(detailedModel.language)
              ? detailedModel.language.join(', ')
              : detailedModel.language}
          </span>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .additional-info {
    margin-bottom: var(--space-4); /* 24px */
  }

  .additional-info h3 {
    font-size: 1.125rem;
    font-weight: var(--font-weight-semibold);
    color: var(--text);
    margin: 0 0 var(--space-2) 0; /* 8px → 12px closest */
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--space-3); /* 16px */
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: var(--space-2); /* 8px */
    padding: var(--space-3); /* 16px */
    background: var(--panel-alt-bg);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg); /* 16px */
    transition: all 0.2s ease;
  }

  .info-item:hover {
    background: var(--panel-bg);
    border-color: var(--accent);
  }

  .info-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .info-value {
    font-size: 1rem;
    color: var(--text);
    font-weight: 500;
  }

  @media (max-width: 768px) {
    .info-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
